//! BytePair and CharPair encoder.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::fmt::Debug;

use bstr::ByteSlice;
use orx_priority_queue::{DaryHeapOfIndices, PriorityQueue, PriorityQueueDecKey};

use crate::bytemap::ByteMap;
use crate::{
    Configuration, EncodeError, Encoder, Fallback, InitializationError, InsertionPosition, Model,
    SpecialToken, SpecialTokenKind, SpecialVocab, TextPart, Token, TokenId, Vocab,
};

type TokenRank = u32;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct RankedPart {
    pub start: u32,
    pub rank:  TokenRank,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct PartState {
    start: u32,
    width: u32,
    prior: u32,
    after: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
struct MergePriority {
    rank:  TokenRank,
    index: u32,
}

type VocabMap = ByteMap<TokenId>;
type RankMap = ByteMap<TokenRank>;
type PieceHeap = DaryHeapOfIndices<u32, MergePriority, 4>;

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
        let mut vocab = self.vocab.iter().collect::<Vec<_>>();
        vocab.sort_by(|(ta, a), (tb, b)| {
            let sa = self.ranks.get(*ta).copied().unwrap();
            let sb = self.ranks.get(*tb).copied().unwrap();
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
    const ENCODE_LINEAR_BYTE_LIMIT: usize = 192;
    const ENCODE_LINEAR_CHAR_LIMIT: usize = 32;

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
        let mut heap = PieceHeap::with_index_bound(Self::ENCODE_BUFFER_SIZE);
        let mut state = Vec::with_capacity(Self::ENCODE_BUFFER_SIZE);
        let end_of_word_len = self.end_of_word.as_ref().map(|e| e.len()).unwrap_or(0);
        for part in parts {
            if part.special != Token::INVALID {
                result.push(part.special);
                continue;
            }
            if part.len() <= self.max_token_bytes
                && part.len() >= self.min_token_bytes
                && let Some(&token) = self.vocab.get(part.as_bytes())
            {
                result.push(token);
                continue;
            }
            let part_count = part.len() - end_of_word_len;
            if part_count > Self::ENCODE_LINEAR_BYTE_LIMIT {
                self.encode_pairs_heap(
                    part.as_bytes(),
                    &mut buffer,
                    &mut heap,
                    &mut state,
                    result,
                    (0..part_count).map(|i| i as u32).map(|i| (i, 1)),
                    fallback,
                )?;
            } else {
                self.encode_pairs(
                    part.as_bytes(),
                    &mut buffer,
                    result,
                    (0..part_count).map(|i| i as u32),
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
        let mut heap = PieceHeap::with_index_bound(Self::ENCODE_BUFFER_SIZE);
        let mut state = Vec::with_capacity(Self::ENCODE_BUFFER_SIZE);
        let mut indices = Vec::with_capacity(Self::ENCODE_BUFFER_SIZE);
        let end_of_word_len = self.end_of_word.as_ref().map(|e| e.len()).unwrap_or(0);
        for part in parts {
            if part.special != Token::INVALID {
                result.push(part.special);
                continue;
            }
            if part.len() <= self.max_token_bytes
                && part.len() >= self.min_token_bytes
                && let Some(&token) = self.vocab.get(part.as_bytes())
            {
                result.push(token);
                continue;
            }
            indices.extend(
                part[..part.len() - end_of_word_len]
                    .char_indices()
                    .map(|(s, _, c)| (s as u32, c.len_utf8() as u32)),
            );
            if indices.len() > Self::ENCODE_LINEAR_CHAR_LIMIT {
                self.encode_pairs_heap(
                    part.as_bytes(),
                    &mut buffer,
                    &mut heap,
                    &mut state,
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
            buffer.clear();
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
                let end = BytePair::fallback_byte_end(piece, self.end_of_word.as_deref());
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

    #[inline(always)]
    fn fallback_byte_end(piece: &[u8], end_of_word: Option<&str>) -> usize {
        end_of_word
            .and_then(|suffix| piece.strip_suffix(suffix.as_bytes()))
            .map_or(piece.len(), <[u8]>::len)
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
    #[allow(clippy::too_many_arguments)]
    fn encode_pairs_heap(
        &self, piece: &[u8], buffer: &mut Vec<RankedPart>, heap: &mut PieceHeap,
        state: &mut Vec<PartState>, result: &mut Vec<TokenId>,
        indices: impl Iterator<Item = (u32, u32)>, fallback: &[Fallback],
    ) -> Result<(), EncodeError> {
        state.clear();
        let mut prior = u32::MAX;
        let mut iter = indices.enumerate().peekable();
        loop {
            if iter.peek().is_none() {
                break;
            }
            let (e, (i, c)) = iter.next().unwrap();
            let next = iter.peek();
            state.push(PartState {
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
            });
            prior = e as _;
        }
        if state.is_empty() {
            heap.clear();
            return Ok(());
        }
        if state.len() > heap.index_bound() {
            *heap = PieceHeap::with_index_bound(state.len());
        } else {
            heap.clear();
        }
        for index in 0..state.len().saturating_sub(1) {
            let part = state[index];
            let next = state[part.after as usize];
            if let Some(&rank) =
                self.ranks.get(&piece[part.start as usize..(next.start + next.width) as usize])
            {
                heap.push(index as u32, MergePriority {
                    rank,
                    index: index as u32,
                });
            }
        }
        BytePair::merge_bpe_parts_heap(piece, heap, state, &self.ranks);
        let mut e = 0;
        while e != u32::MAX {
            let piece = &piece[state[e as usize].start as _
                ..(state[e as usize].start + state[e as usize].width) as _];
            if let Some(&token) = self.vocab.get(piece) {
                result.push(token);
            } else if fallback.first() == Some(&Fallback::Bytes) {
                let end = BytePair::fallback_byte_end(piece, self.end_of_word.as_deref());
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
            e = state[e as usize].after;
        }
        Ok(())
    }

    /// Merges the given parts according to the BPE algorithm, prioritizing merges with the lowest score.
    ///
    /// This version uses a heap for tracking the merge candidates.
    /// The additional allocation overhead compared to the linear search version is amortized for longer pieces.
    #[inline(never)]
    #[cold]
    fn merge_bpe_parts_heap(
        piece: &[u8], heap: &mut PieceHeap, state: &mut [PartState], ranks: &RankMap,
    ) {
        while let Some((index, _)) = heap.pop() {
            let right = state[index as usize].after;
            debug_assert_ne!(right, u32::MAX);
            if heap.contains(&right) {
                heap.remove(&right);
            }

            state[index as usize].width += state[right as usize].width;
            state[index as usize].after = state[right as usize].after;
            let after = state[index as usize].after;
            if after != u32::MAX {
                state[after as usize].prior = index;
            }

            BytePair::update_merge_candidate(piece, heap, state, ranks, index);
            let prior = state[index as usize].prior;
            if prior != u32::MAX {
                BytePair::update_merge_candidate(piece, heap, state, ranks, prior);
            }
        }
    }

    #[inline(always)]
    fn update_merge_candidate(
        piece: &[u8], heap: &mut PieceHeap, state: &[PartState], ranks: &RankMap, index: u32,
    ) {
        let part = state[index as usize];
        let rank = if part.after == u32::MAX {
            None
        } else {
            let next = state[part.after as usize];
            ranks
                .get(&piece[part.start as usize..(next.start + next.width) as usize])
                .copied()
        };
        if let Some(rank) = rank {
            let priority = MergePriority { rank, index };
            if heap.contains(&index) {
                heap.update_key(&index, priority);
            } else {
                heap.push(index, priority);
            }
        } else if heap.contains(&index) {
            heap.remove(&index);
        }
    }
}
