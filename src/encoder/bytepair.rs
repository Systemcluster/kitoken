//! BytePair and CharPair encoder.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::fmt::Debug;

use bstr::ByteSlice;
use hashbrown::HashMap;
use orx_priority_queue::{DaryHeapOfIndices, PriorityQueue, PriorityQueueDecKey};

use crate::{
    Configuration, EncodeError, Encoder, Fallback, InitializationError, InsertionPosition, Model,
    SpecialToken, SpecialTokenKind, SpecialVocab, TextPart, Token, TokenBytes, TokenId, Vocab,
};

type TokenRank = u32;

#[derive(Debug, Clone, Copy)]
struct RankedPart {
    pub start: u32,
    pub rank:  TokenRank,
}

#[derive(Debug, Clone, Copy)]
struct LinkedPart {
    pub start: u32,
    pub width: u32,
    pub prior: u32,
    pub after: u32,
    pub rank:  TokenRank,
}
impl PartialEq for LinkedPart {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.rank == other.rank
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
        (self.rank, self.start).partial_cmp(&(other.rank, other.start)).unwrap()
    }
}

type VocabMap = HashMap<TokenBytes, TokenId>;
type RankMap = HashMap<TokenBytes, TokenRank>;
type PieceHeap = DaryHeapOfIndices<u32, LinkedPart, 4>;

/// BytePair and CharPair encoder.
#[derive(Clone)]
pub(crate) struct BytePair {
    vocab: VocabMap,
    ranks: RankMap,

    unknown:     Option<SpecialToken>,
    end_of_word: Option<String>,
    chars:       bool,
    fallback:    Vec<Fallback>,

    max_token_bytes: usize,
    min_token_bytes: usize,
}
impl Debug for BytePair {
    #[inline(never)]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("BytePair")
            .field("vocab", &format!("VocabMap({})", self.vocab.len()))
            .field("ranks", &format!("RankMap({})", self.ranks.len()))
            .field("unknown", &self.unknown)
            .field("end_of_word", &self.end_of_word)
            .field("chars", &self.chars)
            .field("fallback", &self.fallback)
            .field("max_token_bytes", &self.max_token_bytes)
            .field("min_token_bytes", &self.min_token_bytes)
            .finish()
    }
}
impl Encoder for BytePair {
    #[inline(always)]
    fn encode(&self, text: &str, parts: &mut [TextPart]) -> Result<Vec<TokenId>, EncodeError> {
        if let Some(end_of_word) = &self.end_of_word {
            for part in parts.iter_mut() {
                if part.special == Token::INVALID {
                    part.text.to_mut().push_str(end_of_word);
                }
            }
        }
        let mut result =
            Vec::with_capacity(text.len() / self.min_token_bytes + self.max_token_bytes);
        if self.chars {
            self.encode_chars(parts, &self.fallback, &mut result)?;
        } else {
            self.encode_bytes(parts, &self.fallback, &mut result)?;
        }
        Ok(result)
    }

    #[inline(always)]
    fn model(&self) -> Model {
        let mut vocab = self.vocab.iter().map(|(k, v)| (k.clone(), *v)).collect::<Vec<_>>();
        vocab.sort_by(|(ta, a), (tb, b)| {
            let sa = self.ranks.get(ta).copied().unwrap();
            let sb = self.ranks.get(tb).copied().unwrap();
            match sa.cmp(&sb) {
                Ordering::Equal => a.cmp(b),
                other => other,
            }
        });
        let vocab = vocab.into_iter().map(|(k, v)| (v, k).into()).collect();
        let chars = self.chars;
        Model::BytePair { vocab, chars }
    }
}
impl BytePair {
    const ENCODE_BUFFER_SIZE: usize = 256;
    const ENCODE_LINEAR_LIMIT: usize = 192;

    #[inline(never)]
    pub fn new(
        vocab: Vocab, specials: &SpecialVocab, config: &Configuration, chars: bool,
    ) -> Result<Self, InitializationError> {
        let unknown = specials
            .iter()
            .find(|special| special.kind == SpecialTokenKind::Unknown)
            .cloned();
        let end_of_word = config.templates.iter().find_map(|template| {
            if template.position == InsertionPosition::WordEnd {
                Some(template.content.clone())
            } else {
                None
            }
        });

        let vocab_len = vocab.len();
        let ranks = vocab
            .iter()
            .enumerate()
            .map(|(i, t)| (t.bytes.clone(), i as TokenRank))
            .collect::<RankMap>();
        let vocab = vocab.into_iter().map(|t| t.into()).collect::<VocabMap>();
        if vocab_len != vocab.len() {
            return Err(InitializationError::InvalidEncoder);
        }

        let max_token_bytes = vocab.keys().map(|k| k.len()).max().unwrap().max(1);
        let min_token_bytes = vocab.keys().map(|k| k.len()).min().unwrap().max(1);
        let fallback = config.fallback.clone();

        Ok(Self {
            vocab,
            ranks,
            unknown,
            end_of_word,
            chars,
            fallback,
            max_token_bytes,
            min_token_bytes,
        })
    }
}
impl BytePair {
    /// Encodes the given parts into a sequence of tokens starting at individual bytes.
    #[inline(never)]
    fn encode_bytes(
        &self, parts: &[TextPart], fallback: &[Fallback], result: &mut Vec<TokenId>,
    ) -> Result<(), EncodeError> {
        let mut buffer = Vec::with_capacity(Self::ENCODE_BUFFER_SIZE);
        let end_of_word_len = self.end_of_word.as_ref().map(|e| e.len()).unwrap_or(0);
        for part in parts {
            if part.special != Token::INVALID {
                result.push(part.special);
                continue;
            }
            if part.len() <= self.max_token_bytes && part.len() >= self.min_token_bytes {
                if let Some(&token) = self.vocab.get(part.as_bytes()) {
                    result.push(token);
                    continue;
                }
            }
            if part.len() > Self::ENCODE_LINEAR_LIMIT {
                self.encode_pairs_heap(
                    part.as_bytes(),
                    &mut buffer,
                    result,
                    (0..(part.len() - end_of_word_len)).map(|i| i as u32).map(|i| (i, 1)),
                    fallback,
                )?;
            } else {
                self.encode_pairs(
                    part.as_bytes(),
                    &mut buffer,
                    result,
                    (0..(part.len() - end_of_word_len)).map(|i| i as u32),
                    fallback,
                )?;
            }
            buffer.clear();
        }
        Ok(())
    }

    /// Encodes the given parts into a sequence of tokens starting at individual characters.
    #[inline(never)]
    fn encode_chars(
        &self, parts: &[TextPart], fallback: &[Fallback], result: &mut Vec<TokenId>,
    ) -> Result<(), EncodeError> {
        let mut buffer = Vec::with_capacity(Self::ENCODE_BUFFER_SIZE);
        let mut indices = Vec::with_capacity(Self::ENCODE_BUFFER_SIZE);
        let end_of_word_len = self.end_of_word.as_ref().map(|e| e.len()).unwrap_or(0);
        for part in parts {
            if part.special != Token::INVALID {
                result.push(part.special);
                continue;
            }
            if part.len() <= self.max_token_bytes && part.len() >= self.min_token_bytes {
                if let Some(&token) = self.vocab.get(part.as_bytes()) {
                    result.push(token);
                    continue;
                }
            }
            indices.extend(
                part[..part.len() - end_of_word_len]
                    .char_indices()
                    .map(|(s, _, c)| (s as u32, c.len_utf8() as u32)),
            );
            if indices.len() > Self::ENCODE_LINEAR_LIMIT {
                self.encode_pairs_heap(
                    part.as_bytes(),
                    &mut buffer,
                    result,
                    indices.drain(..),
                    fallback,
                )?;
            } else {
                self.encode_pairs(
                    part.as_bytes(),
                    &mut buffer,
                    result,
                    indices.drain(..).map(|(i, _)| i),
                    fallback,
                )?;
            }
        }
        Ok(())
    }
}
impl BytePair {
    /// Encodes the given piece into a sequence of tokens using the BPE algorithm.
    ///
    /// Returns an error if no token for a part exists in the encoder, no unknown token id is set in the configuration, and no fallback is set.
    #[inline(never)]
    fn encode_pairs(
        &self, piece: &[u8], buffer: &mut Vec<RankedPart>, result: &mut Vec<TokenId>,
        indices: impl Iterator<Item = u32>, fallback: &[Fallback],
    ) -> Result<(), EncodeError> {
        let start = buffer.len();
        buffer.extend(indices.map(|i| RankedPart {
            start: i,
            rank:  TokenRank::MAX,
        }));
        buffer.push(RankedPart {
            start: piece.len() as _,
            rank:  TokenRank::MAX,
        });
        BytePair::merge_bpe_parts(piece, buffer, start, &self.ranks);
        let end = buffer.len() - 1;
        for i in start..end {
            let piece = &piece[buffer[i].start as usize..buffer[i + 1].start as usize];
            if let Some(&token) = self.vocab.get(piece) {
                result.push(token);
            } else if fallback.first() == Some(&Fallback::Bytes) {
                let end = if let Some(end_of_word) = &self.end_of_word {
                    piece.len() - end_of_word.len()
                } else {
                    piece.len()
                };
                self.encode_pairs(
                    piece,
                    buffer,
                    result,
                    0..(end as _),
                    &fallback[fallback.len().min(1)..],
                )?;
            } else if fallback.first() == Some(&Fallback::Unknown) && self.unknown.is_some() {
                result.push(self.unknown.as_ref().unwrap().id);
            } else if fallback.first() == Some(&Fallback::Skip) {
            } else {
                return Err(EncodeError::InvalidPiece(piece.into()));
            }
        }
        Ok(())
    }

    /// Returns the score for the given token in piece between start and end of parts.
    #[inline(always)]
    fn get_rank(
        piece: &[u8], parts: &[RankedPart], start: usize, end: usize, ranks: &RankMap,
    ) -> TokenRank {
        if end < parts.len() {
            ranks
                .get(
                    &piece[unsafe {
                        parts.get_unchecked(start).start as usize
                            ..parts.get_unchecked(end).start as usize
                    }],
                )
                .copied()
                .unwrap_or(TokenRank::MAX)
        } else {
            TokenRank::MAX
        }
    }

    /// Merges the given parts according to the BPE algorithm, prioritizing merges with the lowest score.
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
    fn merge_bpe_parts(piece: &[u8], parts: &mut Vec<RankedPart>, start: usize, ranks: &RankMap) {
        if parts.len() <= start + 1 {
            return;
        }
        let mut min_score = TokenRank::MAX;
        let mut i = start;
        for j in start..parts.len() - 1 {
            parts[j].rank = BytePair::get_rank(piece, &parts[..], j, j + 2, ranks);
            if parts[j].rank < min_score {
                (min_score, i) = (parts[j].rank, j);
            }
        }
        while min_score != TokenRank::MAX {
            if i > start {
                parts[i - 1].rank = BytePair::get_rank(piece, parts, i - 1, i + 2, ranks);
            }
            parts[i].rank = BytePair::get_rank(piece, parts, i, i + 3, ranks);
            parts.remove(i + 1);
            min_score = TokenRank::MAX;
            #[allow(clippy::needless_range_loop)]
            for j in start..parts.len() - 1 {
                if parts[j].rank < min_score {
                    (min_score, i) = (parts[j].rank, j);
                }
            }
        }
    }
}
impl BytePair {
    /// Encodes the given piece into a sequence of tokens using the BPE algorithm.
    ///
    /// This version uses a heap for tracking the merge candidates.
    ///
    /// Returns an error if no token for a part exists in the encoder, no unknown token id is set in the configuration, and no fallback is set.
    #[inline(never)]
    #[cold]
    fn encode_pairs_heap(
        &self, piece: &[u8], buffer: &mut Vec<RankedPart>, result: &mut Vec<TokenId>,
        indices: impl Iterator<Item = (u32, u32)>, fallback: &[Fallback],
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
                width: if next.is_some() {
                    c
                } else {
                    piece.len() as u32 - i
                },
                prior,
                after: if next.is_some() {
                    e as u32 + 1
                } else {
                    u32::MAX
                },
                rank: if let Some((_, (_, n))) = next {
                    self.ranks
                        .get(&piece[i as _..(i + c + n) as _])
                        .copied()
                        .unwrap_or(TokenRank::MAX)
                } else {
                    TokenRank::MAX
                },
            });
            prior = e as _;
        }
        BytePair::merge_bpe_parts_heap(piece, &mut heap, &self.ranks);
        let mut e = 0;
        while e <= prior {
            let part = heap.key_of(&e).unwrap();
            let piece = &piece[part.start as _..(part.start + part.width) as _];
            if let Some(&token) = self.vocab.get(piece) {
                result.push(token);
            } else if fallback.first() == Some(&Fallback::Bytes) {
                let end = if let Some(end_of_word) = &self.end_of_word {
                    piece.len() - end_of_word.len()
                } else {
                    piece.len()
                };
                self.encode_pairs(
                    piece,
                    buffer,
                    result,
                    (0..end).map(|i| i as u32),
                    &fallback[fallback.len().min(1)..],
                )?;
            } else if fallback.first() == Some(&Fallback::Unknown) && self.unknown.is_some() {
                result.push(self.unknown.as_ref().unwrap().id);
            } else if fallback.first() == Some(&Fallback::Skip) {
            } else {
                return Err(EncodeError::InvalidPiece(piece.into()));
            }
            e = part.after;
        }
        Ok(())
    }

    /// Merges the given parts according to the BPE algorithm, prioritizing merges with the lowest score.
    ///
    /// This version uses a heap for tracking the merge candidates.
    /// The additional allocation overhead compared to the linear search version is amortized for longer pieces.
    #[inline(never)]
    #[cold]
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
    fn merge_bpe_parts_heap(piece: &[u8], heap: &mut PieceHeap, ranks: &RankMap) {
        while heap.len() > 1 {
            let &(i, mut part) = heap.peek().unwrap();
            if part.rank == TokenRank::MAX {
                break;
            }
            let next = heap.remove(&part.after);
            part.width += next.width;
            part.after = next.after;
            if part.after != u32::MAX {
                let mut next = heap.key_of(&part.after).unwrap();
                if let Some(&token) =
                    ranks.get(&piece[part.start as _..(next.start + next.width) as _])
                {
                    part.rank = token;
                } else {
                    part.rank = TokenRank::MAX;
                }
                next.prior = i;
                heap.update_key(&part.after, next);
            } else {
                part.rank = TokenRank::MAX;
            }
            if part.prior != u32::MAX {
                let mut prior = heap.key_of(&(part.prior)).unwrap();
                if let Some(&token) =
                    ranks.get(&piece[prior.start as _..(part.start + part.width) as _])
                {
                    prior.rank = token;
                } else {
                    prior.rank = TokenRank::MAX;
                }
                heap.update_key(&part.prior, prior);
            }
            heap.update_key(&i, part);
        }
    }
}
