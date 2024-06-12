use alloc::string::String;
use alloc::vec::Vec;

use bstr::ByteSlice;

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

/// Post-detokenization decoding configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub enum Decoding {
    /// Add a character to the beginning and end, optionally accounting for padding.
    Extend {
        character: char,
        left:      u32,
        right:     u32,
        pad:       bool,
    },
    /// Strip a character from the beginning and end.
    Strip {
        character: char,
        left:      u32,
        right:     u32,
    },
    /// Collapse repeated characters.
    Collapse { character: char },
    /// String replacement.
    Replace {
        pattern:     String,
        replacement: String,
    },
}

impl Decoding {
    #[inline(never)]
    pub fn decode(&self, text: &mut Vec<u8>) {
        use Decoding::*;
        match self {
            Extend {
                character,
                left,
                right,
                pad,
            } => {
                let mut buffer =
                    core::iter::repeat(0).take(character.len_utf8()).collect::<Vec<_>>();
                character.encode_utf8(&mut buffer);
                if *left > 0 {
                    let mut left = *left as usize;
                    if *pad {
                        let leading =
                            text.chars().take(left).take_while(|c| c == character).count();
                        left = left.saturating_sub(leading);
                    }
                    text.splice(..0, core::iter::repeat(&buffer).take(left).flatten().copied());
                }
                if *right > 0 {
                    let mut right = *right as usize;
                    if *pad {
                        let trailing =
                            text.chars().rev().take(right).take_while(|c| c == character).count();
                        right = right.saturating_sub(trailing);
                    }
                    text.extend(core::iter::repeat(&buffer).take(right).flatten().copied());
                }
            }
            Strip {
                character,
                mut left,
                mut right,
            } => {
                let mut slice_start = 0;
                let mut slice_end = 0;
                if left > 0 {
                    for c in text[..].chars() {
                        if c != *character || left == 0 {
                            break;
                        }
                        slice_start += c.len_utf8();
                        left -= 1;
                    }
                }
                if right > 0 {
                    for c in text[slice_start..].chars().rev() {
                        if c != *character || right == 0 {
                            break;
                        }
                        slice_end += c.len_utf8();
                        right -= 1;
                    }
                }
                if slice_start > 0 {
                    text.drain(..slice_start);
                }
                if slice_end > 0 {
                    let len = text.len();
                    text.drain(len - slice_end..);
                }
            }
            Collapse { character } => {
                let mut buffer = [0; 8];
                let mut last = None;
                *text = text
                    .chars()
                    .filter(|c| {
                        if c == character {
                            if Some(c) == last.as_ref() {
                                return false;
                            }
                            last = Some(*c);
                        } else {
                            last = None;
                        }
                        true
                    })
                    .flat_map(|c| c.encode_utf8(&mut buffer).as_bytes().to_vec())
                    .collect();
            }
            Replace {
                pattern,
                replacement,
            } => {
                *text = text.replace(pattern.as_bytes(), replacement.as_bytes());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    #[test]
    fn test_decoding_extend() {
        let mut text = Vec::from(b"aba");
        let decoding = Decoding::Extend {
            character: 'a',
            left:      1,
            right:     2,
            pad:       false,
        };
        decoding.decode(&mut text);
        assert_eq!(text, Vec::from(b"aabaaa"));

        let mut text = Vec::from(b"aba");
        let decoding = Decoding::Extend {
            character: 'a',
            left:      1,
            right:     2,
            pad:       true,
        };
        decoding.decode(&mut text);
        assert_eq!(text, Vec::from(b"abaa"));
    }

    #[test]
    fn test_decoding_strip() {
        let mut text = Vec::from(b"aabaaa");
        let decoding = Decoding::Strip {
            character: 'a',
            left:      1,
            right:     2,
        };
        decoding.decode(&mut text);
        assert_eq!(text, Vec::from(b"aba"));
    }

    #[test]
    fn test_decoding_collapse() {
        let mut text = Vec::from(b"abbbba bbb");
        let decoding = Decoding::Collapse { character: 'b' };
        decoding.decode(&mut text);
        assert_eq!(text, Vec::from(b"aba b"));
    }

    #[test]
    fn test_decoding_replace() {
        let mut text = Vec::from(b"aabbba");
        let decoding = Decoding::Replace {
            pattern:     "bbb".to_owned(),
            replacement: "a".to_owned(),
        };
        decoding.decode(&mut text);
        assert_eq!(text, Vec::from(b"aaaa"));
    }
}
