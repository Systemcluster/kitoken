//! Decoder for the tokenizer.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Debug;

use hashbrown::HashMap;

use crate::{
    Configuration, InsertionPosition, SpecialToken, SpecialTokenKind, SpecialVocab, TokenId, Vocab,
};

/// Errors encountered during decoding.
#[non_exhaustive]
#[derive(Debug, Clone, thiserror::Error)]
pub enum DecodeError {
    /// A token could not be decoded.
    #[error("invalid token {0}")]
    InvalidToken(TokenId),
}

pub(crate) type DecoderMap = HashMap<TokenId, Vec<u8>>;
pub(crate) type SpecialDecoderMap = HashMap<TokenId, SpecialToken>;

/// Decoder for the tokenizer.
#[derive(Clone)]
pub(crate) struct Decoder {
    vocab:    DecoderMap,
    specials: SpecialDecoderMap,

    subword_prefix: Option<String>,

    max_token_bytes: usize,
}
impl Decoder {
    #[inline(always)]
    pub(crate) fn new(vocab: &Vocab, specials: &SpecialVocab, config: &Configuration) -> Self {
        let max_token_bytes = vocab.iter().map(|k| k.len()).max().unwrap().max(1);
        let specials = specials.iter().map(|special| (special.id, special.clone())).collect();
        let vocab = vocab.iter().map(|token| token.into()).collect();
        let subword_prefix = config.templates.iter().find_map(|template| {
            if template.position == InsertionPosition::WordContinuation {
                Some(template.content.clone())
            } else {
                None
            }
        });
        Self {
            vocab,
            specials,
            subword_prefix,
            max_token_bytes,
        }
    }

    #[inline(never)]
    pub(crate) fn decode(
        &self, tokens: &[TokenId], decode_specials: bool,
    ) -> Result<Vec<u8>, DecodeError> {
        let extend = self.subword_prefix.as_deref().unwrap_or_default();
        let mut result = Vec::<u8>::with_capacity(
            tokens.len() * self.max_token_bytes + tokens.len() * extend.len(),
        );
        if !extend.is_empty() {
            Self::decode_with_prefix(
                &mut result,
                tokens,
                extend,
                &self.vocab,
                &self.specials,
                decode_specials,
            )?;
        } else {
            Self::decode_direct(&mut result, tokens, &self.vocab, &self.specials, decode_specials)?;
        }
        Ok(result)
    }

    #[inline(never)]
    #[cfg_attr(
        feature = "multiversion",
        multiversion::multiversion(targets(
            "x86_64+sse3+ssse3+sse4.1+sse4.2+avx+avx2+bmi2+f16c+lzcnt+popcnt",
            "x86_64+sse3+ssse3+sse4.1+sse4.2",
            "aarch64+neon",
            "wasm32+simd128",
        ))
    )]
    fn decode_direct(
        result: &mut Vec<u8>, tokens: &[TokenId], vocab: &DecoderMap, specials: &SpecialDecoderMap,
        decode_specials: bool,
    ) -> Result<(), DecodeError> {
        for token in tokens {
            let bytes = vocab.get(token);
            if let Some(bytes) = bytes {
                result.extend(bytes);
            } else if let Some(special) = specials.get(token) {
                if special.kind != SpecialTokenKind::Control || decode_specials {
                    result.extend(special);
                }
            } else {
                return Err(DecodeError::InvalidToken(*token));
            }
        }
        Ok(())
    }

    #[inline(never)]
    #[cfg_attr(
        feature = "multiversion",
        multiversion::multiversion(targets(
            "x86_64+sse3+ssse3+sse4.1+sse4.2+avx+avx2+bmi2+f16c+lzcnt+popcnt",
            "x86_64+sse3+ssse3+sse4.1+sse4.2",
            "aarch64+neon",
            "wasm32+simd128",
        ))
    )]
    fn decode_with_prefix(
        result: &mut Vec<u8>, tokens: &[TokenId], prefix: &str, vocab: &DecoderMap,
        specials: &SpecialDecoderMap, decode_specials: bool,
    ) -> Result<(), DecodeError> {
        for token in tokens {
            let bytes = vocab.get(token);
            if let Some(bytes) = bytes {
                if !result.is_empty() && !bytes.starts_with(prefix.as_bytes()) {
                    result.push(b' ');
                }
                result.extend(bytes);
            } else if let Some(special) = specials.get(token) {
                if !result.is_empty() {
                    result.push(b' ');
                }
                if special.kind != SpecialTokenKind::Control || decode_specials {
                    result.extend(special);
                }
            } else {
                return Err(DecodeError::InvalidToken(*token));
            }
        }
        Ok(())
    }
}
impl Debug for Decoder {
    #[inline(never)]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("Decoder")
            .field("vocab", &format!("DecoderMap({})", self.vocab.len()))
            .field("specials", &format!("SpecialDecoderMap({})", self.specials.len()))
            .field("subword_prefix", &self.subword_prefix)
            .field("max_token_bytes", &self.max_token_bytes)
            .finish()
    }
}
