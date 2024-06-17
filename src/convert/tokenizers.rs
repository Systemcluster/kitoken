#[cfg(feature = "std")]
use std::fs::File;
#[cfg(feature = "std")]
use std::io::Read;
#[cfg(feature = "std")]
use std::path::Path;

use alloc::collections::VecDeque;
use alloc::format;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::cmp::Ordering;

use bstr::ByteSlice;
use hashbrown::HashMap;

use crate::convert::ConversionError;
use crate::{
    Configuration, Decoding, Definition, DefinitionSource, Kitoken, Metadata, Mode, Normalization,
    Processing, Regex, Scores, SpecialToken, SpecialTokenKind, Split, SplitBehavior,
    UnicodeNormalization, Vocab,
};

mod hf {
    use alloc::string::{String, ToString};
    use alloc::vec::Vec;

    use base64::{alphabet, engine, Engine};
    use hashbrown::HashMap;
    use serde::{Deserialize, Deserializer};

    static BASE64: engine::GeneralPurpose =
        engine::GeneralPurpose::new(&alphabet::STANDARD, engine::general_purpose::PAD);

    fn from_base64<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>, {
        let s: &str = Deserialize::deserialize(deserializer)?;
        let precompiled_charsmap =
            BASE64.decode(s).map_err(|e| serde::de::Error::custom(e.to_string()))?.to_vec();
        Ok(precompiled_charsmap)
    }

    fn default_true() -> bool {
        true
    }
    fn default_split() -> SplitDelimiterBehavior {
        SplitDelimiterBehavior::Isolated
    }
    fn default_right() -> TruncationDirection {
        TruncationDirection::Right
    }
    fn default_prepend_scheme() -> PrependScheme {
        PrependScheme::Always
    }

    #[derive(Deserialize, Debug, Clone, PartialEq)]
    #[allow(clippy::upper_case_acronyms)]
    pub struct BPE {
        pub dropout:                   Option<f64>,
        pub unk_token:                 Option<String>,
        pub continuing_subword_prefix: Option<String>,
        pub end_of_word_suffix:        Option<String>,
        pub fuse_unk:                  Option<bool>,
        pub byte_fallback:             Option<bool>,
        pub vocab:                     HashMap<String, u32>,
        pub merges:                    Vec<String>,
    }

    #[derive(Deserialize, Debug, Clone, PartialEq)]
    pub struct Unigram {
        pub unk_id:        Option<u64>,
        pub vocab:         Vec<(String, f64)>,
        pub byte_fallback: Option<bool>,
    }

    #[derive(Deserialize, Debug, Clone, PartialEq)]
    #[serde(untagged)]
    #[allow(clippy::upper_case_acronyms)]
    pub enum Model {
        BPE(BPE),
        Unigram(Unigram),
    }

    #[derive(Deserialize, Debug, Clone, PartialEq)]
    pub enum Pattern {
        String(String),
        Regex(String),
    }

    #[derive(Deserialize, Debug, Clone, PartialEq)]
    #[serde(tag = "type")]
    #[allow(clippy::upper_case_acronyms, clippy::enum_variant_names)]
    pub enum Normalizer {
        BertNormalizer {
            clean_text:           bool,
            handle_chinese_chars: bool,
            strip_accents:        Option<bool>,
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
            normalizers: Vec<Normalizer>,
        },
        Lowercase,
        Nmt,
        Precompiled {
            #[serde(deserialize_with = "from_base64")]
            precompiled_charsmap: Vec<u8>,
        },
        Replace {
            pattern: Pattern,
            content: String,
        },
        Prepend {
            prepend: String,
        },
    }

    #[derive(Deserialize, Debug, Clone, PartialEq)]
    #[serde(rename_all = "snake_case")]
    pub enum PrependScheme {
        First,
        Never,
        Always,
    }

    #[derive(Deserialize, Debug, Clone, PartialEq)]
    pub enum SplitDelimiterBehavior {
        Removed,
        Isolated,
        MergedWithPrevious,
        MergedWithNext,
        Contiguous,
    }

    #[derive(Deserialize, Debug, Clone, PartialEq)]
    #[serde(tag = "type")]
    #[allow(clippy::enum_variant_names)]
    pub enum PreTokenizer {
        BertPreTokenizer,
        ByteLevel {
            add_prefix_space: bool,
            #[allow(unused)]
            trim_offsets:     bool,
            #[serde(default = "default_true")]
            use_regex:        bool,
        },
        Delimiter {
            delimiter: char,
        },
        Metaspace {
            replacement:      char,
            #[allow(unused)]
            str_rep:          Option<String>,
            #[serde(default = "default_prepend_scheme")]
            prepend_scheme:   PrependScheme,
            add_prefix_space: Option<bool>,
            #[serde(default = "default_true")]
            split:            bool,
        },
        Whitespace,
        Sequence {
            pretokenizers: Vec<PreTokenizer>,
        },
        Split {
            pattern:  Pattern,
            behavior: SplitDelimiterBehavior,
            invert:   bool,
        },
        Punctuation {
            #[serde(default = "default_split")]
            behavior: SplitDelimiterBehavior,
        },
        WhitespaceSplit,
        Digits {
            individual_digits: bool,
        },
        UnicodeScripts,
    }

    #[derive(Deserialize, Debug, Clone, PartialEq)]
    pub enum TemplateSequence {
        A,
        B,
    }
    #[derive(Deserialize, Debug, Clone, PartialEq)]
    pub enum TemplatePiece {
        Sequence {
            id:      TemplateSequence,
            type_id: u32,
        },
        SpecialToken {
            id:      String,
            type_id: u32,
        },
    }

    #[derive(Deserialize, Debug, Clone, PartialEq)]
    pub struct SpecialToken {
        pub id:     String,
        pub ids:    Vec<u32>,
        pub tokens: Vec<String>,
    }

    #[derive(Deserialize, Debug, Clone, PartialEq)]
    #[serde(tag = "type")]
    pub enum PostProcessor {
        #[allow(unused)]
        RobertaProcessing {
            sep:              (String, u32),
            cls:              (String, u32),
            #[allow(unused)]
            trim_offsets:     bool,
            add_prefix_space: bool,
        },
        #[allow(unused)]
        BertProcessing {
            sep: (String, u32),
            cls: (String, u32),
        },
        ByteLevel {
            add_prefix_space: bool,
            #[allow(unused)]
            trim_offsets:     bool,
            #[serde(default = "default_true")]
            use_regex:        bool,
        },
        #[allow(unused)]
        TemplateProcessing {
            single:         Vec<TemplatePiece>,
            pair:           Vec<TemplatePiece>,
            special_tokens: HashMap<String, SpecialToken>,
        },
        Sequence {
            processors: Vec<PostProcessor>,
        },
    }

    #[derive(Deserialize, Debug, Clone, PartialEq)]
    #[serde(tag = "type")]
    #[allow(clippy::upper_case_acronyms, clippy::enum_variant_names)]
    pub enum Decoder {
        BPEDecoder {
            suffix: String,
        },
        ByteLevel {},
        WordPiece {
            prefix:  String,
            cleanup: bool,
        },
        Metaspace {
            replacement:      char,
            #[allow(unused)]
            str_rep:          Option<String>,
            #[serde(default = "default_prepend_scheme")]
            prepend_scheme:   PrependScheme,
            add_prefix_space: Option<bool>,
        },
        CTC {
            pad_token:            String,
            word_delimiter_token: String,
            cleanup:              bool,
        },
        Sequence {
            decoders: Vec<Decoder>,
        },
        Replace {
            pattern: Pattern,
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
    pub enum TruncationDirection {
        Left,
        Right,
    }

    #[derive(Deserialize, Debug, Clone, PartialEq)]
    pub enum TruncationStrategy {
        LongestFirst,
        OnlyFirst,
        OnlySecond,
    }

    #[derive(Deserialize, Debug, Clone, PartialEq)]
    pub struct TruncationParams {
        #[serde(default = "default_right")]
        pub direction:  TruncationDirection,
        pub max_length: usize,
        pub strategy:   TruncationStrategy,
        pub stride:     usize,
    }

    #[derive(Deserialize, Debug, Clone, PartialEq)]
    pub enum PaddingDirection {
        Left,
        Right,
    }

    #[derive(Deserialize, Debug, Clone, PartialEq)]
    pub enum PaddingStrategy {
        BatchLongest,
        Fixed(usize),
    }

    #[derive(Deserialize, Debug, Clone, PartialEq)]
    pub struct PaddingParams {
        pub strategy:           PaddingStrategy,
        pub direction:          PaddingDirection,
        pub pad_to_multiple_of: Option<usize>,
        pub pad_id:             u32,
        pub pad_type_id:        u32,
        pub pad_token:          String,
    }

    #[derive(Deserialize, Debug, Clone, PartialEq)]
    pub struct AddedToken {
        pub id:          u32,
        pub content:     String,
        pub single_word: bool,
        pub lstrip:      bool,
        pub rstrip:      bool,
        pub normalized:  bool,
        pub special:     bool,
    }

    #[derive(Deserialize, Debug, Clone, PartialEq)]
    pub struct Tokenizer {
        pub added_tokens: Option<Vec<AddedToken>>,

        pub normalizer:     Option<Normalizer>,
        pub pre_tokenizer:  Option<PreTokenizer>,
        pub post_processor: Option<PostProcessor>,
        pub decoder:        Option<Decoder>,

        pub truncation: Option<TruncationParams>,
        pub padding:    Option<PaddingParams>,

        pub model: Model,
    }
}

use hf::{AddedToken, Model, Tokenizer};

#[derive(Debug)]
struct ParsedPiece {
    index: u32,
    score: f32,
}

/// Converts a `tokenizers` definition into the definition format used by this crate.
///
/// `data` is the JSON data used by the `tokenizers` library, commonly stored as `tokenizer.json`.
///
/// Returns the tokenizer definition, or an error if the conversion fails.
///
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use kitoken::convert::convert_tokenizers;
/// use kitoken::Kitoken;
///
/// let data = std::fs::read("tests/models/tokenizers/llama2.json")?;
/// let definition = convert_tokenizers(data).unwrap();
///
/// let tokenizer = Kitoken::try_from(definition).unwrap();
/// # Ok(())
/// # }
/// ```
///
/// Additional conversion utilities are defined in [`Definition`] and [`Kitoken`].
///
/// # Format
///
/// The `tokenizers` definition is composed of a JSON object with the following fields:
///
/// - `model`: The model definition.
/// - `added_tokens`: An optional array of added tokens.
/// - `normalizer`: An optional normalizer definition array.
/// - `pre_tokenizer`: An optional pre-tokenizer definition array.
/// - `post_processor`: An optional post-processor definition array.
/// - `decoder`: An optional decoder definition array.
/// - `truncation`: An optional truncation definition.
/// - `padding`: An optional padding definition.
///
/// See the [tokenizers documentation](https://huggingface.co/docs/tokenizers) for more information.
///
/// Tokenizers definitions can contain different model types, including `BPE`, `Unigram`, `WordPiece` and `WordLevel`.
/// This function supports conversion of `BPE` and `Unigram` models.
pub fn convert_tokenizers(data: impl AsRef<[u8]>) -> Result<Definition, ConversionError> {
    let data = data.as_ref();

    let tokenizer = serde_json::from_slice::<Tokenizer>(data).map_err(|e| {
        ConversionError::InvalidData(format!("failed to parse tokenizers definition: {}", e))
    })?;

    let mut config = Configuration::default();

    let mut decode_byte_runes = false;
    let mut decode_byte_chars = false;

    let mut normalizers = VecDeque::from_iter(tokenizer.normalizer);
    let mut pre_tokenizers = VecDeque::from_iter(tokenizer.pre_tokenizer);
    let mut post_processors = VecDeque::from_iter(tokenizer.post_processor);
    let mut decoders = VecDeque::from_iter(tokenizer.decoder);

    // Convert normalizers
    while let Some(normalizer) = normalizers.pop_front() {
        use hf::Normalizer;
        use UnicodeNormalization::*;
        match normalizer {
            Normalizer::BertNormalizer {
                clean_text,
                handle_chinese_chars,
                strip_accents,
                lowercase,
            } => {
                if clean_text {
                    config.normalization.push(Normalization::Replace {
                        pattern:     Regex::new(r"[\s\t\n\r]")?,
                        replacement: " ".to_string(),
                    });
                    config.normalization.push(Normalization::Replace {
                        pattern:     Regex::new(r"\p{C}")?,
                        replacement: "".to_string(),
                    });
                }
                if handle_chinese_chars {
                    config.normalization.push(Normalization::Replace {
                            pattern:     Regex::new(r"([\x{4E00}-\x{9FFF}\x{3400}-\x{4DBF}\x{20000}-\x{2A6DF}\x{2A700}-\x{2B73F}\x{2B740}-\x{2B81F}\x{2B920}-\x{2CEAF}\x{F900}-\x{FAFF}\x{2F800}-\x{2FA1F}])")?,
                            replacement: " $1 ".to_string(),
                        })
                }
                if strip_accents.unwrap_or(lowercase) {
                    config.normalization.push(Normalization::Unicode { scheme: NFD });
                    config.normalization.push(Normalization::Replace {
                        pattern:     Regex::new(r"\p{Mn}")?,
                        replacement: "".to_string(),
                    });
                }
                if lowercase {
                    config.normalization.push(Normalization::CaseFold { upper: false });
                }
            }
            Normalizer::StripNormalizer {
                strip_left,
                strip_right,
            } => {
                if strip_left {
                    config.normalization.push(Normalization::Replace {
                        pattern:     Regex::new(r"^\s+")?,
                        replacement: "".to_string(),
                    });
                }
                if strip_right {
                    config.normalization.push(Normalization::Replace {
                        pattern:     Regex::new(r"\s+$")?,
                        replacement: "".to_string(),
                    });
                }
            }
            Normalizer::StripAccents => {
                config.normalization.push(Normalization::Replace {
                    pattern:     Regex::new(r"\p{M}")?,
                    replacement: "".to_string(),
                });
            }
            Normalizer::NFC => {
                config.normalization.push(Normalization::Unicode { scheme: NFC });
            }
            Normalizer::NFD => {
                config.normalization.push(Normalization::Unicode { scheme: NFD });
            }
            Normalizer::NFKC => {
                config.normalization.push(Normalization::Unicode { scheme: NFKC });
            }
            Normalizer::NFKD => {
                config.normalization.push(Normalization::Unicode { scheme: NFKD });
            }
            Normalizer::Sequence { normalizers: n } => {
                n.into_iter().for_each(|n| normalizers.push_back(n));
            }
            Normalizer::Lowercase => {
                config.normalization.push(Normalization::CaseFold { upper: false });
            }
            Normalizer::Nmt => {
                config.normalization.push(Normalization::NMT);
            }
            Normalizer::Precompiled {
                precompiled_charsmap,
            } => {
                config.normalization.push(Normalization::CharsMap {
                    map: precompiled_charsmap.try_into()?,
                });
            }
            Normalizer::Replace { pattern, content } => {
                use hf::Pattern;
                let pattern = match pattern {
                    Pattern::String(s) => crate::regex::escape(&s).to_string(),
                    Pattern::Regex(r) => r,
                };
                config.normalization.push(Normalization::Replace {
                    pattern:     Regex::new(&pattern)?,
                    replacement: content,
                });
            }
            Normalizer::Prepend { prepend } => {
                config.normalization.push(Normalization::Prepend { prepend });
            }
        }
    }

    // Convert pre-tokenizers
    while let Some(pre_tokenizer) = pre_tokenizers.pop_front() {
        use hf::PreTokenizer;
        match pre_tokenizer {
            PreTokenizer::BertPreTokenizer => {
                config.split.push(Split::Pattern {
                    pattern:  Regex::new(r"\s")?,
                    behavior: SplitBehavior::Remove,
                });
                config.split.push(Split::Pattern {
                    pattern:  Regex::new(r"[\x{0021}-\x{002F}\x{003A}-\x{0040}\x{005B}-\x{0060}\x{007B}-\x{007E}\p{P}]")?,
                    behavior: SplitBehavior::Isolate,
                });
            }
            PreTokenizer::ByteLevel {
                add_prefix_space,
                use_regex,
                ..
            } => {
                decode_byte_chars = true;
                if add_prefix_space {
                    config.normalization.push(Normalization::Extend {
                        character: ' ',
                        left:      1,
                        right:     0,
                        pad:       true,
                    });
                }
                if use_regex {
                    config.split.push(Split::Pattern {
                        pattern:  Regex::new(r"'(?:[sdmt]|ll|ve|re)| ?\p{L}+| ?\p{N}+| ?[^\s\p{L}\p{N}]+|\s+(?!\S)|\s+")?,
                        behavior: SplitBehavior::Isolate,
                    });
                }
            }
            PreTokenizer::Delimiter { delimiter } => {
                config.split.push(Split::Pattern {
                    pattern:  Regex::new(&crate::regex::escape(&delimiter.to_string()))?,
                    behavior: SplitBehavior::Remove,
                });
            }
            PreTokenizer::Metaspace {
                replacement,
                prepend_scheme,
                split,
                add_prefix_space,
                ..
            } => {
                use hf::PrependScheme;
                if add_prefix_space == Some(false) && prepend_scheme != PrependScheme::Never {
                    return Err(ConversionError::UnsupportedConfiguration(
                        "Metaspace pre-tokenizer with prepend_scheme != Never and add_prefix_space = false"
                            .to_string(),
                    ));
                }
                if prepend_scheme != PrependScheme::Never {
                    config.normalization.push(Normalization::Extend {
                        character: ' ',
                        left:      1,
                        right:     0,
                        pad:       true,
                    });
                }
                config.normalization.push(Normalization::Replace {
                    pattern:     Regex::new(r" ")?,
                    replacement: replacement.to_string(),
                });
                if split {
                    config.split.push(Split::Pattern {
                        pattern:  Regex::new(&format!(
                            "{}+",
                            crate::regex::escape(&replacement.to_string())
                        ))?,
                        behavior: SplitBehavior::MergeRight,
                    });
                }
            }
            PreTokenizer::Whitespace => {
                config.split.push(Split::Pattern {
                    pattern:  Regex::new(r"\w+|[^\w\s]+")?,
                    behavior: SplitBehavior::Match,
                });
            }
            PreTokenizer::Sequence { pretokenizers: p } => {
                p.into_iter().for_each(|p| pre_tokenizers.push_back(p));
            }
            PreTokenizer::Split {
                pattern,
                behavior,
                invert,
            } => {
                use hf::{Pattern, SplitDelimiterBehavior};
                let pattern = match pattern {
                    Pattern::String(s) => crate::regex::escape(&s).to_string(),
                    Pattern::Regex(r) => r,
                };
                config.split.push(Split::Pattern {
                    pattern:  Regex::new(&pattern)?,
                    behavior: match behavior {
                        SplitDelimiterBehavior::Removed if invert => SplitBehavior::Match,
                        SplitDelimiterBehavior::Removed => SplitBehavior::Remove,
                        SplitDelimiterBehavior::Isolated => SplitBehavior::Isolate,
                        SplitDelimiterBehavior::MergedWithPrevious => SplitBehavior::MergeLeft,
                        SplitDelimiterBehavior::MergedWithNext => SplitBehavior::MergeRight,
                        SplitDelimiterBehavior::Contiguous => SplitBehavior::Merge,
                    },
                });
            }
            PreTokenizer::Punctuation { behavior } => {
                use hf::SplitDelimiterBehavior;
                config.split.push(Split::Pattern {
                    pattern:  Regex::new(r"[\x{0021}-\x{002F}\x{003A}-\x{0040}\x{005B}-\x{0060}\x{007B}-\x{007E}\p{P}]")?,
                    behavior: match behavior {
                        SplitDelimiterBehavior::Removed => SplitBehavior::Remove,
                        SplitDelimiterBehavior::Isolated => SplitBehavior::Isolate,
                        SplitDelimiterBehavior::MergedWithPrevious => SplitBehavior::MergeLeft,
                        SplitDelimiterBehavior::MergedWithNext => SplitBehavior::MergeRight,
                        SplitDelimiterBehavior::Contiguous => SplitBehavior::Merge,
                    },
                });
            }
            PreTokenizer::WhitespaceSplit => {
                config.normalization.push(Normalization::Replace {
                    pattern:     Regex::new(r"\s+")?,
                    replacement: " ".to_string(),
                });
                config.normalization.push(Normalization::Strip {
                    character: ' ',
                    left:      u32::MAX,
                    right:     u32::MAX,
                });
            }
            PreTokenizer::Digits { individual_digits } => {
                if individual_digits {
                    config.split.push(Split::Pattern {
                        pattern:  Regex::new(r"\p{N}")?,
                        behavior: SplitBehavior::Isolate,
                    });
                } else {
                    config.split.push(Split::Pattern {
                        pattern:  Regex::new(r"\p{N}+")?,
                        behavior: SplitBehavior::Merge,
                    });
                }
            }
            PreTokenizer::UnicodeScripts => {
                return Err(ConversionError::UnsupportedConfiguration(
                    "UnicodeScripts pre-tokenizer".to_string(),
                ));
            }
        }
    }

    // Convert post-processors
    while let Some(post_processor) = post_processors.pop_front() {
        use hf::PostProcessor;
        match post_processor {
            PostProcessor::RobertaProcessing { .. } => {
                log::warn!("RobertaProcessing post-processor is not supported and will be ignored");
            }
            PostProcessor::BertProcessing { .. } => {
                log::warn!("BertProcessing post-processor is not supported and will be ignored");
            }
            PostProcessor::ByteLevel { .. } => {
                if !decode_byte_chars {
                    return Err(ConversionError::UnsupportedConfiguration(
                        "ByteLevel post-processor without ByteLevel pre-tokenizer".to_string(),
                    ));
                }
            }
            PostProcessor::TemplateProcessing { .. } => {
                log::warn!(
                    "TemplateProcessing post-processor is not supported and will be ignored"
                );
            }
            PostProcessor::Sequence { processors: p } => {
                p.into_iter().for_each(|p| post_processors.push_back(p));
            }
        }
    }

    // Convert decoders
    while let Some(decoder) = decoders.pop_front() {
        use hf::Decoder;
        match decoder {
            Decoder::BPEDecoder { .. } => {
                return Err(ConversionError::UnsupportedConfiguration(
                    "BPEDecoder decoder".to_string(),
                ));
            }
            Decoder::ByteLevel { .. } => {
                if !decode_byte_chars {
                    return Err(ConversionError::UnsupportedConfiguration(
                        "ByteLevel decoder without ByteLevel pre-tokenizer".to_string(),
                    ));
                }
            }
            Decoder::WordPiece { .. } => {
                return Err(ConversionError::UnsupportedConfiguration(
                    "WordPiece decoder".to_string(),
                ));
            }
            Decoder::Metaspace {
                prepend_scheme,
                add_prefix_space,
                replacement,
                ..
            } => {
                use hf::PrependScheme;
                if add_prefix_space == Some(false) && prepend_scheme != PrependScheme::Never {
                    return Err(ConversionError::UnsupportedConfiguration(
                        "Metaspace decoder with prepend_scheme != Never and add_prefix_space = false"
                            .to_string(),
                    ));
                }
                if prepend_scheme != PrependScheme::Never {
                    config.decoding.push(Decoding::Strip {
                        character: replacement,
                        left:      1,
                        right:     0,
                    });
                }
                config.decoding.push(Decoding::Replace {
                    pattern:     replacement.to_string(),
                    replacement: " ".to_string(),
                });
            }
            Decoder::CTC {
                pad_token,
                word_delimiter_token,
                cleanup,
            } => {
                config.decoding.push(Decoding::Replace {
                    pattern:     pad_token,
                    replacement: "".to_string(),
                });
                if cleanup {
                    config.decoding.push(Decoding::Replace {
                        pattern:     "[ ](\\.|\\?|\\!|\\,|n't|'m|'s|'ve|'re)".to_string(),
                        replacement: "$1".to_string(),
                    });
                    config.decoding.push(Decoding::Replace {
                        pattern:     " ' ".to_string(),
                        replacement: "'".to_string(),
                    });
                    config.decoding.push(Decoding::Replace {
                        pattern:     " do not".to_string(),
                        replacement: " don't".to_string(),
                    });
                    config.decoding.push(Decoding::Replace {
                        pattern:     word_delimiter_token,
                        replacement: " ".to_string(),
                    });
                }
            }
            Decoder::Sequence { decoders: d } => {
                d.into_iter().for_each(|d| decoders.push_back(d));
            }
            Decoder::Replace { pattern, content } => {
                use hf::Pattern;
                let pattern = match pattern {
                    Pattern::String(s) => s,
                    Pattern::Regex(_) => {
                        return Err(ConversionError::UnsupportedConfiguration(
                            "Replace regex pattern in decoder".to_string(),
                        ));
                    }
                };
                config.decoding.push(Decoding::Replace {
                    pattern,
                    replacement: content,
                });
            }
            Decoder::Fuse => {
                log::info!("Fuse decoder is not used and will be ignored");
            }
            Decoder::Strip {
                content,
                start,
                stop,
            } => {
                config.decoding.push(Decoding::Strip {
                    character: content,
                    left:      start.try_into().map_err(|_| {
                        ConversionError::InvalidData(
                            "Strip decoder start value is too large".to_string(),
                        )
                    })?,
                    right:     stop.try_into().map_err(|_| {
                        ConversionError::InvalidData(
                            "Strip decoder stop value is too large".to_string(),
                        )
                    })?,
                });
            }
            Decoder::ByteFallback => {
                decode_byte_runes = true;
            }
        }
    }

    let get_specials = |unk_token: Option<&String>, unk_id: Option<u32>| {
        let mut specials = HashMap::<Vec<u8>, SpecialToken>::with_capacity(
            tokenizer.added_tokens.as_ref().map_or(0, |added| added.len()),
        );
        for (
            i,
            AddedToken {
                content,
                id,
                special,
                ..
            },
        ) in tokenizer.added_tokens.iter().flatten().enumerate()
        {
            let is_unk = unk_id.as_ref() == Some(id) || unk_token == Some(content);
            specials.insert(content.as_bytes().to_vec(), SpecialToken {
                id:    *id,
                bytes: content.as_bytes().to_vec(),
                kind:  if is_unk {
                    SpecialTokenKind::Unknown
                } else if *special {
                    SpecialTokenKind::Control
                } else {
                    SpecialTokenKind::Priority
                },
                score: i as f32,
                ident: if is_unk {
                    Some("unk".to_string())
                } else {
                    None
                },
            });
        }
        specials
    };

    // Convert vocab
    let (mut vocab, specials, scores) = match tokenizer.model {
        Model::BPE(model) => {
            if decode_byte_chars {
                config.mode = Mode::BytePair;
            } else {
                config.mode = Mode::CharPair;
            }

            let mut vocab = HashMap::<Vec<u8>, u32>::with_capacity(model.vocab.len());
            for (token, id) in model.vocab {
                vocab.insert(token.as_bytes().to_vec(), id);
            }
            let specials = get_specials(model.unk_token.as_ref(), None);
            for special in specials.keys() {
                vocab.remove(special);
            }

            if let Some(unk) = model.unk_token {
                if let Some(special) = specials.get(unk.as_bytes()) {
                    if let Some(true) = model.fuse_unk {
                        config.processing.push(Processing::Collapse { id: special.id });
                    }
                } else {
                    return Err(ConversionError::InvalidData(format!(
                        "Unknown token {:?} not found in specials",
                        unk
                    )));
                }
            }
            if let Some(true) = model.byte_fallback {
                decode_byte_runes = true;
            }

            let merges = model
                .merges
                .into_iter()
                .enumerate()
                .map(|(i, merge)| {
                    let mut parts = merge.splitn(2, ' ');
                    if let (Some(left), Some(right)) = (parts.next(), parts.next()) {
                        Some(([left.as_bytes(), right.as_bytes()].concat(), i))
                    } else {
                        None
                    }
                })
                .collect::<Option<HashMap<_, _>>>();
            let merges = if let Some(merges) = merges {
                merges
            } else {
                return Err(ConversionError::InvalidData("failed to parse BPE merges".to_string()));
            };

            let sort_vocab = |vocab: &mut Vocab| {
                vocab.sort_by(|(a, ai), (b, bi)| {
                    if let (Some(ma), Some(mb)) = (merges.get(a), merges.get(b)) {
                        let comp = ma.cmp(mb);
                        if comp == Ordering::Equal {
                            ai.cmp(bi)
                        } else {
                            comp
                        }
                    } else if merges.get(a).is_some() {
                        Ordering::Less
                    } else if merges.get(b).is_some() {
                        Ordering::Greater
                    } else {
                        ai.cmp(bi)
                    }
                });
            };
            let mut vocab = vocab.into_iter().collect::<Vec<_>>();
            sort_vocab(&mut vocab);

            let mut specials = specials.into_values().collect::<Vec<_>>();
            specials.sort();

            let scores = Vec::with_capacity(0);
            (vocab, specials, scores)
        }
        Model::Unigram(model) => {
            config.mode = Mode::Unigram;

            let mut vocab = HashMap::<Vec<u8>, ParsedPiece>::with_capacity(model.vocab.len());

            for (index, (token, score)) in model.vocab.into_iter().enumerate() {
                vocab.insert(token.as_bytes().to_vec(), ParsedPiece {
                    index: index as u32,
                    score: score as f32,
                });
            }
            let specials = get_specials(None, model.unk_id.map(|id| id as u32));
            for special in specials.keys() {
                vocab.remove(special);
            }

            if let Some(unk) = model.unk_id {
                if let Some((_, special)) =
                    specials.iter().find(|(_, special)| special.id == unk as u32)
                {
                    config.processing.push(Processing::Collapse { id: special.id });
                } else {
                    return Err(ConversionError::InvalidData(format!(
                        "Unknown token {:?} not found in specials",
                        unk
                    )));
                }
            }
            if let Some(true) = model.byte_fallback {
                decode_byte_runes = true;
            }

            let mut vocab = vocab.into_iter().collect::<Vec<_>>();
            vocab.sort_by(|(_, a), (_, b)| match a.score.partial_cmp(&b.score).unwrap() {
                Ordering::Equal => a.index.cmp(&b.index),
                other => other,
            });
            let scores = vocab.iter().map(|(_, piece)| piece.score).collect::<Scores>();
            let vocab =
                vocab.into_iter().map(|(text, piece)| (text, piece.index)).collect::<Vocab>();

            let mut specials = specials.into_values().collect::<Vec<_>>();
            specials.sort();

            (vocab, specials, scores)
        }
    };

    // Replace byte character placeholders
    if decode_byte_chars {
        let (byte_encoder, _) = build_byte_encoder_decoder();
        let replace_byte_chars = |vocab: &mut Vocab| {
            vocab.iter_mut().for_each(|(token, _)| {
                let mut replacement = Vec::with_capacity(token.len());
                for c in token.chars() {
                    if let Some(&replace) = byte_encoder.get(&c) {
                        replacement.push(replace);
                    } else {
                        replacement.extend(c.to_string().as_bytes());
                    }
                }
                *token = replacement;
            });
        };
        replace_byte_chars(&mut vocab);
    }
    // Replace byte rune placeholders
    if decode_byte_runes {
        let replace_byte_runes = |vocab: &mut Vocab| {
            let vocab_map =
                vocab.iter().map(|(token, id)| (token.clone(), *id)).collect::<HashMap<_, _>>();
            *vocab = vocab
                .iter()
                .filter_map(|(token, id)| {
                    if token.len() == 6 && token.starts_with(b"<0x") && token.ends_with(b">") {
                        if let Ok(rune) =
                            u32::from_str_radix(core::str::from_utf8(&token[3..5]).unwrap(), 16)
                        {
                            let rune = [rune as u8].to_vec();
                            if let Some(existing) = vocab_map.get(&rune) {
                                log::debug!(
                                    "Byte rune already in vocab: {:>4} -> {:6?} (skipping {:?})",
                                    format!("{:?}", rune.as_bstr()),
                                    existing,
                                    id
                                );
                                return None;
                            }
                            return Some((rune, *id));
                        }
                    }
                    Some((token.clone(), *id))
                })
                .collect();
        };
        replace_byte_runes(&mut vocab);
    }
    // Remove duplicate tokens
    let deduplicate = |vocab: &mut Vocab| {
        let mut seen = HashMap::new();
        vocab.retain(|(token, id)| {
            if let Some(existing) = seen.get(token) {
                log::debug!(
                    "Removing duplicate token in vocab: {:?} -> {} (existing: {})",
                    token.as_bstr(),
                    id,
                    existing
                );
                false
            } else {
                seen.insert(token.clone(), *id);
                true
            }
        });
    };
    deduplicate(&mut vocab);

    let meta = Metadata {
        source: DefinitionSource::Tokenizers,
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

type ByteEncoder = HashMap<char, u8>;
type ByteDecoder = HashMap<u8, char>;
fn build_byte_encoder_decoder() -> (ByteEncoder, ByteDecoder) {
    let mut encoder = ByteEncoder::default();
    let mut decoder = ByteDecoder::default();
    for i in '!'..='~' {
        encoder.insert(char::from_u32(i as u32).unwrap(), i as u8);
        decoder.insert(i as u8, char::from_u32(i as u32).unwrap());
    }
    for i in '¡'..='¬' {
        encoder.insert(char::from_u32(i as u32).unwrap(), i as u8);
        decoder.insert(i as u8, char::from_u32(i as u32).unwrap());
    }
    for i in '®'..='ÿ' {
        encoder.insert(char::from_u32(i as u32).unwrap(), i as u8);
        decoder.insert(i as u8, char::from_u32(i as u32).unwrap());
    }
    let mut utc = 0;
    for i in 0..=255 {
        #[allow(clippy::map_entry)]
        if !decoder.contains_key(&i) {
            encoder.insert(char::from_u32(256 + utc).unwrap(), i);
            decoder.insert(i, char::from_u32(256 + utc).unwrap());
            utc += 1;
        }
    }
    (encoder, decoder)
}

impl Definition {
    /// Converts a `tokenizers` tokenizer definition into the encoder format used by this crate.
    /// See [`convert_tokenizers`] for more details.
    #[cfg(feature = "std")]
    pub fn from_tokenizers_reader<R: Read>(reader: &mut R) -> Result<Self, ConversionError> {
        let mut data = Vec::with_capacity(1024);
        reader.read_to_end(&mut data)?;
        Self::from_tokenizers_slice(&data)
    }

    /// Converts a `tokenizers` tokenizer definition into the encoder format used by this crate.
    /// See [`convert_tokenizers`] for more details.
    #[cfg(feature = "std")]
    pub fn from_tokenizers_file(path: impl AsRef<Path>) -> Result<Self, ConversionError> {
        let mut file = File::open(path)?;
        Self::from_tokenizers_reader(&mut file)
    }

    /// Converts a `tokenizers` tokenizer definition into the encoder format used by this crate.
    /// See [`convert_tokenizers`] for more details.
    pub fn from_tokenizers_slice(data: &[u8]) -> Result<Self, ConversionError> {
        convert_tokenizers(data)
    }
}

impl Kitoken {
    /// Initializes the tokenizer from a `tokenizers` tokenizer definition.
    /// See [`convert_tokenizers`] for more details.
    #[cfg(feature = "std")]
    pub fn from_tokenizers_reader<R: Read>(reader: &mut R) -> Result<Self, ConversionError> {
        Ok(Self::from_definition(Definition::from_tokenizers_reader(reader)?)?)
    }

    /// Initializes the tokenizer from a `tokenizers` tokenizer definition.
    /// See [`convert_tokenizers`] for more details.
    #[cfg(feature = "std")]
    pub fn from_tokenizers_file(path: impl AsRef<Path>) -> Result<Self, ConversionError> {
        Ok(Self::from_definition(Definition::from_tokenizers_file(path)?)?)
    }

    /// Initializes the tokenizer from a `tokenizers` tokenizer definition.
    /// See [`convert_tokenizers`] for more details.
    pub fn from_tokenizers_slice(data: &[u8]) -> Result<Self, ConversionError> {
        Ok(Self::from_definition(Definition::from_tokenizers_slice(data)?)?)
    }
}
