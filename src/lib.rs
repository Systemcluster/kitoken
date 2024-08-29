//! **Tokenizer for language models.**
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use kitoken::Kitoken;
//! let encoder = Kitoken::from_file("tests/models/llama2.kit")?;
//!
//! let tokens = encoder.encode("Your future belongs to me.", true)?;
//! let string = String::from_utf8(encoder.decode(&tokens, true)?)?;
//!
//! assert!(string == "Your future belongs to me.");
//! # Ok(())
//! # }
//! ```
//!
//! # Overview
//!
//! Kitoken is a fast and versatile tokenizer for language models with support for BPE, Unigram and WordPiece tokenization.
//!
//! Kitoken is compatible with many existing tokenizer formats,
//! including [SentencePiece](https://github.com/google/sentencepiece), [HuggingFace Tokenizers](https://github.com/huggingface/tokenizers), [OpenAI Tiktoken](https://github.com/openai/tiktoken) and [Mistral Tekken](https://docs.mistral.ai/guides/tokenization/),
//! and provides utilities for converting these formats. See [`convert`] for information about supported the formats and conversion utilities.
//!
//! See [`Kitoken`] for the main entry point and additional information.
//!
//! # Cargo features
//!
//! ### Default features
//!
//! - `std`: Enables standard library features, including reading and writing definitions from and to files.
//! - `serialization`: Enables `serde` implementations and methods for serialization and deserialization of definitions.
//! - `normalization`: Enables all input normalization features. When disabled, individual normalizers can be enabled using the following features:
//!   - `normalization-unicode`: Enables unicode input normalization support. This is required for certain models.
//!     Can be disabled to reduce binary size if unicode normalization is not required.
//!   - `normalization-charsmap`: Enables precompiled charsmap input normalization support. This is required for certain models.
//!     Can be disabled to reduce binary size if charsmap normalization is not required.
//! - `convert`: Enables detection and conversion utilities for common tokenizer data formats. When disabled, individual converters can be enabled using the following features:
//!   - `convert-tokenizers`: Enables conversion from HuggingFace Tokenizers tokenizer definitions.
//!   - `convert-sentencepiece`: Enables conversion from SentencePiece tokenizer definitions.
//!   - `convert-tiktoken`: Enables conversion from OpenAI Tiktoken tokenizer definitions.
//!   - `convert-tekken`: Enables conversion from Mistral Tekken tokenizer definitions.
//!   - `convert-detect`: Enables detection of supported formats during deserialization. Enables the serialization feature.
//! - `regex-perf`: Enables additional regex performance optimizations. Can be disabled to reduce binary size.
//! - `multiversion`: Enables the use of multiversion for generating multiple code paths with different CPU feature utilization.
//!
//! ### Optional features
//!
//! - `split`: Enables additional split features including unicode script splitting.
//!   - `split-unicode-script`: Enables unicode script splitting. This is required for certain models.
//!     Disabled by default since it increases binary size and the majority of models don't require it.
//! - `regex-unicode`: Enables support for additional regex unicode patterns including script and segmentation extensions.
//!   Disabled by default since it increases binary size and the majority of models don't make use of these patterns.
//! - `regex-onig`: Enables use of the `oniguruma` regex engine instead of `fancy-regex`.
//!   Generally not recommended since it has worse runtime performance and adds a dependency on the native `oniguruma` library.
//!   However, it may be useful for certain models that require specific regex behavior that is not supported by or differs with `fancy-regex`.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg_hide))]
#![cfg_attr(docsrs, doc(cfg_hide(doc)))]

extern crate alloc;

mod charsmap;
mod config;
mod decoder;
mod definition;
mod encoder;
mod regex;
mod vocab;

#[cfg(feature = "serialization")]
mod serialization;

pub mod convert;

use alloc::boxed::Box;
use alloc::fmt::Debug;
use alloc::string::String;
use alloc::vec::Vec;
use core::str::Utf8Error;

use derive_more::{Deref, DerefMut};
use hashbrown::HashMap;

pub use crate::charsmap::*;
pub use crate::config::*;
pub use crate::decoder::*;
pub use crate::definition::*;
pub use crate::encoder::*;
pub use crate::regex::*;
pub use crate::vocab::*;

#[cfg(feature = "serialization")]
pub use crate::serialization::*;

/// Errors encountered during initialization.
#[non_exhaustive]
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum InitializationError {
    /// The configuration failed to validate.
    #[cfg_attr(feature = "std", error("invalid config: {0}"))]
    InvalidConfig(ConfigurationError),
    /// The encoder and scores must have the same length in unigram mode.
    #[cfg_attr(
        feature = "std",
        error(
            "encoder and scores must have the same length in unigram mode and every token must have a score"
        )
    )]
    InvalidScores,
    /// The encoder and decoder must have the same length and the encoder must not have duplicates.
    #[cfg_attr(
        feature = "std",
        error("encoder and decoder must have the same length (vocab must not have duplicates)")
    )]
    InvalidEncoder,
    /// The special encoder and decoder must have the same length and the special encoder must not have duplicates.
    #[cfg_attr(
        feature = "std",
        error(
            "special encoder and decoder must have the same length (specials must not have duplicates)"
        )
    )]
    InvalidSpecialEncoder,
    /// The split regex failed to compile.
    #[cfg_attr(feature = "std", error("invalid regex: {0}"))]
    InvalidRegex(String),
    /// The special encoder must contain valid utf-8.
    #[cfg_attr(feature = "std", error("invalid utf-8: {0}"))]
    InvalidUtf8(Utf8Error),
}
impl From<ConfigurationError> for InitializationError {
    fn from(e: ConfigurationError) -> Self {
        Self::InvalidConfig(e)
    }
}
impl From<RegexError> for InitializationError {
    fn from(e: RegexError) -> Self {
        Self::InvalidRegex(e.0)
    }
}
impl From<Utf8Error> for InitializationError {
    fn from(e: Utf8Error) -> Self {
        Self::InvalidUtf8(e)
    }
}

#[derive(Clone, Deref, DerefMut)]
struct SpecialsMap(HashMap<TokenBytes, SpecialToken>);
impl FromIterator<(TokenBytes, SpecialToken)> for SpecialsMap {
    #[inline(always)]
    fn from_iter<I: IntoIterator<Item = (TokenBytes, SpecialToken)>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}
impl Debug for SpecialsMap {
    #[inline(never)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.0.values()).finish()
    }
}

/// Kitoken tokenizer.
/// A fast and versatile tokenizer for language models.
#[derive(Debug)]
pub struct Kitoken {
    encoder: Box<dyn Encoder>,
    decoder: Decoder,

    specials: SpecialsMap,

    extract_split: Regex,
    special_split: Regex,

    config: Configuration,
    meta:   Metadata,
}
impl Kitoken {
    /// Creates a tokenizer from the given encoder, specials, scores and config.
    ///
    /// Returns an error if the config is invalid, the special encoder contains invalid utf-8, the encoder or special encoder contain duplicates,
    /// or the encoder and scores have different lengths in unigram mode.
    #[inline(never)]
    pub fn new(
        model: Model, specials: SpecialVocab, config: Configuration, meta: Metadata,
    ) -> Result<Self, InitializationError> {
        if let Err(error) = config.validate() {
            return Err(InitializationError::InvalidConfig(error));
        }

        let special_split = Regex::new(
            &specials
                .iter()
                .filter(|special| !special.extract)
                .map(|special| core::str::from_utf8(&special.bytes))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .map(|s| regex::escape(s))
                .collect::<Vec<_>>()
                .join("|"),
        )?;
        let extract_split = Regex::new(
            &specials
                .iter()
                .filter(|special| special.extract)
                .map(|special| core::str::from_utf8(&special.bytes))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .map(|s| regex::escape(s))
                .collect::<Vec<_>>()
                .join("|"),
        )?;

        let (encoder, decoder) = match model {
            Model::BytePair { vocab, chars } => {
                let decoder = Decoder::new(&vocab, &specials, &config);
                let encoder = Box::new(BytePair::new(vocab, &specials, &config, chars)?) as _;
                (encoder, decoder)
            }
            Model::Unigram { vocab, scores } => {
                let decoder = Decoder::new(&vocab, &specials, &config);
                let encoder = Box::new(Unigram::new(vocab, &specials, &config, scores)?) as _;
                (encoder, decoder)
            }
            Model::WordPiece {
                vocab,
                max_word_chars,
            } => {
                let decoder = Decoder::new(&vocab, &specials, &config);
                let encoder =
                    Box::new(WordPiece::new(vocab, &specials, &config, max_word_chars)) as _;
                (encoder, decoder)
            }
        };

        let specials_len = specials.len();
        let specials = specials
            .into_iter()
            .map(|special| (special.bytes.clone(), special))
            .collect::<SpecialsMap>();
        if specials_len != specials.len() {
            return Err(InitializationError::InvalidSpecialEncoder);
        }

        Ok(Self {
            encoder,
            decoder,
            specials,
            special_split,
            extract_split,
            config,
            meta,
        })
    }

    /// Encodes the given text into a sequence of tokens.
    ///
    /// If `encode_specials` is `true`, control tokens are tokenized with their ids, otherwise they are tokenized with the regular vocabulary.
    ///
    /// Returns a list of tokens, or an error if no token for a part exists in the encoder, and the configuration has no unknown token or skip fallback set.
    #[inline(never)]
    pub fn encode(
        &self, text: impl AsRef<str>, encode_specials: bool,
    ) -> Result<Vec<TokenId>, EncodeError> {
        let text = text.as_ref();
        let mut extracted = if self.extract_split.is_empty() {
            Vec::with_capacity(0)
        } else {
            let mut extracted = self.extract_split.find_iter(text);
            extracted.reverse();
            extracted
        };
        let mut parts = Vec::with_capacity(extracted.len() * 2 + 1);
        let mut posit = 0;
        while posit < text.len() {
            if let Some(next) = extracted.pop() {
                if next.0 > posit {
                    let mut text = text[posit..next.0].into();
                    self.config.normalize(&mut text);
                    parts.push(TextPart {
                        text,
                        special: Token::INVALID,
                    })
                }
                let special = &self.specials[text[next.0..next.1].as_bytes()];
                parts.push(TextPart {
                    text:    text[next.0..next.1].into(),
                    special: if special.kind != SpecialTokenKind::Control || encode_specials {
                        special.id
                    } else {
                        Token::INVALID
                    },
                });
                posit = next.1;
            } else {
                let mut rest = text[posit..text.len()].into();
                self.config.normalize(&mut rest);
                parts.push(TextPart {
                    text:    rest,
                    special: Token::INVALID,
                });
                posit = text.len();
            }
        }
        let mut parts = parts.iter().fold(Vec::with_capacity(text.len() / 6), |mut acc, part| {
            let mut specials = if part.special != Token::INVALID {
                acc.push(part.clone());
                return acc;
            } else if self.special_split.is_empty() {
                Vec::with_capacity(0)
            } else {
                let mut specials = self
                    .special_split
                    .find_iter(&part.text)
                    .into_iter()
                    .map(|(start, end)| {
                        (start, end, &self.specials[part.text[start..end].as_bytes()])
                    })
                    .filter(|(_, _, special)| {
                        special.kind != SpecialTokenKind::Control || encode_specials
                    })
                    .collect::<Vec<_>>();
                specials.reverse();
                specials
            };
            let mut posit = 0;
            while posit < part.text.len() {
                if let Some(next) = specials.pop() {
                    if next.0 > posit {
                        for (start, end) in self.config.split(&part.text[posit..next.0]) {
                            if end > start {
                                acc.push(TextPart {
                                    text:    part.text[posit + start..posit + end].into(),
                                    special: Token::INVALID,
                                });
                            }
                        }
                    }
                    acc.push(TextPart {
                        text:    part.text[next.0..next.1].into(),
                        special: next.2.id,
                    });
                    posit = next.1;
                } else {
                    for (start, end) in self.config.split(&part.text[posit..part.text.len()]) {
                        if end > start {
                            acc.push(TextPart {
                                text:    part.text[posit + start..posit + end].into(),
                                special: Token::INVALID,
                            });
                        }
                    }
                    posit = part.text.len();
                }
            }
            acc
        });
        let mut result = self.encoder.encode(text, &mut parts)?;
        self.config.process(&mut result);
        Ok(result)
    }

    /// Decodes the given sequence of tokens into text.
    ///
    /// If `decode_specials` is `false`, control tokens are ignored.
    ///
    /// Returns a list of bytes, or an error if no byte sequence for a token exists.
    #[inline(never)]
    pub fn decode(
        &self, tokens: impl AsRef<[TokenId]>, decode_specials: bool,
    ) -> Result<Vec<u8>, DecodeError> {
        let tokens = tokens.as_ref();
        let mut result = self.decoder.decode(tokens, decode_specials)?;
        self.config.decode(&mut result);
        Ok(result)
    }
}
