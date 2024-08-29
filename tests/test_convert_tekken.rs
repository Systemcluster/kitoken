//! Test for the conversion of tekken models.

use kitoken::{Definition, Kitoken};

mod util;
use util::*;

#[test]
fn test_serialize_deserialize() {
    init_env();
    eprintln!();
    for model in test_models("tekken", "json") {
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
    test_encode_decode_lines("tekken", "json", "small", false, |model| {
        Kitoken::from_tekken_file(model).unwrap()
    })
}

#[test]
fn test_utf8_full() {
    init_env();
    test_encode_decode_full("tekken", "json", "utf8", false, |model| {
        Kitoken::from_tekken_file(model).unwrap()
    })
}

#[test]
fn test_mixed_lines() {
    init_env();
    test_encode_decode_lines("tekken", "json", "mixed", false, |model| {
        Kitoken::from_tekken_file(model).unwrap()
    })
}
