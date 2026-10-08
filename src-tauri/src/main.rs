#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod remote;

use ghostkeys_lib::{
    engine, handle_tray_pause_resume, handle_tray_start_stop, show_main_window, toggle_widget,
    Config, FileInfo, TypingStatus,
};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

// ============================================================================
// Tauri Commands
// ============================================================================

#[tauri::command]
fn get_config() -> Config {
    engine().get_config()
}

#[tauri::command]
fn set_config(config: Config) {
    engine().set_config(config);
}

#[tauri::command]
fn set_file_content(app: AppHandle, content: String, file_name: String) -> Result<(), String> {
    if engine().is_running() || matches!(engine().get_status(), TypingStatus::Typing | TypingStatus::Countdown | TypingStatus::Paused) {
        return Err("Stop typing before replacing the text".into());
    }
    engine().set_content(content, file_name);
    engine().set_status(TypingStatus::Ready, &app);
    Ok(())
}

#[tauri::command]
fn load_file(path: String) -> Result<FileInfo, String> {
    if engine().is_running() || matches!(engine().get_status(), TypingStatus::Typing | TypingStatus::Countdown | TypingStatus::Paused) {
        return Err("Stop typing before replacing the text".into());
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let name = std::path::Path::new(&path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let char_count = content.chars().count() as u32;

    engine().set_content(content.clone(), name.clone());

    Ok(FileInfo {
        name,
        content,
        char_count,
    })
}

#[tauri::command]
async fn start_typing(app: AppHandle) -> Result<(), String> {
    ghostkeys_lib::platform::ensure_keyboard_access()?;
    let status = engine().get_status();
    if engine().is_running() && !matches!(status, TypingStatus::Paused) {
        return Err("Wait for the current typing session to stop".into());
    }

    match status {
        TypingStatus::Ready
        | TypingStatus::Done
        | TypingStatus::Idle
        | TypingStatus::Error => {
            // Start typing
            let eng = engine().clone();
            if eng.get_progress().total == 0 { return Err("Load text first".into()); }
            eng.set_status(TypingStatus::Countdown, &app);
            tauri::async_runtime::spawn(async move {
                if let Err(e) = eng.run(app.clone()).await {
                    eprintln!("Typing error: {}", e);
                    let _ = app.emit("typing-error", serde_json::json!({ "message": e }));
                }
            });
            Ok(())
        }
        TypingStatus::Typing
        | TypingStatus::Countdown => {
            Err("Already typing".to_string())
        }
        TypingStatus::Paused => {
            // Resume instead
            engine().resume();
            engine().set_status(TypingStatus::Typing, &app);
            Ok(())
        }
    }
}

#[tauri::command]
fn stop_typing(app: AppHandle) {
    engine().stop();
    engine().set_status(TypingStatus::Ready, &app);
}

#[tauri::command]
fn pause_typing(app: AppHandle) {
    engine().pause();
    engine().set_status(TypingStatus::Paused, &app);
}

#[tauri::command]
fn resume_typing(app: AppHandle) {
    engine().resume();
    engine().set_status(TypingStatus::Typing, &app);
}

#[tauri::command]
fn get_state() -> serde_json::Value {
    let progress = engine().get_progress();
    let status = engine().get_status();
    let file_name = engine().get_file_name();

    serde_json::json!({
        "status": status,
        "current_char": progress.current,
        "total_chars": progress.total,
        "file_name": file_name,
    })
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    // Capture app handle for use in global shortcut handler
    let app_handle_for_shortcut: std::sync::Arc<std::sync::Mutex<Option<AppHandle>>> =
        std::sync::Arc::new(std::sync::Mutex::new(None));
    let app_handle_clone = app_handle_for_shortcut.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |_app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        if let Some(ref handle) = *app_handle_clone.lock().unwrap() {
                            handle_tray_start_stop(handle);
                        }
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            load_file,
            start_typing,
            stop_typing,
            pause_typing,
            resume_typing,
            get_config,
            set_config,
            get_state,
            set_file_content,
            ghostkeys_lib::platform::get_platform_status,
            ghostkeys_lib::platform::open_accessibility_settings,
            ghostkeys_lib::background::get_background_target,
            ghostkeys_lib::background::capture_background_target,
            ghostkeys_lib::background::clear_background_target,
        ])
        .setup(move |app| {
            remote::start(app.handle().clone())?;
            // Store app handle for global shortcut handler
            *app_handle_for_shortcut.lock().unwrap() = Some(app.handle().clone());

            // Build tray menu
            let start_stop = MenuItem::with_id(app, "start_stop", "Start/Stop", true, None::<&str>)?;
            let pause_resume =
                MenuItem::with_id(app, "pause_resume", "Pause/Resume", true, None::<&str>)?;
            let show_main = MenuItem::with_id(app, "show_main", "Show Window", true, None::<&str>)?;
            let toggle_widget_item =
                MenuItem::with_id(app, "toggle_widget", "Toggle Widget", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            let menu = Menu::with_items(
                app,
                &[
                    &start_stop,
                    &pause_resume,
                    &show_main,
                    &toggle_widget_item,
                    &quit,
                ],
            )?;

            // Create a simple 16x16 icon (purple square as placeholder)
            let icon_data = create_placeholder_icon();
            let icon = Image::new_owned(icon_data, 16, 16);

            // Create tray icon
            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .tooltip("ghostkeys")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "start_stop" => handle_tray_start_stop(app),
                    "pause_resume" => handle_tray_pause_resume(app),
                    "show_main" => show_main_window(app),
                    "toggle_widget" => toggle_widget(app),
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        toggle_widget(tray.app_handle());
                    }
                })
                .build(app)?;

            // Register global shortcut (Ctrl+Alt+S)
            let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyS);
            if let Err(e) = app.global_shortcut().register(shortcut) {
                eprintln!("Failed to register global shortcut: {}", e);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Create a simple 16x16 placeholder icon (purple ghost color)
fn create_placeholder_icon() -> Vec<u8> {
    let mut data = Vec::with_capacity(16 * 16 * 4);
    let purple = [0x8b, 0x5c, 0xf6, 0xff]; // #8b5cf6
    
    for y in 0..16 {
        for x in 0..16 {
            // Simple ghost shape
            let is_ghost = {
                // Head (circle at top)
                let head_center = (8.0, 6.0);
                let dx = x as f32 - head_center.0;
                let dy = y as f32 - head_center.1;
                let in_head = (dx * dx + dy * dy) < 36.0;
                
                // Body (rectangle below)
                let in_body = y >= 6 && y <= 13 && x >= 3 && x <= 12;
                
                // Wavy bottom
                let in_wave = y >= 12 && y <= 15 && x >= 3 && x <= 12 && 
                    ((x + y) % 3 != 0 || y < 14);
                
                in_head || in_body || in_wave
            };
            
            if is_ghost {
                data.extend_from_slice(&purple);
            } else {
                data.extend_from_slice(&[0, 0, 0, 0]); // Transparent
            }
        }
    }
    
    data
}
