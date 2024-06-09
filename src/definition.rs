//! Kitoken definition format.

use alloc::vec::Vec;

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

use crate::{Configuration, InitializationError, Kitoken, Mode, Scores, Vocab};

/// The source of the definition.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub enum DefinitionSource {
    /// The definition was created by the user.
    None,
    /// The definition was converted from a Sentencepiece model.
    Sentencepiece,
    /// The definition was converted from a Tiktoken definition.
    Tiktoken,
    /// The definition was converted from a Huggingface tokenizer definition.
    Huggingface,
    /// The definition was converted from an unspecified source.
    #[serde(untagged)]
    Other,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
/// Kitoken tokenizer definition metadata.
pub struct Metadata {
    /// The version of Kitoken that created the definition.
    pub version: String,
    /// The source of the definition.
    pub source:  DefinitionSource,
    /// Additional metadata.
    pub meta:    Vec<(String, String)>,
}
impl Default for Metadata {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            source:  DefinitionSource::None,
            meta:    Vec::new(),
        }
    }
}

/// Kitoken tokenizer definition.
///
/// Used for initializing the tokenizer and for serialization and deserialization.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct Definition {
    /// The definition metadata.
    pub meta:     Metadata,
    /// The encoder vocabulary without special tokens.
    /// Sorted by merge priority.
    pub vocab:    Vocab,
    /// The special encoder vocabulary. Prioritized over the vocabulary during encoding and decoding.
    /// Sorted by split priority.
    pub specials: Vocab,
    /// The scores for each token.
    /// Only used in unigram mode.
    pub scores:   Scores,
    /// The tokenizer configuration.
    pub config:   Configuration,
}

impl TryFrom<Definition> for Kitoken {
    type Error = InitializationError;

    fn try_from(value: Definition) -> Result<Self, Self::Error> {
        Kitoken::from_definition(value)
    }
}

impl From<Kitoken> for Definition {
    fn from(value: Kitoken) -> Self {
        value.to_definition()
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
            vocab,
            specials,
            scores,
            config,
            ..
        } = definition;
        Self::new(vocab, specials, scores, config).map(|mut s| {
            s.meta = meta;
            s
        })
    }

    /// Creates a definition from this tokenizer.
    ///
    /// The definition can be used for serialization and initializing the tokenizer with [`Kitoken::from_definition`].
    ///
    /// See [`Definition`] for more details.
    #[inline(never)]
    pub fn to_definition(&self) -> Definition {
        let mut vocab = self.encoder.iter().map(|(k, v)| (k.clone(), v)).collect::<Vec<_>>();
        vocab.sort_by(|(_, a), (_, b)| a.score.partial_cmp(&b.score).unwrap());
        let scores = if self.config.mode == Mode::Unigram {
            vocab.iter().map(|(_, v)| v.score).collect::<Scores>()
        } else {
            Scores::new()
        };
        let vocab = vocab.into_iter().map(|(k, v)| (k, v.token)).collect();
        let mut specials =
            self.special_encoder.iter().map(|(k, v)| (k.clone(), *v)).collect::<Vec<_>>();
        specials.sort_by(|(_, a), (_, b)| a.score.partial_cmp(&b.score).unwrap());
        let specials = specials.into_iter().map(|(k, v)| (k, v.token)).collect();
        let config = self.config.clone();
        let meta = self.meta.clone();
        Definition {
            meta,
            vocab,
            specials,
            scores,
            config,
        }
    }
}
