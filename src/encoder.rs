//! Encoder for the tokenizer.

use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::borrow::Borrow;
use core::fmt::Debug;
use core::ops::Deref;

use crate::{Scores, TokenId, Vocab};

mod bytepair;
mod unigram;
mod wordpiece;

pub(crate) use bytepair::*;
pub(crate) use unigram::*;
pub(crate) use wordpiece::*;

/// Errors encountered during encoding.
#[non_exhaustive]
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum EncodeError {
    /// A piece could not be encoded.
    #[cfg_attr(feature = "std", error("invalid piece {0:?}"))]
    InvalidPiece(Vec<u8>),
}

/// Part of a text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TextPart<'a> {
    pub text:    Cow<'a, str>,
    pub special: TokenId,
}
impl Borrow<[u8]> for TextPart<'_> {
    #[inline(always)]
    fn borrow(&self) -> &[u8] {
        self.text.as_bytes()
    }
}
impl Deref for TextPart<'_> {
    type Target = [u8];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.text.as_bytes()
    }
}

/// Encoder for the tokenizer.
pub(crate) trait Encoder: Debug + Send + Sync + 'static {
    /// Encodes the given parts into a sequence of tokens.
    ///
    /// If `encode_specials` is `true`, control tokens are tokenized with their ids, otherwise they are tokenized with the regular vocabulary.
    ///
    /// Returns an error if no token for a part exists in the encoder, and the configuration has no unknown token or skip fallback set.
    fn encode(&self, text: &str, parts: &mut [TextPart]) -> Result<Vec<TokenId>, EncodeError>;

    /// Returns the vocabulary and scores.
    fn vocab(&self) -> (Vocab, Scores);
}
