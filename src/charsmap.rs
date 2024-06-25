//! Character mapping structure for custom normalization rules.
//! Based on the SentencePiece DoubleArray implementation.

use core::fmt::Debug;

use alloc::format;
use alloc::string::ToString;
use alloc::vec::Vec;

#[cfg(feature = "normalization-charsmap")]
use alloc::string::String;

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

use crate::convert::ConversionError;

#[allow(unused)]
trait UnitExt {
    fn value(&self) -> isize;
    fn label(&self) -> usize;
    fn offset(&self) -> usize;
    fn has_leaf(&self) -> bool;
}
#[allow(unused)]
impl UnitExt for u32 {
    #[inline(always)]
    fn value(&self) -> isize {
        let s = *self as usize;
        (s & ((1 << 31) - 1)) as isize
    }

    #[inline(always)]
    fn label(&self) -> usize {
        let s = *self as usize;
        s & ((1 << 31) | 0xFF)
    }

    #[inline(always)]
    fn offset(&self) -> usize {
        let s = *self as usize;
        (s >> 10) << ((s & (1 << 9)) >> 6)
    }

    #[inline(always)]
    fn has_leaf(&self) -> bool {
        let s = *self as usize;
        (s >> 8) & 1 == 1
    }
}

/// Character mapping structure for custom normalization rules.
///
/// Based on the SentencePiece DoubleArray implementation.
#[derive(Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct CharsMap {
    array:      Vec<u32>,
    normalized: Vec<u8>,
}
#[cfg(feature = "normalization-charsmap")]
impl CharsMap {
    /// Returns the prefix of the input key.
    #[inline(never)]
    fn prefix(&self, key: &[u8]) -> Vec<isize> {
        let mut posit = 0;
        let mut result = Vec::new();
        let mut unit = self.array[posit];
        posit ^= unit.offset();
        for &c in key {
            if c == 0 {
                break;
            }
            posit ^= c as usize;
            unit = self.array[posit];
            if unit.label() != c as usize {
                return result;
            }
            posit ^= unit.offset();
            if unit.has_leaf() {
                result.push(self.array[posit].value());
            }
        }
        result
    }

    /// Transforms the input chunk according to the character mapping.
    #[inline(never)]
    fn transform(&self, chunk: &str) -> Option<&[u8]> {
        let prefix = self.prefix(chunk.as_bytes());
        if prefix.is_empty() {
            None
        } else {
            let start = prefix[0] as usize;
            let mut end = start;
            while end < self.normalized.len() {
                if *self.normalized.get(end)? == 0 {
                    break;
                }
                end += 1;
            }
            Some(&self.normalized[start..end])
        }
    }

    /// Normalizes the input string according to the character mapping.
    #[inline(never)]
    pub fn normalize(&self, original: &str) -> String {
        use bstr::ByteSlice;
        let mut result = String::with_capacity(original.len());
        original.as_bytes().graphemes().for_each(|grapheme| {
            if grapheme.len() < 6 {
                if let Some(transformed) = self.transform(grapheme) {
                    for c in transformed.chars() {
                        result.push(c);
                    }
                    return;
                }
            }
            for (i, c) in grapheme.char_indices() {
                let part = &grapheme[i..i + c.len_utf8()];
                if let Some(transformed) = self.transform(part) {
                    for c in transformed.chars() {
                        result.push(c);
                    }
                } else {
                    result.push(c);
                }
            }
        });
        result
    }
}
impl Debug for CharsMap {
    #[inline(never)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use bstr::ByteSlice;
        f.debug_struct("CharsMap")
            .field("array", &format!("Vec({})", self.array.len()))
            .field("normalized", &format!("String({})", &self.normalized.as_bstr().len()))
            .finish()
    }
}
impl TryFrom<&[u8]> for CharsMap {
    type Error = ConversionError;

    #[inline(never)]
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < 4 {
            return Err(ConversionError::InvalidData("CharsMap data too short".to_string()));
        }
        let size = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        if data.len() < 4 + size {
            return Err(ConversionError::InvalidData("CharsMap data too short".to_string()));
        }
        let mut bytes = [0u8; 4];
        let array = data[4..size]
            .chunks_exact(4)
            .map(|chunk| {
                bytes.copy_from_slice(chunk);
                u32::from_le_bytes(bytes)
            })
            .collect();
        let normalized = data[4 + size..].to_vec();
        Ok(Self { array, normalized })
    }
}
impl TryFrom<Vec<u8>> for CharsMap {
    type Error = ConversionError;

    #[inline(always)]
    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(data.as_slice())
    }
}
impl TryFrom<&Vec<u8>> for CharsMap {
    type Error = ConversionError;

    #[inline(always)]
    fn try_from(data: &Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(data.as_slice())
    }
}
