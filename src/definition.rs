//! Kitoken definition format.
//!
//! Defines the metadata, model, specials, and configuration of a Kitoken tokenizer.

use core::fmt::Debug;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

use crate::{Configuration, InitializationError, Kitoken, Scores, SpecialVocab, Vocab};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
/// Kitoken tokenizer definition metadata.
pub struct Metadata {
    /// The version of Kitoken that created the definition.
    pub version: String,
    /// The source of the definition. Defaults to "kitoken".
    pub source:  String,
    /// Additional metadata.
    pub meta:    Vec<(String, String)>,
}
impl Default for Metadata {
    #[inline(never)]
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            source:  "kitoken".to_string(),
            meta:    Vec::new(),
        }
    }
}

#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[non_exhaustive]
/// Kitoken tokenizer model.
pub enum Model {
    BytePair {
        /// The encoder vocabulary without special tokens.
        /// Sorted by merge priority.
        vocab: Vocab,
        /// Whether to encode the input as characters.
        chars: bool,
    },
    Unigram {
        /// The encoder vocabulary without special tokens.
        /// Sorted by merge priority.
        vocab:  Vocab,
        /// The scores for each token.
        scores: Scores,
    },
    WordPiece {
        /// The encoder vocabulary without special tokens.
        /// Sorted by merge priority.
        vocab:          Vocab,
        /// The maximum number of characters in a piece.
        max_word_chars: u32,
    },
}
impl Model {
    /// Returns the encoder vocabulary.
    #[inline(always)]
    pub fn vocab(&self) -> &Vocab {
        match self {
            Model::BytePair { vocab, .. } => vocab,
            Model::Unigram { vocab, .. } => vocab,
            Model::WordPiece { vocab, .. } => vocab,
        }
    }

    /// Returns the encoder vocabulary as mutable.
    #[inline(always)]
    pub fn vocab_mut(&mut self) -> &mut Vocab {
        match self {
            Model::BytePair { vocab, .. } => vocab,
            Model::Unigram { vocab, .. } => vocab,
            Model::WordPiece { vocab, .. } => vocab,
        }
    }
}
impl Debug for Model {
    #[inline(never)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Model::BytePair { vocab, chars } => f
                .debug_struct("Model::BytePair")
                .field("vocab", &format!("Vocab({})", vocab.len()))
                .field("chars", chars)
                .finish(),
            Model::Unigram { vocab, scores } => f
                .debug_struct("Model::Unigram")
                .field("vocab", &format!("Vocab({})", vocab.len()))
                .field("scores", &format!("Scores({})", scores.len()))
                .finish(),
            Model::WordPiece {
                vocab,
                max_word_chars,
            } => f
                .debug_struct("Model::WordPiece")
                .field("vocab", &format!("Vocab({})", vocab.len()))
                .field("max_word_chars", max_word_chars)
                .finish(),
        }
    }
}

/// Kitoken tokenizer definition.
///
/// Defines the metadata, model, specials, and configuration of a Kitoken tokenizer.
#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct Definition {
    /// The definition metadata.
    pub meta:     Metadata,
    /// The tokenizer model.
    pub model:    Model,
    /// The special encoder vocabulary. Prioritized over the vocabulary during encoding and decoding.
    /// Sorted by split priority.
    pub specials: SpecialVocab,
    /// The tokenizer configuration.
    pub config:   Configuration,
}
impl TryFrom<Definition> for Kitoken {
    type Error = InitializationError;

    #[inline(always)]
    fn try_from(value: Definition) -> Result<Self, Self::Error> {
        Kitoken::from_definition(value)
    }
}
impl From<Kitoken> for Definition {
    #[inline(always)]
    fn from(value: Kitoken) -> Self {
        value.to_definition()
    }
}
impl Debug for Definition {
    #[inline(never)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Definition")
            .field("meta", &self.meta)
            .field("model", &self.model)
            .field("specials", &format!("SpecialVocab({})", self.specials.len()))
            .field("config", &self.config)
            .finish()
    }
}

impl Kitoken {
    /// Creates a tokenizer from the given definition.
    ///
    /// Returns an error if the config is invalid, the special encoder contains invalid utf-8, the encoder or special encoder contain duplicates,
    /// or the encoder and scores have different lengths in unigram mode.
    ///
    /// See [`Definition`] and [`Kitoken::new`] for more details.
    #[inline(always)]
    pub fn from_definition(definition: Definition) -> Result<Self, InitializationError> {
        let Definition {
            meta,
            model,
            specials,
            config,
        } = definition;
        Self::new(model, specials, config, meta)
    }

    /// Creates a definition from this tokenizer.
    ///
    /// The definition can be used for serialization and initializing the tokenizer with [`Kitoken::from_definition`].
    ///
    /// See [`Definition`] for more details.
    #[inline(never)]
    pub fn to_definition(&self) -> Definition {
        let model = self.encoder.model();
        let mut specials = self.specials.iter().map(|(_, v)| v.clone()).collect::<Vec<_>>();
        specials.sort();
        let config = self.config.clone();
        let meta = self.meta.clone();
        Definition {
            meta,
            model,
            specials,
            config,
        }
    }
}
