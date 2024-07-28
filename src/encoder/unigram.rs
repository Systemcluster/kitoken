//! Unigram encoder.

use alloc::format;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::fmt::Debug;

use bstr::ByteSlice;
use hashbrown::HashMap;

use crate::{
    Configuration, EncodeError, Encoder, Fallback, InitializationError, Model, Scores,
    SpecialToken, SpecialTokenKind, SpecialVocab, TextPart, Token, TokenBytes, TokenId, TokenScore,
    Vocab,
};

#[derive(Debug, Clone, Copy)]
struct ScoredToken {
    pub id:    TokenId,
    pub score: TokenScore,
}

#[derive(Debug, Clone, Copy)]
struct SizedPart {
    pub start: usize,
    pub width: usize,
    pub score: f64,
    pub token: TokenId,
}

type ScoredVocabMap = HashMap<TokenBytes, ScoredToken>;

/// Unigram encoder.
#[derive(Clone)]
pub(crate) struct Unigram {
    vocab: ScoredVocabMap,

    unknown:  Option<SpecialToken>,
    fallback: Vec<Fallback>,

    max_token_bytes: usize,
    min_token_bytes: usize,
}
impl Debug for Unigram {
    #[inline(never)]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("Unigram")
            .field("vocab", &format!("ScoredVocabMap({})", self.vocab.len()))
            .field("unknown", &self.unknown)
            .field("fallback", &self.fallback)
            .field("max_token_bytes", &self.max_token_bytes)
            .field("min_token_bytes", &self.min_token_bytes)
            .finish()
    }
}
impl Encoder for Unigram {
    #[inline(always)]
    fn encode(&self, text: &str, parts: &mut [TextPart]) -> Result<Vec<TokenId>, EncodeError> {
        let mut result =
            Vec::with_capacity(text.len() / self.min_token_bytes + self.max_token_bytes);
        self.encode_chars(parts, &self.fallback, &mut result)?;
        Ok(result)
    }

    #[inline(always)]
    fn model(&self) -> Model {
        let mut vocab = self.vocab.iter().map(|(k, v)| (k.clone(), *v)).collect::<Vec<_>>();
        vocab.sort_by(|(_, a), (_, b)| match a.score.partial_cmp(&b.score).unwrap() {
            Ordering::Equal => a.id.cmp(&b.id),
            other => other,
        });
        let scores = vocab.iter().map(|(_, v)| v.score).collect();
        let vocab = vocab.into_iter().map(|(k, v)| (v.id, k).into()).collect();
        Model::Unigram { vocab, scores }
    }
}
impl Unigram {
    const ENCODE_BUFFER_SIZE: usize = 256;

    #[inline(never)]
    pub fn new(
        vocab: Vocab, specials: &SpecialVocab, config: &Configuration, scores: Scores,
    ) -> Result<Self, InitializationError> {
        let unknown = specials
            .iter()
            .find(|special| special.kind == SpecialTokenKind::Unknown)
            .cloned();

        let vocab_len = vocab.len();
        let scores_len = scores.len();
        if vocab_len != scores_len {
            return Err(InitializationError::InvalidScores);
        }
        let vocab = vocab
            .into_iter()
            .zip(scores)
            .map(|(t, s)| {
                (t.bytes.clone(), ScoredToken {
                    id:    t.id,
                    score: s,
                })
            })
            .collect::<ScoredVocabMap>();
        if vocab_len != vocab.len() {
            return Err(InitializationError::InvalidEncoder);
        }

        let max_token_bytes = vocab.keys().map(|k| k.len()).max().unwrap().max(1);
        let min_token_bytes = vocab.keys().map(|k| k.len()).min().unwrap().max(1);

        let fallback = config.fallback.clone();

        Ok(Self {
            vocab,
            unknown,
            fallback,
            max_token_bytes,
            min_token_bytes,
        })
    }

    /// Encodes the given parts into a sequence of tokens starting at individual characters.
    #[inline(never)]
    fn encode_chars(
        &self, parts: &[TextPart], fallback: &[Fallback], result: &mut Vec<TokenId>,
    ) -> Result<(), EncodeError> {
        let mut buffer = Vec::with_capacity(Self::ENCODE_BUFFER_SIZE);
        for part in parts {
            if part.special != Token::INVALID {
                result.push(part.special);
                continue;
            }
            self.encode_unigram(
                part.as_bytes(),
                &mut buffer,
                result,
                part.char_indices().map(|(i, _, _)| i),
                fallback,
            )?;
            buffer.clear();
        }
        Ok(())
    }

    /// Encodes the given piece into a sequence of tokens using the unigram algorithm.
    /// This algorithm merges the highest scored subword units.
    ///
    /// Returns an error if no token for a part exists in the encoder, no unknown token id is set in the configuration, and no fallback is set.
    #[inline(never)]
    fn encode_unigram(
        &self, piece: &[u8], buffer: &mut Vec<SizedPart>, result: &mut Vec<TokenId>,
        indices: impl Iterator<Item = usize>, fallback: &[Fallback],
    ) -> Result<(), EncodeError> {
        let start = buffer.len();
        buffer.extend(indices.map(|c| SizedPart {
            start: c,
            width: 1,
            score: 0.0,
            token: Token::INVALID,
        }));
        buffer.push(SizedPart {
            start: piece.len(),
            width: 1,
            score: 0.0,
            token: Token::INVALID,
        });
        Unigram::merge_parts(piece, buffer, &self.vocab, start, self.max_token_bytes);
        let result_start = result.len();
        let mut sub_end = buffer.len() - 1;
        while sub_end > start {
            if buffer[sub_end].token == Token::INVALID {
                if fallback.first() == Some(&Fallback::Bytes) {
                    let part = &piece[buffer[sub_end - 1].start..buffer[sub_end].start];
                    self.encode_unigram(
                        part,
                        buffer,
                        result,
                        0..part.len(),
                        &fallback[fallback.len().min(1)..],
                    )?;
                } else if fallback.first() == Some(&Fallback::Unknown) && self.unknown.is_some() {
                    result.push(self.unknown.as_ref().unwrap().id);
                } else if fallback.first() == Some(&Fallback::Skip) {
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

    /// Merges the given parts according to the Unigram algorithm
    #[inline(never)]
    #[cfg_attr(
        feature = "multiversion",
        multiversion::multiversion(targets(
            "x86_64/x86-64-v4",
            "x86_64/x86-64-v3",
            "x86_64/x86-64-v2",
            "aarch64+neon",
            "wasm32+simd128",
        ))
    )]
    fn merge_parts(
        piece: &[u8], buffer: &mut [SizedPart], vocab: &ScoredVocabMap, start: usize,
        max_token_bytes: usize,
    ) {
        let end = buffer.len();
        for sub_end in start + 1..end {
            buffer[sub_end].score = 1000000.0;
            for sub_start in (start..sub_end).rev() {
                if (buffer[sub_end].start - buffer[sub_start].start) > max_token_bytes {
                    break;
                }
                if let Some(token) =
                    vocab.get(&piece[buffer[sub_start].start..buffer[sub_end].start])
                {
                    let score = buffer[sub_start].score - token.score as f64;
                    if buffer[sub_end].token == Token::INVALID || score <= buffer[sub_end].score {
                        buffer[sub_end].score = score;
                        buffer[sub_end].width = sub_end - sub_start;
                        buffer[sub_end].token = token.id;
                    }
                }
            }
        }
    }
}
