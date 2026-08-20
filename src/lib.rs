//! # fuzzy_dir
//!
//! A fuzzy matching library specifically made for matching folder names.
//!
//! ## Getting started
//!
//! ```rust
//! let score: u32 = fuzzy_dir::score_dir("this_is_my_folder_name", "myfoldrnam");
//! ```
//!
//! This library also exports a split function which tries to split
//! a folder name into its separate words. See [split](split::split) for
//! more infos.

mod split;

use std::collections::HashMap;

pub use split::split;

const CHARVAL_NOM: f32 = 25.;
const CHARVAL_MAX_DENOM: f32 = 5.;
const FULL_MATCH_SCORE: f32 = 120.;
const CHAR_HIT_MULT: f32 = 2.;
const CHAR_PART_HIT_MULT: f32 = 1.;
const CHAR_MISS_PENALTY: f32 = -5.;
const WORD_MATCH_SCORE: f32 = 20.;

/// Creates a score of how much the input and the pattern match
/// The higher the score the better. There is no max score.
pub fn score_dir(input: &str, pattern: &str) -> u32 {
    let words = split(input)
        .iter()
        .map(|word| word.chars().flat_map(|w| w.to_lowercase()).collect())
        .collect::<Vec<Vec<_>>>();
    let input = input.to_lowercase();
    let pattern = pattern.to_lowercase();
    let pat_chars = pattern.chars().map(Some).collect::<Vec<_>>();

    let pat_len = pat_chars.len() as f32;
    let char_value = CHARVAL_NOM / pat_len.max(CHARVAL_MAX_DENOM);

    let whole_pattern_score = {
        let for_containing = char_value * pat_len;
        if input == pattern {
            FULL_MATCH_SCORE + for_containing
        } else if input.contains(&pattern) {
            for_containing
        } else {
            0.
        }
    };

    let chars_score = {
        let inp_frq = freqmap(input.chars());
        let matches = freqmap(pat_chars.iter().filter_map(|&o| o)) // kill `Some`
            .iter()
            .map(|(&char, &pat_amt)| match inp_frq.get(&char) {
                // Letter that exists and fully fits into the input
                Some(&inp_amt) if pat_amt <= inp_amt => CHAR_HIT_MULT * pat_amt,
                // Even if it was taken already should be higher rated
                Some(&inp_amt) => {
                    CHAR_PART_HIT_MULT * (pat_amt - inp_amt) + CHAR_HIT_MULT * inp_amt
                }
                None => CHAR_MISS_PENALTY * pat_amt,
            })
            .sum::<f32>();
        char_value * matches
    };

    let words_score = {
        let mut pat_chars = pat_chars;
        let matches = words
            .into_iter()
            .map(|word| {
                let (start, len) = (0..pat_chars.len())
                    .map(|start| {
                        (
                            start,
                            // Find the largest subslice
                            word.iter()
                                .zip(&pat_chars[start..])
                                .take_while(|(wc, pc)| pc.is_some_and(|pc| pc == **wc))
                                .count(),
                        )
                    })
                    .max_by_key(|&(_, count)| count)
                    .unwrap_or_default();

                pat_chars[start..start + len].fill(None); // Zeroize it so one char doesn't trigger multiple times

                (len as f32).sqrt() // First char gives the most score, others less and less
            })
            .sum::<f32>();
        matches * WORD_MATCH_SCORE
    };

    (whole_pattern_score + chars_score + words_score).round() as u32
}

fn freqmap(chars: impl Iterator<Item = char>) -> HashMap<char, f32> {
    let mut map = HashMap::with_capacity(chars.size_hint().0);
    for char in chars {
        *map.entry(char).or_default() += 1.;
    }
    map
}

#[cfg(test)]
mod tests {
    use super::score_dir as score;

    #[test]
    fn test_simple() {
        assert_eq!(score("test", "test"), score("test", "test"));
        assert_eq!(score("test", "uoa"), 0);

        assert!(score("test", "test") > score("test", "tes"));
        assert!(score("ttest", "tt") > score("ttest", "t"));
    }

    #[test]
    fn test_advanced() {
        assert!(score("helloworld", "world") > score("helloworld", "elwo"));
        assert!(score("helloworld", "hello") > score("helloworld", "hellohello"));
    }

    #[test]
    fn test_negative_queries() {
        assert!(score("helloworld", "ellovvvv") == 0);
        assert!(score("helloworld", "wx") == 0);
    }

    #[test]
    fn test_separate_word_scoring() {
        assert!(score("test", "tt") > score("test", "t"));
        assert!(score("test-abc", "ta") > score("test-abc", "te"));
        assert!(score("test abc", "ta") > score("test abc", "te"));
        assert!(score("test_abc", "ta") > score("test_abc", "te"));

        assert!(score("test_abc_a", "taa") > score("test_abc_a", "te"));
        assert!(score("test_abc_a", "taa") > score("test_abc_a", "tea"));

        assert!(score("testAbc", "ta") > score("testAbc", "te"));
    }
}
