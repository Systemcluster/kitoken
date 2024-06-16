//! Test for the conversion of HuggingFace Tokenizers models.

use kitoken::convert::*;
use kitoken::Kitoken;

mod util;
use util::*;

#[test]
fn test_serialize_deserialize() {
    init_env();
    let mut models = std::fs::read_dir(test_models_path().join("tokenizers"))
        .unwrap()
        .map(|x| x.unwrap())
        .filter(|x| x.file_type().unwrap().is_file())
        .collect::<Vec<_>>();
    models.sort_by_key(|x| x.file_name());
    for model in models {
        log::info!("converting: {:?}", model);
        let data = std::fs::read(model.path()).unwrap();
        let definition1 = convert_tokenizers(data).unwrap();
        let tokenizer = Kitoken::from_definition(definition1.clone()).unwrap();
        let definition2 = tokenizer.to_definition();
        test_definitions_same(definition1, definition2);
    }
}

#[test]
fn test_small_lines() {
    init_env();
    let mut models = std::fs::read_dir(test_models_path().join("tokenizers"))
        .unwrap()
        .map(|x| x.unwrap())
        .filter(|x| x.file_type().unwrap().is_file())
        .collect::<Vec<_>>();
    models.sort_by_key(|x| x.file_name());
    for model in models {
        let name = model.file_name();
        let name = name.to_str().unwrap();
        let name = name.trim_end_matches(".json");
        let tokens = std::fs::metadata(
            test_data_path()
                .join("tokenizers")
                .join(["small_tokens_", name, ".txt"].concat()),
        );
        if !tokens.is_ok_and(|tokens| tokens.is_file()) {
            log::warn!("small_input.txt: {:?}: skipping (no token data)", model);
            continue;
        }
        let output = std::fs::metadata(
            test_data_path()
                .join("tokenizers")
                .join(["small_output_", name, ".txt"].concat()),
        );
        log::info!("small_input.txt: {:?}", model);
        let tokenizer = Kitoken::from_tokenizers_file(model.path()).unwrap();
        if output.is_ok_and(|output| output.is_file()) {
            test_encode_decode_lines_different(
                &tokenizer,
                "small_input.txt",
                ["tokenizers/small_tokens_", name, ".txt"].concat(),
                ["tokenizers/small_output_", name, ".txt"].concat(),
                false,
            );
        } else {
            test_encode_decode_lines_same(
                &tokenizer,
                "small_input.txt",
                ["tokenizers/small_tokens_", name, ".txt"].concat(),
                false,
            );
        }
    }
}

#[test]
fn test_utf8_full() {
    init_env();
    let mut models = std::fs::read_dir(test_models_path().join("tokenizers"))
        .unwrap()
        .map(|x| x.unwrap())
        .filter(|x| x.file_type().unwrap().is_file())
        .collect::<Vec<_>>();
    models.sort_by_key(|x| x.file_name());
    for model in models {
        let name = model.file_name();
        let name = name.to_str().unwrap();
        let name = name.trim_end_matches(".json");
        let tokens = std::fs::metadata(
            test_data_path()
                .join("tokenizers")
                .join(["utf8_tokens_", name, ".txt"].concat()),
        );
        if !tokens.is_ok_and(|tokens| tokens.is_file()) {
            log::warn!("utf8_input.txt: {:?}: skipping (no token data)", model);
            continue;
        }
        let output = std::fs::metadata(
            test_data_path()
                .join("tokenizers")
                .join(["utf8_output_", name, ".txt"].concat()),
        );
        log::info!("utf8_input.txt: {:?}", model);
        let tokenizer = Kitoken::from_tokenizers_file(model.path()).unwrap();
        if output.is_ok_and(|output| output.is_file()) {
            test_encode_decode_full_different(
                &tokenizer,
                "utf8_input.txt",
                ["tokenizers/utf8_tokens_", name, ".txt"].concat(),
                ["tokenizers/utf8_output_", name, ".txt"].concat(),
            );
        } else {
            test_encode_decode_full_same(
                &tokenizer,
                "utf8_input.txt",
                ["tokenizers/utf8_tokens_", name, ".txt"].concat(),
            );
        }
    }
}
