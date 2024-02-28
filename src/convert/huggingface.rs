#[cfg(feature = "std")]
use std::fs::File;
#[cfg(feature = "std")]
use std::io::Read;
#[cfg(feature = "std")]
use std::path::Path;

use alloc::format;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::cmp::Ordering;

use base64::{alphabet, engine, Engine};
use hashbrown::{HashMap, HashSet};
use serde::{Deserialize, Deserializer};

use crate::convert::ConversionError;
use crate::{
    Configuration, Definition, DefinitionSource, Kitoken, Metadata, Mode, Scores,
    UnicodeNormalization, Vocab,
};

static BASE64: engine::GeneralPurpose =
    engine::GeneralPurpose::new(&alphabet::STANDARD, engine::general_purpose::PAD);


fn from_base64<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>, {
    let s: &str = Deserialize::deserialize(deserializer)?;
    let precompiled_charsmap =
        BASE64.decode(s).map_err(|e| serde::de::Error::custom(e.to_string()))?;
    Ok(precompiled_charsmap)
}

fn default_true() -> bool {
    true
}

fn default_split() -> HfSplitDelimiterBehavior {
    HfSplitDelimiterBehavior::Isolated
}

fn default_right() -> HfTruncationDirection {
    HfTruncationDirection::Right
}

fn default_prepend_scheme() -> HfPrependScheme {
    HfPrependScheme::Always
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
struct HfBPE {
    dropout:                   Option<f64>,
    unk_token:                 Option<String>,
    continuing_subword_prefix: Option<String>,
    end_of_word_suffix:        Option<String>,
    fuse_unk:                  Option<bool>,
    byte_fallback:             Option<bool>,
    vocab:                     HashMap<String, u32>,
    merges:                    Vec<String>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
struct HfUnigram {
    unk_id:        Option<u64>,
    vocab:         Vec<(String, f64)>,
    byte_fallback: Option<bool>,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
enum HfModel {
    BPE(HfBPE),
    Unigram(HfUnigram),
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
enum HfPattern {
    String(String),
    Regex(String),
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
enum HfNormalizer {
    BertNormalizer {
        clean_text:           bool,
        handle_chinese_chars: bool,
        strip_accents:        bool,
        lowercase:            bool,
    },
    StripNormalizer {
        strip_left:  bool,
        strip_right: bool,
    },
    StripAccents,
    NFC,
    NFD,
    NFKC,
    NFKD,
    Sequence {
        normalizers: Vec<HfNormalizer>,
    },
    Lowercase,
    Nmt,
    Precompiled {
        #[serde(deserialize_with = "from_base64")]
        precompiled_charsmap: Vec<u8>,
    },
    Replace {
        pattern: HfPattern,
        content: String,
    },
    Prepend {
        prepend: String,
    },
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
enum HfPrependScheme {
    First,
    Never,
    Always,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
enum HfSplitDelimiterBehavior {
    Removed,
    Isolated,
    MergedWithPrevious,
    MergedWithNext,
    Contiguous,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
enum HfPreTokenizer {
    BertPreTokenizer {
        clean_text:           bool,
        handle_chinese_chars: bool,
        strip_accents:        bool,
        lowercase:            bool,
    },
    ByteLevel {
        add_prefix_space: bool,
        trim_offsets:     bool,
        #[serde(default = "default_true")]
        use_regex:        bool,
    },
    Delimiter {
        delimiter: String,
    },
    Metaspace {
        replacement:      String,
        add_prefix_space: bool,
    },
    Whitespace,
    Sequence {
        pretokenizers: Vec<HfPreTokenizer>,
    },
    Split {
        pattern:  HfPattern,
        behavior: HfSplitDelimiterBehavior,
        invert:   bool,
    },
    Punctuation {
        #[serde(default = "default_split")]
        behavior: HfSplitDelimiterBehavior,
    },
    WhitespaceSplit,
    Digits {
        individual_digits: bool,
    },
    UnicodeScripts,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
enum HfTemplateSequence {
    A,
    B,
}
#[derive(Deserialize, Debug, Clone, PartialEq)]
enum HfTemplatePiece {
    Sequence {
        id:      HfTemplateSequence,
        type_id: u32,
    },
    SpecialToken {
        id:      String,
        type_id: u32,
    },
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
struct HfSpecialToken {
    id:     String,
    ids:    Vec<u32>,
    tokens: Vec<String>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
enum HfPostProcessor {
    RobertaProcessing {
        sep:              (String, u32),
        cls:              (String, u32),
        trim_offsets:     bool,
        add_prefix_space: bool,
    },
    BertProcessing {
        sep: (String, u32),
        cls: (String, u32),
    },
    ByteLevel {
        add_prefix_space: bool,
        trim_offsets:     bool,
        #[serde(default = "default_true")]
        use_regex:        bool,
    },
    TemplateProcessing {
        single:         Vec<HfTemplatePiece>,
        pair:           Vec<HfTemplatePiece>,
        special_tokens: HashMap<String, HfSpecialToken>,
    },
    Sequence {
        processors: Vec<HfPostProcessor>,
    },
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
enum HfDecoder {
    BPE {
        suffix: String,
    },
    ByteLevel {
        add_prefix_space: bool,
        trim_offsets:     bool,
        #[serde(default = "default_true")]
        use_regex:        bool,
    },
    WordPiece {
        prefix:  String,
        cleanup: bool,
    },
    Metaspace {
        replacement:      char,
        add_prefix_space: bool,
        #[serde(default = "default_prepend_scheme")]
        prepend_scheme:   HfPrependScheme,
    },
    CTC {
        pad_token:            String,
        word_delimiter_token: String,
        cleanup:              bool,
    },
    Sequence {
        decoders: Vec<HfDecoder>,
    },
    Replace {
        pattern: HfPattern,
        content: String,
    },
    Fuse,
    Strip {
        content: char,
        start:   u64,
        stop:    u64,
    },
    ByteFallback,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
enum HfTruncationDirection {
    Left,
    Right,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
enum HfTruncationStrategy {
    LongestFirst,
    OnlyFirst,
    OnlySecond,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
struct HfTruncationParams {
    #[serde(default = "default_right")]
    pub direction:  HfTruncationDirection,
    pub max_length: usize,
    pub strategy:   HfTruncationStrategy,
    pub stride:     usize,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
enum HfPaddingDirection {
    Left,
    Right,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
enum HfPaddingStrategy {
    BatchLongest,
    Fixed(usize),
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
struct HfPaddingParams {
    pub strategy:           HfPaddingStrategy,
    pub direction:          HfPaddingDirection,
    pub pad_to_multiple_of: Option<usize>,
    pub pad_id:             u32,
    pub pad_type_id:        u32,
    pub pad_token:          String,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
struct HfTokenizer {
    normalizer:     Option<HfNormalizer>,
    pre_tokenizer:  Option<HfPreTokenizer>,
    model:          HfModel,
    post_processor: Option<HfPostProcessor>,
    decoder:        Option<HfDecoder>,

    truncation: Option<HfTruncationParams>,
    padding:    Option<HfPaddingParams>,
}

pub fn convert_huggingface(data: impl AsRef<[u8]>) -> Result<Definition, ConversionError> {
    let data = data.as_ref();

    let mut config = Configuration {
        ..Configuration::default()
    };


    let tokenizer = serde_json::from_slice::<HfTokenizer>(data);
    match tokenizer {
        Ok(_) => {
            unimplemented!()
        }
        Err(e) => {
            eprintln!("{:?}", e);
            unimplemented!()
        }
    }
}

impl Definition {
    /// Converts a `huggingface` tokenizer definition into the encoder format used by this crate.
    /// See [`convert_sentencepiece`] for more details.
    #[cfg(feature = "std")]
    pub fn from_huggingface_reader<R: Read>(reader: &mut R) -> Result<Self, ConversionError> {
        let mut data = Vec::with_capacity(1024);
        reader.read_to_end(&mut data)?;
        Self::from_huggingface_slice(&data)
    }

    /// Converts a `huggingface` tokenizer definition into the encoder format used by this crate.
    /// See [`convert_huggingface`] for more details.
    #[cfg(feature = "std")]
    pub fn from_huggingface_file(path: impl AsRef<Path>) -> Result<Self, ConversionError> {
        let mut file = File::open(path)?;
        Self::from_huggingface_reader(&mut file)
    }

    /// Converts a `huggingface` tokenizer definition into the encoder format used by this crate.
    /// See [`convert_huggingface`] for more details.
    pub fn from_huggingface_slice(data: &[u8]) -> Result<Self, ConversionError> {
        convert_huggingface(data)
    }
}

impl Kitoken {
    /// Initializes the tokenizer from a `huggingface` tokenizer definition.
    /// See [`convert_huggingface`] for more details.
    #[cfg(feature = "std")]
    pub fn from_huggingface_reader<R: Read>(reader: &mut R) -> Result<Self, ConversionError> {
        Ok(Self::from_definition(Definition::from_huggingface_reader(reader)?)?)
    }

    /// Initializes the tokenizer from a `huggingface` tokenizer definition.
    /// See [`convert_huggingface`] for more details.
    #[cfg(feature = "std")]
    pub fn from_huggingface_file(path: impl AsRef<Path>) -> Result<Self, ConversionError> {
        Ok(Self::from_definition(Definition::from_huggingface_file(path)?)?)
    }

    /// Initializes the tokenizer from a `huggingface` tokenizer definition.
    /// See [`convert_huggingface`] for more details.
    pub fn from_huggingface_slice(data: &[u8]) -> Result<Self, ConversionError> {
        Ok(Self::from_definition(Definition::from_huggingface_slice(data)?)?)
    }
}
