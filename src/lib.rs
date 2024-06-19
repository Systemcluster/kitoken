//! **Tokenizer for language models.**
//!
//! ```no_run
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
//! Kitoken is a fast and versatile tokenizer for language models with support for BPE and Unigram tokenization.
//!
//! Kitoken is compatible with many existing tokenizer formats,
//! including [SentencePiece](https://github.com/google/sentencepiece), [HuggingFace Tokenizers](https://github.com/huggingface/tokenizers) and [OpenAI Tiktoken](https://github.com/openai/tiktoken),
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
//! - `unicode-normalization`: Enables unicode input normalization support. This is required for certain models.
//!   Can be disabled to reduce binary size if normalization is not required.
//! - `convert`: Enables conversion utilities for common tokenizer data formats. When disabled, individual converters can be enabled using the following features:
//!   - `convert-tokenizers`: Enables conversion from HuggingFace Tokenizers tokenizer definitions.
//!   - `convert-sentencepiece`: Enables conversion from SentencePiece tokenizer definitions.
//!   - `convert-tiktoken`: Enables conversion from tiktoken tokenizer definitions.
//! - `regex-perf`: Enables additional regex performance optimizations. Can be disabled to reduce binary size.
//!
//! ### Optional features
//!
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
mod definition;
mod regex;

#[cfg(feature = "serialization")]
mod serialization;

pub mod convert;

use alloc::borrow::Cow;
use alloc::fmt::Debug;
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::str::Utf8Error;

use bstr::ByteSlice;
use debug_ignore::DebugIgnore;
use hashbrown::HashMap;
use orx_priority_queue::{DaryHeapOfIndices, PriorityQueue, PriorityQueueDecKey};

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

pub use crate::charsmap::*;
pub use crate::config::*;
pub use crate::definition::*;
pub use crate::regex::*;

#[cfg(feature = "serialization")]
pub use crate::serialization::*;

/// Special token type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub enum SpecialTokenKind {
    /// Placeholder for unknown tokens during encoding.
    Unknown,
    /// Control tokens like padding, beginning of sequence, end of sequence, and similar.
    Control,
    /// Priotitized during encoding.
    Priority,
}

/// Special token.
#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct SpecialToken {
    /// The token id. The numeric value of the token.
    pub id:      u32,
    /// The token bytes. The byte sequence of the token.
    pub bytes:   Vec<u8>,
    /// The token type.
    pub kind:    SpecialTokenKind,
    /// Common identifier for the token. Used for control tokens like "cls", "sep", "pad", "mask" and similar.
    pub ident:   Option<String>,
    /// The token score. Used for prioritizing special tokens during encoding.
    pub score:   f32,
    /// Whether the token should be split pre-normalization.
    pub extract: bool,
}
impl Eq for SpecialToken {}
impl PartialOrd for SpecialToken {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for SpecialToken {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut ord = self.kind.cmp(&other.kind);
        if ord == Ordering::Equal {
            ord = self.score.partial_cmp(&other.score).unwrap();
        }
        if ord == Ordering::Equal {
            ord = self.id.cmp(&other.id);
        }
        ord
    }
}
impl Debug for SpecialToken {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SpecialToken")
            .field("id", &self.id)
            .field("bytes", &self.bytes.as_bstr())
            .field("kind", &self.kind)
            .field("ident", &self.ident)
            .field("score", &self.score)
            .field("extract", &self.extract)
            .finish()
    }
}

/// List of token bytes and their id.
pub type Vocab = Vec<(Vec<u8>, u32)>;
/// List of special tokens.
pub type SpecialVocab = Vec<SpecialToken>;
/// List of token ids and their score.
pub type Scores = Vec<f32>;

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
    InvalidRegex(RegexError),
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
        Self::InvalidRegex(e)
    }
}
impl From<Utf8Error> for InitializationError {
    fn from(e: Utf8Error) -> Self {
        Self::InvalidUtf8(e)
    }
}

/// Errors encountered during encoding.
#[non_exhaustive]
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum EncodeError {
    /// A piece could not be encoded.
    #[cfg_attr(feature = "std", error("invalid piece {0:?}"))]
    InvalidPiece(Vec<u8>),
}

/// Errors encountered during decoding.
#[non_exhaustive]
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum DecodeError {
    /// A token could not be decoded.
    #[cfg_attr(feature = "std", error("invalid token {0}"))]
    InvalidToken(u32),
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Token {
    token: u32,
    score: f32,
}
#[derive(Debug, Clone, Copy)]
pub(crate) struct Score {
    score: u32,
}

pub(crate) type ScoreMap = HashMap<Vec<u8>, Score>;
pub(crate) type EncoderMap = HashMap<Vec<u8>, Token>;
pub(crate) type SpecialEncoderMap = HashMap<Vec<u8>, SpecialToken>;
pub(crate) type DecoderMap = HashMap<u32, Vec<u8>>;
pub(crate) type SpecialDecoderMap = HashMap<u32, SpecialToken>;

pub(crate) type PieceHeap = DaryHeapOfIndices<u32, LinkedPart, 4>;

const ENCODE_LINEAR_LIMIT: usize = 192;
const ENCODE_BUFFER_SIZE: usize = 256;

/// Kitoken tokenizer.
/// A fast and versatile tokenizer for language models.
#[derive(Debug)]
pub struct Kitoken {
    encoder:       DebugIgnore<EncoderMap>,
    score_encoder: DebugIgnore<ScoreMap>,
    decoder:       DebugIgnore<DecoderMap>,

    special_encoder: DebugIgnore<SpecialEncoderMap>,
    special_decoder: DebugIgnore<SpecialDecoderMap>,

    extract_split: Regex,
    special_split: Regex,

    config:  Configuration,
    meta:    Metadata,
    unknown: Option<(u32, Vec<u8>)>,

    max_token_bytes: usize,
    min_token_bytes: usize,
}
impl Kitoken {
    /// Creates a tokenizer from the given encoder, specials, scores and config.
    ///
    /// Returns an error if the config is invalid, the special encoder contains invalid utf-8, the encoder or special encoder contain duplicates,
    /// or the encoder and scores have different lengths in unigram mode.
    #[inline(never)]
    pub fn new(
        vocab: impl Into<Vocab>, specials: impl Into<SpecialVocab>, scores: impl Into<Scores>,
        config: Configuration,
    ) -> Result<Self, InitializationError> {
        if let Err(error) = config.validate() {
            return Err(InitializationError::InvalidConfig(error));
        }

        let vocab: Vocab = vocab.into();
        let specials: SpecialVocab = specials.into();
        let scores: Scores = scores.into();

        if config.mode == Mode::Unigram && vocab.len() != scores.len() {
            return Err(InitializationError::InvalidScores);
        }

        let decoder = vocab.iter().map(|(k, v)| (*v, k.clone())).collect::<DecoderMap>();
        if vocab.len() != decoder.len() {
            return Err(InitializationError::InvalidEncoder);
        }
        let special_decoder = specials
            .iter()
            .map(|special| (special.id, special.clone()))
            .collect::<SpecialDecoderMap>();
        if specials.len() != special_decoder.len() {
            return Err(InitializationError::InvalidSpecialEncoder);
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

        let score_encoder = vocab
            .iter()
            .enumerate()
            .map(|(i, (k, _))| (k.clone(), Score { score: i as u32 }))
            .collect::<ScoreMap>();
        let encoder = vocab
            .into_iter()
            .enumerate()
            .map(|(i, (k, v))| {
                if config.mode == Mode::Unigram {
                    (k, Token {
                        token: v,
                        score: scores[i],
                    })
                } else {
                    (k, Token {
                        token: v,
                        score: 0.0,
                    })
                }
            })
            .collect::<EncoderMap>();

        let unknown = specials
            .iter()
            .find(|special| special.kind == SpecialTokenKind::Unknown)
            .map(|special| (special.id, special.bytes.clone()));

        let special_encoder = specials
            .into_iter()
            .map(|special| (special.bytes.clone(), special))
            .collect::<SpecialEncoderMap>();

        let max_token_bytes = encoder.keys().map(|k| k.len()).max().unwrap().max(1);
        let min_token_bytes = encoder.keys().map(|k| k.len()).min().unwrap().max(1);
        let meta = Metadata::default();

        Ok(Self {
            score_encoder: DebugIgnore(score_encoder),
            encoder: DebugIgnore(encoder),
            decoder: DebugIgnore(decoder),
            special_encoder: DebugIgnore(special_encoder),
            special_decoder: DebugIgnore(special_decoder),
            special_split,
            extract_split,
            config,
            meta,
            unknown,
            max_token_bytes,
            min_token_bytes,
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
    ) -> Result<Vec<u32>, EncodeError> {
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
                        special: false,
                    })
                }
                parts.push(TextPart {
                    text:    text[next.0..next.1].into(),
                    special: true,
                });
                posit = next.1;
            } else {
                let mut rest = text[posit..text.len()].into();
                self.config.normalize(&mut rest);
                parts.push(TextPart {
                    text:    rest,
                    special: false,
                });
                posit = text.len();
            }
        }
        let parts = parts.iter().fold(Vec::with_capacity(text.len() / 6), |mut acc, part| {
            let mut specials = if part.special {
                if encode_specials
                    || self.special_encoder[part.text.as_bytes()].kind != SpecialTokenKind::Control
                {
                    acc.push(part.clone());
                    return acc;
                }
                Vec::with_capacity(0)
            } else if self.special_split.is_empty() {
                Vec::with_capacity(0)
            } else {
                let mut specials = self
                    .special_split
                    .find_iter(&part.text)
                    .into_iter()
                    .filter(|special| {
                        if encode_specials {
                            true
                        } else {
                            self.special_encoder
                                .get(part.text[special.0..special.1].as_bytes())
                                .filter(|special| special.kind != SpecialTokenKind::Control)
                                .is_some()
                        }
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
                                    special: false,
                                });
                            }
                        }
                    }
                    acc.push(TextPart {
                        text:    part.text[next.0..next.1].into(),
                        special: true,
                    });
                    posit = next.1;
                } else {
                    for (start, end) in self.config.split(&part.text[posit..part.text.len()]) {
                        if end > start {
                            acc.push(TextPart {
                                text:    part.text[posit + start..posit + end].into(),
                                special: false,
                            });
                        }
                    }
                    posit = part.text.len();
                }
            }
            acc
        });
        let mut result = self.encode_parts(text, &parts, encode_specials)?;
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
        &self, tokens: impl AsRef<[u32]>, decode_specials: bool,
    ) -> Result<Vec<u8>, DecodeError> {
        let tokens = tokens.as_ref();
        let mut result = Vec::<u8>::with_capacity(tokens.len() * self.max_token_bytes);
        for token in tokens {
            let bytes = self.decoder.get(token);
            if let Some(bytes) = bytes {
                result.extend(bytes);
            } else if let Some(special) = self.special_decoder.get(token) {
                if special.kind != SpecialTokenKind::Control || decode_specials {
                    result.extend(&special.bytes);
                }
            } else {
                return Err(DecodeError::InvalidToken(*token));
            }
        }
        self.config.decode(&mut result);
        Ok(result)
    }

    /// Encodes the given piece into a sequence of tokens.
    ///
    /// If `encode_specials` is `true`, control tokens are tokenized with their ids, otherwise they are tokenized with the regular vocabulary.
    ///
    /// Returns an error if no token for a part exists in the encoder, and the configuration has no unknown token or skip fallback set.
    #[inline(never)]
    fn encode_parts(
        &self, text: &str, parts: &[TextPart], encode_specials: bool,
    ) -> Result<Vec<u32>, EncodeError> {
        if parts.is_empty() {
            return Ok(Vec::new());
        }
        let mut result =
            Vec::with_capacity(text.len() / self.min_token_bytes + self.max_token_bytes);
        match self.config.mode {
            Mode::BytePair => {
                let mut buffer = Vec::with_capacity(ENCODE_BUFFER_SIZE);
                for part in parts {
                    let piece = &part.text;
                    if part.special {
                        let special = &self.special_encoder[piece.as_bytes()];
                        if special.kind != SpecialTokenKind::Control || encode_specials {
                            result.push(special.id);
                            continue;
                        }
                    }
                    if piece.len() <= self.max_token_bytes && piece.len() >= self.min_token_bytes {
                        if let Some(&token) = self.encoder.get(piece.as_bytes()) {
                            result.push(token.token);
                            continue;
                        }
                    }
                    if piece.len() > ENCODE_LINEAR_LIMIT {
                        self.encode_pairs_heap(
                            piece.as_bytes(),
                            &mut buffer,
                            &mut result,
                            (0..piece.len()).map(|i| (i as _, 1)),
                            &self.config.fallback,
                        )?;
                    } else {
                        self.encode_pairs(
                            piece.as_bytes(),
                            &mut buffer,
                            &mut result,
                            0..piece.len(),
                            &self.config.fallback,
                        )?;
                    }
                    buffer.clear();
                }
            }
            Mode::CharPair => {
                let mut buffer = Vec::with_capacity(ENCODE_BUFFER_SIZE);
                let mut indices = Vec::with_capacity(ENCODE_BUFFER_SIZE);
                for part in parts {
                    let piece = &part.text;
                    if part.special {
                        let special = &self.special_encoder[piece.as_bytes()];
                        if special.kind != SpecialTokenKind::Control || encode_specials {
                            result.push(special.id);
                            continue;
                        }
                    }
                    if piece.len() <= self.max_token_bytes && piece.len() >= self.min_token_bytes {
                        if let Some(&token) = self.encoder.get(piece.as_bytes()) {
                            result.push(token.token);
                            continue;
                        }
                    }
                    indices.extend(piece.char_indices());
                    if indices.len() > ENCODE_LINEAR_LIMIT {
                        self.encode_pairs_heap(
                            piece.as_bytes(),
                            &mut buffer,
                            &mut result,
                            indices.drain(..).map(|(i, c)| (i as _, c.len_utf8() as _)),
                            &self.config.fallback,
                        )?;
                    } else {
                        self.encode_pairs(
                            piece.as_bytes(),
                            &mut buffer,
                            &mut result,
                            indices.drain(..).map(|(i, _)| i),
                            &self.config.fallback,
                        )?;
                    }
                    buffer.clear();
                }
            }
            Mode::Unigram => {
                let mut buffer = Vec::with_capacity(ENCODE_BUFFER_SIZE);
                for part in parts {
                    let piece = &part.text;
                    if part.special {
                        let special = &self.special_encoder[piece.as_bytes()];
                        if special.kind != SpecialTokenKind::Control || encode_specials {
                            result.push(special.id);
                            continue;
                        }
                    }
                    self.encode_unigram(
                        piece.as_bytes(),
                        &mut buffer,
                        &mut result,
                        piece.char_indices().map(|(i, _)| i),
                        &self.config.fallback,
                    )?;
                    buffer.clear();
                }
            }
        }
        Ok(result)
    }

    /// Returns the score for the given token in piece between start and end of parts.
    #[inline(always)]
    fn get_score(&self, piece: &[u8], parts: &[Part], start: usize, end: usize) -> u32 {
        if end < parts.len() {
            self.score_encoder
                .get(
                    &piece[unsafe {
                        parts.get_unchecked(start).start..parts.get_unchecked(end).start
                    }],
                )
                .map(|t| t.score)
                .unwrap_or(u32::MAX)
        } else {
            u32::MAX
        }
    }

    /// Merges the given parts according to the BPE algorithm, prioritizing merges with the lowest score.
    #[inline(never)]
    fn merge_bpe_parts(&self, piece: &[u8], parts: &mut Vec<Part>, start: usize) {
        if parts.len() <= start + 1 {
            return;
        }
        let mut min_score = u32::MAX;
        let mut i = start;
        for j in start..parts.len() - 1 {
            parts[j].score = self.get_score(piece, &parts[..], j, j + 2);
            if parts[j].score < min_score {
                min_score = parts[j].score;
                i = j;
            }
        }
        while min_score != u32::MAX {
            parts[i].score = self.get_score(piece, parts, i, i + 3);
            if i > start {
                parts[i - 1].score = self.get_score(piece, parts, i - 1, i + 2);
            }
            parts.remove(i + 1);
            min_score = u32::MAX;
            for (j, &Part { score, .. }) in parts[start..parts.len() - 1].iter().enumerate() {
                if score < min_score {
                    min_score = score;
                    i = j + start;
                }
            }
        }
    }

    /// Encodes the given piece into a sequence of tokens using the BPE algorithm.
    ///
    /// Returns an error if no token for a part exists in the encoder, no unknown token id is set in the configuration, and no fallback is set.
    #[inline(never)]
    fn encode_pairs(
        &self, piece: &[u8], buffer: &mut Vec<Part>, result: &mut Vec<u32>,
        indices: impl Iterator<Item = usize>, fallback: &[ModeFallback],
    ) -> Result<(), EncodeError> {
        let start = buffer.len();
        buffer.extend(indices.map(|i| Part {
            start: i,
            score: u32::MAX,
        }));
        buffer.push(Part {
            start: piece.len(),
            score: u32::MAX,
        });
        self.merge_bpe_parts(piece, buffer, start);
        let end = buffer.len() - 1;
        for i in start..end {
            let piece = &piece[buffer[i].start..buffer[i + 1].start];
            if let Some(token) = self.encoder.get(piece) {
                result.push(token.token);
            } else if fallback.first() == Some(&ModeFallback::Bytes) {
                self.encode_pairs(
                    piece,
                    buffer,
                    result,
                    0..piece.len(),
                    &fallback[fallback.len().min(1)..],
                )?;
            } else if fallback.first() == Some(&ModeFallback::Unknown) && self.unknown.is_some() {
                result.push(self.unknown.as_ref().unwrap().0);
            } else if fallback.first() == Some(&ModeFallback::Skip) {
            } else {
                return Err(EncodeError::InvalidPiece(piece.into()));
            }
        }
        Ok(())
    }

    /// Merges the given parts according to the BPE algorithm, prioritizing merges with the lowest score.
    ///
    /// This version uses a heap for tracking the merge candidates.
    /// The additional allocation overhead compared to the linear search version is amortized for longer pieces.
    #[inline(never)]
    fn merge_bpe_parts_heap(&self, piece: &[u8], heap: &mut PieceHeap) {
        while heap.len() > 1 {
            let &(i, mut part) = heap.peek().unwrap();
            if part.score == u32::MAX {
                break;
            }
            let next = heap.remove(&part.after);
            part.width += next.width;
            part.after = next.after;
            if part.after != u32::MAX {
                let mut next = heap.key_of(&part.after).unwrap();
                if let Some(token) =
                    self.score_encoder.get(&piece[part.start as _..(next.start + next.width) as _])
                {
                    part.score = token.score;
                } else {
                    part.score = u32::MAX;
                }
                next.prior = i;
                heap.update_key(&part.after, next);
            } else {
                part.score = u32::MAX;
            }
            if part.prior != u32::MAX {
                let mut prior = heap.key_of(&(part.prior)).unwrap();
                if let Some(token) =
                    self.score_encoder.get(&piece[prior.start as _..(part.start + part.width) as _])
                {
                    prior.score = token.score;
                } else {
                    prior.score = u32::MAX;
                }
                heap.update_key(&part.prior, prior);
            }
            heap.update_key(&i, part);
        }
    }

    /// Encodes the given piece into a sequence of tokens using the BPE algorithm.
    ///
    /// This version uses a heap for tracking the merge candidates.
    ///
    /// Returns an error if no token for a part exists in the encoder, no unknown token id is set in the configuration, and no fallback is set.
    #[inline(never)]
    fn encode_pairs_heap(
        &self, piece: &[u8], buffer: &mut Vec<Part>, result: &mut Vec<u32>,
        indices: impl Iterator<Item = (u32, u32)>, fallback: &[ModeFallback],
    ) -> Result<(), EncodeError> {
        let mut heap = PieceHeap::with_index_bound(piece.len());
        let mut prior = u32::MAX;
        let mut iter = indices.enumerate().peekable();
        loop {
            if iter.peek().is_none() {
                break;
            }
            let (e, (i, c)) = iter.next().unwrap();
            let next = iter.peek();
            heap.push(e as _, LinkedPart {
                start: i,
                width: c,
                prior,
                after: if next.is_some() {
                    e as u32 + 1
                } else {
                    u32::MAX
                },
                score: if let Some((_, (_, n))) = next {
                    self.score_encoder
                        .get(&piece[i as _..(i + c + n) as _])
                        .map(|t| t.score)
                        .unwrap_or(u32::MAX)
                } else {
                    u32::MAX
                },
            });
            prior = e as _;
        }
        self.merge_bpe_parts_heap(piece, &mut heap);
        let mut e = 0;
        while e <= prior {
            let part = heap.key_of(&e).unwrap();
            let piece = &piece[part.start as _..(part.start + part.width) as _];
            if let Some(token) = self.encoder.get(piece) {
                result.push(token.token);
            } else if fallback.first() == Some(&ModeFallback::Bytes) {
                self.encode_pairs(
                    piece,
                    buffer,
                    result,
                    0..piece.len(),
                    &fallback[fallback.len().min(1)..],
                )?;
            } else if fallback.first() == Some(&ModeFallback::Unknown) && self.unknown.is_some() {
                result.push(self.unknown.as_ref().unwrap().0);
            } else if fallback.first() == Some(&ModeFallback::Skip) {
            } else {
                return Err(EncodeError::InvalidPiece(piece.into()));
            }
            e = part.after;
        }
        Ok(())
    }

    /// Encodes the given piece into a sequence of tokens using the unigram algorithm.
    /// This algorithm merges the highest scored subword units.
    ///
    /// Returns an error if no token for a part exists in the encoder, no unknown token id is set in the configuration, and no fallback is set.
    #[inline(never)]
    fn encode_unigram(
        &self, piece: &[u8], buffer: &mut Vec<SizedPart>, result: &mut Vec<u32>,
        indices: impl Iterator<Item = usize>, fallback: &[ModeFallback],
    ) -> Result<(), EncodeError> {
        let start = buffer.len();
        buffer.extend(indices.map(|c| SizedPart {
            start: c,
            width: 1,
            score: 0.0,
            token: u32::MAX,
        }));
        buffer.push(SizedPart {
            start: piece.len(),
            width: 1,
            score: 0.0,
            token: u32::MAX,
        });
        let end = buffer.len();
        for sub_end in start + 1..end {
            buffer[sub_end].score = 1000000.0;
            for sub_start in (start..sub_end).rev() {
                if (buffer[sub_end].start - buffer[sub_start].start) > self.max_token_bytes {
                    break;
                }
                if let Some(token) =
                    self.encoder.get(&piece[buffer[sub_start].start..buffer[sub_end].start])
                {
                    let score = buffer[sub_start].score - token.score;
                    if buffer[sub_end].token == u32::MAX || score < buffer[sub_end].score {
                        buffer[sub_end].score = score;
                        buffer[sub_end].width = sub_end - sub_start;
                        buffer[sub_end].token = token.token;
                    }
                }
            }
        }
        let result_start = result.len();
        let mut sub_end = end - 1;
        while sub_end > start {
            if buffer[sub_end].token == u32::MAX {
                if fallback.first() == Some(&ModeFallback::Bytes) {
                    let part = &piece[buffer[sub_end - 1].start..buffer[sub_end].start];
                    self.encode_unigram(
                        part,
                        buffer,
                        result,
                        0..part.len(),
                        &fallback[fallback.len().min(1)..],
                    )?;
                } else if fallback.first() == Some(&ModeFallback::Unknown) && self.unknown.is_some()
                {
                    result.push(self.unknown.as_ref().unwrap().0);
                } else if fallback.first() == Some(&ModeFallback::Skip) {
                } else {
                    let part = &piece[buffer[sub_end - 1].start..buffer[sub_end].start];
                    return Err(EncodeError::InvalidPiece(part.into()));
                }
                sub_end -= buffer[sub_end].width;
                continue;
            }
            result.push(buffer[sub_end].token);
            sub_end -= buffer[sub_end].width;
        }
        result[result_start..].reverse();
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct TextPart<'a> {
    text:    Cow<'a, str>,
    special: bool,
}

#[derive(Debug, Clone, Copy)]
struct Part {
    start: usize,
    score: u32,
}

#[derive(Debug, Clone, Copy)]
struct LinkedPart {
    start: u32,
    width: u32,
    prior: u32,
    after: u32,
    score: u32,
}
impl PartialEq for LinkedPart {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}
impl Eq for LinkedPart {}
impl PartialOrd for LinkedPart {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for LinkedPart {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> Ordering {
        (self.score, self.start).partial_cmp(&(other.score, other.start)).unwrap()
    }
}

#[derive(Debug, Clone, Copy)]
struct SizedPart {
    start: usize,
    width: usize,
    score: f32,
    token: u32,
}
