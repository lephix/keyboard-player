use std::collections::HashSet;

use bevy::prelude::*;
use rand::Rng;

use crate::data::text_model::Language;

const EN_HIDE_RATIO: f64 = 1.0 / 3.0;
const EN_MAX_PER_WORD: usize = 1;

#[derive(Resource, Default, Clone)]
pub struct HiddenChars {
    pub positions: Vec<HashSet<usize>>,
}

pub const HIDDEN_PLACEHOLDER: char = '■';

pub fn generate_hidden_positions(lines: &[String], language: Language) -> HiddenChars {
    let mut rng = rand::thread_rng();
    let positions = lines
        .iter()
        .map(|line| match language {
            Language::En => generate_english_hidden(line, &mut rng),
            Language::Zh => generate_chinese_hidden(line, &mut rng),
        })
        .collect();
    HiddenChars { positions }
}

fn generate_english_hidden(line: &str, rng: &mut impl Rng) -> HashSet<usize> {
    let mut hidden = HashSet::new();
    let chars: Vec<char> = line.chars().collect();
    let letter_count: usize = chars.iter().filter(|c| c.is_alphabetic()).count();
    let max_hidden = ((letter_count as f64) * EN_HIDE_RATIO).floor() as usize;

    let mut words: Vec<Vec<usize>> = Vec::new();
    let mut current_word: Vec<usize> = Vec::new();

    for (i, &ch) in chars.iter().enumerate() {
        if ch.is_whitespace() {
            if !current_word.is_empty() {
                words.push(current_word.clone());
                current_word.clear();
            }
        } else {
            if ch.is_alphabetic() {
                current_word.push(i);
            }
        }
    }
    if !current_word.is_empty() {
        words.push(current_word);
    }

    let mut hidden_count = 0;
    for letter_positions in &words {
        if hidden_count >= max_hidden {
            break;
        }
        if !letter_positions.is_empty() {
            let idx = rng.gen_range(0..letter_positions.len());
            hidden.insert(letter_positions[idx]);
            hidden_count += 1;
        }
    }

    hidden
}

fn generate_chinese_hidden(line: &str, rng: &mut impl Rng) -> HashSet<usize> {
    let mut hidden = HashSet::new();

    let delimiters = ['，', '。', '！', '？', '；', '、', ',', '.', '!', '?', ';'];
    let mut sentence_start = 0;
    let chars: Vec<char> = line.chars().collect();

    let mut i = 0;
    while i <= chars.len() {
        let is_end = i == chars.len() || delimiters.contains(&chars[i]);

        if is_end && i > sentence_start {
            let hanzi_positions: Vec<usize> = (sentence_start..i)
                .filter(|&idx| {
                    let ch = chars[idx];
                    ch >= '\u{4e00}' && ch <= '\u{9fff}'
                })
                .collect();

            if hanzi_positions.len() > 2 {
                let idx = rng.gen_range(0..hanzi_positions.len());
                hidden.insert(hanzi_positions[idx]);
            }

            if is_end && i < chars.len() {
                sentence_start = i + 1;
            }
        }

        if is_end {
            sentence_start = i + 1;
        }
        i += 1;
    }

    hidden
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn english_hidden_positions_are_valid_char_indices() {
        let mut rng = StdRng::seed_from_u64(42);
        let line = "hello world test";
        let hidden = generate_english_hidden(line, &mut rng);
        let chars: Vec<char> = line.chars().collect();

        for &pos in &hidden {
            assert!(pos < chars.len(), "position {} out of bounds", pos);
            assert!(
                chars[pos].is_alphabetic(),
                "position {} is not alphabetic",
                pos
            );
        }
    }

    #[test]
    fn english_hidden_respects_max_ratio() {
        let mut rng = StdRng::seed_from_u64(99);
        let line = "the quick brown fox jumps over the lazy dog";
        let hidden = generate_english_hidden(line, &mut rng);
        let letter_count = line.chars().filter(|c| c.is_alphabetic()).count();
        let max_hidden = ((letter_count as f64) * EN_HIDE_RATIO).floor() as usize;

        assert!(hidden.len() <= max_hidden);
    }

    #[test]
    fn english_hidden_at_most_one_per_word() {
        let mut rng = StdRng::seed_from_u64(7);
        let line = "hello world test foo bar";
        let hidden = generate_english_hidden(line, &mut rng);
        let chars: Vec<char> = line.chars().collect();

        let mut in_word = false;
        let mut word_hidden_counts: Vec<usize> = Vec::new();
        let mut current_count = 0;

        for (i, &ch) in chars.iter().enumerate() {
            if ch.is_whitespace() {
                if in_word {
                    word_hidden_counts.push(current_count);
                    current_count = 0;
                    in_word = false;
                }
            } else {
                in_word = true;
                if hidden.contains(&i) {
                    current_count += 1;
                }
            }
        }
        if in_word {
            word_hidden_counts.push(current_count);
        }

        for (idx, &count) in word_hidden_counts.iter().enumerate() {
            assert!(count <= 1, "word {} has {} hidden chars", idx, count);
        }
    }

    #[test]
    fn chinese_hidden_at_most_one_per_sentence() {
        let mut rng = StdRng::seed_from_u64(42);
        let line = "床前明月光，疑是地上霜。";
        let hidden = generate_chinese_hidden(line, &mut rng);
        let chars: Vec<char> = line.chars().collect();

        for &pos in &hidden {
            assert!(pos < chars.len());
            let ch = chars[pos];
            assert!(
                ch >= '\u{4e00}' && ch <= '\u{9fff}',
                "hidden char '{}' is not CJK",
                ch
            );
        }

        assert!(hidden.len() <= 2);
    }

    #[test]
    fn empty_line_produces_no_hidden() {
        let mut rng = StdRng::seed_from_u64(42);
        assert!(generate_english_hidden("", &mut rng).is_empty());
        assert!(generate_chinese_hidden("", &mut rng).is_empty());
    }

    #[test]
    fn single_word_line() {
        let mut rng = StdRng::seed_from_u64(42);
        let hidden = generate_english_hidden("hi", &mut rng);
        assert!(hidden.len() <= 1);
    }

    #[test]
    fn generate_positions_multiple_lines() {
        let lines = vec!["hello world".to_string(), "foo bar baz".to_string()];
        let result = generate_hidden_positions(&lines, Language::En);
        assert_eq!(result.positions.len(), 2);
    }
}
