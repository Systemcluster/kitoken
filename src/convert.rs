//! Definitions for converting and initializing the tokenizer from different tokenizer formats.

use alloc::string::String;

use crate::InitializationError;

mod util;

#[cfg(feature = "convert-sentencepiece")]
mod sentencepiece;
#[cfg(feature = "convert-sentencepiece")]
pub use sentencepiece::*;

#[cfg(feature = "convert-tiktoken")]
mod tiktoken;
#[cfg(feature = "convert-tiktoken")]
pub use tiktoken::*;

#[cfg(feature = "convert-huggingface")]
mod huggingface;
#[cfg(feature = "convert-huggingface")]
pub use huggingface::*;


/// Errors encountered when the conversion fails.
#[non_exhaustive]
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum ConversionError {
    /// The data is invalid. See the error message for more information
    #[cfg_attr(feature = "std", error("invalid data: {0}"))]
    InvalidData(String),
    /// A string contains invalid utf-8.
    #[cfg_attr(feature = "std", error("invalid utf-8: {0}"))]
    InvalidUtf8(String),
    /// A string contains invalid base64.
    #[cfg_attr(feature = "std", error("invalid base64: {0}"))]
    InvalidBase64(String),
    /// A string contains an invalid number.
    #[cfg_attr(feature = "std", error("invalid token {0}"))]
    InvalidNumber(String),
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
