//! WordPiece encoder.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::fmt::Debug;
use core::iter::Peekable;

use bstr::ByteSlice;
use hashbrown::HashMap;

use crate::{
    Configuration, EncodeError, Encoder, Fallback, InsertionPosition, Model, SpecialToken,
    SpecialTokenKind, SpecialVocab, TextPart, Token, TokenBytes, TokenId, Vocab,
};

type VocabMap = HashMap<TokenBytes, TokenId>;

/// WordPiece encoder.
#[derive(Clone)]
pub(crate) struct WordPiece {
    start:        VocabMap,
    continuation: VocabMap,

    unknown:        Option<SpecialToken>,
    subword_prefix: Option<String>,
    fallback:       Vec<Fallback>,

    max_word_chars:  usize,
    max_token_bytes: usize,
    min_token_bytes: usize,
}
impl Debug for WordPiece {
    #[inline(never)]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("WordPiece")
            .field("start", &format!("VocabMap({})", self.start.len()))
            .field("continuation", &format!("VocabMap({})", self.continuation.len()))
            .field("unknown", &self.unknown)
            .field("subword_prefix", &self.subword_prefix)
            .field("fallback", &self.fallback)
            .field("max_word_chars", &self.max_word_chars)
            .field("max_token_bytes", &self.max_token_bytes)
            .field("min_token_bytes", &self.min_token_bytes)
            .finish()
    }
}
impl Encoder for WordPiece {
    #[inline(always)]
    fn encode(&self, text: &str, parts: &mut [TextPart]) -> Result<Vec<TokenId>, EncodeError> {
        let mut result =
            Vec::with_capacity(text.len() / self.min_token_bytes + self.max_token_bytes);
        self.encode_chars(parts, &self.fallback, &mut result)?;
        Ok(result)
    }

    #[inline(always)]
    fn model(&self) -> Model {
        let mut vocab = self
            .start
            .iter()
            .map(|(k, v)| (k.clone(), *v).into())
            .chain(self.continuation.iter().map(|(k, v)| {
                let prefixed = [
                    self.subword_prefix.as_deref().unwrap_or_default().as_bytes(),
                    k,
                ]
                .concat();
                (prefixed, *v).into()
            }))
            .collect::<Vocab>();
        vocab.sort_by(|Token { bytes: a, id: ai }, Token { bytes: b, id: bi }| {
            let comp = ai.cmp(bi);
            if comp == Ordering::Equal {
                a.cmp(b)
            } else {
                comp
            }
        });
        let max_word_chars = self.max_word_chars as u32;
        Model::WordPiece {
            vocab,
            max_word_chars,
        }
    }
}
impl WordPiece {
    #[inline(never)]
    pub fn new(
        vocab: Vocab, specials: &SpecialVocab, config: &Configuration, max_word_chars: u32,
    ) -> Self {
        let unknown = specials
            .iter()
            .find(|special| special.kind == SpecialTokenKind::Unknown)
            .cloned();
        let subword_prefix = config.templates.iter().find_map(|template| {
            if template.position == InsertionPosition::WordContinuation {
                Some(template.content.clone())
            } else {
                None
            }
        });

        let start = if let Some(subword_prefix) = &subword_prefix {
            vocab
                .iter()
                .filter(|token| !token.starts_with(subword_prefix.as_bytes()))
                .map(|token| token.into())
                .collect::<VocabMap>()
        } else {
            vocab.iter().map(|token| token.into()).collect::<VocabMap>()
        };
        let continuation = if let Some(subword_prefix) = &subword_prefix {
            vocab
                .iter()
                .filter(|&token| token.starts_with(subword_prefix.as_bytes()))
                .map(|token| (token.bytes[subword_prefix.len()..].to_vec(), token.id))
                .collect::<VocabMap>()
        } else {
            VocabMap::with_capacity(0)
        };

        let max_word_chars = max_word_chars as usize;
        let max_token_bytes = start.keys().map(|k| k.len()).max().unwrap().max(1);
        let min_token_bytes = start.keys().map(|k| k.len()).min().unwrap().max(1);

        let fallback = config.fallback.clone();

        Self {
            start,
            continuation,
            unknown,
            subword_prefix,
            fallback,
            max_word_chars,
            max_token_bytes,
            min_token_bytes,
        }
    }

    /// Encodes the given parts into a sequence of tokens starting at individual characters.
    #[inline(never)]
    fn encode_chars(
        &self, parts: &[TextPart], fallback: &[Fallback], result: &mut Vec<TokenId>,
    ) -> Result<(), EncodeError> {
        for part in parts {
            if part.special != Token::INVALID {
                result.push(part.special);
                continue;
            }
            self.encode_wordpiece(
                part.as_bytes(),
                result,
                part.char_indices().map(|(i, e, _)| (i, e)),
                fallback.iter().copied().peekable(),
            )?;
        }
        Ok(())
    }

    /// Encodes the given bytes into a sequence of tokens using the WordPiece algorithm.
    #[inline(never)]
    fn encode_wordpiece(
        &self, bytes: &[u8], result: &mut Vec<TokenId>,
        mut indices: impl DoubleEndedIterator<Item = (usize, usize)> + Clone,
        mut fallback: Peekable<impl Iterator<Item = Fallback>>,
    ) -> Result<(), EncodeError> {
        if bytes.len() < self.min_token_bytes
            || self.max_word_chars > 0 && indices.clone().count() > self.max_word_chars
        {
            if fallback.peek() == Some(&Fallback::Unknown) && self.unknown.is_some() {
                result.push(self.unknown.as_ref().unwrap().id);
            } else if fallback.peek() == Some(&Fallback::Skip) {
            } else {
                return Err(EncodeError::InvalidPiece(bytes[..].to_vec()));
            }
            return Ok(());
        }
        let init = result.len();
        let mut first = true;
        let mut until = 0;
        let stop = [(0, bytes.len())];
        while let Some((start, e)) = indices.next() {
            if start < until {
                continue;
            }
            let inner = core::iter::once((0, e)).chain(indices.clone()).chain(stop).rev();
            for (_, end) in inner {
                let piece = bytes[start..end].to_vec();
                let token = if first {
                    self.start.get(&piece).copied()
                } else {
                    self.continuation.get(&piece).copied()
                };
                if let Some(token) = token {
                    result.push(token);
                    first = false;
                    until = end;
                    break;
                }
            }
            if until <= start {
                result.truncate(init);
                if fallback.peek() == Some(&Fallback::Unknown) && self.unknown.is_some() {
                    result.push(self.unknown.as_ref().unwrap().id);
                } else if fallback.peek() == Some(&Fallback::Skip) {
                } else {
                    return Err(EncodeError::InvalidPiece(bytes[start..].to_vec()));
                }
                break;
            }
        }
        Ok(())
    }
}
