#[cfg(feature = "std")]
use std::fs::File;
#[cfg(feature = "std")]
use std::io::Read;
#[cfg(feature = "std")]
use std::path::Path;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use base64::{alphabet, engine, Engine};
use bstr::ByteSlice;

use crate::convert::ConversionError;
use crate::{
    Configuration, Definition, Fallback, InsertionPosition, Kitoken, Metadata, Model, Regex,
    SpecialToken, SpecialTokenKind, SpecialVocab, Split, SplitBehavior, Template, Vocab,
};

static BASE64: engine::GeneralPurpose =
    const { engine::GeneralPurpose::new(&alphabet::STANDARD, engine::general_purpose::PAD) };

/// Converts a `tiktoken` tokenizer definition into the definition format used by this crate.
///
/// `data` is the raw data format used by the `tiktoken` tokenizer.
///
/// Returns the tokenizer definition, or an error if the conversion fails.
///
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use kitoken::convert::convert_tiktoken;
/// use kitoken::Kitoken;
///
/// let data = std::fs::read("tests/models/tiktoken/cl100k_base.tiktoken")?;
/// let definition = convert_tiktoken(data).unwrap();
///
/// let tokenizer = Kitoken::try_from(definition).unwrap();
/// # Ok(())
/// # }
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

    let mut vocab = Vocab::with_capacity(lines.len());
    for (i, line) in lines.into_iter().enumerate() {
        let (l, r) = line
            .split_once_str(" ")
            .ok_or_else(|| ConversionError::InvalidData(format!("wrong format in line {i}")))?;
        let bytes = BASE64.decode(l).map_err(|e| {
            ConversionError::InvalidData(format!("invalid base64 in line {i}: {}", e))
        })?;
        let token = r
            .as_bstr()
            .to_str()
            .map_err(|e| ConversionError::InvalidData(format!("invalid utf-8 in line {i}: {}", e)))?
            .parse::<u32>()
            .map_err(|e| {
                ConversionError::InvalidData(format!("invalid number in line {i}: {}", e))
            })?;
        vocab.push((bytes, token).into());
    }

    let mut config = Configuration::default();
    config.fallback.push(Fallback::Skip);

    let mut specials = Vec::<(String, u32)>::with_capacity(2048);
    let reserved = move |name, count, start, pos| {
        (start..count + start)
            .enumerate()
            .map(move |(n, i)| (format!("<|{name}reserved_special_token_{i}|>"), (pos + n) as u32))
    };
    let sequential = move |list: &'static [&'static str], pos| {
        list.iter().enumerate().map(move |(n, s)| (s.to_string(), (pos + n) as u32))
    };
    match vocab.len() {
        len @ 200000 => {
            log::debug!("Detected llama4 vocab");
            config.split.push(Split::Pattern { pattern:
                Regex::new(&[
                    r"[^\r\n\p{L}\p{N}]?[\p{Lu}\p{Lt}\p{Lm}\p{Lo}\p{M}]*[\p{Ll}\p{Lm}\p{Lo}\p{M}]+(?i:'s|'t|'re|'ve|'m|'ll|'d)?",
                    r"[^\r\n\p{L}\p{N}]?[\p{Lu}\p{Lt}\p{Lm}\p{Lo}\p{M}]+[\p{Ll}\p{Lm}\p{Lo}\p{M}]*(?i:'s|'t|'re|'ve|'m|'ll|'d)?",
                    r"\p{N}{1,3}",
                    r" ?[^\s\p{L}\p{N}]+[\r\n/]*",
                    r"\s*[\r\n]+",
                    r"\s+(?!\S)",
                ].join("|"))?.into(),
                behavior: SplitBehavior::Isolate
            });
            config.templates.push(Template {
                content:  "<|begin_of_text|>".to_string(),
                position: InsertionPosition::SequenceStart,
            });
            config.templates.push(Template {
                content:  "<|end_of_text|>".to_string(),
                position: InsertionPosition::SequenceEnd,
            });
            // Ref: https://github.com/meta-llama/llama-models/blob/main/models/llama4/tokenizer.py
            specials.extend(sequential(
                &[
                    "<|begin_of_text|>",
                    "<|end_of_text|>",
                    "<|fim_prefix|>",
                    "<|fim_middle|>",
                    "<|fim_suffix|>",
                    "<|header_start|>",
                    "<|header_end|>",
                    "<|eom|>",
                    "<|eot|>",
                    "<|step|>",
                ],
                len,
            ));
            specials.extend(reserved("text_post_train_", 6, 0, len + specials.len()));
            specials.extend(sequential(
                &[
                    "<|python_start|>",
                    "<|python_end|>",
                    "<|finetune_right_pad|>",
                ],
                len + specials.len(),
            ));
            specials.extend(reserved("text_post_train_", 61, 8, len + specials.len()));
            specials.extend(sequential(
                &[
                    "<|image_start|>",
                    "<|image_end|>",
                    "<|vision_reserved_special_token_0|>",
                    "<|vision_reserved_special_token_1|>",
                    "<|tile_x_separator|>",
                    "<|tile_y_separator|>",
                    "<|vision_reserved_special_token_2|>",
                    "<|vision_reserved_special_token_3|>",
                    "<|vision_reserved_special_token_4|>",
                    "<|vision_reserved_special_token_5|>",
                    "<|image|>",
                    "<|vision_reserved_special_token_6|>",
                    "<|patch|>",
                ],
                len + specials.len(),
            ));
            specials.extend(reserved("vision_", 1041, 7, len + specials.len()));
            specials.extend(reserved("reasoning_", 7, 0, len + specials.len()));
            specials.extend(sequential(
                &["<|reasoning_thinking_start|>", "<|reasoning_thinking_end|>"],
                len + specials.len(),
            ));
        }
        199990.. => {
            log::debug!("Detected o200k vocab");
            config.split.push(Split::Pattern { pattern:
            Regex::new(&[
                r"[^\r\n\p{L}\p{N}]?[\p{Lu}\p{Lt}\p{Lm}\p{Lo}\p{M}]*[\p{Ll}\p{Lm}\p{Lo}\p{M}]+(?i:'s|'t|'re|'ve|'m|'ll|'d)?",
                r"[^\r\n\p{L}\p{N}]?[\p{Lu}\p{Lt}\p{Lm}\p{Lo}\p{M}]+[\p{Ll}\p{Lm}\p{Lo}\p{M}]*(?i:'s|'t|'re|'ve|'m|'ll|'d)?",
                r"\p{N}{1,3}",
                r" ?[^\s\p{L}\p{N}]+[\r\n/]*",
                r"\s*[\r\n]+",
                r"\s+(?!\S)",
            ].join("|"))?.into(),
            behavior: SplitBehavior::Isolate
        });
            specials.extend([
                ("<|endoftext|>".to_string(), 199999),
                ("<|endofprompt|>".to_string(), 200018),
            ]);
        }
        100000.. => {
            log::debug!("Detected cl100k vocab");
            config.split.push(Split::Pattern { pattern:
            Regex::new(r"'(?i:[sdmt]|ll|ve|re)|[^\r\n\p{L}\p{N}]?+\p{L}+|\p{N}{1,3}| ?[^\s\p{L}\p{N}]++[\r\n]*|\s*[\r\n]|\s+(?!\S)")?.into(),
            behavior: SplitBehavior::Isolate
        });
            specials.extend(
                [
                    ("<|endoftext|>", 100257),
                    ("<|fim_prefix|>", 100258),
                    ("<|fim_middle|>", 100259),
                    ("<|fim_suffix|>", 100260),
                    ("<|endofprompt|>", 100276),
                    ("<|im_start|>", 100264),
                    ("<|im_end|>", 100265),
                ]
                .map(|(s, n)| (s.to_string(), n)),
            );
        }
        _ => {
            log::debug!("Detected p50k vocab");
            config.split.push(Split::Pattern {
                pattern:  Regex::new(
                    r"'(?:[sdmt]|ll|ve|re)|\s?\p{L}+|\s?\p{N}+|\s?[^\s\p{L}\p{N}]+",
                )?
                .into(),
                behavior: SplitBehavior::Isolate,
            });
            specials.extend(
                [
                    ("<|endoftext|>", 50256),
                    ("<|fim_prefix|>", 50281),
                    ("<|fim_middle|>", 50282),
                    ("<|fim_suffix|>", 50283),
                ]
                .map(|(s, n)| (s.to_string(), n)),
            );
        }
    };
    let mut specials = specials
        .iter()
        .enumerate()
        .map(|(i, &(ref s, t))| SpecialToken {
            id:      t,
            bytes:   s.as_bytes().to_vec(),
            kind:    SpecialTokenKind::Control,
            ident:   match s.as_str() {
                "<|begin_of_text|>" => Some("bos"),
                "<|end_of_text|>" | "<|endoftext|>" => Some("eos"),
                "<|eot|>" => Some("eot"),
                "<|eom|>" => Some("eom"),
                "<|finetune_right_pad|>" => Some("pad"),
                _ => None,
            }
            .map(|s| s.to_string()),
            score:   i as f32,
            extract: true,
        })
        .collect::<SpecialVocab>();
    specials.sort();

    let model = Model::BytePair {
        vocab,
        chars: false,
    };

    let meta = Metadata {
        source: "tiktoken".to_string(),
        ..Metadata::default()
    };

    Ok(Definition {
        meta,
        model,
        specials,
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
