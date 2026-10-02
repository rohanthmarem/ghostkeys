use once_cell::sync::Lazy;
use std::collections::HashMap;

/// QWERTY keyboard neighbor map for adjacent key mistakes
pub static NEIGHBORS: Lazy<HashMap<char, Vec<char>>> = Lazy::new(|| {
    let mut map = HashMap::new();

    // Top row
    map.insert('q', vec!['w', 'a']);
    map.insert('w', vec!['q', 'e', 'a', 's']);
    map.insert('e', vec!['w', 'r', 's', 'd']);
    map.insert('r', vec!['e', 't', 'd', 'f']);
    map.insert('t', vec!['r', 'y', 'f', 'g']);
    map.insert('y', vec!['t', 'u', 'g', 'h']);
    map.insert('u', vec!['y', 'i', 'h', 'j']);
    map.insert('i', vec!['u', 'o', 'j', 'k']);
    map.insert('o', vec!['i', 'p', 'k', 'l']);
    map.insert('p', vec!['o', 'l', '[']);

    // Middle row
    map.insert('a', vec!['q', 'w', 's', 'z']);
    map.insert('s', vec!['a', 'w', 'e', 'd', 'z', 'x']);
    map.insert('d', vec!['s', 'e', 'r', 'f', 'x', 'c']);
    map.insert('f', vec!['d', 'r', 't', 'g', 'c', 'v']);
    map.insert('g', vec!['f', 't', 'y', 'h', 'v', 'b']);
    map.insert('h', vec!['g', 'y', 'u', 'j', 'b', 'n']);
    map.insert('j', vec!['h', 'u', 'i', 'k', 'n', 'm']);
    map.insert('k', vec!['j', 'i', 'o', 'l', 'm', ',']);
    map.insert('l', vec!['k', 'o', 'p', ';', ',', '.']);

    // Bottom row
    map.insert('z', vec!['a', 's', 'x']);
    map.insert('x', vec!['z', 's', 'd', 'c']);
    map.insert('c', vec!['x', 'd', 'f', 'v']);
    map.insert('v', vec!['c', 'f', 'g', 'b']);
    map.insert('b', vec!['v', 'g', 'h', 'n']);
    map.insert('n', vec!['b', 'h', 'j', 'm']);
    map.insert('m', vec!['n', 'j', 'k', ',']);

    // Numbers
    map.insert('1', vec!['2', 'q']);
    map.insert('2', vec!['1', '3', 'q', 'w']);
    map.insert('3', vec!['2', '4', 'w', 'e']);
    map.insert('4', vec!['3', '5', 'e', 'r']);
    map.insert('5', vec!['4', '6', 'r', 't']);
    map.insert('6', vec!['5', '7', 't', 'y']);
    map.insert('7', vec!['6', '8', 'y', 'u']);
    map.insert('8', vec!['7', '9', 'u', 'i']);
    map.insert('9', vec!['8', '0', 'i', 'o']);
    map.insert('0', vec!['9', '-', 'o', 'p']);

    map
});

/// Mistake types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MistakeType {
    /// Type an adjacent key instead
    AdjacentKey,
    /// Swap two consecutive characters
    Transposition,
    /// Skip a character (omission)
    Omission,
    /// Type a character twice
    DoubleTap,
    /// Wrong capitalization
    Capitalization,
}

impl MistakeType {
    /// Get a random mistake type with weighted probabilities
    pub fn random() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let roll: f64 = rng.gen();

        // Weights: adjacent 40%, transposition 20%, omission 15%, double-tap 15%, caps 10%
        if roll < 0.40 {
            MistakeType::AdjacentKey
        } else if roll < 0.60 {
            MistakeType::Transposition
        } else if roll < 0.75 {
            MistakeType::Omission
        } else if roll < 0.90 {
            MistakeType::DoubleTap
        } else {
            MistakeType::Capitalization
        }
    }
}

/// Result of generating a mistake
#[derive(Debug, Clone)]
pub struct MistakeResult {
    /// Characters to type (might be wrong or modified)
    pub chars_to_type: Vec<char>,
    /// Whether a mistake was actually made
    pub mistake_made: bool,
    /// How many characters were "consumed" from input
    pub chars_consumed: usize,
    /// Type of mistake made (if any)
    pub mistake_type: Option<MistakeType>,
}

/// Generate a potential mistake for typing
pub fn generate_mistake(
    current_char: char,
    next_char: Option<char>,
    mistake_rate: f64,
) -> MistakeResult {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    // Check if we should make a mistake
    // Keep composed Unicode intact: backspace removes graphemes, not scalar values.
    if !current_char.is_ascii() || current_char.is_control() || rng.gen::<f64>() >= mistake_rate {
        return MistakeResult {
            chars_to_type: vec![current_char],
            mistake_made: false,
            chars_consumed: 1,
            mistake_type: None,
        };
    }

    let mistake_type = MistakeType::random();

    match mistake_type {
        MistakeType::AdjacentKey => {
            if let Some(wrong_char) = get_adjacent_key(current_char) {
                MistakeResult {
                    chars_to_type: vec![wrong_char],
                    mistake_made: true,
                    chars_consumed: 1,
                    mistake_type: Some(MistakeType::AdjacentKey),
                }
            } else {
                // No adjacent key found, type correctly
                MistakeResult {
                    chars_to_type: vec![current_char],
                    mistake_made: false,
                    chars_consumed: 1,
                    mistake_type: None,
                }
            }
        }

        MistakeType::Transposition => {
            if let Some(next) = next_char.filter(|c| c.is_ascii() && !c.is_control()) {
                // Swap current and next
                MistakeResult {
                    chars_to_type: vec![next, current_char],
                    mistake_made: true,
                    chars_consumed: 2,
                    mistake_type: Some(MistakeType::Transposition),
                }
            } else {
                MistakeResult {
                    chars_to_type: vec![current_char],
                    mistake_made: false,
                    chars_consumed: 1,
                    mistake_type: None,
                }
            }
        }

        MistakeType::Omission => {
            // Skip this character entirely
            MistakeResult {
                chars_to_type: vec![],
                mistake_made: true,
                chars_consumed: 1,
                mistake_type: Some(MistakeType::Omission),
            }
        }

        MistakeType::DoubleTap => {
            // Type character twice
            MistakeResult {
                chars_to_type: vec![current_char, current_char],
                mistake_made: true,
                chars_consumed: 1,
                mistake_type: Some(MistakeType::DoubleTap),
            }
        }

        MistakeType::Capitalization => {
            let wrong_char = if current_char.is_uppercase() {
                current_char.to_lowercase().next().unwrap_or(current_char)
            } else if current_char.is_lowercase() {
                current_char.to_uppercase().next().unwrap_or(current_char)
            } else {
                current_char
            };

            if wrong_char != current_char {
                MistakeResult {
                    chars_to_type: vec![wrong_char],
                    mistake_made: true,
                    chars_consumed: 1,
                    mistake_type: Some(MistakeType::Capitalization),
                }
            } else {
                MistakeResult {
                    chars_to_type: vec![current_char],
                    mistake_made: false,
                    chars_consumed: 1,
                    mistake_type: None,
                }
            }
        }
    }
}

/// Get a random adjacent key for the given character
fn get_adjacent_key(c: char) -> Option<char> {
    use rand::seq::SliceRandom;

    let lower = c.to_lowercase().next()?;
    let neighbors = NEIGHBORS.get(&lower)?;
    let mut rng = rand::thread_rng();
    let neighbor = neighbors.choose(&mut rng)?;

    // Preserve case
    if c.is_uppercase() {
        Some(neighbor.to_uppercase().next().unwrap_or(*neighbor))
    } else {
        Some(*neighbor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unicode_is_never_split_or_transposed_for_mistakes() {
        for _ in 0..100 {
            for c in ['é', '👻', '\u{0301}', '\u{200d}'] {
                let result = generate_mistake(c, Some('a'), 1.0);
                assert_eq!(result.chars_to_type, vec![c]);
                assert!(!result.mistake_made);
                let result = generate_mistake('a', Some(c), 1.0);
                assert_eq!(result.chars_consumed, 1);
            }
        }
    }
}
