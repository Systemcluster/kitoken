//! Utilities for converting different tokenizer definitions into a format compatible with this crate.

use core::cmp::Ordering;

use base64::{alphabet, engine, Engine};
use bstr::{BString, ByteSlice};
use log::debug;
use sentencepiece_model::{ModelType, SentencePiece, SentencePieceModel, Type};

use crate::convert::ConversionError;
use crate::{Config, EncoderMap, Mode, UnicodeNormalization};

type ByteEncoder = IndexMap<Vec<u8>, u8>;
type ByteDecoder = IndexMap<u8, Vec<u8>>;
fn build_byte_encoder_decoder() -> (ByteEncoder, ByteDecoder) {
    let mut encoder = ByteEncoder::default();
    let mut decoder = ByteDecoder::default();
    for i in '!'..='~' {
        encoder.insert(char::from_u32(i as u32).unwrap().to_string().into(), i as u8);
        decoder.insert(i as u8, char::from_u32(i as u32).unwrap().to_string().into());
    }
    for i in '¡'..='¬' {
        encoder.insert(char::from_u32(i as u32).unwrap().to_string().into(), i as u8);
        decoder.insert(i as u8, char::from_u32(i as u32).unwrap().to_string().into());
    }
    for i in '®'..='ÿ' {
        encoder.insert(char::from_u32(i as u32).unwrap().to_string().into(), i as u8);
        decoder.insert(i as u8, char::from_u32(i as u32).unwrap().to_string().into());
    }
    let mut utc = 0;
    for i in 0..=255 {
        #[allow(clippy::map_entry)]
        if !decoder.contains_key(&i) {
            encoder.insert(char::from_u32(256 + utc).unwrap().to_string().into(), i);
            decoder.insert(i, char::from_u32(256 + utc).unwrap().to_string().into());
            utc += 1;
        }
    }
    (encoder, decoder)
}

/// Converts vocab and merges into the encoder format used by this crate.
/// Vocab and merges are a common format for storing tokenizer definitions.
///
/// `vocab` is a map from tokens to ids.
/// `merges` is a list of merges in the form `(left, right)`.
/// `special_tokens` is a list of special tokens included in the vocab.
/// `byte_encoding` enables decoding of encoded bytes included in the vocab.
///
/// Returns the encoder and special encoder for the vocab and merges, or an error if the conversion fails.
///
/// This function supports conversion of encoded bytes included in the vocab of some tokenizers with the `byte_encoding` argument.
/// See [`ByteEncoding`] for more information.
// pub fn convert_vocab_and_merges(
//     vocab: IndexMap<BString, u16>, merges: Vec<(BString, BString)>, special_tokens: Vec<BString>,
//     byte_encoding: ByteEncoding,
// ) -> Result<(EncoderMap, EncoderMap), ConversionError> {
//     let mut encoder = vocab
//         .into_iter()
//         .map(|(k, v)| (k.bytes().as_bytes().to_vec(), v as u32))
//         .collect::<EncoderMap>();

//     if byte_encoding == ByteEncoding::HexByte {
//         let mut dupes = 0;
//         encoder = encoder
//             .iter()
//             .map(|(k, v)| {
//                 if k.len() > 2 && k.starts_with(b"0x") {
//                     let rune = u32::from_str_radix(std::str::from_utf8(&k[2..]).unwrap(), 16)
//                         .map_err(|e| ConversionError::InvalidNumber(format!("{:?}", e)))?;
//                     let rune = [rune as u8].to_vec();
//                     if !encoder.contains_key(&rune) {
//                         return Ok((rune, *v));
//                     } else {
//                         dupes += 1;
//                         debug!(
//                             "duplicate rune: {:?} ({:?}) -> {:?}",
//                             k,
//                             std::str::from_utf8(&rune),
//                             encoder.get(&rune)
//                         );
//                     }
//                 }
//                 Ok((k.clone(), *v))
//             })
//             .collect::<Result<EncoderMap, _>>()?;
//         if dupes > 0 {
//             debug!("skipped {} duplicates", dupes);
//         }
//     }

//     let merges = merges
//         .iter()
//         .enumerate()
//         .map(|(i, (l, r))| ([l.bytes().as_bytes(), r.bytes().as_bytes()].concat(), i))
//         .collect::<IndexMap<_, _>>();

//     let mut special_tokens = special_tokens;
//     special_tokens.sort_by(|a, b| {
//         merges
//             .get(b.bytes().as_bytes())
//             .unwrap_or(&usize::MAX)
//             .cmp(merges.get(a.bytes().as_bytes()).unwrap_or(&usize::MAX))
//     });
//     let mut special_encoder = EncoderMap::default();
//     for special in special_tokens {
//         let special = special.bytes().as_bytes().to_vec();
//         let special_token = encoder.remove(&special).unwrap();
//         special_encoder.insert(special, special_token);
//     }

//     encoder.sort_by(|va, _, vb, _| {
//         merges.get(vb).unwrap_or(&usize::MAX).cmp(merges.get(va).unwrap_or(&usize::MAX))
//     });

//     if byte_encoding == ByteEncoding::CharByte {
//         let (byte_encoder, _) = build_byte_encoder_decoder();
//         let mut replaced_encoder = EncoderMap::default();
//         for (k, &v) in encoder.iter() {
//             let mut replacement = Vec::new();
//             for c in k.chars() {
//                 if let Some(&b) = byte_encoder.get(&c.to_string().as_bytes().to_vec()) {
//                     replacement.push(b);
//                 } else {
//                     replacement.extend(c.to_string().bytes());
//                 }
//             }
//             replaced_encoder.insert(replacement, v);
//         }
//         encoder = replaced_encoder;
//     }

//     Ok((encoder, special_encoder))
// }

// #[derive(Debug, Deserialize)]
// struct HuggingFaceTokenizer {}

// pub fn convert_huggingface(data: impl AsRef<[u8]>) {
//     let data = data.as_ref();
// }

/// Converts a `tiktoken` tokenizer definition into the encoder format used by this crate.
///
/// `data` is the raw data format used by the `tiktoken` tokenizer.
///
/// Returns the encoder for the data, or an error if the conversion fails.
///
/// The tiktoken definition data is composed of lines of the form `<token bytes> <token id>`, where `<token bytes>` is a base64-encoded byte sequence and `<token id>` is a decimal number.
/// Tiktoken definitions don't include special tokens.

#[derive(Debug)]
struct ParsedPiece {
    piece: SentencePiece,
    text:  BString,
}

/// Converts a `sentencepiece` model into the encoder format used by this crate.
/// Sentencepiece models are used and generated by the `sentencepiece` library.
///
/// `data` is the raw model data generated by the `sentencepiece` tokenizer.
///
/// Returns the encoder special encoder, scores and config for the model, or an error if the conversion fails.
pub fn convert_sentencepiece(
    data: impl AsRef<[u8]>,
) -> Result<(Vec<(Vec<u8>, u32)>, Vec<(Vec<u8>, u32)>, Vec<(u32, f32)>, Config), ConversionError> {
    let data = data.as_ref();

    let model = SentencePieceModel::from_slice(data).map_err(|e| {
        ConversionError::InvalidData(format!("failed to parse sentencepiece model: {:?}", e))
    })?;

    let mut vocab = IndexMap::<u32, ParsedPiece>::default();
    let mut tokens = IndexMap::<BString, u32>::default();
    let mut duplicates = Vec::<(BString, u32)>::new();
    let mut special_tokens = Vec::<(BString, u32)>::new();

    let mut config = Config {
        mode: Mode::CharPair,
        ..Config::default()
    };

    let mut model_type = ModelType::Bpe;

    debug!("trainer: {:#?}", model.trainer().unwrap());

    if let Some(trainer) = model.trainer() {
        let mut splits = Vec::new();
        if trainer.split_digits() {
            splits.push(r"[0-9]".to_string());
        } else if trainer.split_by_number() {
            splits.push(r"[0-9]+".to_string());
        }
        if trainer.split_by_unicode_script() {
            splits.push(r"\p{L}+".to_string());
        }
        if trainer.treat_whitespace_as_suffix() {
            splits.iter_mut().for_each(|s| s.push_str(r" ?"));
        } else {
            splits.iter_mut().for_each(|s| s.insert_str(0, r" ?"));
            splits.push(r" ?[^\t\n\f\r \p{L}0-9]+".to_string());
            // splits.push(r"[\t\n\f\r ]+(?![^\t\n\f\r ])".to_string());
        }
        if trainer.split_by_whitespace() {
            splits.push(r"[\t\n\f\r ]+".to_string());
        }
        config.split = splits.join("|");

        config.skip_whitespace = !trainer.allow_whitespace_only_pieces();
        config.unknown_token_id = Some(trainer.unk_id() as u32);
        config.unknown_token = Some(trainer.unk_surface().as_bytes().to_vec());

        model_type = trainer.model_type();
    } else {
        config.split =
            r" ?\p{L}+| ?[0-9]+| ?[^\t\n\f\r \p{L}0-9]+|[\t\n\f\r ]+(?![^\t\n\f\r ])|[\t\n\f\r ]+"
                .to_string();
    }
    if let Some(normalizer) = model.normalizer() {
        config.normalization = match normalizer.name() {
            "nmt_nfkc" => UnicodeNormalization::NFKCNMT,
            "nfkc" => UnicodeNormalization::NFKC,
            "nmt_nfkc_cf" => UnicodeNormalization::NFKCNMTCF,
            "nfkc_cf" => UnicodeNormalization::NFKCCF,
            _ => UnicodeNormalization::None,
        };
        config.trim_whitespace = normalizer.remove_extra_whitespaces();
        config.collapse_whitespace = normalizer.remove_extra_whitespaces();
        config.prefix_whitespace = normalizer.add_dummy_prefix();
    }


    for (index, piece) in model.pieces.iter().enumerate() {
        let piece_text = piece
            .piece
            .as_ref()
            .ok_or_else(|| ConversionError::InvalidData(format!("piece {} has no text", index)))?;
        let piece_type = piece.r#type();

        let text = if piece_type == Type::Byte {
            // byte encoding in the form `<0xAA>` where `AA` is a hexadecimal number representing a single byte
            let rune = &piece_text[3..5];
            let rune = u32::from_str_radix(std::str::from_utf8(rune.as_bytes()).unwrap(), 16)
                .map_err(|e| ConversionError::InvalidNumber(format!("{:?}", e)))?;
            let text = [rune as u8].to_vec().into();
            debug!("unicode: {:?} -> {:?}", piece_text, text);
            text
        } else {
            let text: BString = piece_text.to_string().replace('▁', " ").into();
            if piece_type == Type::UserDefined || piece_type == Type::Control {
                special_tokens.push((text.clone(), index as u32));
            }
            text
        };

        let mut insert = true;
        if let Some(existing) = tokens.get(&text) {
            let existing_piece = &vocab.get_mut(existing).unwrap().piece;
            let existing_type = existing_piece.r#type();
            duplicates.push((text.clone(), *existing));
            if piece_type == Type::Byte && existing_type != Type::Byte {
                insert = false;
            }
            debug!(
                "duplicate token: {} ({:?}) -> {} ({:?}) ({})",
                index,
                piece,
                existing,
                existing_piece,
                if insert { "replacing" } else { "skipping" }
            );
        }
        if insert {
            if let Some(id) = tokens.remove(&text) {
                vocab.remove(&id);
            }
            tokens.insert(text.clone(), index as u32);
            vocab.insert(index as u32, ParsedPiece {
                piece: piece.clone(),
                text,
            });
        }
    }

    let mut merges = IndexMap::<Vec<u8>, u32>::default();
    for (index, piece) in &vocab {
        for split in 1..piece.text.len() {
            let left = &piece.text[..split];
            let right = &piece.text[split..];
            if let (Some(_), Some(_)) = (tokens.get(left), tokens.get(right)) {
                merges.insert([left, right].concat(), *index);
            }
        }
    }
    merges.sort_by(|_, a, _, b| {
        if let (Some(pa), Some(pb)) = (vocab.get(a), vocab.get(b)) {
            if let (Some(sa), Some(sb)) = (pa.piece.score, pb.piece.score) {
                return sa.partial_cmp(&sb).unwrap();
            }
        }
        Ordering::Equal
    });

    special_tokens.sort_by(|(va, _), (vb, _)| {
        if let (Some(ma), Some(mb)) =
            (merges.get(va.bytes().as_bytes()), merges.get(vb.bytes().as_bytes()))
        {
            return mb.cmp(ma);
        }
        if let Some(ma) = merges.get(va.bytes().as_bytes()) {
            return ma.cmp(&u32::MAX);
        }
        if let Some(mb) = merges.get(vb.bytes().as_bytes()) {
            return u32::MAX.cmp(mb);
        }
        Ordering::Equal
    });
    let mut special_encoder = EncoderMap::default();
    for (text, index) in special_tokens {
        let special = text.bytes().as_bytes().to_vec();
        vocab.remove(&index).unwrap();
        special_encoder.insert(special, index);
    }

    let mut vocab = vocab.into_iter().collect::<Vec<_>>();
    if model_type == ModelType::Bpe {
        vocab.sort_by(|(_, va), (_, vb)| {
            if let (Some(ma), Some(mb)) =
                (merges.get(va.text.bytes().as_bytes()), merges.get(vb.text.bytes().as_bytes()))
            {
                return ma.cmp(mb);
            }
            if let Some(ma) = merges.get(va.text.bytes().as_bytes()) {
                return ma.cmp(&u32::MAX);
            }
            if let Some(mb) = merges.get(vb.text.bytes().as_bytes()) {
                return u32::MAX.cmp(mb);
            }
            Ordering::Equal
        });
    } else {
        vocab.sort_by(|(_, va), (_, vb)| {
            vb.piece.score().partial_cmp(&va.piece.score()).unwrap_or(Ordering::Equal)
        });
    }

    // let file = std::fs::File::create("spiece.txt").unwrap();
    // let mut writer = std::io::BufWriter::new(file);
    // use std::io::Write;
    // for (key, value) in special_encoder.iter() {
    //     writeln!(writer, "\"{}\": {}", value, key.to_str_lossy()).unwrap();
    // }
    // writeln!(writer, "------").unwrap();
    // for (key, value) in vocab.iter() {
    //     writeln!(writer, "\"{}\": {}: {}", key, value.piece.score(), value.text).unwrap();
    // }

    let mut encoder = EncoderMap::default();
    for (index, piece) in vocab.iter() {
        encoder.insert(piece.text.bytes().as_bytes().to_vec(), *index);
    }

    let scores = vocab
        .iter()
        .map(|(_, v)| (v.text.bytes().as_bytes().to_owned(), v.piece.score()))
        .collect();

    Ok((encoder, special_encoder, config, scores))
}
