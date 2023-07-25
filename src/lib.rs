//! **Tokenizer for language models.**
//!
//! Supports BPE and Unigram tokenization. Usable in native and WASM environments.
//!
//! # Overview
//!
//! Kitoken is a fast and versatile tokenizer for language models with support for BPE and Unigram tokenization.
//!
//! Kitoken is compatible with many existing tokenizer formats,
//! including [SentencePiece](https://github.com/google/sentencepiece), [HuggingFace Tokenizers](https://github.com/huggingface/tokenizers) and [tiktoken](https://github.com/openai/tiktoken),
//! while outperforming them in many scenarios.
//!
//! See [`Kitoken`] for the main entry point and additional information.
//!
//! # Examples
//!
//! ### Loading a Kitoken definition
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use kitoken::Kitoken;
//! let tokenizer = Kitoken::from_file("tests/models/llama2.kit")?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Loading a SentencePiece model
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use kitoken::convert::convert_sentencepiece;
//! use kitoken::Kitoken;
//!
//! // Directly initialize the tokenizer from a SentencePiece model
//! let tokenizer = Kitoken::from_sentencepiece_file("tests/models/llama2.model")?;
//!
//! // Or convert a SentencePiece model to a Kitoken definition
//! let model = std::fs::read("tests/models/xlnet_base_cased.model")?;
//! let definition = convert_sentencepiece(&model)?;
//! let tokenizer = Kitoken::try_from(definition)?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Encoding and decoding
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # use kitoken::Kitoken;
//! # let tokenizer = Kitoken::from_file("tests/models/llama2.kit")?;
//! let tokens = tokenizer.encode("Your future belongs to me <|endoftext|>", true)?;
//! let text = String::from_utf8(tokenizer.decode(&tokens)?)?;
//! assert_eq!(text, "Your future belongs to me <|endoftext|>");
//! # Ok(())
//! # }
//! ```
//!
//! See [`convert`] for information about supported formats and conversion utilities.
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
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

extern crate alloc;

mod config;
mod definition;
mod normalization;
mod regex;

#[cfg(feature = "serialization")]
mod serialization;

pub mod convert;

use alloc::string::ToString;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::num::NonZeroUsize;
use core::str::Utf8Error;

use hashbrown::HashMap;
use orx_priority_queue::{DaryHeapOfIndices, PriorityQueue, PriorityQueueDecKey};

pub use crate::config::*;
pub use crate::definition::*;

#[cfg(feature = "serialization")]
pub use crate::serialization::*;

use crate::regex::*;

/// List of token bytes and their id.
pub type Vocab = Vec<(Vec<u8>, u32)>;
/// List of token ids and their score.
pub type Scores = Vec<f32>;

/// Errors encountered during initialization.
#[non_exhaustive]
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum InitializationError {
    /// The configuration failed to validate.
    #[cfg_attr(feature = "std", error("invalid config: {0}"))]
    InvalidConfig(ValidationError),
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
impl From<ValidationError> for InitializationError {
    fn from(e: ValidationError) -> Self {
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

pub(crate) type EncoderMap = HashMap<Vec<u8>, Token>;
pub(crate) type DecoderMap = HashMap<u32, Vec<u8>>;

pub(crate) type PieceHeap = DaryHeapOfIndices<usize, LinkedPart, 4>;

static HEAP_PIECE_SIZE: usize = 256;

/// Kitoken tokenizer.
/// A fast and versatile tokenizer for language models.
#[derive(Debug)]
pub struct Kitoken {
    encoder: EncoderMap,
    decoder: DecoderMap,

    special_encoder: EncoderMap,
    special_decoder: DecoderMap,

    split:         Regex,
    special_split: Regex,

    config: Configuration,

    max_token_bytes: usize,
}
impl Kitoken {
    /// Creates a tokenizer from the given encoder, specials, scores and config.
    ///
    /// Returns an error if the config is invalid, the special encoder contains invalid utf-8, the encoder or special encoder contain duplicates,
    /// or the encoder and scores have different lengths in unigram mode.
    #[inline(never)]
    pub fn new(
        vocab: impl Into<Vocab>, specials: impl Into<Vocab>, scores: impl Into<Scores>,
        config: Configuration,
    ) -> Result<Self, InitializationError> {
        if let Err(error) = config.validate() {
            return Err(InitializationError::InvalidConfig(error));
        }

        let vocab: Vocab = vocab.into();
        let specials: Vocab = specials.into();
        let scores: Scores = scores.into();

        if config.mode == Mode::Unigram && vocab.len() != scores.len() {
            return Err(InitializationError::InvalidScores);
        }

        let split = Regex::new(&config.split)?;
        let special_split = Regex::new(
            &specials
                .iter()
                .map(|(s, _)| core::str::from_utf8(s))
                .collect::<Result<Vec<_>, _>>()?
                .iter()
                .map(|s| fancy_regex::escape(s))
                .collect::<Vec<_>>()
                .join("|"),
        )?;

        let decoder = vocab.iter().map(|(k, v)| (*v, k.clone())).collect::<DecoderMap>();
        if vocab.len() != decoder.len() {
            return Err(InitializationError::InvalidEncoder);
        }
        let special_decoder = specials.iter().map(|(k, v)| (*v, k.clone())).collect::<DecoderMap>();
        if specials.len() != special_decoder.len() {
            return Err(InitializationError::InvalidSpecialEncoder);
        }

        let encoder = vocab
            .into_iter()
            .enumerate()
            .map(|(i, (k, v))| {
                let score = if config.mode == Mode::Unigram {
                    scores[i]
                } else {
                    i as f32
                };
                Some((k, Token { token: v, score }))
            })
            .collect::<Option<EncoderMap>>()
            .ok_or(InitializationError::InvalidScores)?;
        let special_encoder = specials
            .into_iter()
            .enumerate()
            .map(|(i, (k, v))| {
                (k, Token {
                    token: v,
                    score: i as f32,
                })
            })
            .collect::<EncoderMap>();

        let max_token_bytes = encoder.keys().map(|k| k.len()).max().unwrap();

        Ok(Self {
            encoder,
            decoder,
            split,
            special_encoder,
            special_decoder,
            special_split,
            config,
            max_token_bytes,
        })
    }

    /// Encodes the given text into a sequence of tokens.
    ///
    /// If `encode_specials` is `true`, the text is first split around special tokens which are separately encoded with the special encoder.
    ///
    /// Returns a list of tokens, or an error if no token for a part exists in the encoder and no unknown token id is set in the configuration.
    #[inline(never)]
    pub fn encode(
        &self, text: impl AsRef<str>, encode_specials: bool,
    ) -> Result<Vec<u32>, EncodeError> {
        let text = text.as_ref();
        let normalized = self.config.normalize_encode_input_enabled().then(|| {
            let mut text = text.to_string();
            self.config.normalize_encode_input(&mut text);
            text
        });
        let text = normalized.as_ref().map_or(text, |text| text.as_str());
        if text.is_empty() {
            return Ok(Vec::new());
        }
        let parts = self.split_into_parts(text, encode_specials);
        let mut result = self.encode_parts(text, &parts)?;
        if self.config.normalize_encode_output_enabled() {
            self.config.normalize_encode_output(&mut result);
        }
        Ok(result)
    }

    /// Decodes the given sequence of tokens into text.
    ///
    /// Returns a list of bytes, or an error if no byte sequence for a token exists in the decoder and no unknown token is set in the configuration.
    #[inline(never)]
    pub fn decode(&self, tokens: impl AsRef<[u32]>) -> Result<Vec<u8>, DecodeError> {
        let tokens = tokens.as_ref();
        let mut result = Vec::<u8>::with_capacity(tokens.len() * 3);
        for token in tokens {
            if let Some((unk_id, unk)) = &self.config.specials.unk {
                if token == unk_id {
                    result.extend(unk);
                    continue;
                }
            }
            let bytes = self
                .decoder
                .get(token)
                .or_else(|| self.special_decoder.get(token))
                .ok_or(DecodeError::InvalidToken(*token))?;
            result.extend(bytes);
        }
        if self.config.normalize_decode_output_enabled() {
            self.config.normalize_decode_output(&mut result);
        }
        Ok(result)
    }

    /// Splits the given text into parts according to the split regex.
    ///
    /// If `split_specials` is `true`, the text is first split around special tokens.
    ///
    /// Returns a list of parts.
    #[inline(never)]
    fn split_into_parts(&self, text: &str, split_specials: bool) -> Vec<TextPart> {
        let mut parts = Vec::<TextPart>::with_capacity(text.len() / 3);
        let mut posit = 0;
        loop {
            let next_special = if split_specials {
                self.special_split.find(&text[posit..])
            } else {
                None
            };
            let end = next_special.map_or(text.len(), |(start, _)| start + posit);
            let mut last = 0;
            for (start, end) in self.split.find_iter(&text[posit..end]) {
                if start > last {
                    parts.push(TextPart {
                        start:   last + posit,
                        end:     unsafe { NonZeroUsize::new_unchecked(start + posit) },
                        special: false,
                    });
                }
                if end > start {
                    parts.push(TextPart {
                        start:   start + posit,
                        end:     unsafe { NonZeroUsize::new_unchecked(end + posit) },
                        special: false,
                    });
                }
                last = end
            }
            if last + posit < end {
                parts.push(TextPart {
                    start:   last + posit,
                    end:     unsafe { NonZeroUsize::new_unchecked(end) },
                    special: false,
                });
            }
            if let Some((start, end)) = next_special {
                parts.push(TextPart {
                    start:   start + posit,
                    end:     unsafe { NonZeroUsize::new_unchecked(end + posit) },
                    special: true,
                });
                posit += end
            } else {
                break;
            }
        }
        parts
    }

    /// Encodes the given piece into a sequence of tokens.
    ///
    /// Returns an error if no token for a part exists in the encoder and no unknown token id is set in the configuration.
    #[inline(never)]
    fn encode_parts(&self, text: &str, parts: &[TextPart]) -> Result<Vec<u32>, EncodeError> {
        let mut result = Vec::with_capacity(text.len() / 3);
        match self.config.mode {
            Mode::BytePair => {
                let mut buffer = Vec::with_capacity(HEAP_PIECE_SIZE);
                for part in parts {
                    let piece = &text[part.start..part.end.get()];
                    if part.special {
                        result.push(self.special_encoder[piece.as_bytes()].token);
                        continue;
                    }
                    if let Some(&token) = self.encoder.get(piece.as_bytes()) {
                        result.push(token.token);
                        continue;
                    }
                    if piece.len() >= HEAP_PIECE_SIZE {
                        self.encode_pairs_heap(
                            piece.as_bytes(),
                            &mut buffer,
                            &mut result,
                            (0..piece.len()).map(|i| (i, 1)),
                            false,
                        )?;
                    } else {
                        self.encode_pairs(
                            piece.as_bytes(),
                            &mut buffer,
                            &mut result,
                            0..piece.len(),
                            false,
                        )?;
                    }
                    buffer.clear();
                }
            }
            Mode::CharPair => {
                let mut buffer = Vec::with_capacity(HEAP_PIECE_SIZE);
                let mut indices = Vec::with_capacity(HEAP_PIECE_SIZE);
                for part in parts {
                    let piece = &text[part.start..part.end.get()];
                    if part.special {
                        result.push(self.special_encoder[piece.as_bytes()].token);
                        continue;
                    }
                    if let Some(&token) = self.encoder.get(piece.as_bytes()) {
                        result.push(token.token);
                        continue;
                    }
                    indices.extend(piece.char_indices());
                    if indices.len() >= HEAP_PIECE_SIZE {
                        self.encode_pairs_heap(
                            piece.as_bytes(),
                            &mut buffer,
                            &mut result,
                            indices.drain(..).map(|(i, c)| (i, c.len_utf8())),
                            true,
                        )?;
                    } else {
                        self.encode_pairs(
                            piece.as_bytes(),
                            &mut buffer,
                            &mut result,
                            indices.drain(..).map(|(i, _)| i),
                            true,
                        )?;
                    }
                    buffer.clear();
                }
            }
            Mode::Unigram => {
                let mut buffer = Vec::with_capacity(HEAP_PIECE_SIZE);
                for part in parts {
                    let piece = &text[part.start..part.end.get()];
                    if part.special {
                        result.push(self.special_encoder[piece.as_bytes()].token);
                        continue;
                    }
                    self.encode_unigram(
                        piece.as_bytes(),
                        &mut buffer,
                        &mut result,
                        piece.char_indices().map(|(i, _)| i),
                    )?;
                    buffer.clear();
                }
            }
        }
        Ok(result)
    }

    /// Merges the given parts according to the BPE algorithm, prioritizing merges with the lowest score.
    #[inline(never)]
    fn merge_bpe_parts(&self, piece: &[u8], parts: &mut Vec<Part>, start: usize) {
        if parts.len() <= start + 1 {
            return;
        }
        let update_bpe_part = {
            #[inline(always)]
            |piece: &[u8], parts: &mut [Part], start: usize, end: usize| {
                if end >= parts.len()
                    || parts[end].start - parts[start].start > self.max_token_bytes
                {
                    parts[start].score = f32::MAX;
                } else if let Some(entry) =
                    self.encoder.get(&piece[parts[start].start..parts[end].start])
                {
                    parts[start].score = entry.score;
                    parts[start].token = entry.token;
                } else {
                    parts[start].score = f32::MAX;
                }
            }
        };
        for i in start..parts.len() - 1 {
            update_bpe_part(piece, &mut parts[..], i, i + 2);
        }
        while parts.len() > start + 1 {
            let mut min_score = f32::MAX;
            let mut i = 0;
            for (j, &Part { score, .. }) in parts[..parts.len() - 1].iter().skip(start).enumerate()
            {
                if score < min_score {
                    min_score = score;
                    i = j;
                }
            }
            if min_score == f32::MAX {
                break;
            }
            update_bpe_part(piece, parts, i, i + 3);
            if i > start {
                update_bpe_part(piece, parts, i - 1, i + 2);
                parts[i - 1].token = u32::MAX;
            }
            parts.remove(i + 1);
        }
    }

    /// Encodes the given piece into a sequence of tokens using the BPE algorithm.
    ///
    /// Returns an error if no token for a part exists in the encoder and no unknown token id is set in the configuration.
    #[inline(never)]
    fn encode_pairs(
        &self, piece: &[u8], buffer: &mut Vec<Part>, result: &mut Vec<u32>,
        indices: impl Iterator<Item = usize>, byte_pair_fallback: bool,
    ) -> Result<(), EncodeError> {
        let start = buffer.len();
        buffer.extend(indices.map(|i| Part {
            start: i,
            score: f32::MAX,
            token: u32::MAX,
        }));
        buffer.push(Part {
            start: piece.len(),
            score: f32::MAX,
            token: u32::MAX,
        });
        self.merge_bpe_parts(piece, buffer, start);
        let end = buffer.len() - 1;
        for i in start..end {
            if buffer[i].token != u32::MAX {
                result.push(buffer[i].token);
                continue;
            }
            let piece = &piece[buffer[i].start..buffer[i + 1].start];
            if let Some(token) = self.encoder.get(piece) {
                result.push(token.token);
            } else if byte_pair_fallback {
                self.encode_pairs(piece, buffer, result, 0..piece.len(), false)?;
            } else if let Some((unk_id, _)) = self.config.specials.unk {
                result.push(unk_id);
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
            let (i, mut part) = heap.pop().unwrap();
            if part.score == f32::MAX {
                heap.push(i, part);
                break;
            }
            let next = heap.remove(&part.after);
            part.width += next.width;
            part.after = next.after;
            if part.after != usize::MAX {
                let mut next = heap.remove(&part.after);
                if let Some(token) = self.encoder.get(&piece[part.start..next.start + next.width]) {
                    part.score = token.score;
                    part.token = token.token;
                } else {
                    part.score = f32::MAX;
                }
                next.prior = i;
                heap.push(part.after, next);
            } else {
                part.score = f32::MAX;
            }
            if part.prior != usize::MAX {
                let mut prior = heap.remove(&(part.prior));
                if let Some(token) = self.encoder.get(&piece[prior.start..part.start + part.width])
                {
                    prior.score = token.score;
                } else {
                    prior.score = f32::MAX;
                }
                prior.token = u32::MAX;
                heap.push(part.prior, prior);
            }
            heap.push(i, part);
        }
    }

    /// Encodes the given piece into a sequence of tokens using the BPE algorithm.
    ///
    /// This version uses a heap for tracking the merge candidates.
    #[inline(never)]
    fn encode_pairs_heap(
        &self, piece: &[u8], buffer: &mut Vec<Part>, result: &mut Vec<u32>,
        indices: impl Iterator<Item = (usize, usize)>, byte_pair_fallback: bool,
    ) -> Result<(), EncodeError> {
        let mut heap = PieceHeap::with_index_bound(piece.len());
        let mut prior = usize::MAX;
        let mut iter = indices.enumerate().peekable();
        loop {
            if iter.peek().is_none() {
                break;
            }
            let (e, (i, c)) = iter.next().unwrap();
            let next = iter.peek();
            heap.push(e, LinkedPart {
                start: i,
                width: c,
                prior,
                after: if next.is_some() { e + 1 } else { usize::MAX },
                score: if let Some((_, (_, n))) = next {
                    self.encoder.get(&piece[i..i + c + n]).map(|t| t.score).unwrap_or(f32::MAX)
                } else {
                    f32::MAX
                },
                token: u32::MAX,
            });
            prior = e;
        }
        self.merge_bpe_parts_heap(piece, &mut heap);
        let mut e = 0;
        while e <= prior {
            let part = heap.key_of(&e).unwrap();
            if part.token != u32::MAX {
                result.push(part.token);
                e = part.after;
                continue;
            }
            let piece = &piece[part.start..part.start + part.width];
            if let Some(token) = self.encoder.get(piece) {
                result.push(token.token);
            } else if byte_pair_fallback {
                self.encode_pairs(piece, buffer, result, 0..piece.len(), false)?;
            } else if let Some((unk_id, _)) = self.config.specials.unk {
                result.push(unk_id);
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
    /// Returns an error if no token for a part exists in the encoder and no unknown token id is set in the configuration.
    #[inline(never)]
    fn encode_unigram(
        &self, piece: &[u8], buffer: &mut Vec<SizedPart>, result: &mut Vec<u32>,
        indices: impl Iterator<Item = usize>,
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
                if let Some((unk_id, _)) = self.config.specials.unk {
                    result.push(unk_id);
                } else {
                    let part = &piece[buffer[sub_end - 1].start..buffer[sub_end].start];
                    return Err(EncodeError::InvalidPiece(part.into()));
                }
                sub_end -= 1;
                continue;
            }
            result.push(buffer[sub_end].token);
            sub_end -= buffer[sub_end].width;
        }
        result[result_start..].reverse();
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
struct TextPart {
    start:   usize,
    end:     NonZeroUsize,
    special: bool,
}

#[derive(Debug, Clone, Copy)]
struct Part {
    start: usize,
    score: f32,
    token: u32,
}

#[derive(Debug, Clone, Copy)]
struct LinkedPart {
    start: usize,
    width: usize,
    prior: usize,
    after: usize,
    score: f32,
    token: u32,
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
