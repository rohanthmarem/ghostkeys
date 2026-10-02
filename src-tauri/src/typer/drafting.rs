use crate::config::{Config, DraftingMode};
use rand::Rng;

/// A local rhythm model, not a semantic author: phrase momentum persists across
/// keys, and all tentative text is erased before the source text is emitted.
pub(super) struct DraftingRhythm {
    mode: DraftingMode,
    words_left: usize,
    words_since_revision: usize,
    pace: f64,
}

pub(super) struct Step {
    pub pause_ms: u64,
    pub key_delay_ms: u64,
    pub tentative: String,
    pub review_ms: u64,
}

impl DraftingRhythm {
    pub fn new(mode: DraftingMode) -> Self {
        Self {
            mode,
            words_left: 0,
            words_since_revision: 0,
            pace: 1.0,
        }
    }

    pub fn step(&mut self, chars: &[char], i: usize, config: &Config, rng: &mut impl Rng) -> Step {
        let slow = self.mode == DraftingMode::Slow;
        let word_start = !chars[i].is_whitespace() && (i == 0 || chars[i - 1].is_whitespace());
        let mut pause_ms = 0;
        let mut tentative = String::new();
        let mut review_ms = 0;
        if word_start {
            let previous = chars[..i].iter().rposition(|c| !c.is_whitespace());
            let gap_start = previous.map_or(0, |p| p + 1);
            let paragraph = chars[gap_start..i].contains(&'\n');
            let punctuation = previous.map(|p| {
                // Closing quotes/brackets don't hide the sentence boundary.
                let mut end = p;
                while end > 0 && matches!(chars[end], '"' | '\'' | '”' | '’' | ')' | ']') {
                    end -= 1;
                }
                chars[end]
            });
            let sentence =
                punctuation.is_some_and(|c| matches!(c, '.' | '!' | '?' | '。' | '！' | '？'));
            let clause = punctuation.is_some_and(|c| matches!(c, ',' | ';' | ':' | '—'));
            if paragraph {
                pause_ms = if slow {
                    rng.gen_range(5000..=14000)
                } else {
                    rng.gen_range(2000..=6000)
                };
            } else if i == 0 || sentence {
                pause_ms = if slow {
                    rng.gen_range(2200..=6500)
                } else {
                    rng.gen_range(700..=2400)
                };
            } else if self.words_left == 0 {
                pause_ms = if slow {
                    rng.gen_range(2500..=9000)
                } else {
                    rng.gen_range(900..=3500)
                };
            } else if clause && rng.gen_bool(if slow { 0.65 } else { 0.4 }) {
                pause_ms = if slow {
                    rng.gen_range(1000..=3200)
                } else {
                    rng.gen_range(350..=1200)
                };
            }
            if self.words_left == 0 || sentence || paragraph {
                self.words_left = if slow {
                    rng.gen_range(3..=7)
                } else {
                    rng.gen_range(6..=12)
                };
                self.pace = if config.burst_typing {
                    rng.gen_range(0.72..=1.12)
                } else {
                    1.0
                };
            }
            self.words_left -= 1;
            self.words_since_revision += 1;
            // Cooldown avoids constant editing. Only newly emitted plain ASCII
            // is erased; never cursor back into the user's existing document.
            if self.words_since_revision >= 5 && rng.gen_bool(if slow { 0.18 } else { 0.09 }) {
                tentative = tentative_word(&chars[i..], rng);
                if !tentative.is_empty() {
                    self.words_since_revision = 0;
                    review_ms = if slow {
                        rng.gen_range(1200..=3800)
                    } else {
                        rng.gen_range(450..=1500)
                    };
                }
            }
        }
        let jitter = rng.gen_range(-1.0..=1.0) * config.wpm_variance;
        let hesitation = if word_start { 1.2 } else { 1.0 };
        let key_delay_ms = (super::timing::base_delay_ms(config.base_wpm) as f64
            * self.pace
            * (1.0 + jitter)
            * hesitation)
            .max(20.0) as u64;
        Step {
            pause_ms,
            key_delay_ms,
            tentative,
            review_ms,
        }
    }
}

fn tentative_word(chars: &[char], rng: &mut impl Rng) -> String {
    let len = chars.iter().take_while(|c| c.is_ascii_alphabetic()).count();
    if !(5..=18).contains(&len)
        || chars.get(len).is_some_and(|c| {
            !c.is_ascii_whitespace() && !matches!(c, '.' | ',' | ';' | ':' | '!' | '?')
        })
    {
        return String::new();
    }
    let word: String = chars[..len].iter().collect();
    let alternative = match word.to_ascii_lowercase().as_str() {
        "however" => Some("but"),
        "because" => Some("since"),
        "perhaps" => Some("maybe"),
        "important" => Some("useful"),
        "therefore" => Some("so"),
        "usually" => Some("often"),
        _ => None,
    };
    if let Some(alternative) = alternative.filter(|_| rng.gen_bool(0.5)) {
        let mut result = alternative.to_string();
        if chars[0].is_ascii_uppercase() {
            result[..1].make_ascii_uppercase();
        }
        result
    } else {
        // An unfinished word is reconsidered and restarted, rather than
        // fabricating arbitrary sentences or modifying the supplied final copy.
        chars[..rng.gen_range(2..len)].iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};

    #[test]
    fn pauses_follow_structure_and_stay_out_of_words() {
        let chars: Vec<char> = "Plan this.\n\nAnother idea".chars().collect();
        for mode in [DraftingMode::Slow, DraftingMode::Fast] {
            let mut rhythm = DraftingRhythm::new(mode);
            let config = Config {
                drafting_mode: mode,
                ..Config::default()
            };
            let mut rng = StdRng::seed_from_u64(7);
            assert!(rhythm.step(&chars, 0, &config, &mut rng).pause_ms > 0);
            assert_eq!(rhythm.step(&chars, 1, &config, &mut rng).pause_ms, 0);
            let paragraph = rhythm.step(&chars, 12, &config, &mut rng).pause_ms;
            assert!(
                paragraph
                    >= if mode == DraftingMode::Slow {
                        5000
                    } else {
                        2000
                    }
            );
        }
    }

    #[test]
    fn revisions_do_not_split_unicode_or_code_tokens() {
        let mut rng = StdRng::seed_from_u64(42);
        for word in [
            "cafés",
            "hello\u{301}",
            "👻hello",
            "hello_world",
            "helloWorld()",
            "hello@example.com",
        ] {
            assert!(tentative_word(&word.chars().collect::<Vec<_>>(), &mut rng).is_empty());
        }
    }
}
