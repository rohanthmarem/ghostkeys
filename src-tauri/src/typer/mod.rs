pub mod keyboard;
pub mod mistakes;
pub mod timing;

use crate::config::{Config, TypingProgress, TypingStatus};
use keyboard::KeyboardSimulator;
use mistakes::generate_mistake;
use parking_lot::Mutex;
use rand::Rng;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::watch;
use tokio::time::sleep;

/// Shared state for the typing engine
pub struct TypingEngine {
    /// Current status
    status: Mutex<TypingStatus>,
    /// Current configuration
    config: Mutex<Config>,
    /// Content to type
    content: Mutex<Option<String>>,
    /// File name
    file_name: Mutex<Option<String>>,
    /// Current character index
    current_index: Mutex<usize>,
    /// Stop signal
    stop_signal: AtomicBool,
    /// Pause signal
    pause_signal: AtomicBool,
    running: AtomicBool,
    /// Pause watcher sender
    pause_tx: Mutex<Option<watch::Sender<bool>>>,
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
            current_index: Mutex::new(0),
            stop_signal: AtomicBool::new(false),
            pause_signal: AtomicBool::new(false),
            running: AtomicBool::new(false),
            pause_tx: Mutex::new(None),
        }
    }

    pub fn get_status(&self) -> TypingStatus {
        *self.status.lock()
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn set_status(&self, status: TypingStatus, app: &AppHandle) {
        *self.status.lock() = status;
        let _ = app.emit("typing-state-changed", serde_json::json!({ "status": status }));
    }

    pub fn get_config(&self) -> Config {
        self.config.lock().clone()
    }

    pub fn set_config(&self, config: Config) {
        *self.config.lock() = config;
    }

    pub fn set_content(&self, content: String, file_name: String) {
        *self.content.lock() = Some(content);
        *self.file_name.lock() = Some(file_name);
        *self.current_index.lock() = 0;
    }

    pub fn get_file_name(&self) -> Option<String> {
        self.file_name.lock().clone()
    }

    pub fn get_progress(&self) -> TypingProgress {
        let content = self.content.lock();
        let total = content.as_ref().map(|c| c.chars().count()).unwrap_or(0) as u32;
        let current = *self.current_index.lock() as u32;
        let percent = if total > 0 {
            (current as f32 / total as f32) * 100.0
        } else {
            0.0
        };
        TypingProgress {
            current,
            total,
            percent,
        }
    }

    pub fn stop(&self) {
        self.stop_signal.store(true, Ordering::SeqCst);
        // Also unpause if paused so the loop can exit
        if self.pause_signal.load(Ordering::SeqCst) {
            self.pause_signal.store(false, Ordering::SeqCst);
            if let Some(tx) = self.pause_tx.lock().as_ref() {
                let _ = tx.send(false);
            }
        }
    }

    pub fn pause(&self) {
        self.pause_signal.store(true, Ordering::SeqCst);
    }

    pub fn resume(&self) {
        self.pause_signal.store(false, Ordering::SeqCst);
        if let Some(tx) = self.pause_tx.lock().as_ref() {
            let _ = tx.send(false);
        }
    }

    pub fn is_stopped(&self) -> bool {
        self.stop_signal.load(Ordering::SeqCst)
    }

    pub fn is_paused(&self) -> bool {
        self.pause_signal.load(Ordering::SeqCst)
    }

    /// Run the typing simulation
    pub async fn run(self: Arc<Self>, app: AppHandle) -> Result<(), String> {
        if self.running.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
            return Err("Wait for the current typing session to stop".into());
        }
        struct RunningSession(Arc<TypingEngine>);
        impl Drop for RunningSession {
            fn drop(&mut self) { self.0.running.store(false, Ordering::SeqCst); }
        }
        let _session = RunningSession(self.clone());
        // Reset signals
        self.stop_signal.store(false, Ordering::SeqCst);
        self.pause_signal.store(false, Ordering::SeqCst);
        *self.current_index.lock() = 0;

        // Create pause watcher
        let (pause_tx, _pause_rx) = watch::channel(false);
        *self.pause_tx.lock() = Some(pause_tx);

        // Get content
        let content = {
            let guard = self.content.lock();
            guard.clone().ok_or("No content to type")?
        };
        let chars: Vec<char> = content.chars().collect();
        let total_chars = chars.len();

        if total_chars == 0 {
            return Err("Content is empty".to_string());
        }

        // Countdown
        let countdown_secs = self.config.lock().countdown_seconds;
        self.set_status(TypingStatus::Countdown, &app);

        for i in (1..=countdown_secs).rev() {
            if self.is_stopped() {
                self.set_status(TypingStatus::Ready, &app);
                return Ok(());
            }
            let _ = app.emit("countdown-tick", serde_json::json!({ "remaining": i }));
            sleep(Duration::from_secs(1)).await;
        }

        // Start typing
        self.set_status(TypingStatus::Typing, &app);

        // Create keyboard simulator in a blocking context
        let engine = self.clone();
        let app_clone = app.clone();
        
        let result = tokio::task::spawn_blocking(move || {
            let mut keyboard = KeyboardSimulator::new()?;
            let mut config = engine.config.lock().clone();
            if !crate::background::get_background_target().is_null() { config.mistake_rate = 0.0; }
            let mut rng = rand::thread_rng();
            
            let mut i = 0;
            
            while i < total_chars {
                // Check stop signal
                if engine.stop_signal.load(Ordering::SeqCst) {
                    return Ok::<(), String>(());
                }
                
                // Check pause signal
                while engine.pause_signal.load(Ordering::SeqCst) {
                    std::thread::sleep(Duration::from_millis(100));
                    if engine.stop_signal.load(Ordering::SeqCst) {
                        return Ok(());
                    }
                }
                
                let current_char = chars[i];
                let next_char = chars.get(i + 1).copied();

                // Calculate delay using enhanced word-aware timing
                let delay = timing::calculate_delay_v2(
                    &config,
                    &chars,
                    i,
                    total_chars,
                );
                
                // Maybe generate a mistake
                let mistake_result = generate_mistake(
                    current_char,
                    next_char,
                    config.mistake_rate,
                );
                
                // Type the character(s)
                for c in &mistake_result.chars_to_type {
                    keyboard.type_char(*c)?;
                    std::thread::sleep(Duration::from_millis(delay / 2));
                }
                
                // If a mistake was made, maybe correct it
                if mistake_result.mistake_made {
                    let should_correct = rng.gen::<f64>() < config.correction_rate;
                    
                    if should_correct {
                        // Wait before noticing mistake
                        std::thread::sleep(Duration::from_millis(timing::notice_mistake_delay()));
                        
                        // Backspace to remove wrong chars
                        let backspace_delay = timing::backspace_delay(&config);
                        keyboard.backspace_n(mistake_result.chars_to_type.len(), backspace_delay)?;
                        
                        // Type correctly
                        for j in 0..mistake_result.chars_consumed {
                            if i + j < total_chars {
                                keyboard.type_char(chars[i + j])?;
                                std::thread::sleep(Duration::from_millis(delay));
                            }
                        }
                    }
                }
                
                // Move forward
                i += mistake_result.chars_consumed;
                
                // Update progress
                *engine.current_index.lock() = i;
                let progress = engine.get_progress();
                let _ = app_clone.emit("typing-progress", progress);
                
                // Wait
                std::thread::sleep(Duration::from_millis(delay));
            }
            
            Ok(())
        })
        .await
        .map_err(|e| format!("Typing task failed: {}", e))?;

        if let Err(e) = result {
            self.set_status(TypingStatus::Error, &app);
            let _ = app.emit("typing-error", serde_json::json!({ "message": e }));
            return Err(e);
        }

        // Done
        if !self.is_stopped() {
            self.set_status(TypingStatus::Done, &app);
        } else {
            self.set_status(TypingStatus::Ready, &app);
        }

        Ok(())
    }
}
