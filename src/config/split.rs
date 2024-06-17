use alloc::string::ToString;
use alloc::vec::Vec;

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

use crate::Regex;

/// Split behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub enum SplitBehavior {
    /// Keep the matching parts, discard the non-matching parts.
    Match,
    /// Keep the non-matching parts, discard the matching parts.
    Remove,
    /// Isolate the matching parts, keep the non-matching parts.
    Isolate,
    /// Merge consecutive matching parts, keep the non-matching parts.
    Merge,
    /// Isolate the matching parts, merging single isolates with the left non-matching part, keep the non-matching parts.
    MergeLeft,
    /// Isolate the matching parts, merging single isolates with the right non-matching part, keep the non-matching parts.
    MergeRight,
}

/// Pre-tokenization split configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub enum Split {
    /// Split by regular expression.
    Pattern {
        pattern:  Regex,
        behavior: SplitBehavior,
    },
    /// Split by character.
    Character {
        character: char,
        behavior:  SplitBehavior,
    },
}

impl Split {
    #[inline(never)]
    pub fn split(&self, text: &str) -> Vec<(usize, usize)> {
        if text.is_empty() {
            return Vec::new();
        }
        use Split::*;
        use SplitBehavior::*;
        let (mut matches, behavior) = match self {
            Pattern { pattern, behavior } => (pattern.find_iter(text), *behavior),
            Character {
                character,
                behavior,
            } => {
                let matches = if character.len_utf8() == 0 {
                    Vec::new()
                } else if character.len_utf8() == 1 {
                    memchr::memchr_iter(*character as u8, text.as_bytes())
                        .map(|a| (a, a + 1))
                        .collect()
                } else {
                    memchr::memmem::find_iter(text.as_bytes(), character.to_string().as_bytes())
                        .map(|a| (a, a + 1))
                        .collect()
                };
                (matches, *behavior)
            }
        };
        match behavior {
            Match => {}
            Remove => {
                invert(&mut matches, text.len());
            }
            Isolate => {
                expand(&mut matches, text.len());
            }
            Merge => {
                merge(&mut matches);
                expand(&mut matches, text.len());
            }
            MergeLeft => {
                merge_left(&mut matches, text.len());
            }
            MergeRight => {
                merge_right(&mut matches, text.len());
            }
        }
        matches
    }
}

/// Inverts the matches leaving the gaps.
#[inline(always)]
fn invert(matches: &mut Vec<(usize, usize)>, len: usize) {
    let mut last = 0;
    *matches = matches.iter().fold(Vec::new(), |mut acc, (start, end)| {
        if *start != last {
            acc.push((last, *start));
        }
        last = *end;
        acc
    });
    if last < len {
        matches.push((last, len));
    }
}

/// Expands the matches to include the gaps.
#[inline(always)]
fn expand(matches: &mut Vec<(usize, usize)>, len: usize) {
    let mut last = 0;
    *matches = matches.iter().fold(Vec::new(), |mut acc, (start, end)| {
        if *start != last {
            acc.push((last, *start));
        }
        last = *end;
        acc.push((*start, *end));
        acc
    });
    if last < len {
        matches.push((last, len));
    }
}

/// Merges consecutive matches.
#[inline(always)]
fn merge(matches: &mut Vec<(usize, usize)>) {
    if matches.is_empty() {
        return;
    }
    let mut last = 0;
    *matches = matches.iter().fold(Vec::new(), |mut acc, (start, end)| {
        if *start == last && !acc.is_empty() {
            acc.last_mut().unwrap().1 = *end;
        } else {
            acc.push((*start, *end));
        }
        last = *end;
        acc
    });
}

/// Merge the first match after a gap with the gap and expand.
#[inline(always)]
fn merge_left(matches: &mut Vec<(usize, usize)>, len: usize) {
    let mut last = 0;
    *matches = matches.iter().fold(Vec::new(), |mut acc, (start, end)| {
        if *start != last {
            acc.push((last, *end));
        } else {
            acc.push((*start, *end));
        }
        last = *end;
        acc
    });
    if last < len {
        matches.push((last, len));
    }
}

/// Merge the last match before a gap with the gap and expand.
#[inline(always)]
fn merge_right(matches: &mut Vec<(usize, usize)>, len: usize) {
    if matches.is_empty() {
        matches.push((0, len));
        return;
    }
    let mut last = 0;
    *matches = matches.iter().fold(Vec::new(), |mut acc, (start, end)| {
        if *start != last && !acc.is_empty() {
            acc.last_mut().unwrap().1 = *start;
        }
        acc.push((*start, *end));
        last = *end;
        acc
    });
    if last < len {
        matches.last_mut().unwrap().1 = len;
    }
    if matches[0].0 != 0 {
        matches.insert(0, (0, matches[0].0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Regex;
    use alloc::vec::Vec;

    #[test]
    fn test_split_match() {
        let split = Split::Pattern {
            pattern:  Regex::new(r"[ ]").unwrap(),
            behavior: SplitBehavior::Match,
        };
        let text = "aaa bbb  ccc   ddd";
        let matches = split.split(text);
        #[rustfmt::skip]
        let expected = Vec::from([(3, 4), (7, 8), (8, 9), (12, 13), (13, 14), (14, 15)]);
        assert_eq!(matches, expected);
    }

    #[test]
    fn test_split_match_char() {
        let split = Split::Character {
            character: ' ',
            behavior:  SplitBehavior::Match,
        };
        let text = "aaa bbb  ccc   ddd";
        let matches = split.split(text);
        #[rustfmt::skip]
        let expected = Vec::from([(3, 4), (7, 8), (8, 9), (12, 13), (13, 14), (14, 15)]);
        assert_eq!(matches, expected);
    }

    #[test]
    fn test_split_remove() {
        let split = Split::Pattern {
            pattern:  Regex::new(r"[ ]").unwrap(),
            behavior: SplitBehavior::Remove,
        };
        let text = "aaa bbb  ccc   ddd";
        let matches = split.split(text);
        #[rustfmt::skip]
        let expected = Vec::from([(0, 3), (4, 7), (9, 12), (15, 18)]);
        assert_eq!(matches, expected);
    }

    #[test]
    fn test_split_remove_char() {
        let split = Split::Character {
            character: ' ',
            behavior:  SplitBehavior::Remove,
        };
        let text = "aaa bbb  ccc   ddd";
        let matches = split.split(text);
        #[rustfmt::skip]
        let expected = Vec::from([(0, 3), (4, 7), (9, 12), (15, 18)]);
        assert_eq!(matches, expected);
    }

    #[test]
    fn test_split_isolate() {
        let split = Split::Pattern {
            pattern:  Regex::new(r"[ ]").unwrap(),
            behavior: SplitBehavior::Isolate,
        };
        let text = "aaa bbb  ccc   ddd";
        let matches = split.split(text);
        #[rustfmt::skip]
        let expected = Vec::from([(0, 3), (3, 4), (4, 7), (7, 8), (8, 9), (9, 12), (12, 13), (13, 14), (14, 15), (15, 18)]);
        assert_eq!(matches, expected);
    }

    #[test]
    fn test_split_isolate_char() {
        let split = Split::Character {
            character: ' ',
            behavior:  SplitBehavior::Isolate,
        };
        let text = "aaa bbb  ccc   ddd";
        let matches = split.split(text);
        #[rustfmt::skip]
        let expected = Vec::from([(0, 3), (3, 4), (4, 7), (7, 8), (8, 9), (9, 12), (12, 13), (13, 14), (14, 15), (15, 18)]);
        assert_eq!(matches, expected);
    }

    #[test]
    fn test_split_merge() {
        let split = Split::Pattern {
            pattern:  Regex::new(r"[ ]").unwrap(),
            behavior: SplitBehavior::Merge,
        };
        let text = "aaa bbb  ccc   ddd";
        let matches = split.split(text);
        #[rustfmt::skip]
        let expected = Vec::from([(0, 3), (3, 4), (4, 7), (7, 9), (9, 12), (12, 15), (15, 18)]);
        assert_eq!(matches, expected);
    }

    #[test]
    fn test_split_merge_char() {
        let split = Split::Character {
            character: ' ',
            behavior:  SplitBehavior::Merge,
        };
        let text = "aaa bbb  ccc   ddd";
        let matches = split.split(text);
        #[rustfmt::skip]
        let expected = Vec::from([(0, 3), (3, 4), (4, 7), (7, 9), (9, 12), (12, 15), (15, 18)]);
        assert_eq!(matches, expected);
    }

    #[test]
    fn test_split_merge_left() {
        let split = Split::Pattern {
            pattern:  Regex::new(r"[ ]").unwrap(),
            behavior: SplitBehavior::MergeLeft,
        };
        let text = "aaa bbb  ccc   ddd";
        let matches = split.split(text);
        #[rustfmt::skip]
        let expected = Vec::from([(0, 4), (4, 8), (8, 9), (9, 13), (13, 14), (14, 15), (15, 18)]);
        assert_eq!(matches, expected);
    }

    #[test]
    fn test_split_merge_left_char() {
        let split = Split::Character {
            character: ' ',
            behavior:  SplitBehavior::MergeLeft,
        };
        let text = "aaa bbb  ccc   ddd";
        let matches = split.split(text);
        #[rustfmt::skip]
        let expected = Vec::from([(0, 4), (4, 8), (8, 9), (9, 13), (13, 14), (14, 15), (15, 18)]);
        assert_eq!(matches, expected);
    }

    #[test]
    fn test_split_merge_right() {
        let split = Split::Pattern {
            pattern:  Regex::new(r"[ ]").unwrap(),
            behavior: SplitBehavior::MergeRight,
        };
        let text = "aaa bbb  ccc   ddd";
        let matches = split.split(text);
        #[rustfmt::skip]
        let expected = Vec::from([(0, 3), (3, 7), (7, 8), (8, 12), (12, 13), (13, 14), (14, 18)]);
        assert_eq!(matches, expected);
    }

    #[test]
    fn test_split_merge_right_char() {
        let split = Split::Character {
            character: ' ',
            behavior:  SplitBehavior::MergeRight,
        };
        let text = "aaa bbb  ccc   ddd";
        let matches = split.split(text);
        #[rustfmt::skip]
        let expected = Vec::from([(0, 3), (3, 7), (7, 8), (8, 12), (12, 13), (13, 14), (14, 18)]);
        assert_eq!(matches, expected);
    }
}
