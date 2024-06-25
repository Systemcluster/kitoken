//! Utilities for converting different tokenizer formats into Kitoken definitions.
//!
//! Additional methods for initializing from supported formats are also available in [`Definition`](crate::Definition) and [`Kitoken`](crate::Kitoken).

use alloc::string::String;

use crate::{InitializationError, RegexError};

#[cfg(feature = "convert-sentencepiece")]
mod sentencepiece;
#[cfg(feature = "convert-sentencepiece")]
pub use sentencepiece::*;

#[cfg(feature = "convert-tiktoken")]
mod tiktoken;
#[cfg(feature = "convert-tiktoken")]
pub use tiktoken::*;

#[cfg(feature = "convert-tokenizers")]
mod tokenizers;
#[cfg(feature = "convert-tokenizers")]
pub use tokenizers::*;

/// Errors encountered when the conversion fails.
#[non_exhaustive]
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum ConversionError {
    /// The data is invalid. See the error message for more information.
    #[cfg_attr(feature = "std", error("invalid data: {0}"))]
    InvalidData(String),
    /// The configuration is not supported.
    #[cfg_attr(feature = "std", error("unsupported configuration: {0}"))]
    UnsupportedConfiguration(String),
    /// A regex failed to compile.
    #[cfg_attr(feature = "std", error("invalid regex: {0}"))]
    InvalidRegex(String),
    /// The tokenizer failed to initialize.
    #[cfg_attr(feature = "std", error("{0}"))]
    InitializationError(InitializationError),
    /// Reading the data failed.
    #[cfg(feature = "std")]
    #[error("{0}")]
    IOError(#[from] std::io::Error),
}
impl From<InitializationError> for ConversionError {
    fn from(e: InitializationError) -> Self {
        Self::InitializationError(e)
    }
}
impl From<RegexError> for ConversionError {
    fn from(e: RegexError) -> Self {
        Self::InvalidRegex(e.0)
    }
}
