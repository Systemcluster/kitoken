//! Kitoken definition format.

use core::cmp::Ordering;
use core::fmt::Debug;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

use crate::{Configuration, InitializationError, Kitoken, Mode, Scores, SpecialVocab, Vocab};

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
    /// The definition was converted from a Tokenizers definition.
    Tokenizers,
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
#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct Definition {
    /// The definition metadata.
    pub meta:     Metadata,
    /// The encoder vocabulary without special tokens.
    /// Sorted by merge priority.
    pub vocab:    Vocab,
    /// The special encoder vocabulary. Prioritized over the vocabulary during encoding and decoding.
    /// Sorted by split priority.
    pub specials: SpecialVocab,
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
impl Debug for Definition {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Definition")
            .field("meta", &self.meta)
            .field("vocab", &format!("Vocab({})", self.vocab.len()))
            .field("specials", &format!("Vocab({})", self.specials.len()))
            .field("scores", &format!("Scores({})", self.vocab.len()))
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
        let mut vocab = self.encoder.iter().map(|(k, v)| (k.clone(), *v)).collect::<Vec<_>>();
        if self.config.mode == Mode::Unigram {
            vocab.sort_by(|(_, a), (_, b)| match a.score.partial_cmp(&b.score).unwrap() {
                Ordering::Equal => a.token.cmp(&b.token),
                other => other,
            });
        } else {
            vocab.sort_by(|(ta, a), (tb, b)| {
                let sa = self.score_encoder.get(ta).copied().unwrap();
                let sb = self.score_encoder.get(tb).copied().unwrap();
                match sa.score.cmp(&sb.score) {
                    Ordering::Equal => a.token.cmp(&b.token),
                    other => other,
                }
            });
        };
        let scores = if self.config.mode == Mode::Unigram {
            vocab.iter().map(|(_, v)| v.score).collect::<Scores>()
        } else {
            Scores::new()
        };
        let vocab = vocab.into_iter().map(|(k, v)| (k, v.token)).collect();
        let mut specials = self.special_encoder.iter().map(|(_, v)| v.clone()).collect::<Vec<_>>();
        specials.sort();
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
