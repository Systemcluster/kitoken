use alloc::string::String;
use alloc::vec::Vec;
use core::borrow::Borrow;
use core::cmp::Ordering;
use core::fmt::{Debug, Display};

use bstr::ByteSlice;
use derive_more::{AsMut, AsRef, Deref, DerefMut, Index, IndexMut};

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

/// Numeric identifier of a token.
pub type TokenId = u32;
/// Byte sequence of a token.
pub type TokenBytes = Vec<u8>;
/// Score of a token.
pub type TokenScore = f32;

/// Token structure.
#[derive(Clone, AsRef, AsMut, Deref, DerefMut, Index, IndexMut)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct Token {
    pub id:    TokenId,
    #[as_ref]
    #[as_mut]
    #[deref]
    #[deref_mut]
    #[index]
    #[index_mut]
    pub bytes: TokenBytes,
}
impl Token {
    pub const INVALID: TokenId = u32::MAX;
}
impl Display for Token {
    #[inline(never)]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_tuple("Token").field(&self.id).field(&self.bytes.as_bstr()).finish()
    }
}
impl Debug for Token {
    #[inline(never)]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("Token")
            .field("id", &self.id)
            .field("bytes", &self.bytes.as_bstr())
            .finish()
    }
}
impl PartialEq for Token {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.bytes == other.bytes
    }
}
impl Eq for Token {}
impl PartialOrd for Token {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Token {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}
impl Borrow<TokenBytes> for Token {
    #[inline(always)]
    fn borrow(&self) -> &TokenBytes {
        &self.bytes
    }
}
impl Borrow<[u8]> for Token {
    #[inline(always)]
    fn borrow(&self) -> &[u8] {
        &self.bytes
    }
}
impl IntoIterator for Token {
    type IntoIter = alloc::vec::IntoIter<u8>;
    type Item = u8;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.bytes.into_iter()
    }
}
impl<'a> IntoIterator for &'a Token {
    type IntoIter = alloc::slice::Iter<'a, u8>;
    type Item = &'a u8;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.bytes.iter()
    }
}
impl From<Token> for (TokenBytes, TokenId) {
    #[inline(always)]
    fn from(value: Token) -> (TokenBytes, TokenId) {
        (value.bytes, value.id)
    }
}
impl From<&Token> for (TokenBytes, TokenId) {
    #[inline(always)]
    fn from(value: &Token) -> (TokenBytes, TokenId) {
        (value.bytes.clone(), value.id)
    }
}
impl From<Token> for (TokenId, TokenBytes) {
    #[inline(always)]
    fn from(value: Token) -> (TokenId, TokenBytes) {
        (value.id, value.bytes)
    }
}
impl From<&Token> for (TokenId, TokenBytes) {
    #[inline(always)]
    fn from(value: &Token) -> (TokenId, TokenBytes) {
        (value.id, value.bytes.clone())
    }
}
impl From<(TokenBytes, TokenId)> for Token {
    #[inline(always)]
    fn from(value: (TokenBytes, TokenId)) -> Token {
        Token {
            id:    value.1,
            bytes: value.0,
        }
    }
}
impl From<(TokenId, TokenBytes)> for Token {
    #[inline(always)]
    fn from(value: (TokenId, TokenBytes)) -> Token {
        Token {
            id:    value.0,
            bytes: value.1,
        }
    }
}

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

/// Identifier for special tokens.
///
/// Used for control tokens like "cls", "sep", "pad", "mask" and similar.
pub type SpecialTokenIdent = String;

/// Special token structure.
#[derive(Clone, AsRef, AsMut, Deref, DerefMut, Index, IndexMut)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct SpecialToken {
    /// The token id. The numeric value of the token.
    pub id:      TokenId,
    /// The token bytes. The byte sequence of the token.
    #[as_ref]
    #[as_mut]
    #[deref]
    #[deref_mut]
    #[index]
    #[index_mut]
    pub bytes:   TokenBytes,
    /// The token type.
    pub kind:    SpecialTokenKind,
    /// Common identifier for the token. Used for control tokens like "cls", "sep", "pad", "mask" and similar.
    pub ident:   Option<SpecialTokenIdent>,
    /// The token score. Used for prioritizing special tokens during encoding.
    pub score:   TokenScore,
    /// Whether the token should be split pre-normalization.
    pub extract: bool,
}
impl Display for SpecialToken {
    #[inline(never)]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_tuple("SpecialToken")
            .field(&self.id)
            .field(&self.bytes.as_bstr())
            .field(&self.kind)
            .field(&self.ident)
            .field(&self.score)
            .field(&self.extract)
            .finish()
    }
}
impl Debug for SpecialToken {
    #[inline(never)]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
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
impl PartialEq for SpecialToken {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.kind == other.kind && self.bytes == other.bytes
    }
}
impl Eq for SpecialToken {}
impl PartialOrd for SpecialToken {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for SpecialToken {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> Ordering {
        let mut ord = self.kind.cmp(&other.kind);
        if ord == Ordering::Equal {
            ord = self.score.partial_cmp(&other.score).unwrap_or(Ordering::Equal);
        }
        if ord == Ordering::Equal {
            ord = self.id.cmp(&other.id);
        }
        ord
    }
}
impl Borrow<TokenBytes> for SpecialToken {
    #[inline(always)]
    fn borrow(&self) -> &TokenBytes {
        &self.bytes
    }
}
impl Borrow<[u8]> for SpecialToken {
    #[inline(always)]
    fn borrow(&self) -> &[u8] {
        &self.bytes
    }
}
impl IntoIterator for SpecialToken {
    type IntoIter = alloc::vec::IntoIter<u8>;
    type Item = u8;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.bytes.into_iter()
    }
}
impl<'a> IntoIterator for &'a SpecialToken {
    type IntoIter = alloc::slice::Iter<'a, u8>;
    type Item = &'a u8;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.bytes.iter()
    }
}
impl From<SpecialToken> for Token {
    #[inline(always)]
    fn from(value: SpecialToken) -> Token {
        Token {
            id:    value.id,
            bytes: value.bytes,
        }
    }
}
impl From<&SpecialToken> for Token {
    #[inline(always)]
    fn from(value: &SpecialToken) -> Token {
        Token {
            id:    value.id,
            bytes: value.bytes.clone(),
        }
    }
}

/// List of tokens.
pub type Vocab = Vec<Token>;
/// List of special tokens.
pub type SpecialVocab = Vec<SpecialToken>;
/// List of token scores.
pub type Scores = Vec<TokenScore>;
