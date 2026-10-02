pub mod config;
pub mod platform;
pub mod preferences;
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
    if engine().is_running() {
        engine().stop();
        return;
    }
    let status = engine().get_status();
    match status {
        TypingStatus::Typing | TypingStatus::Countdown | TypingStatus::Paused => {
            engine().stop();
        }
        TypingStatus::Ready | TypingStatus::Done | TypingStatus::Error => {
            if let Err(error) = engine().start(app.clone()) {
                engine().report_error(error, app);
                show_main_window(app);
            }
        }
        _ => {}
    }
}

pub fn handle_tray_pause_resume(app: &AppHandle) {
    let status = engine().get_status();
    match status {
        TypingStatus::Typing => {
            engine().pause(app);
        }
        TypingStatus::Paused => {
            if let Err(error) = engine().resume(app) {
                engine().report_error(error, app);
            }
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
        }
    }
}

pub fn show_main_window(app: &AppHandle) {
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.show();
        let _ = main.set_focus();
    }
}
