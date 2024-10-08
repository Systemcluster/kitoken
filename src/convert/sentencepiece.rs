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

use bstr::ByteSlice;
use hashbrown::HashMap;
use sentencepiece_model::{ModelType, SentencePieceModel, Type};

use crate::convert::ConversionError;
use crate::{
    Configuration, Decoding, Definition, Fallback, InsertionPosition, Kitoken, Metadata, Model,
    Normalization, Processing, Regex, Scores, SpecialToken, SpecialTokenKind, SpecialVocab, Split,
    SplitBehavior, Template, Token, UnicodeNormalization, Vocab,
};

/// Converts a `sentencepiece` model into the definition format used by this crate.
///
/// `data` is the raw model data generated by the `sentencepiece` tokenizer.
///
/// Returns the tokenizer definition, or an error if the conversion fails.
///
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use kitoken::convert::convert_sentencepiece;
/// use kitoken::Kitoken;
///
/// let data = std::fs::read("tests/models/sentencepiece/llama2.model")?;
/// let definition = convert_sentencepiece(data).unwrap();
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
/// SentencePiece models are used and generated by the `sentencepiece` tokenizer.
///
/// SentencePiece models can contain different model types, including `BPE`, `Unigram`, `Char` and `Word`.
/// This function supports conversion of `BPE` and `Unigram` models.
pub fn convert_sentencepiece(data: impl AsRef<[u8]>) -> Result<Definition, ConversionError> {
    let data = data.as_ref();
    let model = SentencePieceModel::from_slice(data).map_err(|e| {
        ConversionError::InvalidData(format!("failed to parse sentencepiece model: {:?}", e))
    })?;
    convert_sentencepiece_model(model)
}
fn convert_sentencepiece_model(model: SentencePieceModel) -> Result<Definition, ConversionError> {
    let mut config = Configuration::default();
    config.fallback.push(Fallback::Unknown);
    config.fallback.push(Fallback::Skip);

    let mut model_type = ModelType::Unigram;
    let mut treat_whitespace_as_suffix = false;
    let mut remove_extra_whitespaces = true;
    let mut add_dummy_prefix = true;
    let mut unk_id = None;
    let mut specials = HashMap::<Vec<u8>, SpecialToken>::default();

    if let Some(trainer) = model.trainer() {
        treat_whitespace_as_suffix = trainer.treat_whitespace_as_suffix();
        specials.insert(trainer.unk_piece().as_bytes().to_vec(), SpecialToken {
            id:      trainer.unk_id() as _,
            bytes:   trainer.unk_surface().as_bytes().to_vec(),
            kind:    SpecialTokenKind::Unknown,
            ident:   Some("unk".to_string()),
            score:   0.0,
            extract: false,
        });
        unk_id = Some(trainer.unk_id() as _);
        specials.insert(trainer.bos_piece().as_bytes().to_vec(), SpecialToken {
            id:      trainer.bos_id() as _,
            bytes:   trainer.bos_piece().as_bytes().to_vec(),
            kind:    SpecialTokenKind::Control,
            ident:   Some("bos".to_string()),
            score:   0.0,
            extract: false,
        });
        config.templates.push(Template {
            content:  trainer.bos_piece().to_string(),
            position: InsertionPosition::SequenceStart,
        });
        specials.insert(trainer.eos_piece().as_bytes().to_vec(), SpecialToken {
            id:      trainer.eos_id() as _,
            bytes:   trainer.eos_piece().as_bytes().to_vec(),
            kind:    SpecialTokenKind::Control,
            ident:   Some("eos".to_string()),
            score:   0.0,
            extract: false,
        });
        config.templates.push(Template {
            content:  trainer.eos_piece().to_string(),
            position: InsertionPosition::SequenceEnd,
        });
        specials.insert(trainer.pad_piece().as_bytes().to_vec(), SpecialToken {
            id:      trainer.pad_id() as _,
            bytes:   trainer.pad_piece().as_bytes().to_vec(),
            kind:    SpecialTokenKind::Control,
            ident:   Some("pad".to_string()),
            score:   0.0,
            extract: false,
        });
        model_type = trainer.model_type();
        if trainer.byte_fallback() {
            config.fallback.insert(0, Fallback::Bytes);
        }
    }

    if model.pieces.len() > u32::MAX as usize {
        return Err(ConversionError::InvalidData(format!(
            "too many pieces: {}",
            model.pieces.len()
        )));
    }

    let mut vocab = HashMap::<Vec<u8>, ParsedPiece>::with_capacity(model.pieces.len());

    for (index, piece) in model.pieces.iter().enumerate() {
        let text = piece
            .piece
            .as_ref()
            .ok_or_else(|| ConversionError::InvalidData(format!("piece {} has no text", index)))?;
        let piece_type = piece.r#type();

        let text = if piece_type == Type::Byte {
            // byte encoding in the form `<0xAA>`
            let rune = &text[3..5];
            let rune = u32::from_str_radix(rune, 16)
                .map_err(|e| ConversionError::InvalidData(format!("{:?}", e)))?;
            [rune as u8].to_vec()
        } else {
            text.as_bytes().to_vec()
        };

        if piece_type == Type::UserDefined
            || piece_type == Type::Control
            || piece_type == Type::Unknown
            || piece_type == Type::Unused
        {
            if piece_type == Type::Unknown {
                if unk_id.is_some() && unk_id != Some(index as u32) {
                    log::warn!("Multiple unknown pieces in vocab");
                } else if unk_id.is_none() {
                    specials.insert(text.clone(), SpecialToken {
                        bytes:   text.clone(),
                        id:      index as u32,
                        score:   index as f32,
                        kind:    SpecialTokenKind::Unknown,
                        ident:   Some("unk".to_string()),
                        extract: false,
                    });
                    unk_id = Some(index as u32);
                }
            } else if piece_type == Type::Unused {
                log::warn!("Skipping unused piece {} ({:?})", index, piece.piece);
            } else {
                specials.insert(text.clone(), SpecialToken {
                    bytes:   text.clone(),
                    id:      index as u32,
                    score:   index as f32,
                    kind:    match piece_type {
                        Type::Control => SpecialTokenKind::Control,
                        _ => SpecialTokenKind::Priority,
                    },
                    ident:   None,
                    extract: false,
                });
            }
            continue;
        }
        if let Some(existing) = vocab.get(&text) {
            let existing_type = existing.type_;
            if piece_type == Type::Byte && existing_type != Type::Byte {
                log::debug!(
                    "Byte piece already in vocab: {:>4} -> {:6?} (skipping {:?})",
                    format!("{:?}", text.as_bstr()),
                    existing.index,
                    index
                );
                continue;
            }
        }
        if let Some(skipped) = vocab.insert(text.clone(), ParsedPiece {
            index: index as u32,
            score: piece.score(),
            type_: piece.r#type(),
        }) {
            log::debug!(
                "Byte piece already in vocab: {:>4} -> {:6?} (replacing {:?})",
                format!("{:?}", text.as_bstr()),
                index,
                skipped.index
            );
        };
    }
    specials.iter_mut().for_each(|(_, special)| {
        special.score = 1.0 / (special.score + 1.0);
    });

    if let Some(normalizer) = model.normalizer() {
        use UnicodeNormalization::*;
        match normalizer.name() {
            "nmt_nfkc" => {
                config.normalization.push(Normalization::Unicode { scheme: NFKC });
                config.normalization.push(Normalization::NMT);
            }
            "nfkc" => {
                config.normalization.push(Normalization::Unicode { scheme: NFKC });
            }
            "nmt_nfkc_cf" => {
                config.normalization.push(Normalization::Unicode { scheme: NFKC });
                config.normalization.push(Normalization::NMT);
                config.normalization.push(Normalization::CaseFold { upper: false });
            }
            "nfkc_cf" => {
                config.normalization.push(Normalization::Unicode { scheme: NFKC });
                config.normalization.push(Normalization::CaseFold { upper: false });
            }
            "identity" => {}
            "user_defined" => {
                if normalizer.precompiled_charsmap().is_empty() {
                    return Err(ConversionError::InvalidData(
                        "user_defined normalizer has no precompiled charsmap".to_string(),
                    ));
                }
                config.normalization.push(Normalization::CharsMap {
                    map: normalizer.precompiled_charsmap().try_into()?,
                });
            }
            name => {
                return Err(ConversionError::UnsupportedConfiguration(format!(
                    "{} normalizer",
                    name
                )));
            }
        };
        remove_extra_whitespaces = normalizer.remove_extra_whitespaces();
        add_dummy_prefix = normalizer.add_dummy_prefix();
    }

    if remove_extra_whitespaces {
        config.normalization.push(Normalization::Strip {
            character: ' ',
            left:      u32::MAX,
            right:     u32::MAX,
        });
        config.normalization.push(Normalization::Collapse { character: ' ' });
        if let Some(unk) = unk_id {
            config.processing.push(Processing::Collapse { id: unk });
        }
        if treat_whitespace_as_suffix {
            config.split.push(Split::Pattern {
                pattern:  '▁'.into(),
                behavior: SplitBehavior::MergeLeft,
            });
        } else {
            config.split.push(Split::Pattern {
                pattern:  '▁'.into(),
                behavior: SplitBehavior::MergeRight,
            });
        }
    } else if treat_whitespace_as_suffix {
        config.split.push(Split::Pattern {
            pattern:  Regex::new(r"▁+")?.into(),
            behavior: SplitBehavior::MergeLeft,
        });
    } else {
        config.split.push(Split::Pattern {
            pattern:  Regex::new(r"▁+")?.into(),
            behavior: SplitBehavior::MergeRight,
        });
    }
    config.normalization.push(Normalization::Replace {
        pattern:     " ".into(),
        replacement: "▁".to_string(),
    });
    if add_dummy_prefix {
        config.normalization.push(Normalization::Extend {
            character: '▁',
            left:      if treat_whitespace_as_suffix { 0 } else { 1 },
            right:     if treat_whitespace_as_suffix { 1 } else { 0 },
            pad:       false,
        });
        config.decoding.push(Decoding::Strip {
            character: '▁',
            left:      if treat_whitespace_as_suffix { 0 } else { 1 },
            right:     if treat_whitespace_as_suffix { 1 } else { 0 },
        });
    };
    config.decoding.push(Decoding::Replace {
        pattern:     "▁".into(),
        replacement: " ".to_string(),
    });

    let (model, specials) = match model_type {
        ModelType::Bpe => {
            let create_merges = |vocab: &HashMap<Vec<u8>, ParsedPiece>| {
                let mut merges = HashMap::<u32, f32>::with_capacity(vocab.len() * 3);
                for (text, piece) in vocab.iter() {
                    for split in 1..text.len() {
                        let left = &text[..split];
                        let right = &text[split..];
                        if let (Some(_), Some(_)) = (vocab.get(left), vocab.get(right)) {
                            if !merges.contains_key(&piece.index) {
                                merges.insert(piece.index, piece.score);
                            }
                        }
                    }
                }
                merges
            };
            let vocab_merges = create_merges(&vocab);

            let sort_vocab = |vocab: &mut Vocab, merges: &HashMap<u32, f32>| {
                vocab.sort_by(|Token { id: ai, .. }, Token { id: bi, .. }| {
                    if let (Some(ma), Some(mb)) = (merges.get(ai), merges.get(bi)) {
                        let comp = mb.partial_cmp(ma).unwrap();
                        if comp == Ordering::Equal {
                            ai.cmp(bi)
                        } else {
                            comp
                        }
                    } else if merges.get(ai).is_some() {
                        Ordering::Less
                    } else if merges.get(bi).is_some() {
                        Ordering::Greater
                    } else {
                        ai.cmp(bi)
                    }
                });
            };
            let mut vocab = vocab
                .into_iter()
                .map(|(text, piece)| (text, piece.index).into())
                .collect::<Vocab>();
            sort_vocab(&mut vocab, &vocab_merges);

            let mut specials =
                specials.into_iter().map(|(_, special)| special).collect::<SpecialVocab>();
            specials.sort();

            (Model::BytePair { vocab, chars: true }, specials)
        }
        ModelType::Unigram => {
            let mut vocab = vocab.into_iter().collect::<Vec<_>>();
            vocab.sort_by(|(_, a), (_, b)| match a.score.partial_cmp(&b.score).unwrap() {
                Ordering::Equal => a.index.cmp(&b.index),
                other => other,
            });
            let scores = vocab.iter().map(|(_, piece)| piece.score).collect::<Scores>();
            let vocab = vocab
                .into_iter()
                .map(|(text, piece)| (text, piece.index).into())
                .collect::<Vocab>();
            let mut specials =
                specials.into_iter().map(|(_, special)| special).collect::<SpecialVocab>();
            specials.sort();

            (Model::Unigram { vocab, scores }, specials)
        }
        _ => {
            return Err(ConversionError::UnsupportedConfiguration(format!(
                "{:?} model type",
                model_type
            )));
        }
    };

    let meta = Metadata {
        source: "sentencepiece".to_string(),
        ..Metadata::default()
    };

    Ok(Definition {
        meta,
        model,
        specials,
        config,
    })
}

#[derive(Debug)]
struct ParsedPiece {
    index: u32,
    score: f32,
    type_: Type,
}

impl Definition {
    /// Converts a `sentencepiece` model into the encoder format used by this crate.
    /// See [`convert_sentencepiece`] for more details.
    #[cfg(feature = "std")]
    pub fn from_sentencepiece_reader<R: Read>(reader: &mut R) -> Result<Self, ConversionError> {
        let mut data = Vec::with_capacity(1024);
        reader.read_to_end(&mut data)?;
        Self::from_sentencepiece_slice(&data)
    }

    /// Converts a `sentencepiece` model into the encoder format used by this crate.
    /// See [`convert_sentencepiece`] for more details.
    #[cfg(feature = "std")]
    pub fn from_sentencepiece_file(path: impl AsRef<Path>) -> Result<Self, ConversionError> {
        let mut file = File::open(path)?;
        Self::from_sentencepiece_reader(&mut file)
    }

    /// Converts a `sentencepiece` model into the encoder format used by this crate.
    /// See [`convert_sentencepiece`] for more details.
    pub fn from_sentencepiece_slice(data: &[u8]) -> Result<Self, ConversionError> {
        convert_sentencepiece(data)
    }

    /// Converts a `sentencepiece` model into the encoder format used by this crate.
    /// See [`convert_sentencepiece`] for more details.
    pub fn from_sentencepiece_model(model: SentencePieceModel) -> Result<Self, ConversionError> {
        convert_sentencepiece_model(model)
    }
}

impl Kitoken {
    /// Initializes the tokenizer from a `sentencepiece` model.
    /// See [`convert_sentencepiece`] for more details.
    #[cfg(feature = "std")]
    pub fn from_sentencepiece_reader<R: Read>(reader: &mut R) -> Result<Self, ConversionError> {
        Ok(Self::from_definition(Definition::from_sentencepiece_reader(reader)?)?)
    }

    /// Initializes the tokenizer from a `sentencepiece` model.
    /// See [`convert_sentencepiece`] for more details.
    #[cfg(feature = "std")]
    pub fn from_sentencepiece_file(path: impl AsRef<Path>) -> Result<Self, ConversionError> {
        Ok(Self::from_definition(Definition::from_sentencepiece_file(path)?)?)
    }

    /// Initializes the tokenizer from a `sentencepiece` model.
    /// See [`convert_sentencepiece`] for more details.
    pub fn from_sentencepiece_slice(data: &[u8]) -> Result<Self, ConversionError> {
        Ok(Self::from_definition(Definition::from_sentencepiece_slice(data)?)?)
    }

    /// Initializes the tokenizer from a `sentencepiece` model.
    /// See [`convert_sentencepiece`] for more details.
    pub fn from_sentencepiece_model(model: SentencePieceModel) -> Result<Self, ConversionError> {
        Ok(Self::from_definition(Definition::from_sentencepiece_model(model)?)?)
    }
}
