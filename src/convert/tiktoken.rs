#[cfg(feature = "std")]
use std::fs::File;
#[cfg(feature = "std")]
use std::io::Read;
#[cfg(feature = "std")]
use std::path::Path;

use alloc::borrow::ToOwned;
use alloc::format;
use alloc::string::ToString;
use alloc::vec::Vec;

use base64::{alphabet, engine, Engine};
use bstr::ByteSlice;

use crate::convert::ConversionError;
use crate::{Configuration, Definition, DefinitionSource, Kitoken, Metadata};

static BASE64: engine::GeneralPurpose =
    engine::GeneralPurpose::new(&alphabet::STANDARD, engine::general_purpose::PAD);

/// Converts a `tiktoken` tokenizer definition into the definition format used by this crate.
///
/// `data` is the raw data format used by the `tiktoken` tokenizer.
///
/// Returns the tokenizer definition, or an error if the conversion fails.
///
/// # Examples
///
/// ```
/// use kitoken::convert::convert_tiktoken;
/// use kitoken::Kitoken;
///
/// let data = include_bytes!("../../tests/models/cl100k_base.tiktoken");
/// let definition = convert_tiktoken(data).unwrap();
///
/// let tokenizer = Kitoken::try_from(definition).unwrap();
/// ```
///
///  Additional conversion utilities are defined in [`Definition`] and [`Kitoken`].
///
/// # Format
///
/// The tiktoken definition data is composed of lines of the form `<token bytes> <token id>`,
/// where `<token bytes>` is a base64-encoded byte sequence and `<token id>` is a decimal number.
/// The lines are ordered by merge priority, with the first line having the highest priority.
///
/// Tiktoken definitions don't include special tokens or a split regex.
/// This function chooses values for both based on the number of tokens in the vocabulary according to the defaults used by the `tiktoken` tokenizer.
/// Depending on the data and requirements, these values may have to be adjusted manually.
pub fn convert_tiktoken(data: impl AsRef<[u8]>) -> Result<Definition, ConversionError> {
    let data = data.as_ref();
    let lines = data
        .split(|u| *u == b'\n')
        .map(|l| l.trim_with(|u| u == '\r'))
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>();

    let mut vocab = Vec::with_capacity(lines.len());
    for (i, line) in lines.into_iter().enumerate() {
        let (l, r) = line
            .split_once_str(" ")
            .ok_or_else(|| ConversionError::InvalidData(format!("wrong format in line {i}")))?;
        let bytes = BASE64.decode(l).map_err(|e| {
            ConversionError::InvalidBase64(format!("invalid base64 in line {i}: {}", e))
        })?;
        let token = r
            .as_bstr()
            .to_str()
            .map_err(|e| ConversionError::InvalidUtf8(format!("invalid utf-8 in line {i}: {}", e)))?
            .parse::<u32>()
            .map_err(|e| {
                ConversionError::InvalidNumber(format!("invalid number in line {i}: {}", e))
            })?;
        vocab.push((bytes, token.to_owned()));
    }

    let specials;
    let mut config = Configuration::default();
    if vocab.len() >= 100000 {
        config.split =
            r"(?i:'s|'t|'re|'ve|'m|'ll|'d)|[^\r\n\p{L}\p{N}]?\p{L}+|\p{N}{1,3}| ?[^\s\p{L}\p{N}]+[\r\n]*|\s*[\r\n]+|\s+(?!\S)|\s+".to_string();
        specials = Vec::from([
            ("<|endoftext|>".into(), 100257),
            ("<|fim_prefix|>".into(), 100258),
            ("<|fim_middle|>".into(), 100259),
            ("<|fim_suffix|>".into(), 100260),
            ("<|endofprompt|>".into(), 100276),
            ("<|im_start|>".into(), 100264),
            ("<|im_end|>".into(), 100265),
        ]);
    } else {
        config.split =
            r"'s|'t|'re|'ve|'m|'ll|'d| ?\p{L}+| ?\p{N}+| ?[^\s\p{L}\p{N}]+|\s+(?!\S)|\s+"
                .to_string();
        specials = Vec::from([
            ("<|endoftext|>".into(), 50256),
            ("<|fim_prefix|>".into(), 50281),
            ("<|fim_middle|>".into(), 50282),
            ("<|fim_suffix|>".into(), 50283),
        ]);
    }
    let scores = Vec::new();

    let meta = Metadata {
        source: DefinitionSource::Tiktoken,
        ..Metadata::default()
    };

    Ok(Definition {
        meta,
        vocab,
        specials,
        scores,
        config,
    })
}

impl Definition {
    /// Converts a `tiktoken` model into the encoder format used by this crate.
    /// See [`convert_tiktoken`] for more details.
    #[cfg(feature = "std")]
    pub fn from_tiktoken_reader<R: Read>(reader: &mut R) -> Result<Self, ConversionError> {
        let mut data = Vec::with_capacity(1024);
        reader.read_to_end(&mut data)?;
        Self::from_tiktoken_slice(&data)
    }

    /// Converts a `tiktoken` model into the encoder format used by this crate.
    /// See [`convert_tiktoken`] for more details.
    #[cfg(feature = "std")]
    pub fn from_tiktoken_file(path: impl AsRef<Path>) -> Result<Self, ConversionError> {
        let mut file = File::open(path)?;
        Self::from_tiktoken_reader(&mut file)
    }

    /// Converts a `tiktoken` tokenizer definition into the encoder format used by this crate.
    /// See [`convert_tiktoken`] for more details.
    pub fn from_tiktoken_slice(data: &[u8]) -> Result<Self, ConversionError> {
        convert_tiktoken(data)
    }
}

impl Kitoken {
    /// Initializes the tokenizer from a `tiktoken` model.
    /// See [`convert_tiktoken`] for more details.
    #[cfg(feature = "std")]
    pub fn from_tiktoken_reader<R: Read>(reader: &mut R) -> Result<Self, ConversionError> {
        Ok(Self::from_definition(Definition::from_tiktoken_reader(reader)?)?)
    }

    /// Initializes the tokenizer from a `tiktoken` model.
    /// See [`convert_tiktoken`] for more details.
    #[cfg(feature = "std")]
    pub fn from_tiktoken_file(path: impl AsRef<Path>) -> Result<Self, ConversionError> {
        Ok(Self::from_definition(Definition::from_tiktoken_file(path)?)?)
    }

    /// Initializes the tokenizer from a `tiktoken` model.
    /// See [`convert_tiktoken`] for more details.
    pub fn from_tiktoken_slice(data: &[u8]) -> Result<Self, ConversionError> {
        Ok(Self::from_definition(Definition::from_tiktoken_slice(data)?)?)
    }
}
