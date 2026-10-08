pub mod config;
pub mod background;
pub mod platform;
pub mod typer;

use once_cell::sync::Lazy;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use typer::TypingEngine;

// Re-export types for use in main.rs
pub use config::{Config, FileInfo, TypingStatus};

/// Global typing engine instance
static ENGINE: Lazy<Arc<TypingEngine>> = Lazy::new(|| Arc::new(TypingEngine::new()));

/// Get the typing engine
pub fn engine() -> &'static Arc<TypingEngine> {
    &ENGINE
}

// ============================================================================
// Tray Menu Handlers
// ============================================================================

pub fn handle_tray_start_stop(app: &AppHandle) {
    let status = engine().get_status();
    match status {
        TypingStatus::Typing | TypingStatus::Countdown | TypingStatus::Paused => {
            engine().stop();
            engine().set_status(TypingStatus::Ready, app);
        }
        TypingStatus::Ready | TypingStatus::Done => {
            if engine().is_running() || engine().get_progress().total == 0 { return; }
            if let Err(message) = crate::platform::ensure_keyboard_access() {
                use tauri::Emitter;
                let _ = app.emit("typing-error", serde_json::json!({ "message": message }));
                return;
            }
            engine().set_status(TypingStatus::Countdown, app);
            let app = app.clone();
            let engine = engine().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = engine.run(app.clone()).await {
                    eprintln!("Typing error: {}", e);
                }
            });
        }
        _ => {}
    }
}

pub fn handle_tray_pause_resume(app: &AppHandle) {
    let status = engine().get_status();
    match status {
        TypingStatus::Typing => {
            engine().pause();
            engine().set_status(TypingStatus::Paused, app);
        }
        TypingStatus::Paused => {
            engine().resume();
            engine().set_status(TypingStatus::Typing, app);
        }
        _ => {}
    }
}

pub fn toggle_widget(app: &AppHandle) {
    if let Some(widget) = app.get_webview_window("widget") {
        if widget.is_visible().unwrap_or(false) {
            let _ = widget.hide();
        } else {
            let _ = widget.show();
            let _ = widget.set_focus();
        }
    }
}

pub fn show_main_window(app: &AppHandle) {
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.show();
        let _ = main.set_focus();
    }
}
