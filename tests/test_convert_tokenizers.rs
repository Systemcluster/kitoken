//! Test for the conversion of HuggingFace Tokenizers models.

#![cfg_attr(
    feature = "multiversion",
    feature(
        allocator_api,
        lahfsahf_target_feature,
        avx512_target_feature,
        aarch64_ver_target_feature
    )
)]

use kitoken::convert::*;
use kitoken::Kitoken;

mod util;
use util::*;

#[test]
fn test_serialize_deserialize() {
    init_env();
    eprintln!();
    for model in test_models("tokenizers", "json") {
        log::info!("converting: {}", model.to_string_lossy());
        let data = std::fs::read(model).unwrap();
        let definition1 = convert_tokenizers(data).unwrap();
        let tokenizer = Kitoken::from_definition(definition1.clone()).unwrap();
        let definition2 = tokenizer.to_definition();
        test_definitions_same(definition1, definition2);
    }
}

#[test]
fn test_small_lines() {
    init_env();
    test_encode_decode_lines("tokenizers", "json", "small", true, |model| {
        Kitoken::from_tokenizers_file(model).unwrap()
    })
}

#[test]
fn test_utf8_full() {
    init_env();
    test_encode_decode_full("tokenizers", "json", "utf8", true, |model| {
        Kitoken::from_tokenizers_file(model).unwrap()
    })
}

#[test]
fn test_mixed_lines() {
    init_env();
    test_encode_decode_lines("tokenizers", "json", "mixed", true, |model| {
        Kitoken::from_tokenizers_file(model).unwrap()
    })
}
