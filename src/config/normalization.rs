use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use once_cell::race::OnceBox;

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

use crate::charsmap::CharsMap;
use crate::Regex;

/// Unicode normalization scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// Input normalization configuration.
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
        pattern:     Regex,
        replacement: String,
    },
    /// Precompiled character map.
    CharsMap { map: CharsMap },
}

impl Normalization {
    #[inline(never)]
    pub fn normalize(&self, text: &mut Cow<str>) {
        static NMT_REGEX_SPACE: OnceBox<Regex> = OnceBox::new();

        use Normalization::*;
        match self {
            #[cfg(feature = "unicode-normalization")]
            Unicode { scheme } => {
                use unicode_normalization::UnicodeNormalization as _;
                use UnicodeNormalization::*;
                match scheme {
                    NFC => {
                        *text = text.nfc().collect();
                    }
                    NFD => {
                        *text = text.nfd().collect();
                    }
                    NFKC => {
                        *text = text.nfkc().collect();
                    }
                    NFKD => {
                        *text = text.nfkd().collect();
                    }
                }
            }
            #[cfg(not(feature = "unicode-normalization"))]
            Unicode { scheme } => {
                panic!("Unicode normalization must be enabled for {:?} normalization", scheme);
            }
            NMT => {
                text.to_mut().retain(|c| !matches!(c, '\u{1}'..='\u{8}' | '\u{e}'..='\u{1f}' | '\u{b}' | '\u{7f}' | '\u{8f}' | '\u{9f}'));
                let replacer_space =
                    NMT_REGEX_SPACE.get_or_init(|| Box::new(Regex::new("[\u{0}\u{a}\u{c}\u{d}\u{1680}\u{200B}-\u{200F}\u{2028}\u{2029}\u{2581}\u{feff}\u{fffd}]").unwrap()));
                *text.to_mut() = replacer_space.replace_all(text, " ");
            }
            CaseFold { upper } => {
                if *upper {
                    *text = text.to_uppercase().into();
                } else {
                    *text = text.to_lowercase().into();
                }
            }
            Append { append } => {
                text.to_mut().push_str(append);
            }
            Prepend { prepend } => {
                text.to_mut().insert_str(0, prepend);
            }
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
                    unsafe {
                        text.to_mut()
                            .as_mut_vec()
                            .splice(..0, core::iter::repeat(&buffer).take(left).flatten().copied());
                    };
                }
                if *right > 0 {
                    let mut right = *right as usize;
                    if *pad {
                        let trailing =
                            text.chars().rev().take(right).take_while(|c| c == character).count();
                        right = right.saturating_sub(trailing);
                    }
                    unsafe {
                        text.to_mut()
                            .as_mut_vec()
                            .extend(core::iter::repeat(&buffer).take(right).flatten().copied());
                    };
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
                    text.to_mut().drain(..slice_start);
                }
                if slice_end > 0 {
                    let len = text.len();
                    text.to_mut().drain(len - slice_end..);
                }
            }
            Collapse { character } => {
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
                    .collect();
            }
            Replace {
                pattern,
                replacement,
            } => {
                if pattern.as_ref() == pattern.escape().as_ref() && !replacement.contains("$") {
                    *text.to_mut() = text.replace(pattern.as_ref(), replacement);
                } else {
                    *text.to_mut() = pattern.replace_all(text, replacement);
                }
            }
            CharsMap { .. } => {
                // TODO
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalization_nmt() {
        let mut text = Cow::Borrowed("aaa\u{200D}bbb\u{8f}");
        let normalization = Normalization::NMT;
        normalization.normalize(&mut text);
        assert_eq!(text, "aaa bbb");
    }

    #[test]
    fn test_normalization_case_fold() {
        let mut text = Cow::Borrowed("AAA bbb");
        let normalization = Normalization::CaseFold { upper: false };
        normalization.normalize(&mut text);
        assert_eq!(text, "aaa bbb");

        let mut text = Cow::Borrowed("AAA bbb");
        let normalization = Normalization::CaseFold { upper: true };
        normalization.normalize(&mut text);
        assert_eq!(text, "AAA BBB");
    }

    #[test]
    fn test_normalization_append() {
        let mut text = Cow::Borrowed("aaa");
        let normalization = Normalization::Append {
            append: " bbb".to_string(),
        };
        normalization.normalize(&mut text);
        assert_eq!(text, "aaa bbb");
    }

    #[test]
    fn test_normalization_prepend() {
        let mut text = Cow::Borrowed("bbb");
        let normalization = Normalization::Prepend {
            prepend: "aaa ".to_string(),
        };
        normalization.normalize(&mut text);
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
        normalization.normalize(&mut text);
        assert_eq!(text, "aabbbaaa");

        let mut text = Cow::Borrowed("aba");
        let normalization = Normalization::Extend {
            character: 'a',
            left:      2,
            right:     3,
            pad:       true,
        };
        normalization.normalize(&mut text);
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
        normalization.normalize(&mut text);
        assert_eq!(text, "aba");
    }

    #[test]
    fn test_normalization_collapse() {
        let mut text = Cow::Borrowed("abbbba bbb");
        let normalization = Normalization::Collapse { character: 'b' };
        normalization.normalize(&mut text);
        assert_eq!(text, "aba b");
    }

    #[test]
    fn test_normalization_replace() {
        let mut text = Cow::Borrowed("aba bbb");
        let normalization = Normalization::Replace {
            pattern:     Regex::new(r"b").unwrap(),
            replacement: "a".to_string(),
        };
        normalization.normalize(&mut text);
        assert_eq!(text, "aaa aaa");
    }
}
