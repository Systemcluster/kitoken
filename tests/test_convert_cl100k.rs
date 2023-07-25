//! Test for the conversion of the tiktoken cl100k model.

use std::sync::{Once, OnceLock};

use kitoken::convert::*;
use kitoken::Kitoken;

mod util;
use util::*;

fn init_kitoken() -> &'static Kitoken {
    static INIT_ENV: Once = Once::new();
    INIT_ENV.call_once(|| {
        simple_logger::init_with_level(log::Level::Debug).unwrap();
    });
    static TOKENIZER: OnceLock<Kitoken> = OnceLock::new();
    TOKENIZER.get_or_init(|| {
        let data = std::fs::read(test_models_path().join("cl100k_base.tiktoken")).unwrap();
        let definition = convert_tiktoken(data).unwrap();
        eprintln!("{:?}", definition.config);
        Kitoken::from_definition(definition).unwrap()
    })
}

#[test]
fn test_serialize_deserialize() {
    let data = std::fs::read(test_models_path().join("cl100k_base.tiktoken")).unwrap();
    let definition1 = convert_tiktoken(data).unwrap();
    let tokenizer = Kitoken::from_definition(definition1.clone()).unwrap();
    let definition2 = tokenizer.to_definition();
    assert_eq!(definition1.vocab.len(), definition2.vocab.len(), "vocab lengths are equal");
    assert_eq!(
        definition1.specials.len(),
        definition2.specials.len(),
        "special vocab lengths are equal"
    );
    assert_eq!(definition1.scores.len(), definition2.scores.len(), "scores lengths are equal");
    assert_eq!(&definition1, &definition2, "definitions are equal");
}

#[test]
fn test_small_lines() {
    let tokenizer = init_kitoken();
    test_encode_decode_lines_same(tokenizer, "small_input.txt", "small_tokens_cl100k.txt", false);
}

#[test]
fn test_utf8_full() {
    let tokenizer = init_kitoken();
    test_encode_decode_full_same(tokenizer, "utf8_input.txt", "utf8_tokens_cl100k.txt");
}
