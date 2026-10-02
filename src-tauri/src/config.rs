use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub base_wpm: u32,
    pub wpm_variance: f64,
    pub mistake_rate: f64,
    pub correction_rate: f64,
    pub punctuation_pause: u64,
    pub paragraph_pause: u64,
    pub thinking_pause_chance: f64,
    pub thinking_pause_duration: u64,
    pub burst_typing: bool,
    pub countdown_seconds: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            base_wpm: 60,
            wpm_variance: 0.3,
            mistake_rate: 0.03,
            correction_rate: 0.7,
            punctuation_pause: 300,
            paragraph_pause: 800,
            thinking_pause_chance: 0.02,
            thinking_pause_duration: 1500,
            burst_typing: true,
            countdown_seconds: 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TypingStatus {
    Idle,
    Ready,
    Countdown,
    Typing,
    Paused,
    Done,
    Error,
}

impl Default for TypingStatus {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingProgress {
    pub current: u32,
    pub total: u32,
    pub percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatePayload {
    pub status: TypingStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountdownPayload {
    pub remaining: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPayload {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    pub name: String,
    pub content: String,
    pub char_count: u32,
}

impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if !(20..=200).contains(&self.base_wpm) {
            return Err("Typing speed must be between 20 and 200 WPM.".into());
        }
        if !(3..=15).contains(&self.countdown_seconds) {
            return Err("Countdown must be between 3 and 15 seconds.".into());
        }
        for (value, max, name) in [
            (self.wpm_variance, 0.5, "Speed variation"),
            (self.mistake_rate, 0.15, "Added typos"),
            (self.correction_rate, 1.0, "Typos corrected"),
            (self.thinking_pause_chance, 0.1, "Thinking pauses"),
        ] {
            if !value.is_finite() || !(0.0..=max).contains(&value) {
                return Err(format!("{name} is outside the allowed range."));
            }
        }
        if self.punctuation_pause > 1000
            || self.paragraph_pause > 3000
            || !(500..=5000).contains(&self.thinking_pause_duration)
        {
            return Err("Pause duration is outside the allowed range.".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_config_is_valid() {
        assert!(Config::default().validate().is_ok());
    }
    #[test]
    fn invalid_speed_and_countdown_are_rejected() {
        let mut config = Config::default();
        config.base_wpm = 0;
        assert!(config.validate().is_err());
        config.base_wpm = 60;
        config.countdown_seconds = 0;
        assert!(config.validate().is_err());
    }
    #[test]
    fn invalid_probabilities_and_durations_are_rejected() {
        for invalid in [f64::NAN, f64::INFINITY, -0.1, 1.1] {
            let mut config = Config::default();
            config.correction_rate = invalid;
            assert!(config.validate().is_err());
        }
        let mut config = Config::default();
        config.paragraph_pause = u64::MAX;
        assert!(config.validate().is_err());
    }
}
