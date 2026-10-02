#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use ghostkeys_lib::preferences::{self, PreferenceState, Preferences, ShortcutRecording};
use ghostkeys_lib::{
    engine, handle_tray_pause_resume, handle_tray_start_stop, platform, show_main_window,
    toggle_widget, Config, FileInfo, TypingStatus,
};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use tauri_plugin_store::StoreExt;

// ============================================================================
// Tauri Commands
// ============================================================================

#[tauri::command]
fn get_config() -> Config {
    engine().get_config()
}

#[tauri::command]
fn set_config(config: Config, app: AppHandle) -> Result<(), String> {
    engine().set_config(config.clone())?;
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store.set(
        "config",
        serde_json::to_value(config).map_err(|e| e.to_string())?,
    );
    store
        .save()
        .map_err(|e| format!("Could not save settings: {e}"))
}

#[tauri::command]
fn set_file_content(content: String, file_name: String, app: AppHandle) -> Result<(), String> {
    engine().set_content(content, file_name, &app)?;
    preferences::save_draft(&app)
}

#[tauri::command]
fn set_selection(start: usize, end: usize, app: AppHandle) -> Result<(), String> {
    engine().set_selection(start, end, &app)
}

#[tauri::command]
fn load_file(path: String) -> Result<FileInfo, String> {
    let content =
        std::fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {}", e))?;

    let name = std::path::Path::new(&path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let content = ghostkeys_lib::typer::normalize_content(&content);
    let char_count = content.chars().count() as u32;

    Ok(FileInfo {
        name,
        content,
        char_count,
    })
}

#[tauri::command]
async fn start_typing(app: AppHandle) -> Result<(), String> {
    if engine().get_status() == TypingStatus::Paused {
        engine().resume(&app)
    } else {
        engine().start(app)
    }
}

#[tauri::command]
fn stop_typing() {
    engine().stop();
}

#[tauri::command]
fn pause_typing(app: AppHandle) {
    engine().pause(&app);
}

#[tauri::command]
fn resume_typing(app: AppHandle) -> Result<(), String> {
    engine().resume(&app)
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
        "content": engine().get_content(),
        "error_message": engine().get_error_message(),
    })
}

#[tauri::command]
fn get_session_timing() -> ghostkeys_lib::typer::SessionTiming {
    engine().session_timing()
}

#[tauri::command]
fn get_platform_info() -> platform::PlatformInfo {
    platform::info()
}

#[tauri::command]
fn request_accessibility() -> bool {
    platform::request_accessibility()
}

#[tauri::command]
fn open_accessibility_settings() -> Result<(), String> {
    platform::open_accessibility_settings()
}

#[tauri::command]
fn show_widget(app: AppHandle) -> Result<(), String> {
    ghostkeys_lib::show_progress_widget(&app).map_err(|e| e.to_string())
}

#[tauri::command]
fn show_main(app: AppHandle) {
    show_main_window(&app);
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    tauri::Builder::default()
        .manage(ShortcutRecording(std::sync::atomic::AtomicBool::new(false)))
        .manage(PreferenceState(parking_lot::Mutex::new(
            Preferences::default(),
        )))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    if event.state() == ShortcutState::Pressed
                        && !app
                            .state::<ShortcutRecording>()
                            .0
                            .load(std::sync::atomic::Ordering::SeqCst)
                    {
                        let keys = app.state::<PreferenceState>().0.lock().shortcuts();
                        if let Ok(keys) = keys {
                            if *shortcut == keys[0] {
                                handle_tray_start_stop(app);
                            } else if *shortcut == keys[1] {
                                handle_tray_pause_resume(app);
                            }
                        }
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            load_file,
            preferences::get_preferences,
            preferences::set_shortcut_recording,
            preferences::set_preferences,
            set_selection,
            start_typing,
            stop_typing,
            pause_typing,
            resume_typing,
            get_config,
            set_config,
            get_state,
            set_file_content,
            get_platform_info,
            request_accessibility,
            open_accessibility_settings,
            show_widget,
            get_session_timing,
            show_main,
        ])
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .setup(move |app| {
            // Restore valid settings and the optional local draft.
            if let Ok(store) = app.store("settings.json") {
                if let Some(value) = store.get("config") {
                    if let Ok(config) = serde_json::from_value::<Config>(value) {
                        let _ = engine().set_config(config);
                    }
                }
            }
            if let Ok(store) = app.store("settings.json") {
                let prefs = store
                    .get("preferences")
                    .and_then(|v| serde_json::from_value::<Preferences>(v).ok())
                    .filter(|p| p.validate().is_ok())
                    .unwrap_or_default();
                engine().set_auto_pause(prefs.auto_pause);
                if prefs.remember_draft {
                    if let Some(draft) = store.get("draft") {
                        if let Some(content) = draft["content"].as_str() {
                            let _ = engine().set_content(
                                content.into(),
                                draft["name"].as_str().unwrap_or("Untitled").into(),
                                app.handle(),
                            );
                        }
                    }
                }
                *app.state::<PreferenceState>().0.lock() = prefs;
            }

            #[cfg(target_os = "macos")]
            {
                // The native Edit menu supplies Command-C/V/X/A and undo to WKWebView.
                app.set_menu(Menu::default(app.handle())?)?;
                show_main_window(app.handle());
            }
            ghostkeys_lib::widget::configure(app.handle())?;

            // Build tray menu
            let start_stop =
                MenuItem::with_id(app, "start_stop", "Start/Stop", true, None::<&str>)?;
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

            // Match the monochrome app mark in the menu bar.
            let icon_data = create_tray_icon();
            let icon = Image::new_owned(icon_data, 16, 16);

            // Create tray icon
            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .tooltip("ghostkeys")
                .icon_as_template(cfg!(target_os = "macos"))
                .menu(&menu)
                .show_menu_on_left_click(cfg!(target_os = "macos"))
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "start_stop" => handle_tray_start_stop(app),
                    "pause_resume" => handle_tray_pause_resume(app),
                    "show_main" => show_main_window(app),
                    "toggle_widget" => toggle_widget(app),
                    "quit" => {
                        engine().stop();
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
                        if !cfg!(target_os = "macos") {
                            toggle_widget(tray.app_handle());
                        }
                    }
                })
                .build(app)?;

            let keys = app.state::<PreferenceState>().0.lock().shortcuts()?;
            for key in keys {
                if let Err(e) = app.global_shortcut().register(key) {
                    engine()
                        .report_error(format!("Could not register shortcut: {e}"), app.handle());
                }
            }

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| match event {
            #[cfg(target_os = "macos")]
            tauri::RunEvent::Reopen { .. } => show_main_window(app),
            tauri::RunEvent::ExitRequested { .. } => engine().stop(),
            _ => {}
        });
}

/// Three key strokes, rendered as a macOS template image.
fn create_tray_icon() -> Vec<u8> {
    let mut data = Vec::with_capacity(16 * 16 * 4);
    for y in 0..16 {
        for x in 0..16 {
            let mark = ((3..=4).contains(&x) || (11..=12).contains(&x)) && (5..=10).contains(&y)
                || (7..=8).contains(&x) && (2..=13).contains(&y);
            data.extend_from_slice(if mark {
                &[0x17, 0x17, 0x17, 0xff]
            } else {
                &[0, 0, 0, 0]
            });
        }
    }
    data
}
