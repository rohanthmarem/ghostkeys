mod control;
pub mod keyboard;
pub mod mistakes;
pub mod timing;

use crate::config::{Config, TypingProgress, TypingStatus};
use control::RunControl;
use keyboard::{KeyboardOutput, KeyboardSimulator};
use mistakes::generate_mistake;
use parking_lot::Mutex;
use rand::Rng;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

pub fn normalize_content(content: &str) -> String {
    content.replace("\r\n", "\n").replace('\r', "\n")
}

/// A run keeps its own cancellation state until its worker has fully exited.
/// Reserving `active` before spawning prevents hotkey/UI starts from overlapping.
pub struct TypingEngine {
    status: Mutex<TypingStatus>,
    config: Mutex<Config>,
    content: Mutex<Option<String>>,
    file_name: Mutex<Option<String>>,
    error_message: Mutex<Option<String>>,
    current_index: Mutex<usize>,
    active: Mutex<Option<Arc<RunControl>>>,
    resuming: AtomicBool,
    auto_pause: AtomicBool,
    target_app: Mutex<Option<i32>>,
    selection: Mutex<Option<(usize, usize)>>,
}

impl Default for TypingEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TypingEngine {
    pub fn new() -> Self {
        Self {
            status: Mutex::new(TypingStatus::Idle),
            config: Mutex::new(Config::default()),
            content: Mutex::new(None),
            file_name: Mutex::new(None),
            error_message: Mutex::new(None),
            current_index: Mutex::new(0),
            active: Mutex::new(None),
            resuming: AtomicBool::new(false),
            auto_pause: AtomicBool::new(true),
            target_app: Mutex::new(None),
            selection: Mutex::new(None),
        }
    }

    pub fn get_status(&self) -> TypingStatus {
        *self.status.lock()
    }

    pub fn set_status(&self, status: TypingStatus, app: &AppHandle) {
        *self.status.lock() = status;
        if status == TypingStatus::Countdown {
            if let Some(widget) = app.get_webview_window("widget") {
                let _ = widget.show();
            }
        }
        let _ = app.emit(
            "typing-state-changed",
            serde_json::json!({ "status": status }),
        );
    }

    pub fn get_config(&self) -> Config {
        self.config.lock().clone()
    }
    pub fn set_config(&self, config: Config) -> Result<(), String> {
        config.validate()?;
        let active = self.active.lock();
        if active.is_some() {
            return Err("Stop typing before changing settings.".into());
        }
        *self.config.lock() = config;
        Ok(())
    }

    pub fn set_content(
        &self,
        content: String,
        file_name: String,
        app: &AppHandle,
    ) -> Result<(), String> {
        let active = self.active.lock();
        if active.is_some() {
            return Err("Stop typing before changing the text.".into());
        }
        let content = normalize_content(&content);
        let status = if content.is_empty() {
            TypingStatus::Idle
        } else {
            TypingStatus::Ready
        };
        *self.content.lock() = Some(content);
        *self.selection.lock() = None;
        *self.file_name.lock() = Some(file_name);
        *self.error_message.lock() = None;
        *self.current_index.lock() = 0;
        self.emit_progress(app);
        self.set_status(status, app);
        Ok(())
    }

    pub fn set_auto_pause(&self, enabled: bool) {
        self.auto_pause.store(enabled, Ordering::SeqCst);
    }

    pub fn set_selection(&self, start: usize, end: usize, app: &AppHandle) -> Result<(), String> {
        let active = self.active.lock();
        if active.is_some() {
            return Err("Stop typing before changing the selection.".into());
        }
        let len = self.get_content().unwrap_or_default().chars().count();
        if start > end || end > len {
            return Err("Selection is outside the text.".into());
        }
        *self.selection.lock() = if start == end {
            None
        } else {
            Some((start, end))
        };
        *self.current_index.lock() = 0;
        self.emit_progress(app);
        Ok(())
    }

    fn session_chars(&self) -> Vec<char> {
        let chars: Vec<char> = self.get_content().unwrap_or_default().chars().collect();
        match *self.selection.lock() {
            Some((start, end)) => chars[start..end].to_vec(),
            None => chars,
        }
    }

    fn target_is_focused(&self) -> bool {
        let target = *self.target_app.lock();
        target.is_some() && crate::platform::focused_application() == target
    }

    fn guarded_wait(&self, control: &RunControl, duration: Duration, app: &AppHandle) -> bool {
        let mut remaining = duration;
        loop {
            if !control.wait(Duration::ZERO) {
                return false;
            }
            if cfg!(target_os = "macos")
                && self.auto_pause.load(Ordering::SeqCst)
                && !self.target_is_focused()
            {
                control.pause();
                self.set_status(TypingStatus::Paused, app);
                continue;
            }
            if remaining.is_zero() {
                return true;
            }
            let slice = remaining.min(Duration::from_millis(50));
            if !control.wait(slice) {
                return false;
            }
            remaining = remaining.saturating_sub(slice);
        }
    }

    pub fn get_content(&self) -> Option<String> {
        self.content.lock().clone()
    }
    pub fn get_file_name(&self) -> Option<String> {
        self.file_name.lock().clone()
    }
    pub fn get_error_message(&self) -> Option<String> {
        self.error_message.lock().clone()
    }

    pub fn get_progress(&self) -> TypingProgress {
        let total = match *self.selection.lock() {
            Some((start, end)) => end - start,
            None => self
                .content
                .lock()
                .as_ref()
                .map(|c| c.chars().count())
                .unwrap_or(0),
        } as u32;
        let current = (*self.current_index.lock() as u32).min(total);
        TypingProgress {
            current,
            total,
            percent: if total > 0 {
                current as f32 / total as f32 * 100.0
            } else {
                0.0
            },
        }
    }

    fn emit_progress(&self, app: &AppHandle) {
        let _ = app.emit("typing-progress", self.get_progress());
    }

    pub fn report_error(&self, error: String, app: &AppHandle) {
        *self.error_message.lock() = Some(error.clone());
        self.set_status(TypingStatus::Error, app);
        let _ = app.emit("typing-error", serde_json::json!({ "message": error }));
    }

    pub fn stop(&self) {
        if let Some(control) = self.active.lock().as_ref() {
            control.stop();
        }
    }

    pub fn is_running(&self) -> bool {
        self.active.lock().is_some()
    }

    pub fn pause(&self, app: &AppHandle) {
        if let Some(control) = self.active.lock().as_ref() {
            if self.get_status() == TypingStatus::Typing && !control.is_stopped() {
                control.pause();
                self.set_status(TypingStatus::Paused, app);
            }
        }
    }

    pub fn resume(self: &Arc<Self>, app: &AppHandle) -> Result<(), String> {
        if let Err(error) = crate::platform::ensure_accessibility() {
            self.stop();
            return Err(error);
        }
        let active = self.active.lock();
        let Some(control) = active.as_ref().cloned() else {
            return Ok(());
        };
        if !control.is_paused()
            || control.is_stopped()
            || self.resuming.swap(true, Ordering::SeqCst)
        {
            return Ok(());
        }
        // Clicking Resume can focus ghostkeys. Give the user time to return to
        // the destination before releasing the paused keyboard worker.
        let seconds = self.get_config().countdown_seconds.max(3);
        self.set_status(TypingStatus::Countdown, app);
        let app = app.clone();
        let engine = self.clone();
        tauri::async_runtime::spawn(async move {
            for remaining in (1..=seconds).rev() {
                if control.is_stopped() {
                    return;
                }
                let _ = app.emit(
                    "countdown-tick",
                    serde_json::json!({ "remaining": remaining }),
                );
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            let active = engine.active.lock();
            if active
                .as_ref()
                .is_some_and(|current| Arc::ptr_eq(current, &control))
                && !control.is_stopped()
            {
                if cfg!(target_os = "macos")
                    && engine.auto_pause.load(Ordering::SeqCst)
                    && !engine.target_is_focused()
                {
                    engine.set_status(TypingStatus::Paused, &app);
                    let _ = app.emit(
                        "session-notice",
                        "Return to the destination app, then resume.",
                    );
                } else {
                    engine.set_status(TypingStatus::Typing, &app);
                    control.resume();
                }
                engine.resuming.store(false, Ordering::SeqCst);
            }
        });
        Ok(())
    }

    pub fn start(self: &Arc<Self>, app: AppHandle) -> Result<(), String> {
        let mut active = self.active.lock();
        if active.is_some() {
            return Err("A typing session is already running or stopping.".into());
        }
        crate::platform::ensure_accessibility()?;
        let chars = self.session_chars();
        if chars.is_empty() {
            return Err("Content is empty.".into());
        }
        let config = self.get_config();
        if config.base_wpm == 0 {
            return Err("Typing speed must be greater than zero.".into());
        }
        let control = Arc::new(RunControl::new());
        *active = Some(control.clone());
        self.resuming.store(false, Ordering::SeqCst);
        *self.current_index.lock() = 0;
        *self.error_message.lock() = None;
        self.emit_progress(&app);
        self.set_status(TypingStatus::Countdown, &app);
        let engine = self.clone();
        tauri::async_runtime::spawn(async move {
            let worker_engine = engine.clone();
            let worker_app = app.clone();
            let worker_control = control.clone();
            let result = tauri::async_runtime::spawn_blocking(move || {
                worker_engine.run_worker(&worker_app, &worker_control, &chars, &config)
            })
            .await
            .map_err(|e| format!("Typing task failed: {e}"))
            .and_then(|result| result);

            let mut active = engine.active.lock();
            if control.is_stopped() {
                *engine.current_index.lock() = 0;
                engine.emit_progress(&app);
                engine.set_status(TypingStatus::Ready, &app);
            } else if let Err(error) = result {
                engine.report_error(error, &app);
            } else {
                engine.set_status(TypingStatus::Done, &app);
            }
            *active = None;
        });
        Ok(())
    }

    fn run_worker(
        &self,
        app: &AppHandle,
        control: &RunControl,
        chars: &[char],
        config: &Config,
    ) -> Result<(), String> {
        for remaining in (1..=config.countdown_seconds).rev() {
            if control.is_stopped() {
                return Ok(());
            }
            let _ = app.emit(
                "countdown-tick",
                serde_json::json!({ "remaining": remaining }),
            );
            if !control.wait(Duration::from_secs(1)) {
                return Ok(());
            }
        }
        if !control.wait(Duration::ZERO) {
            return Ok(());
        }
        if cfg!(target_os = "macos") && self.auto_pause.load(Ordering::SeqCst) {
            let target = crate::platform::focused_application()
                .filter(|pid| *pid != std::process::id() as i32)
                .ok_or("Click a text field in another app before the countdown ends.")?;
            *self.target_app.lock() = Some(target);
        }
        let mut keyboard = KeyboardSimulator::new()?;
        self.set_status(TypingStatus::Typing, app);
        type_sequence(
            &mut keyboard,
            chars,
            config,
            |duration| self.guarded_wait(control, duration, app),
            |i| {
                *self.current_index.lock() = i;
                self.emit_progress(app);
            },
        )
    }
}

/// Shared production/test loop. Every physical key is preceded by a cancellation
/// checkpoint; injected outputs let us verify corrections without desktop access.
fn type_sequence(
    keyboard: &mut impl KeyboardOutput,
    chars: &[char],
    config: &Config,
    mut wait: impl FnMut(Duration) -> bool,
    mut progress: impl FnMut(usize),
) -> Result<(), String> {
    let mut rng = rand::thread_rng();
    let mut i = 0;
    while i < chars.len() {
        if !wait(Duration::ZERO) {
            return Ok(());
        }
        let delay = timing::calculate_delay_v2(config, chars, i, chars.len());
        let mistake = generate_mistake(chars[i], chars.get(i + 1).copied(), config.mistake_rate);
        for c in &mistake.chars_to_type {
            if !wait(Duration::ZERO) {
                return Ok(());
            }
            keyboard.type_char(*c)?;
            if !wait(Duration::from_millis(delay)) {
                return Ok(());
            }
        }
        if mistake.mistake_made && rng.gen::<f64>() < config.correction_rate {
            if !wait(Duration::from_millis(timing::notice_mistake_delay())) {
                return Ok(());
            }
            for _ in &mistake.chars_to_type {
                if !wait(Duration::ZERO) {
                    return Ok(());
                }
                keyboard.backspace()?;
                if !wait(Duration::from_millis(timing::backspace_delay(config))) {
                    return Ok(());
                }
            }
            for c in chars.iter().skip(i).take(mistake.chars_consumed) {
                if !wait(Duration::ZERO) {
                    return Ok(());
                }
                keyboard.type_char(*c)?;
                if !wait(Duration::from_millis(delay)) {
                    return Ok(());
                }
            }
        }
        i += mistake.chars_consumed;
        progress(i);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Default)]
    struct Buffer(String);
    impl KeyboardOutput for Buffer {
        fn type_char(&mut self, c: char) -> Result<(), String> {
            self.0.push(c);
            Ok(())
        }
        fn backspace(&mut self) -> Result<(), String> {
            self.0.pop();
            Ok(())
        }
    }

    #[test]
    fn clean_output_preserves_unicode_tabs_and_newlines() {
        let chars: Vec<char> = "café 👻\n\tHello!".chars().collect();
        let config = Config {
            mistake_rate: 0.0,
            ..Config::default()
        };
        let mut output = Buffer::default();
        let mut progress = 0;
        let mut timed_waits = 0;
        type_sequence(
            &mut output,
            &chars,
            &config,
            |duration| {
                if !duration.is_zero() {
                    timed_waits += 1;
                }
                true
            },
            |i| progress = i,
        )
        .unwrap();
        assert_eq!(output.0, chars.iter().collect::<String>());
        assert_eq!(progress, chars.len());
        assert_eq!(
            timed_waits,
            chars.len(),
            "one pacing delay per key, not two"
        );
    }

    #[test]
    fn corrected_mistakes_reproduce_source_text() {
        let chars: Vec<char> = "The quick brown fox.\nA new line\tand a tab."
            .chars()
            .collect();
        let config = Config {
            mistake_rate: 1.0,
            correction_rate: 1.0,
            ..Config::default()
        };
        for _ in 0..100 {
            let mut output = Buffer::default();
            type_sequence(&mut output, &chars, &config, |_| true, |_| {}).unwrap();
            assert_eq!(output.0, chars.iter().collect::<String>());
        }
    }

    #[test]
    fn cancellation_during_a_delay_prevents_further_keys() {
        let chars: Vec<char> = "stop after one".chars().collect();
        let config = Config {
            mistake_rate: 0.0,
            ..Config::default()
        };
        let mut output = Buffer::default();
        type_sequence(
            &mut output,
            &chars,
            &config,
            |duration| duration.is_zero(),
            |_| {},
        )
        .unwrap();
        assert_eq!(output.0, "s");
    }

    #[test]
    fn active_sessions_reject_config_edits() {
        let engine = TypingEngine::new();
        *engine.active.lock() = Some(Arc::new(RunControl::new()));
        let config = Config {
            base_wpm: 100,
            ..Config::default()
        };
        assert!(engine.set_config(config).is_err());
        assert_eq!(engine.get_config().base_wpm, 60);
    }

    #[test]
    fn selection_uses_unicode_characters_and_preserves_draft() {
        let engine = TypingEngine::new();
        *engine.content.lock() = Some("a👻café z".into());
        *engine.selection.lock() = Some((1, 6));
        assert_eq!(engine.session_chars().iter().collect::<String>(), "👻café");
        assert_eq!(engine.get_progress().total, 5);
        assert_eq!(engine.get_content().unwrap(), "a👻café z");
    }

    #[test]
    fn normalizes_windows_and_classic_mac_line_endings() {
        assert_eq!(normalize_content("a\r\nb\rc\n"), "a\nb\nc\n");
    }

    #[test]
    fn progress_counts_unicode_characters_instead_of_utf8_bytes() {
        let engine = TypingEngine::new();
        *engine.content.lock() = Some("aé👻\n".into());
        *engine.current_index.lock() = 4;
        let progress = engine.get_progress();
        assert_eq!(progress.total, 4);
        assert_eq!(progress.current, 4);
        assert_eq!(progress.percent, 100.0);
    }
}
