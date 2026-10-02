use rand::Rng;
use rand_distr::{Distribution, Normal};

use crate::config::Config;

/// Calculate the base delay between keystrokes in milliseconds
pub fn base_delay_ms(wpm: u32) -> u64 {
    // Average word is 5 characters
    // WPM = words per minute
    // chars per minute = WPM * 5
    // chars per second = WPM * 5 / 60
    // ms per char = 60000 / (WPM * 5) = 12000 / WPM
    (12000 / wpm.max(1) as u64).max(20) // Minimum 20ms
}

/// Add gaussian variance to a delay
pub fn add_variance(delay_ms: u64, variance: f64) -> u64 {
    if variance <= 0.0 {
        return delay_ms;
    }

    let mean = delay_ms as f64;
    let std_dev = mean * variance;

    // Create normal distribution
    let normal = match Normal::new(mean, std_dev) {
        Ok(n) => n,
        Err(_) => return delay_ms,
    };

    let mut rng = rand::thread_rng();
    let varied = normal.sample(&mut rng);

    // Clamp to reasonable range (50% to 200% of original)
    let min = (delay_ms as f64 * 0.5) as u64;
    let max = (delay_ms as f64 * 2.0) as u64;

    (varied as u64).clamp(min.max(10), max)
}

/// Check if character is a word boundary
fn is_word_boundary(c: char) -> bool {
    c.is_whitespace()
        || matches!(
            c,
            '.' | ','
                | ';'
                | ':'
                | '!'
                | '?'
                | '"'
                | '\''
                | '('
                | ')'
                | '['
                | ']'
                | '{'
                | '}'
                | '-'
                | '/'
                | '\\'
        )
}

/// Get which hand types a character (simplified QWERTY layout)
/// Returns 0 for left hand, 1 for right hand, 2 for either/special
fn get_hand(c: char) -> u8 {
    match c.to_ascii_lowercase() {
        'q' | 'w' | 'e' | 'r' | 't' | 'a' | 's' | 'd' | 'f' | 'g' | 'z' | 'x' | 'c' | 'v' | 'b'
        | '1' | '2' | '3' | '4' | '5' | '`' | '~' => 0,
        'y' | 'u' | 'i' | 'o' | 'p' | 'h' | 'j' | 'k' | 'l' | 'n' | 'm' | '6' | '7' | '8' | '9'
        | '0' | '-' | '=' | '[' | ']' | '\\' | ';' | '\'' | ',' | '.' | '/' => 1,
        _ => 2,
    }
}

/// Check if two characters form a common digraph (typed faster due to muscle memory)
fn is_common_digraph(prev: char, curr: char) -> bool {
    let pair = format!("{}{}", prev.to_ascii_lowercase(), curr.to_ascii_lowercase());
    matches!(
        pair.as_str(),
        // Common English digraphs
        "th" | "he" | "in" | "er" | "an" | "re" | "on" | "at" | "en" | "nd" |
        "ti" | "es" | "or" | "te" | "of" | "ed" | "is" | "it" | "al" | "ar" |
        "st" | "to" | "nt" | "ng" | "ha" | "as" | "ou" | "io" | "le" |
        "ve" | "co" | "me" | "de" | "hi" | "ri" | "ro" | "ic" | "ne" | "ea" |
        "ra" | "ce" | "li" | "ch" | "ll" | "be" | "ma" | "si" | "om" | "ur" |
        // Common programming digraphs
        "if" | "el" | "fo" | "wh" | "tu" | "rn" | "fu" | "nc" |
        "ct" | "va" | "et" | "ue" | "tr" | "fa" |
        "ls" | "nu" | "un" | "fi" | "cl" | "ss"
    )
}

/// Calculate word position context
pub struct WordContext {
    /// Characters since last word boundary
    pub chars_in_word: usize,
    /// Is current char at word start
    pub is_word_start: bool,
    /// Is current char at word end (next is boundary)
    pub is_word_end: bool,
    /// Average word length for momentum calculation
    pub word_length_estimate: usize,
}

impl WordContext {
    pub fn analyze(chars: &[char], current_index: usize) -> Self {
        let current_char = chars.get(current_index).copied().unwrap_or(' ');
        let prev_char = if current_index > 0 {
            chars.get(current_index - 1).copied()
        } else {
            None
        };
        let next_char = chars.get(current_index + 1).copied();

        // Count chars since last word boundary
        let mut chars_in_word = 0;
        let mut i = current_index;
        while i > 0 {
            i -= 1;
            if is_word_boundary(chars[i]) {
                break;
            }
            chars_in_word += 1;
        }

        // Estimate word length by looking ahead
        let mut word_length_estimate = chars_in_word;
        let mut j = current_index;
        while j < chars.len() && !is_word_boundary(chars[j]) {
            word_length_estimate += 1;
            j += 1;
        }

        let is_word_start =
            prev_char.map(is_word_boundary).unwrap_or(true) && !is_word_boundary(current_char);
        let is_word_end =
            next_char.map(is_word_boundary).unwrap_or(true) && !is_word_boundary(current_char);

        Self {
            chars_in_word,
            is_word_start,
            is_word_end,
            word_length_estimate,
        }
    }
}

/// Calculate delay for a specific character with context (enhanced version)
pub fn calculate_delay_v2(
    config: &Config,
    chars: &[char],
    current_index: usize,
    total_chars: usize,
) -> u64 {
    let mut rng = rand::thread_rng();
    let base = base_delay_ms(config.base_wpm);

    let current_char = chars[current_index];
    let prev_char = if current_index > 0 {
        Some(chars[current_index - 1])
    } else {
        None
    };

    let word_ctx = WordContext::analyze(chars, current_index);

    // Start with base delay
    let mut delay = base as f64;

    // === WORD RHYTHM ===

    // Word start hesitation - slight pause when beginning a new word
    if word_ctx.is_word_start {
        delay *= 1.15 + rng.gen::<f64>() * 0.15; // 15-30% slower at word start
    }

    // Mid-word momentum - type faster as you flow through a word
    if word_ctx.chars_in_word > 0 && word_ctx.chars_in_word < word_ctx.word_length_estimate {
        // Build momentum: faster in the middle of words
        let word_progress =
            word_ctx.chars_in_word as f64 / word_ctx.word_length_estimate.max(1) as f64;
        // Peak speed at ~40% through the word
        let momentum = if word_progress < 0.4 {
            0.85 + 0.15 * (1.0 - word_progress / 0.4) // Getting faster
        } else {
            0.85 + 0.1 * ((word_progress - 0.4) / 0.6) // Slightly slowing
        };
        delay *= momentum;
    }

    // === DIGRAPH AND HAND PATTERNS ===

    if let Some(prev) = prev_char {
        // Common digraphs are typed faster (muscle memory)
        if is_common_digraph(prev, current_char) {
            delay *= 0.75 + rng.gen::<f64>() * 0.1; // 15-25% faster
        }

        // Hand alternation is faster than same-hand sequences
        let prev_hand = get_hand(prev);
        let curr_hand = get_hand(current_char);
        if prev_hand != curr_hand && prev_hand < 2 && curr_hand < 2 {
            delay *= 0.88 + rng.gen::<f64>() * 0.08; // 4-12% faster
        } else if prev_hand == curr_hand && prev_hand < 2 {
            // Same hand is slightly slower
            delay *= 1.05 + rng.gen::<f64>() * 0.1; // 5-15% slower
        }
    }

    // === WORD BOUNDARIES ===

    // Space after word - thinking about next word
    if current_char == ' ' {
        delay *= 1.2 + rng.gen::<f64>() * 0.3; // 20-50% slower for spaces
    }

    // After punctuation - longer pause
    if let Some(prev) = prev_char {
        if matches!(prev, '.' | '!' | '?') {
            // End of sentence - longer pause
            delay += config.punctuation_pause as f64 * (0.8 + rng.gen::<f64>() * 0.4);
        } else if matches!(prev, ',' | ';' | ':') {
            // Mid-sentence punctuation
            delay += (config.punctuation_pause as f64 * 0.5) * (0.7 + rng.gen::<f64>() * 0.6);
        }
    }

    // Newlines/paragraphs
    if current_char == '\n' {
        delay += config.paragraph_pause as f64 * (0.6 + rng.gen::<f64>() * 0.8);
    }

    // === BURST TYPING ===

    // Occasional burst of faster typing (flow state)
    if config.burst_typing && rng.gen::<f64>() < 0.08 {
        // Burst affects multiple characters, so make it subtle
        delay *= 0.6 + rng.gen::<f64>() * 0.15;
    }

    // === THINKING PAUSES ===

    // More likely at word boundaries
    let thinking_chance = if word_ctx.is_word_start {
        config.thinking_pause_chance * 1.5 // More likely to think before starting word
    } else if is_word_boundary(current_char) {
        config.thinking_pause_chance * 2.0 // Even more likely at boundaries
    } else {
        config.thinking_pause_chance * 0.3 // Rarely in middle of word
    };

    if rng.gen::<f64>() < thinking_chance {
        let thinking = add_variance(config.thinking_pause_duration, 0.4);
        delay += thinking as f64;
    }

    // === WARMUP AND FATIGUE ===

    // Warmup - slower at the very start
    let warmup_chars = 30;
    if current_index < warmup_chars {
        let warmup_factor =
            1.0 + 0.35 * (1.0 - (current_index as f64 / warmup_chars as f64)).powi(2);
        delay *= warmup_factor;
    }

    // Fatigue near the end
    let progress = current_index as f64 / total_chars.max(1) as f64;
    if progress > 0.85 {
        let fatigue_factor = 1.0 + 0.15 * ((progress - 0.85) / 0.15);
        delay *= fatigue_factor;
    }

    // === FINAL VARIANCE ===

    // Add small random variance to everything
    let final_delay = delay * (0.9 + rng.gen::<f64>() * 0.2);

    // Apply configured variance
    let with_variance = add_variance(final_delay as u64, config.wpm_variance * 0.5);

    with_variance.max(8) // Minimum 8ms between keystrokes
}

/// Legacy function for backward compatibility
pub fn calculate_delay(
    config: &Config,
    char: char,
    prev_char: Option<char>,
    chars_typed: usize,
    total_chars: usize,
) -> u64 {
    let mut rng = rand::thread_rng();
    let base = base_delay_ms(config.base_wpm);
    let mut delay = add_variance(base, config.wpm_variance);

    // Warmup period - type slower at the start
    let warmup_chars = 20;
    if chars_typed < warmup_chars {
        let warmup_factor = 1.0 + (0.4 * (1.0 - (chars_typed as f64 / warmup_chars as f64)));
        delay = (delay as f64 * warmup_factor) as u64;
    }

    // Add punctuation pause
    if let Some(prev) = prev_char {
        if matches!(prev, '.' | '!' | '?' | ',' | ';' | ':') {
            delay += config.punctuation_pause;
        }
    }

    // Add paragraph pause for newlines
    if char == '\n' || prev_char == Some('\n') {
        delay += config.paragraph_pause;
    }

    // Random thinking pause
    if rng.gen::<f64>() < config.thinking_pause_chance {
        // Add variance to thinking pause too
        let thinking = add_variance(config.thinking_pause_duration, 0.3);
        delay += thinking;
    }

    // Burst typing - occasionally type faster
    if config.burst_typing && rng.gen::<f64>() < 0.1 {
        delay = (delay as f64 * 0.5) as u64;
    }

    // Slow down near the end (fatigue simulation)
    let progress = chars_typed as f64 / total_chars.max(1) as f64;
    if progress > 0.8 {
        let fatigue_factor = 1.0 + (0.2 * (progress - 0.8) / 0.2);
        delay = (delay as f64 * fatigue_factor) as u64;
    }

    delay.max(10) // Minimum 10ms between keystrokes
}

/// Calculate delay for backspace (usually faster)
pub fn backspace_delay(config: &Config) -> u64 {
    let base = base_delay_ms(config.base_wpm);
    let faster = (base as f64 * 0.7) as u64; // 30% faster
    add_variance(faster, config.wpm_variance * 0.5).max(10)
}

/// Calculate delay before noticing a mistake
pub fn notice_mistake_delay() -> u64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(50..500)
}
