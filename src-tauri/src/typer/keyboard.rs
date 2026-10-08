use enigo::{Enigo, Keyboard, Settings};
use std::thread;
use std::time::Duration;

/// Wrapper around enigo for keyboard simulation
pub struct KeyboardSimulator {
    enigo: Option<Enigo>,
    #[cfg(target_os = "macos")]
    background: Option<crate::background::mac::BackgroundWriter>,
}

impl KeyboardSimulator {
    pub fn new() -> Result<Self, String> {
        crate::platform::ensure_keyboard_access()?;
        #[cfg(target_os = "macos")]
        if let Some(background) = crate::background::mac::BackgroundWriter::new()? {
            return Ok(Self { enigo: None, background: Some(background) });
        }
        let enigo = Enigo::new(&Settings::default())
            .map_err(|e| format!("Failed to create keyboard simulator: {}", e))?;
        Ok(Self { enigo: Some(enigo), #[cfg(target_os = "macos")] background: None })
    }
    
    /// Type a single character
    pub fn type_char(&mut self, c: char) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        if let Some(background) = &mut self.background { return background.type_char(c); }
        // Control characters must be real keys in browser editors.
        if matches!(c, '\n' | '\r' | '\t') {
            let key = if c == '\t' { enigo::Key::Tab } else { enigo::Key::Return };
            return self.enigo.as_mut().unwrap().key(key, enigo::Direction::Click)
                .map_err(|e| format!("Failed to press control key: {}", e));
        }
        self.enigo
            .as_mut().unwrap()
            .text(&c.to_string())
            .map_err(|e| format!("Failed to type character '{}': {}", c, e))
    }
    
    /// Type a string of characters
    pub fn type_text(&mut self, text: &str) -> Result<(), String> {
        for c in text.chars() { self.type_char(c)?; }
        Ok(())
    }
    
    /// Press backspace
    pub fn backspace(&mut self) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        if let Some(background) = &mut self.background { return background.backspace(); }
        use enigo::Key;
        self.enigo
            .as_mut().unwrap()
            .key(Key::Backspace, enigo::Direction::Click)
            .map_err(|e| format!("Failed to press backspace: {}", e))
    }
    
    /// Press backspace multiple times
    pub fn backspace_n(&mut self, n: usize, delay_ms: u64) -> Result<(), String> {
        for _ in 0..n {
            self.backspace()?;
            if delay_ms > 0 {
                thread::sleep(Duration::from_millis(delay_ms));
            }
        }
        Ok(())
    }
}

impl Default for KeyboardSimulator {
    fn default() -> Self {
        Self::new().expect("Failed to create default KeyboardSimulator")
    }
}
