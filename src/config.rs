//! Configuration for the tokenizer.

use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

mod decoding;
mod normalization;
mod processing;
mod split;

pub use decoding::*;
pub use normalization::*;
pub use processing::*;
pub use split::*;

use crate::TokenId;

/// Tokenization mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub enum Mode {
    /// A variation of the original BPE algorithm. Merges inputs starting from individual bytes.
    BytePair,
    /// A variation of the modified BPE algorithm. Merges inputs starting from individual characters.
    CharPair,
    /// The Unigram subword algorithm.
    Unigram,
    /// The WordPiece subword algorithm.
    WordPiece,
}
impl Default for Mode {
    fn default() -> Self {
        Self::CharPair
    }
}

/// Tokenization mode fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub enum ModeFallback {
    /// Skip pieces that cannot be tokenized.
    Skip,
    /// Replace pieces that cannot be tokenized with the unknown token.
    Unknown,
    /// Merge pieces that cannot be tokenized starting from individual bytes.
    Bytes,
}


/// Template insertion position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub enum InsertionPosition {
    WordStart,
    WordContinuation,
    WordEnd,
    SequenceStart,
    SequenceContinuation,
    SequenceEnd,
    SubSequenceStart,
    SubSequenceContinuation,
    SubSequenceEnd,
}

/// Output template.
///
/// Specifies additional data to insert into the tokenization input.
/// The `content` field contains the data to insert, and the `position` field specifies where to insert it.
///
/// Only [`InsertionPosition::WordEnd`] and [`InsertionPosition::WordContinuation`] are used during tokenization.
/// The other positions exist for manual lookup and future use.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct Template {
    pub content:  String,
    pub position: InsertionPosition,
}

/// Errors returned when the configuration fails to validate.
#[non_exhaustive]
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum ConfigurationError {
    /// The feature required for the configuration is not enabled.
    #[cfg_attr(feature = "std", error("required feature not enabled: {0}"))]
    FeatureDisabled(String),
}

/// Configuration for the tokenizer.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct Configuration {
    /// The tokenization mode.
    pub mode:          Mode,
    /// The tokenization mode fallback.
    pub fallback:      Vec<ModeFallback>,
    /// The input normalization scheme.
    pub normalization: Vec<Normalization>,
    /// The pre-tokenization split behavior.
    pub split:         Vec<Split>,
    /// The post-tokenization processing.
    pub processing:    Vec<Processing>,
    /// The post-decode processing.
    pub decoding:      Vec<Decoding>,
    /// The input templates.
    pub templates:     Vec<Template>,
}

impl Configuration {
    /// Validates the configuration.
    ///
    /// Returns an error if the configuration is invalid.
    #[inline(never)]
    pub fn validate(&self) -> Result<(), ConfigurationError> {
        #[cfg(not(feature = "normalization-unicode"))]
        if self
            .normalization
            .iter()
            .any(|norm| matches!(norm, Normalization::Unicode { .. }))
        {
            use alloc::string::ToString;
            return Err(ConfigurationError::FeatureDisabled("normalization-unicode".to_string()));
        }
        #[cfg(not(feature = "normalization-charsmap"))]
        if self
            .normalization
            .iter()
            .any(|norm| matches!(norm, Normalization::CharsMap { .. }))
        {
            use alloc::string::ToString;
            return Err(ConfigurationError::FeatureDisabled("normalization-charsmap".to_string()));
        }
        #[cfg(not(feature = "split-unicode-script"))]
        if self.split.iter().any(|split| matches!(split, Split::UnicodeScript)) {
            use alloc::string::ToString;
            return Err(ConfigurationError::FeatureDisabled("split-unicode-script".to_string()));
        }
        Ok(())
    }

    /// Normalizes the input before tokenization.
    #[inline(never)]
    pub fn normalize(&self, text: &mut Cow<str>) {
        if text.is_empty() {
            return;
        }
        for norm in &self.normalization {
            norm.normalize(text);
        }
    }

    /// Splits the input into parts to tokenize.
    #[inline(never)]
    pub fn split(&self, text: &str) -> Vec<(usize, usize)> {
        if text.is_empty() {
            return Vec::new();
        }
        if self.split.is_empty() {
            return Vec::from([(0, text.len())]);
        }
        if self.split.len() == 1 {
            return self.split[0].split(text);
        }
        let mut matches = Vec::from([(0, text.len())]);
        for split in &self.split {
            let split_matches = matches.iter().map(|&(start, end)| {
                let mut split_match = split.split(&text[start..end]);
                split_match.iter_mut().for_each(|(split_start, split_end)| {
                    *split_start += start;
                    *split_end += start;
                });
                split_match
            });
            matches = split_matches.flatten().collect();
        }
        matches
    }

    /// Processes the tokens after tokenization.
    #[inline(never)]
    pub fn process(&self, tokens: &mut Vec<TokenId>) {
        if tokens.is_empty() {
            return;
        }
        for processing in &self.processing {
            processing.process(tokens);
        }
    }

    /// Postprocesses the bytes after detokenization.
    #[inline(never)]
    pub fn decode(&self, tokens: &mut Vec<u8>) {
        if tokens.is_empty() {
            return;
        }
        for decoding in &self.decoding {
            decoding.decode(tokens);
        }
    }
}
