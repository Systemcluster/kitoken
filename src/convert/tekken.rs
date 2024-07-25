#[cfg(feature = "std")]
use std::fs::File;
#[cfg(feature = "std")]
use std::io::Read;
#[cfg(feature = "std")]
use std::path::Path;

use alloc::format;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::convert::ConversionError;
use crate::{
    Configuration, Definition, Fallback, InsertionPosition, Kitoken, Metadata, Model, Regex,
    SpecialToken, SpecialTokenKind, SpecialVocab, Split, SplitBehavior, Template, Token, Vocab,
};

mod ms {
    use alloc::string::String;
    use alloc::vec::Vec;
    use base64::{alphabet, engine, Engine};
    use serde::{Deserialize, Deserializer};

    static BASE64: engine::GeneralPurpose =
        const { engine::GeneralPurpose::new(&alphabet::STANDARD, engine::general_purpose::PAD) };

    fn from_base64<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>, {
        let s: &str = Deserialize::deserialize(deserializer)?;
        let precompiled_charsmap =
            BASE64.decode(s).map_err(|e| serde::de::Error::custom(e.to_string()))?.to_vec();
        Ok(precompiled_charsmap)
    }

    #[derive(Deserialize, Debug, Clone, PartialEq)]
    pub struct Config {
        pub pattern:                    String,
        #[allow(unused)]
        pub num_vocab_tokens:           Option<usize>,
        #[allow(unused)]
        pub default_vocab_size:         Option<usize>,
        pub default_num_special_tokens: Option<usize>,
        pub version:                    String,
    }

    #[derive(Deserialize, Debug, Clone, PartialEq)]
    pub struct Token {
        pub rank:        usize,
        #[serde(deserialize_with = "from_base64")]
        pub token_bytes: Vec<u8>,
        #[allow(unused)]
        pub token_str:   Option<String>,
    }

    #[derive(Deserialize, Debug, Clone, PartialEq)]
    pub struct Tokenizer {
        pub config: Config,
        pub vocab:  Vec<Token>,
    }
}

use ms::Tokenizer;

/// Converts a `tekken` tokenizer definition into the definition format used by this crate.
///
/// `data` is the JSON data used by the `tekken` library, commonly stored as `tekken.json`.
///
/// Returns the tokenizer definition, or an error if the conversion fails.
///
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use kitoken::convert::convert_tekken;
/// use kitoken::Kitoken;
///
/// let data = std::fs::read("tests/models/tekken/nemo.json")?;
/// let definition = convert_tekken(data).unwrap();
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
/// The `tekken` definition is composed of a JSON object with the following fields:
///
/// - `config`: The model configuration with the following fields:
///   - `pattern`: The regex pattern used to split the input.
///   - `num_vocab_tokens`: The number of vocabulary tokens.
///   - `default_vocab_size`: The default vocabulary size.
///   - `default_num_special_tokens`: The default number of special tokens.
///   - `version`: The version of the model.
/// - `vocab`: The vocabulary with elements with the following fields:
///   - `rank`: The rank of the token.
///   - `token_bytes`: The token bytes.
///   - `token_str`: The string representation of the token.
///
/// See the [tekken documentation](https://docs.mistral.ai/guides/tokenization/) for more information.
pub fn convert_tekken(data: impl AsRef<[u8]>) -> Result<Definition, ConversionError> {
    let data = data.as_ref();

    let tokenizer = serde_json::from_slice::<Tokenizer>(data)
        .map_err(|e| ConversionError::InvalidData(format!("invalid JSON: {}", e)))?;

    if tokenizer.config.version != "v3" {
        return Err(ConversionError::UnsupportedConfiguration(format!(
            "unsupported version: {}",
            tokenizer.config.version
        )));
    }

    let specials = &[
        ("<unk>", Some("unk".to_string()), false),
        ("<s>", Some("bos".to_string()), false),
        ("</s>", Some("eos".to_string()), false),
        ("[INST]", None, true),
        ("[/INST]", None, true),
        ("[AVAILABLE_TOOLS]", None, true),
        ("[/AVAILABLE_TOOLS]", None, true),
        ("[TOOL_RESULTS]", None, true),
        ("[/TOOL_RESULTS]", None, true),
        ("[TOOL_CALLS]", None, true),
        ("<pad>", Some("pad".to_string()), false),
        ("[PREFIX]", None, true),
        ("[MIDDLE]", None, true),
        ("[SUFFIX]", None, true),
    ];

    let specials_len = tokenizer.config.default_num_special_tokens.unwrap_or(specials.len());
    let vocab_len = tokenizer.config.default_vocab_size.unwrap_or(tokenizer.vocab.len());
    if vocab_len > tokenizer.vocab.len() + specials_len {
        return Err(ConversionError::InvalidData(format!(
            "too many tokens: {} > {}",
            vocab_len,
            tokenizer.vocab.len() + specials_len
        )));
    }

    if vocab_len >= u32::MAX as usize {
        return Err(ConversionError::InvalidData(format!(
            "too many pieces in vocab: {}",
            vocab_len
        )));
    }
    if specials_len >= u32::MAX as usize {
        return Err(ConversionError::InvalidData(format!(
            "too many pieces in specials: {}",
            specials_len
        )));
    }

    let mut config = Configuration::default();
    config.fallback.push(Fallback::Unknown);
    config.fallback.push(Fallback::Skip);

    config.split.push(Split::Pattern {
        pattern:  Regex::new(&tokenizer.config.pattern)?.into(),
        behavior: SplitBehavior::Isolate,
    });

    let mut specials = specials
        .iter()
        .enumerate()
        .map(|(i, (s, d, e))| SpecialToken {
            id:      i as u32,
            bytes:   s.as_bytes().to_vec(),
            kind:    SpecialTokenKind::Control,
            ident:   d.clone(),
            score:   i as f32,
            extract: *e,
        })
        .collect::<SpecialVocab>();
    specials[0].kind = SpecialTokenKind::Unknown;
    if specials.len() < specials_len {
        for i in specials.len()..specials_len {
            specials.push(SpecialToken {
                id:      i as u32,
                bytes:   format!("<SPECIAL_{}>", i).as_bytes().to_vec(),
                kind:    SpecialTokenKind::Control,
                ident:   None,
                score:   i as f32,
                extract: true,
            });
        }
    }
    specials.sort();

    let mut vocab = Vocab::with_capacity(vocab_len);
    // This will throw away any tokens beyond the set vocab size.
    // This is consistent with the behavior of `tekken`.
    for token in tokenizer.vocab.into_iter().take(vocab_len - specials.len()) {
        vocab.push(Token {
            id:    token.rank as u32 + specials.len() as u32,
            bytes: token.token_bytes,
        });
    }
    vocab.sort();

    let model = Model::BytePair {
        vocab,
        chars: false,
    };

    config.templates.push(Template {
        content:  "<s>".to_string(),
        position: InsertionPosition::SequenceStart,
    });
    config.templates.push(Template {
        content:  "</s>".to_string(),
        position: InsertionPosition::SequenceEnd,
    });

    let meta = Metadata {
        source: "tekken".to_string(),
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
    /// Converts a `tekken` model into the encoder format used by this crate.
    /// See [`convert_tekken`] for more details.
    #[cfg(feature = "std")]
    pub fn from_tekken_reader<R: Read>(reader: &mut R) -> Result<Self, ConversionError> {
        let mut data = Vec::with_capacity(1024);
        reader.read_to_end(&mut data)?;
        Self::from_tekken_slice(&data)
    }

    /// Converts a `tekken` model into the encoder format used by this crate.
    /// See [`convert_tekken`] for more details.
    #[cfg(feature = "std")]
    pub fn from_tekken_file(path: impl AsRef<Path>) -> Result<Self, ConversionError> {
        let mut file = File::open(path)?;
        Self::from_tekken_reader(&mut file)
    }

    /// Converts a `tekken` tokenizer definition into the encoder format used by this crate.
    /// See [`convert_tekken`] for more details.
    pub fn from_tekken_slice(data: &[u8]) -> Result<Self, ConversionError> {
        convert_tekken(data)
    }
}

impl Kitoken {
    /// Initializes the tokenizer from a `tekken` model.
    /// See [`convert_tekken`] for more details.
    #[cfg(feature = "std")]
    pub fn from_tekken_reader<R: Read>(reader: &mut R) -> Result<Self, ConversionError> {
        Ok(Self::from_definition(Definition::from_tekken_reader(reader)?)?)
    }

    /// Initializes the tokenizer from a `tekken` model.
    /// See [`convert_tekken`] for more details.
    #[cfg(feature = "std")]
    pub fn from_tekken_file(path: impl AsRef<Path>) -> Result<Self, ConversionError> {
        Ok(Self::from_definition(Definition::from_tekken_file(path)?)?)
    }

    /// Initializes the tokenizer from a `tekken` model.
    /// See [`convert_tekken`] for more details.
    pub fn from_tekken_slice(data: &[u8]) -> Result<Self, ConversionError> {
        Ok(Self::from_definition(Definition::from_tekken_slice(data)?)?)
    }
}
