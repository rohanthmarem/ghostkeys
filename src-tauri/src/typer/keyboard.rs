use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::thread;
use std::time::Duration;

/// Wrapper around enigo for keyboard simulation
pub struct KeyboardSimulator {
    enigo: Enigo,
}

impl KeyboardSimulator {
    pub fn new() -> Result<Self, String> {
        crate::platform::ensure_accessibility()?;
        let enigo = Enigo::new(&Settings::default())
            .map_err(|e| format!("Failed to create keyboard simulator: {}", e))?;
        Ok(Self { enigo })
    }

    /// Type a single character
    pub fn type_char(&mut self, c: char) -> Result<(), String> {
        // CoreGraphics text events do not reliably represent Return and Tab.
        // In particular enigo's newline workaround inserts a zero-width space.
        if let Some(key) = control_key(c) {
            return self
                .enigo
                .key(key, Direction::Click)
                .map_err(|e| format!("Failed to type control key: {e}"));
        }
        self.enigo
            .text(&c.to_string())
            .map_err(|e| format!("Failed to type character '{}': {}", c, e))
    }

    /// Type a string of characters
    pub fn type_text(&mut self, text: &str) -> Result<(), String> {
        for c in super::normalize_content(text).chars() {
            self.type_char(c)?;
        }
        Ok(())
    }

    /// Press backspace
    pub fn backspace(&mut self) -> Result<(), String> {
        use enigo::Key;
        self.enigo
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

fn control_key(c: char) -> Option<Key> {
    match c {
        '\n' | '\r' => Some(Key::Return),
        '\t' => Some(Key::Tab),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_characters_use_real_keys_and_unicode_uses_text() {
        assert_eq!(control_key('\n'), Some(Key::Return));
        assert_eq!(control_key('\t'), Some(Key::Tab));
        for c in ['a', 'é', '👻', ' '] {
            assert_eq!(control_key(c), None);
        }
    }
}

impl Default for KeyboardSimulator {
    fn default() -> Self {
        Self::new().expect("Failed to create default KeyboardSimulator")
    }
}

/// Operations used by the typing engine, independently of the OS event backend.
pub(crate) trait KeyboardOutput {
    fn type_char(&mut self, c: char) -> Result<(), String>;
    fn backspace(&mut self) -> Result<(), String>;
}

impl KeyboardOutput for KeyboardSimulator {
    fn type_char(&mut self, c: char) -> Result<(), String> {
        KeyboardSimulator::type_char(self, c)
    }
    fn backspace(&mut self) -> Result<(), String> {
        KeyboardSimulator::backspace(self)
    }
}
