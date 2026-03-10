//! Definitions for loading tokenizer models from the web.

use std::string::ToString;

use alloc::string::String;

use crate::{Definition, DeserializationError, InitializationError, Kitoken, util};

/// Errors encountered when loading models from the web.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum WebRequestError {
    /// The URL is invalid.
    #[error("{0}")]
    InvalidURL(String),
    /// The web request failed.
    #[error("{0}")]
    RequestError(#[from] reqwest::Error),
    /// The tokenizer failed to deserialize.
    #[error("{0}")]
    DeserializationError(#[from] DeserializationError),
}
impl From<InitializationError> for WebRequestError {
    fn from(e: InitializationError) -> Self {
        Self::DeserializationError(e.into())
    }
}
impl From<std::io::Error> for WebRequestError {
    fn from(e: std::io::Error) -> Self {
        Self::DeserializationError(e.into())
    }
}

impl Definition {
    /// Loads the tokenizer definition from the web.
    pub fn from_web(url: &str) -> Result<Self, WebRequestError> {
        let url = util::parse_url(url);
        if !url.starts_with("http:") && !url.starts_with("https:") {
            return Err(WebRequestError::InvalidURL(url.to_string()));
        };
        let mut definition = match reqwest::blocking::get(&url).and_then(|r| r.bytes()) {
            Ok(r) => Definition::from_slice(&r).map_err(|e| e.into()),
            Err(e) => return Err(e.into()),
        };
        if let Ok(ref mut definition) = definition {
            definition.meta.source = url;
        }
        definition
    }
}

impl Kitoken {
    /// Loads the tokenizer definition from the web and initializes the tokenizer.
    /// See [`Kitoken::from_definition`] for more details.
    pub fn from_web(url: &str) -> Result<Self, WebRequestError> {
        let definition = Definition::from_web(url)?;
        Ok(Self::from_definition(definition)?)
    }
}
