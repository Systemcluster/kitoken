//! Character mapping structure for normalization.

use alloc::vec::Vec;

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

use crate::convert::ConversionError;

/// Character mapping structure for normalization.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct CharsMap {}

impl TryFrom<&[u8]> for CharsMap {
    type Error = ConversionError;

    fn try_from(_: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}
impl TryFrom<Vec<u8>> for CharsMap {
    type Error = ConversionError;

    fn try_from(_: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}
