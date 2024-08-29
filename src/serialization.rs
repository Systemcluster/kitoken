//! Definitions for serializing and deserializing the tokenizer.

#[cfg(feature = "std")]
use std::fs::File;
#[cfg(feature = "std")]
use std::io::{Read, Result as IOResult, Write};

use alloc::string::{String, ToString};
use alloc::vec::Vec;

#[cfg(feature = "convert-detect")]
use crate::convert::ConversionError;
use crate::{Definition, InitializationError, Kitoken};

const MAGIC: &[u8] = b"kitoken";
const VERSION: &[u8] = &[0, 1];

/// Errors encountered when deserializing the tokenizer.
#[non_exhaustive]
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum DeserializationError {
    /// The data is invalid. See the error message for more information.
    #[cfg_attr(feature = "std", error("{0}"))]
    InvalidData(String),
    /// The tokenizer failed to initialize.
    #[cfg_attr(feature = "std", error("{0}"))]
    InitializationError(InitializationError),
    /// Reading the data failed.
    #[cfg(feature = "std")]
    #[error("{0}")]
    IOError(#[from] std::io::Error),
}
impl From<InitializationError> for DeserializationError {
    fn from(e: InitializationError) -> Self {
        Self::InitializationError(e)
    }
}

impl Definition {
    /// Deserializes the tokenizer definition from a reader.
    /// The format is detected automatically when the `convert-detect` feature is enabled.
    #[cfg(feature = "std")]
    pub fn from_reader<R: Read>(reader: &mut R) -> Result<Self, DeserializationError> {
        let data = {
            let mut data = Vec::new();
            reader.read_to_end(&mut data)?;
            data
        };
        Self::from_slice(&data)
    }

    /// Deserializes the tokenizer definition from a file.
    /// The format is detected automatically when the `convert-detect` feature is enabled.
    #[cfg(feature = "std")]
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, DeserializationError> {
        let mut file = File::open(path)?;
        Self::from_reader(&mut file)
    }

    #[cfg(not(feature = "convert-detect"))]
    /// Deserializes the tokenizer definition from bytes.
    /// The format is detected automatically when the `convert-detect` feature is enabled.
    pub fn from_slice(slice: &[u8]) -> Result<Self, DeserializationError> {
        if slice.len() < MAGIC.len() + VERSION.len() {
            return Err(DeserializationError::InvalidData("invalid size".to_string()));
        }
        if &slice[..MAGIC.len()] != MAGIC {
            return Err(DeserializationError::InvalidData("invalid magic".to_string()));
        }
        if &slice[MAGIC.len()..MAGIC.len() + VERSION.len()] != VERSION {
            return Err(DeserializationError::InvalidData("invalid version".to_string()));
        }
        let definition = postcard::from_bytes(&slice[MAGIC.len() + VERSION.len()..])
            .map_err(|e| DeserializationError::InvalidData(e.to_string()))?;
        Ok(definition)
    }

    #[cfg(feature = "convert-detect")]
    /// Deserializes the tokenizer definition from bytes.
    /// The format is detected automatically when the `convert-detect` feature is enabled.
    pub fn from_slice(slice: &[u8]) -> Result<Self, DeserializationError> {
        let formats = &[
            |slice: &[u8]| {
                if slice.len() < MAGIC.len() + VERSION.len() {
                    return Err(ConversionError::InvalidData("invalid size".to_string()));
                }
                if &slice[..MAGIC.len()] != MAGIC {
                    return Err(ConversionError::InvalidData("invalid magic".to_string()));
                }
                if &slice[MAGIC.len()..MAGIC.len() + VERSION.len()] != VERSION {
                    return Err(ConversionError::InvalidData("invalid version".to_string()));
                }
                postcard::from_bytes(&slice[MAGIC.len() + VERSION.len()..])
                    .map_err(|e| ConversionError::InvalidData(e.to_string()))
            },
            #[cfg(feature = "convert-tiktoken")]
            Definition::from_tiktoken_slice,
            #[cfg(feature = "convert-sentencepiece")]
            Definition::from_sentencepiece_slice,
            #[cfg(feature = "convert-tokenizers")]
            Definition::from_tokenizers_slice,
            #[cfg(feature = "convert-tekken")]
            Definition::from_tekken_slice,
        ];
        formats
            .iter()
            .find_map(|f| f(slice).ok())
            .ok_or_else(|| DeserializationError::InvalidData("unknown format".to_string()))
    }

    /// Serializes the tokenizer definition to a writer.
    #[cfg(feature = "std")]
    pub fn to_writer<W: Write>(&self, writer: &mut W) -> IOResult<()> {
        writer.write_all(MAGIC)?;
        writer.write_all(VERSION)?;
        let data = postcard::to_allocvec(self).unwrap();
        writer.write_all(&data)?;
        Ok(())
    }

    /// Serializes the tokenizer definition to a file.
    #[cfg(feature = "std")]
    pub fn to_file<P: AsRef<std::path::Path>>(&self, path: P) -> IOResult<()> {
        let mut file = File::create(path)?;
        self.to_writer(&mut file)
    }

    /// Serializes the tokenizer definition to bytes.
    pub fn to_vec(&self) -> Vec<u8> {
        let data = postcard::to_allocvec(self).unwrap();
        let mut vec = Vec::with_capacity(MAGIC.len() + VERSION.len() + data.len());
        vec.extend_from_slice(MAGIC);
        vec.extend_from_slice(VERSION);
        vec.extend_from_slice(&data);
        vec
    }
}

impl Kitoken {
    /// Deserializes the tokenizer definition from a reader and initializes the tokenizer.
    /// The format is detected automatically when the `convert-detect` feature is enabled.
    /// See [`Kitoken::from_definition`] for more details.
    #[cfg(feature = "std")]
    pub fn from_reader<R: Read>(reader: &mut R) -> Result<Self, DeserializationError> {
        let definition = Definition::from_reader(reader)?;
        Ok(Self::from_definition(definition)?)
    }

    /// Deserializes the tokenizer definition from a file and initializes the tokenizer.
    /// The format is detected automatically when the `convert-detect` feature is enabled.
    /// See [`Kitoken::from_definition`] for more details.
    #[cfg(feature = "std")]
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, DeserializationError> {
        let definition = Definition::from_file(path)?;
        Ok(Self::from_definition(definition)?)
    }

    /// Deserializes the tokenizer definition from bytes and initializes the tokenizer.
    /// The format is detected automatically when the `convert-detect` feature is enabled.
    /// See [`Kitoken::from_definition`] for more details.
    pub fn from_slice(slice: &[u8]) -> Result<Self, DeserializationError> {
        let definition = Definition::from_slice(slice)?;
        Ok(Self::from_definition(definition)?)
    }

    /// Creates a definition from this tokenizer and serializes it to a writer.
    /// See [`Kitoken::to_definition`] for more details.
    #[cfg(feature = "std")]
    pub fn to_writer<W: Write>(&self, writer: &mut W) -> IOResult<()> {
        let definition = self.to_definition();
        definition.to_writer(writer)
    }

    /// Creates a definition from this tokenizer and serializes it to a file.
    /// See [`Kitoken::to_definition`] for more details.
    #[cfg(feature = "std")]
    pub fn to_file<P: AsRef<std::path::Path>>(&self, path: P) -> IOResult<()> {
        let definition = self.to_definition();
        definition.to_file(path)
    }

    /// Creates a definition from this tokenizer and serializes it to bytes.
    /// See [`Kitoken::to_definition`] for more details.
    pub fn to_vec(&self) -> Vec<u8> {
        let definition = self.to_definition();
        definition.to_vec()
    }
}
