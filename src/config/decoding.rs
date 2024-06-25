//! Post-detokenization output decoding.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use bstr::ByteSlice;

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

use crate::Regex;

/// Replacement pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub enum DecodingReplacePattern {
    /// Replace a character.
    Character(char),
    /// Replace a string.
    String(String),
    /// Replace by regular expression.
    /// Invalid UTF-8 bytes are replaced with `U+FFFD`.
    Regex(Regex),
}
impl From<char> for DecodingReplacePattern {
    #[inline(always)]
    fn from(character: char) -> Self {
        Self::Character(character)
    }
}
impl From<String> for DecodingReplacePattern {
    #[inline(always)]
    fn from(pattern: String) -> Self {
        Self::String(pattern)
    }
}
impl From<&str> for DecodingReplacePattern {
    #[inline(always)]
    fn from(pattern: &str) -> Self {
        Self::String(pattern.to_string())
    }
}
impl From<Regex> for DecodingReplacePattern {
    #[inline(always)]
    fn from(regex: Regex) -> Self {
        Self::Regex(regex)
    }
}

/// Post-detokenization output decoding configuration.
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
        pattern:     DecodingReplacePattern,
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
                decode_extend(text, *character, *left, *right, *pad);
            }
            Strip {
                character,
                left,
                right,
            } => {
                decode_strip(text, *character, *left, *right);
            }
            Collapse { character } => {
                decode_collapse(text, *character);
            }
            Replace {
                pattern,
                replacement,
            } => {
                decode_replace(text, pattern, replacement);
            }
        }
    }
}

#[inline(never)]
fn decode_extend(text: &mut Vec<u8>, character: char, left: u32, right: u32, pad: bool) {
    let mut buffer = core::iter::repeat(0).take(character.len_utf8()).collect::<Vec<_>>();
    character.encode_utf8(&mut buffer);
    if left > 0 {
        let mut left = left as usize;
        if pad {
            let leading = text.chars().take(left).take_while(|&c| c == character).count();
            left = left.saturating_sub(leading);
        }
        text.splice(..0, core::iter::repeat(&buffer).take(left).flatten().copied());
    }
    if right > 0 {
        let mut right = right as usize;
        if pad {
            let trailing = text.chars().rev().take(right).take_while(|&c| c == character).count();
            right = right.saturating_sub(trailing);
        }
        text.extend(core::iter::repeat(&buffer).take(right).flatten().copied());
    }
}

#[inline(never)]
fn decode_strip(text: &mut Vec<u8>, character: char, mut left: u32, mut right: u32) {
    let mut slice_start = 0;
    let mut slice_end = 0;
    if left > 0 {
        for c in text[..].chars() {
            if c != character || left == 0 {
                break;
            }
            slice_start += c.len_utf8();
            left -= 1;
        }
    }
    if right > 0 {
        for c in text[slice_start..].chars().rev() {
            if c != character || right == 0 {
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

#[inline(never)]
fn decode_collapse(text: &mut Vec<u8>, character: char) {
    let mut buffer = [0; 8];
    let mut last = None;
    *text = text
        .chars()
        .filter(|&c| {
            if c == character {
                if Some(c) == last {
                    return false;
                }
                last = Some(c);
            } else {
                last = None;
            }
            true
        })
        .flat_map(|c| c.encode_utf8(&mut buffer).as_bytes().to_vec())
        .collect();
}

#[inline(never)]
fn decode_replace(text: &mut Vec<u8>, pattern: &DecodingReplacePattern, replacement: &str) {
    match pattern {
        DecodingReplacePattern::Character(character) => {
            let bytes = character.to_string().into_bytes();
            *text = text.replace(bytes, replacement.as_bytes());
        }
        DecodingReplacePattern::String(pattern) => {
            let bytes = pattern.as_bytes();
            *text = text.replace(bytes, replacement.as_bytes());
        }
        DecodingReplacePattern::Regex(regex) => {
            *text = regex.replace_all(&text.to_str_lossy(), replacement).into();
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
            pattern:     "bbb".into(),
            replacement: "a".to_owned(),
        };
        decoding.decode(&mut text);
        assert_eq!(text, Vec::from(b"aaaa"));
    }
}
