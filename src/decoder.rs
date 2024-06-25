//! Decoder for the tokenizer.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Debug;

use hashbrown::HashMap;

use crate::{SpecialToken, SpecialTokenKind, SpecialVocab, TokenId, Vocab};

/// Errors encountered during decoding.
#[non_exhaustive]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum DecodeError {
    /// A token could not be decoded.
    #[cfg_attr(feature = "std", error("invalid token {0}"))]
    InvalidToken(TokenId),
}

pub(crate) type DecoderMap = HashMap<TokenId, Vec<u8>>;
pub(crate) type SpecialDecoderMap = HashMap<TokenId, SpecialToken>;

/// Decoder for the tokenizer.
#[derive(Clone)]
pub(crate) struct Decoder {
    vocab:    DecoderMap,
    specials: SpecialDecoderMap,

    max_token_bytes: usize,
}
impl Decoder {
    #[inline(always)]
    pub(crate) fn new(vocab: &Vocab, specials: &SpecialVocab) -> Self {
        let max_token_bytes = vocab.iter().map(|k| k.len()).max().unwrap().max(1);
        let specials = specials.iter().map(|special| (special.id, special.clone())).collect();
        let vocab = vocab.iter().map(|token| token.into()).collect();
        Self {
            vocab,
            specials,
            max_token_bytes,
        }
    }

    #[inline(never)]
    pub(crate) fn decode(
        &self, tokens: &[TokenId], decode_specials: bool,
    ) -> Result<Vec<u8>, DecodeError> {
        let mut result = Vec::<u8>::with_capacity(tokens.len() * self.max_token_bytes);
        for token in tokens {
            let bytes = self.vocab.get(token);
            if let Some(bytes) = bytes {
                result.extend(bytes);
            } else if let Some(special) = self.specials.get(token) {
                if special.kind != SpecialTokenKind::Control || decode_specials {
                    result.extend(special);
                }
            } else {
                return Err(DecodeError::InvalidToken(*token));
            }
        }
        Ok(result)
    }
}
impl Debug for Decoder {
    #[inline(never)]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("Decoder")
            .field("map", &format!("DecoderMap({})", self.vocab.len()))
            .field("specials", &format!("SpecialDecoderMap({})", self.specials.len()))
            .field("max_token_bytes", &self.max_token_bytes)
            .finish()
    }
}
