//! Test for the conversion of the sentencepiece nerdstash bpe model.

use kitoken::convert::*;
use kitoken::Kitoken;

mod util;
use util::*;

static MODEL_PATH: &str = "sentencepiece/nerdstash.model";

#[test]
fn test_serialize_deserialize() {
    init_env();
    let data = std::fs::read(test_models_path().join(MODEL_PATH)).unwrap();
    let definition1 = convert_sentencepiece(data).unwrap();
    let tokenizer = Kitoken::from_definition(definition1.clone()).unwrap();
    let definition2 = tokenizer.to_definition();
    test_definitions_same(definition1, definition2);
}

#[test]
fn test_small_lines() {
    init_env();
    let tokenizer = Kitoken::from_sentencepiece_file(test_models_path().join(MODEL_PATH)).unwrap();
    test_encode_decode_lines_same(
        &tokenizer,
        "small_input.txt",
        "small_tokens_nerdstash.txt",
        false,
    );
}

#[test]
fn test_utf8_full() {
    init_env();
    let tokenizer = Kitoken::from_sentencepiece_file(test_models_path().join(MODEL_PATH)).unwrap();
    test_encode_decode_full_same(&tokenizer, "utf8_input.txt", "utf8_tokens_nerdstash.txt");
}
