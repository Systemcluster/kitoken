//! Configuration for the tokenizer.

use alloc::string::String;
use alloc::vec::Vec;

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

use crate::normalization::*;

/// Tokenization mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub enum Mode {
    /// The original BPE algorithm, which merges pairs of bytes.
    BytePair,
    /// A variant of the BPE algorithm, which merges pairs of characters before falling back to `BytePair`.
    CharPair,
    /// The Unigram subword algorithm.
    Unigram,
}
impl Default for Mode {
    fn default() -> Self {
        Self::BytePair
    }
}

/// Unicode normalization scheme.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub enum Normalization {
    /// Unicode normalization form C.
    NFC,
    /// Unicode normalization form D.
    NFD,
    /// Unicode normalization form KC.
    NFKC,
    /// Unicode normalization form KD.
    NFKD,
    /// NMT normalization.
    NMT,
    /// Case folding.
    CaseFold {
        upper: bool,
    },
    /// Trim whitespace.
    TrimWhitespace {
        left:  bool,
        right: bool,
    },
    /// Add whitespace.
    AddWhitespace {
        left:  bool,
        right: bool,
    },
    /// Strip accents.
    StripAccents,
    Precompiled,
    Replace {
        pattern:     String,
        replacement: String,
    },
    Prepend {
        prepend: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub enum Split {
    /// Split on whitespace.
    Whitespace,
    /// Split on whitespace and punctuation.
    WhitespacePunctuation,
    /// A custom split regex.
    Custom(String),
}

// /// Normalization configuration for encoding and decoding.
// #[derive(Debug, Clone, Default, PartialEq, Eq)]
// #[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
// pub struct Normalization {
//     /// Unicode normalization scheme for inputs.
//     pub unicode:             UnicodeNormalization,
//     /// Whether to trim whitespace from the beginning and end during encoding and encoding.
//     pub trim_whitespace:     bool,
//     /// Whether to collapse whitespace in inputs.
//     pub collapse_whitespace: bool,
//     /// Whether to add prefix whitespace to inputs.
//     pub prefix_whitespace:   bool,
//     /// Whether to collapse unknown tokens in outputs.
//     pub collapse_unknown:    bool,
// }

/// Special tokens.
///
/// The first value of each member is the token id, the second value are the bytes of the token.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct Specials {
    /// The unknown token.
    pub unk:  Option<(u32, Vec<u8>)>,
    /// The padding token.
    pub pad:  Option<(u32, Vec<u8>)>,
    /// The beginning of sequence token.
    pub bos:  Option<(u32, Vec<u8>)>,
    /// The end of sequence token.
    pub eos:  Option<(u32, Vec<u8>)>,
    /// The separator token.
    pub sep:  Option<(u32, Vec<u8>)>,
    /// The mask token.
    pub mask: Option<(u32, Vec<u8>)>,
}

/// Configuration for the tokenizer.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct Configuration {
    /// The regex used to split text into parts during encoding.
    pub split:         String,
    /// The tokenization mode.
    pub mode:          Mode,
    /// The normalization scheme.
    pub normalization: Normalization,
    /// The special tokens.
    pub specials:      Specials,
}

/// Errors returned when the configuration fails to validate.
#[non_exhaustive]
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum ValidationError {
    /// The unknown token `specials.unk` must be set when `collapse_unknown` is enabled.
    #[cfg_attr(
        feature = "std",
        error("unknown_token_id must be set when collapse_unknown is enabled")
    )]
    InvalidCollapseUnknown,
    /// The normalization scheme is not supported. The `unicode-normalization` feature must be enabled to use normalization.
    #[cfg_attr(
        feature = "std",
        error(
            "unsupported normalization: {0:?} (the `unicode-normalization` feature must be enabled)"
        )
    )]
    InvalidNormalization(UnicodeNormalization),
}

impl Configuration {
    /// Validates the configuration.
    ///
    /// Returns an error if the configuration is invalid.
    pub fn validate(&self) -> Result<(), ValidationError> {
        let normalization = &self.normalization;
        if normalization.collapse_unknown && self.specials.unk.is_none() {
            return Err(ValidationError::InvalidCollapseUnknown);
        }
        if normalize_unicode_in_place(&mut String::new(), normalization.unicode).is_none() {
            return Err(ValidationError::InvalidNormalization(normalization.unicode));
        }
        Ok(())
    }

    /// Returns whether normalization of input data before encoding is enabled.
    pub fn normalize_encode_input_enabled(&self) -> bool {
        let normalization = &self.normalization;
        normalization.unicode != UnicodeNormalization::None
            || normalization.collapse_whitespace
            || normalization.trim_whitespace
            || normalization.prefix_whitespace
    }

    pub(crate) fn normalize_encode_input(&self, text: &mut String) {
        let normalization = &self.normalization;
        if normalization.unicode != UnicodeNormalization::None {
            normalize_unicode_in_place(text, normalization.unicode);
        }
        if normalization.collapse_whitespace {
            collapse_whitespace_in_place(text);
        }
        if normalization.trim_whitespace {
            trim_whitespace_in_place(unsafe { text.as_mut_vec() }, None);
        }
        if normalization.prefix_whitespace {
            text.insert(0, ' ');
        }
    }

    /// Returns whether normalization of output data after encoding is enabled.
    pub fn normalize_encode_output_enabled(&self) -> bool {
        let normalization = &self.normalization;
        normalization.collapse_unknown && self.specials.unk.is_some()
    }

    pub(crate) fn normalize_encode_output(&self, tokens: &mut Vec<u32>) {
        let normalization = &self.normalization;
        if normalization.collapse_unknown {
            if let Some((unk_id, _)) = self.specials.unk {
                collapse_tokens_in_place(tokens, unk_id);
            }
        }
    }

    /// Returns whether normalization of output data after decoding is enabled.
    pub fn normalize_decode_output_enabled(&self) -> bool {
        let normalization = &self.normalization;
        normalization.trim_whitespace || normalization.prefix_whitespace
    }

    pub(crate) fn normalize_decode_output(&self, text: &mut Vec<u8>) {
        let normalization = &self.normalization;
        if normalization.trim_whitespace {
            trim_whitespace_in_place(text, self.specials.unk.as_ref().map(|(_, unk)| unk.as_ref()));
        }
        if normalization.prefix_whitespace
            && !text.is_empty()
            && text[0] == b' '
            && (self.specials.unk.is_none()
                || !text.starts_with(self.specials.unk.as_ref().map(|(_, unk)| unk).unwrap()))
        {
            text.remove(0);
        }
    }
}
