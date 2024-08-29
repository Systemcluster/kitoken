//! Test for the conversion of OpenAI Tiktoken models.

use kitoken::{Definition, Kitoken};

mod util;
use util::*;

#[test]
fn test_serialize_deserialize() {
    init_env();
    eprintln!();
    for model in test_models("tiktoken", "tiktoken") {
        log::info!("converting: {}", model.to_string_lossy());
        let data = std::fs::read(model).unwrap();
        let definition1 = Definition::from_slice(&data).unwrap();
        let tokenizer = Kitoken::from_definition(definition1.clone()).unwrap();
        let definition2 = tokenizer.to_definition();
        test_definitions_same(definition1, definition2);
    }
}

#[test]
fn test_small_lines() {
    init_env();
    test_encode_decode_lines("tiktoken", "tiktoken", "small", true, |model| {
        Kitoken::from_tiktoken_file(model).unwrap()
    })
}

#[test]
fn test_utf8_full() {
    init_env();
    test_encode_decode_full("tiktoken", "tiktoken", "utf8", true, |model| {
        Kitoken::from_tiktoken_file(model).unwrap()
    })
}

#[test]
fn test_mixed_lines() {
    init_env();
    test_encode_decode_lines("tiktoken", "tiktoken", "mixed", true, |model| {
        Kitoken::from_tiktoken_file(model).unwrap()
    })
}
