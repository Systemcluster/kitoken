//! Pre-tokenization input normalization.

use core::ops::Range;

use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use once_cell::race::OnceBox;
#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

use crate::{CharsMap, Regex};

/// Unicode normalization scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub enum UnicodeNormalization {
    /// Unicode normalization form C.
    NFC,
    /// Unicode normalization form D.
    NFD,
    /// Unicode normalization form KC.
    NFKC,
    /// Unicode normalization form KD.
    NFKD,
}

/// Replacement pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub enum NormalizationReplacePattern {
    /// Replace a character.
    Character(char),
    /// Replace a string.
    String(String),
    /// Replace by regular expression.
    Regex(Regex),
}
impl From<char> for NormalizationReplacePattern {
    #[inline(always)]
    fn from(character: char) -> Self {
        Self::Character(character)
    }
}
impl From<String> for NormalizationReplacePattern {
    #[inline(always)]
    fn from(pattern: String) -> Self {
        Self::String(pattern)
    }
}
impl From<&str> for NormalizationReplacePattern {
    #[inline(always)]
    fn from(pattern: &str) -> Self {
        Self::String(pattern.to_string())
    }
}
impl From<Regex> for NormalizationReplacePattern {
    #[inline(always)]
    fn from(pattern: Regex) -> Self {
        Self::Regex(pattern)
    }
}

/// Condition for conditional normalization.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub enum NormalizationCondition {
    StartOfText,
    EndOfText,
}

/// Pre-tokenization input normalization configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub enum Normalization {
    /// Unicode normalization.
    Unicode { scheme: UnicodeNormalization },
    /// NMT normalization.
    NMT,
    /// Case folding.
    CaseFold { upper: bool },
    /// Append a string to the end.
    Append { append: String },
    /// Prepend a string to the beginning.
    Prepend { prepend: String },
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
    /// Pattern replacement.
    Replace {
        pattern:     NormalizationReplacePattern,
        replacement: String,
    },
    /// Precompiled character map.
    CharsMap { map: CharsMap },
    /// Conditional normalization.
    Conditional {
        condition:     NormalizationCondition,
        normalization: Box<Normalization>,
    },
}

impl Normalization {
    #[inline(never)]
    pub fn normalize(&self, text: &mut Cow<str>, position: Range<usize>) {
        use Normalization::*;
        match self {
            Unicode { scheme } => {
                normalize_unicode(text, *scheme);
            }
            NMT => {
                normalize_nmt(text);
            }
            CaseFold { upper } => {
                normalize_casefold(text, *upper);
            }
            Append { append } => {
                normalize_append(text, append);
            }
            Prepend { prepend } => {
                normalize_prepend(text, prepend);
            }
            Extend {
                character,
                left,
                right,
                pad,
            } => {
                normalize_extend(text, *character, *left, *right, *pad);
            }
            Strip {
                character,
                left,
                right,
            } => {
                normalize_strip(text, *character, *left, *right);
            }
            Collapse { character } => {
                normalize_collapse(text, *character);
            }
            Replace {
                pattern,
                replacement,
            } => {
                normalize_replace(text, pattern, replacement);
            }
            CharsMap { map } => {
                normalize_charsmap(text, map);
            }
            Conditional {
                condition,
                normalization,
            } => {
                if match condition {
                    NormalizationCondition::StartOfText => position.start == 0,
                    NormalizationCondition::EndOfText => position.end == usize::MAX,
                } {
                    normalization.normalize(text, position);
                }
            }
        }
    }
}

#[cfg(feature = "normalization-unicode")]
#[inline(never)]
fn normalize_unicode(text: &mut Cow<str>, scheme: UnicodeNormalization) {
    use unicode_normalization::UnicodeNormalization as _;
    use UnicodeNormalization::*;
    match scheme {
        NFC => {
            *text.to_mut() = text.nfc().collect();
        }
        NFD => {
            *text.to_mut() = text.nfd().collect();
        }
        NFKC => {
            *text.to_mut() = text.nfkc().collect();
        }
        NFKD => {
            *text.to_mut() = text.nfkd().collect();
        }
    }
}
#[cfg(not(feature = "normalization-unicode"))]
#[inline(never)]
fn normalize_unicode(_text: &mut Cow<str>, _scheme: UnicodeNormalization) {
    log::warn!("Unicode normalization must be enabled for Unicode normalization");
}

#[inline(never)]
fn normalize_nmt(text: &mut Cow<str>) {
    text.to_mut()
        .retain(|c| !matches!(c, '\u{1}'..='\u{8}' | '\u{e}'..='\u{1f}' | '\u{b}' | '\u{7f}' | '\u{8f}' | '\u{9f}'));
    static NMT_REGEX_SPACE: OnceBox<Regex> = const { OnceBox::new() };
    let replacer_space = NMT_REGEX_SPACE.get_or_init(|| {
            Box::new(Regex::new("[\u{0}\u{a}\u{c}\u{d}\u{1680}\u{200B}-\u{200F}\u{2028}\u{2029}\u{2581}\u{feff}\u{fffd}]")
                .unwrap())
    });
    *text.to_mut() = replacer_space.replace_all(text, " ");
}

#[inline(never)]
fn normalize_casefold(text: &mut Cow<str>, upper: bool) {
    if upper {
        *text.to_mut() = text.to_uppercase();
    } else {
        *text.to_mut() = text.to_lowercase();
    }
}

#[inline(never)]
fn normalize_append(text: &mut Cow<str>, append: &str) {
    text.to_mut().push_str(append);
}

#[inline(never)]
fn normalize_prepend(text: &mut Cow<str>, prepend: &str) {
    text.to_mut().insert_str(0, prepend);
}

#[inline(never)]
fn normalize_extend(text: &mut Cow<str>, character: char, left: u32, right: u32, pad: bool) {
    let mut buffer = core::iter::repeat(0).take(character.len_utf8()).collect::<Vec<_>>();
    character.encode_utf8(&mut buffer);
    if left > 0 {
        let mut left = left as usize;
        if pad {
            let leading = text.chars().take(left).take_while(|&c| c == character).count();
            left = left.saturating_sub(leading);
        }
        unsafe {
            text.to_mut()
                .as_mut_vec()
                .splice(..0, core::iter::repeat(&buffer).take(left).flatten().copied());
        };
    }
    if right > 0 {
        let mut right = right as usize;
        if pad {
            let trailing = text.chars().rev().take(right).take_while(|&c| c == character).count();
            right = right.saturating_sub(trailing);
        }
        unsafe {
            text.to_mut()
                .as_mut_vec()
                .extend(core::iter::repeat(&buffer).take(right).flatten().copied());
        };
    }
}

#[inline(never)]
fn normalize_strip(text: &mut Cow<str>, character: char, mut left: u32, mut right: u32) {
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
        text.to_mut().drain(..slice_start);
    }
    if slice_end > 0 {
        let len = text.len();
        text.to_mut().drain(len - slice_end..);
    }
}

#[inline(never)]
fn normalize_collapse(text: &mut Cow<str>, character: char) {
    let mut last = None;
    *text.to_mut() = text
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
        .collect();
}

#[inline(never)]
fn normalize_replace(
    text: &mut Cow<str>, pattern: &NormalizationReplacePattern, replacement: &str,
) {
    match pattern {
        NormalizationReplacePattern::Character(character) => {
            *text.to_mut() = text.replace(*character, replacement);
        }
        NormalizationReplacePattern::String(pattern) => {
            *text.to_mut() = text.replace(pattern, replacement);
        }
        NormalizationReplacePattern::Regex(pattern) => {
            *text.to_mut() = pattern.replace_all(text, replacement);
        }
    }
}

#[cfg(feature = "normalization-charsmap")]
#[inline(never)]
fn normalize_charsmap(text: &mut Cow<str>, map: &CharsMap) {
    *text = map.normalize(text).into();
}
#[cfg(not(feature = "normalization-charsmap"))]
#[inline(never)]
fn normalize_charsmap(_text: &mut Cow<str>, _map: &CharsMap) {
    log::warn!("CharsMap normalization must be enabled for CharsMap normalization");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalization_nmt() {
        let mut text = Cow::Borrowed("aaa\u{200D}bbb\u{8f}");
        let normalization = Normalization::NMT;
        normalization.normalize(&mut text, 0..usize::MAX);
        assert_eq!(text, "aaa bbb");
    }

    #[test]
    fn test_normalization_case_fold() {
        let mut text = Cow::Borrowed("AAA bbb");
        let normalization = Normalization::CaseFold { upper: false };
        normalization.normalize(&mut text, 0..usize::MAX);
        assert_eq!(text, "aaa bbb");

        let mut text = Cow::Borrowed("AAA bbb");
        let normalization = Normalization::CaseFold { upper: true };
        normalization.normalize(&mut text, 0..usize::MAX);
        assert_eq!(text, "AAA BBB");
    }

    #[test]
    fn test_normalization_append() {
        let mut text = Cow::Borrowed("aaa");
        let normalization = Normalization::Append {
            append: " bbb".to_string(),
        };
        normalization.normalize(&mut text, 0..usize::MAX);
        assert_eq!(text, "aaa bbb");
    }

    #[test]
    fn test_normalization_prepend() {
        let mut text = Cow::Borrowed("bbb");
        let normalization = Normalization::Prepend {
            prepend: "aaa ".to_string(),
        };
        normalization.normalize(&mut text, 0..usize::MAX);
        assert_eq!(text, "aaa bbb");
    }

    #[test]
    fn test_normalization_extend() {
        let mut text = Cow::Borrowed("bbb");
        let normalization = Normalization::Extend {
            character: 'a',
            left:      2,
            right:     3,
            pad:       false,
        };
        normalization.normalize(&mut text, 0..usize::MAX);
        assert_eq!(text, "aabbbaaa");

        let mut text = Cow::Borrowed("aba");
        let normalization = Normalization::Extend {
            character: 'a',
            left:      2,
            right:     3,
            pad:       true,
        };
        normalization.normalize(&mut text, 0..usize::MAX);
        assert_eq!(text, "aabaaa");
    }

    #[test]
    fn test_normalization_strip() {
        let mut text = Cow::Borrowed("aaabaaaa");
        let normalization = Normalization::Strip {
            character: 'a',
            left:      2,
            right:     3,
        };
        normalization.normalize(&mut text, 0..usize::MAX);
        assert_eq!(text, "aba");
    }

    #[test]
    fn test_normalization_collapse() {
        let mut text = Cow::Borrowed("abbbba bbb");
        let normalization = Normalization::Collapse { character: 'b' };
        normalization.normalize(&mut text, 0..usize::MAX);
        assert_eq!(text, "aba b");
    }

    #[test]
    fn test_normalization_replace() {
        let mut text = Cow::Borrowed("aba bbb");
        let normalization = Normalization::Replace {
            pattern:     Regex::new(r"b").unwrap().into(),
            replacement: "a".to_string(),
        };
        normalization.normalize(&mut text, 0..usize::MAX);
        assert_eq!(text, "aaa aaa");
    }

    #[test]
    fn test_normalization_conditional() {
        let mut text = Cow::Borrowed("aba bbb");
        let normalization = Normalization::Conditional {
            condition:     NormalizationCondition::StartOfText,
            normalization: Box::new(Normalization::Replace {
                pattern:     Regex::new(r"b").unwrap().into(),
                replacement: "a".to_string(),
            }),
        };
        normalization.normalize(&mut text, 0..usize::MAX);
        assert_eq!(text, "aaa aaa");

        let mut text = Cow::Borrowed("aba bbb");
        normalization.normalize(&mut text, 1..usize::MAX);
        assert_eq!(text, "aba bbb");

        let mut text = Cow::Borrowed("aba bbb");
        let normalization = Normalization::Conditional {
            condition:     NormalizationCondition::EndOfText,
            normalization: Box::new(Normalization::Replace {
                pattern:     Regex::new(r"b").unwrap().into(),
                replacement: "a".to_string(),
            }),
        };
        normalization.normalize(&mut text, 0..usize::MAX);
        assert_eq!(text, "aaa aaa");

        let mut text = Cow::Borrowed("aba bbb");
        normalization.normalize(&mut text, 0..4);
        assert_eq!(text, "aba bbb");
    }
}
