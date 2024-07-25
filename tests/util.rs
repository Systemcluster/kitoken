#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::Once;

use bstr::ByteSlice;
use console::style;

use kitoken::{Definition, Kitoken, Model};

static INIT_ENV: Once = Once::new();

pub fn init_env() {
    INIT_ENV.call_once(|| {
        simple_logger::SimpleLogger::new()
            .with_level(log::Level::Debug.to_level_filter())
            .env()
            .init()
            .unwrap();
    });
}

pub fn test_models_path() -> std::path::PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/models")
}

pub fn test_data_path() -> std::path::PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data")
}

pub fn test_models(path: impl AsRef<str>, extension: impl AsRef<str>) -> Vec<PathBuf> {
    let mut models = std::fs::read_dir(test_models_path().join(path.as_ref()))
        .unwrap()
        .map(|x| x.unwrap())
        .filter(|x| x.file_type().unwrap().is_file())
        .collect::<Vec<_>>();
    models.sort_by_key(|x| x.file_name());
    let models_arg = std::env::var_os("MODELS")
        .map(|file| {
            file.to_string_lossy()
                .split(",")
                .map(|file| file.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let models = models
        .into_iter()
        .map(|x| x.path())
        .filter(|x| x.extension().filter(|&e| e == extension.as_ref()).is_some())
        .filter(|x| {
            let name = x.file_name().unwrap().to_string_lossy();
            if !models_arg.is_empty() && !models_arg.iter().any(|model| name.starts_with(model)) {
                log::info!("skipping: {}", x.to_string_lossy());
                return false;
            }
            true
        })
        .collect::<Vec<_>>();
    models
}

pub fn test_encode_decode_lines(
    path: impl AsRef<str>, extension: impl AsRef<str>, input: impl AsRef<str>, specials: bool,
    init: impl Fn(&Path) -> Kitoken,
) {
    eprintln!();
    let path = path.as_ref();
    let extension = extension.as_ref();
    let input = input.as_ref();
    for model in test_models(path, extension) {
        let name = model.file_name().unwrap().to_string_lossy();
        let name = name.trim_end_matches(extension).trim_end_matches('.');
        let tokens = std::fs::metadata(
            test_data_path().join(path).join([input, "_tokens_", name, ".txt"].concat()),
        );
        if !tokens.is_ok_and(|tokens| tokens.is_file()) {
            log::warn!(
                "{}_input.txt: {}: skipping (no token data)",
                input,
                model.to_string_lossy()
            );
            continue;
        }
        let output = std::fs::metadata(
            test_data_path().join(path).join([input, "_output_", name, ".txt"].concat()),
        );
        log::info!("{}_input.txt: {}", input, model.to_string_lossy());
        let tokenizer = init(&model);
        if output.is_ok_and(|output| output.is_file()) {
            test_encode_decode_lines_different(
                &tokenizer,
                [input, "_input.txt"].concat(),
                [path, "/", input, "_tokens_", name, ".txt"].concat(),
                [path, "/", input, "_output_", name, ".txt"].concat(),
                specials,
                false,
            );
        } else {
            test_encode_decode_lines_same(
                &tokenizer,
                [input, "_input.txt"].concat(),
                [path, "/", input, "_tokens_", name, ".txt"].concat(),
                specials,
                false,
            );
        }
    }
}

pub fn test_encode_decode_full(
    path: impl AsRef<str>, extension: impl AsRef<str>, input: impl AsRef<str>, specials: bool,
    init: impl Fn(&Path) -> Kitoken,
) {
    eprintln!();
    let path = path.as_ref();
    let extension = extension.as_ref();
    let input = input.as_ref();
    for model in test_models(path, extension) {
        let name = model.file_name().unwrap().to_string_lossy();
        let name = name.trim_end_matches(extension).trim_end_matches('.');
        let tokens = std::fs::metadata(
            test_data_path().join(path).join([input, "_tokens_", name, ".txt"].concat()),
        );
        if !tokens.is_ok_and(|tokens| tokens.is_file()) {
            log::warn!(
                "{}_input.txt: {}: skipping (no token data)",
                input,
                model.to_string_lossy()
            );
            continue;
        }
        let output = std::fs::metadata(
            test_data_path().join(path).join([input, "_output_", name, ".txt"].concat()),
        );
        log::info!("{}_input.txt: {}", input, model.to_string_lossy());
        let tokenizer = init(&model);
        if output.is_ok_and(|output| output.is_file()) {
            test_encode_decode_full_different(
                &tokenizer,
                [input, "_input.txt"].concat(),
                [path, "/", input, "_tokens_", name, ".txt"].concat(),
                [path, "/", input, "_output_", name, ".txt"].concat(),
                specials,
            );
        } else {
            test_encode_decode_full_same(
                &tokenizer,
                [input, "_input.txt"].concat(),
                [path, "/", input, "_tokens_", name, ".txt"].concat(),
                specials,
            );
        }
    }
}

pub fn read_lines(path: impl Into<PathBuf>) -> Vec<String> {
    let path = path.into();
    let lines = std::fs::read_to_string(path).unwrap();
    lines
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| line.to_owned().replace("\\n", "\n").replace("\\s", " ").replace("\\x", ""))
        .collect()
}

pub fn read_full(path: impl Into<PathBuf>) -> String {
    let path = path.into();
    std::fs::read_to_string(path).unwrap()
}

pub fn read_token_lines(path: impl Into<PathBuf>) -> Vec<Vec<u32>> {
    let path = path.into();
    let lines = std::fs::read_to_string(path).unwrap();
    lines
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| line.split(',').map(|token| token.trim().parse().unwrap()).collect::<Vec<_>>())
        .collect()
}

pub fn read_tokens_full(path: impl Into<PathBuf>) -> Vec<u32> {
    let path = path.into();
    let lines = std::fs::read_to_string(path).unwrap();
    lines
        .lines()
        .filter(|line| !line.is_empty())
        .flat_map(|line| {
            line.split(',').map(|token| token.trim().parse().unwrap()).collect::<Vec<_>>()
        })
        .collect()
}

pub fn check_encode_decode_same(
    tokenizer: &Kitoken, input: &str, tokens: &[u32], encode_specials: bool,
) -> (bool, Vec<u32>, bool, Vec<u8>) {
    check_encode_decode_different(tokenizer, input, tokens, input, encode_specials)
}

pub fn check_encode_decode_different(
    tokenizer: &Kitoken, input: &str, tokens: &[u32], output: &str, encode_specials: bool,
) -> (bool, Vec<u32>, bool, Vec<u8>) {
    let encode_result = tokenizer.encode(input, encode_specials).unwrap();
    let encode_ok = encode_result == tokens;
    let decode_result = tokenizer.decode(&encode_result, encode_specials).unwrap();
    let decode_ok = decode_result == output.as_bytes();
    (encode_ok, encode_result, decode_ok, decode_result)
}

pub fn test_encode_decode_full_same(
    tokenizer: &Kitoken, input: impl Into<PathBuf>, tokens: impl Into<PathBuf>,
    encode_specials: bool,
) {
    let input = input.into();
    test_encode_decode_full_different(tokenizer, &input, tokens, &input, encode_specials);
}

pub fn test_encode_decode_full_different(
    tokenizer: &Kitoken, input: impl Into<PathBuf>, tokens: impl Into<PathBuf>,
    output: impl Into<PathBuf>, encode_specials: bool,
) {
    let output = output.into();
    let tokens = tokens.into();
    let tokens_dump_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join(PathBuf::from(&tokens).file_name().unwrap())
        .with_extension("error.txt");
    let output_dump_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join(PathBuf::from(&output).file_name().unwrap())
        .with_extension("error.txt");
    let input = read_full(test_data_path().join(input.into()));
    let tokens = read_tokens_full(test_data_path().join(tokens));
    let output = read_full(test_data_path().join(output));
    let (encode_ok, encode_result, decode_ok, decode_result) =
        check_encode_decode_different(tokenizer, &input, &tokens, &output, encode_specials);
    if !encode_ok {
        let line = style("encode mismatch".to_string()).on_red();
        eprintln!("{}", line);
    }
    if !decode_ok {
        let line = style("decode mismatch".to_string()).on_red();
        eprintln!("{}", line);
    }
    if !encode_ok && std::env::var("DUMP_ERRORS").is_ok() {
        use std::io::Write;
        let file = std::fs::File::create(&tokens_dump_path).unwrap();
        let mut writer = std::io::BufWriter::new(file);
        writer
            .write_all(
                encode_result
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
                    .as_bytes(),
            )
            .unwrap();
    }
    if !decode_ok && std::env::var_os("DUMP_ERRORS").is_some() {
        use std::io::Write;
        let file = std::fs::File::create(&output_dump_path).unwrap();
        let mut writer = std::io::BufWriter::new(file);
        writer.write_all(decode_result.as_bstr()).unwrap();
    }
    assert!(encode_ok, "encoding passes");
    assert!(decode_ok, "decoding passes");
}

pub fn test_encode_decode_lines_same(
    tokenizer: &Kitoken, input: impl Into<PathBuf>, tokens: impl Into<PathBuf>,
    encode_specials: bool, exit_on_error: bool,
) {
    let input = input.into();
    test_encode_decode_lines_different(
        tokenizer,
        &input,
        tokens,
        &input,
        encode_specials,
        exit_on_error,
    );
}

pub fn test_encode_decode_lines_different(
    tokenizer: &Kitoken, input: impl Into<PathBuf>, tokens: impl Into<PathBuf>,
    output: impl Into<PathBuf>, encode_specials: bool, exit_on_error: bool,
) {
    let input_lines = read_lines(test_data_path().join(input.into()));
    let tokens_lines = read_token_lines(test_data_path().join(tokens.into()));
    let output_lines = read_lines(test_data_path().join(output.into()));
    assert_eq!(
        input_lines.len(),
        tokens_lines.len(),
        "input lines and output token lines have same number of lines"
    );
    assert_eq!(
        input_lines.len(),
        output_lines.len(),
        "input lines and output lines have same number of lines"
    );
    let sep = style(":").dim();
    let mut results = Vec::with_capacity(input_lines.len());
    for (i, ((input, tokens), output)) in
        input_lines.iter().zip(tokens_lines.iter()).zip(output_lines.iter()).enumerate()
    {
        let (encode_ok, encode_result, decode_ok, decode_result) =
            check_encode_decode_different(tokenizer, input, tokens, output, encode_specials);
        if !encode_ok {
            let line = style(format!("encode mismatch #{}", i + 1)).on_red();
            eprintln!("{}{} {:?}", line, sep, encode_result);
            let line = style(format!("expected        #{}", i + 1)).bold().red();
            eprintln!("{}{} {:?}", line, sep, tokens);
        }
        if !decode_ok {
            let line = style(format!("decode mismatch #{}", i + 1)).on_red();
            eprintln!("{}{} {:?}", line, sep, decode_result.as_bstr());
            let line = style(format!("expected        #{}", i + 1)).bold().red();
            eprintln!("{}{} {:?}", line, sep, output.as_bytes().as_bstr());
        }
        if encode_ok && decode_ok {
            let line = style(format!("ok #{}", i + 1)).on_green();
            eprintln!("{}{} {:?}", line, sep, input);
        } else {
            if encode_ok {
                let line = style(format!("encode ok       #{}", i + 1)).green();
                eprintln!("{}{} {:?}", line, sep, input);
            }
            if decode_ok {
                let line = style(format!("decode ok       #{}", i + 1)).green();
                eprintln!("{}{} {:?}", line, sep, decode_result.as_bstr());
            }
            if exit_on_error {
                std::process::exit(1);
            }
        }
        results.push((encode_ok, decode_ok));
    }
    let (encode_ok, decode_ok) =
        results
            .iter()
            .fold((true, true), |(encode_ok, decode_ok), (encode_ok2, decode_ok2)| {
                (encode_ok && *encode_ok2, decode_ok && *decode_ok2)
            });
    let failures = results.iter().filter(|(encode_ok, decode_ok)| !encode_ok || !decode_ok).count();
    let passes = results.len() - failures;
    eprintln!("\n{}", style(format!("{} passes", passes)).on_green());
    if failures > 0 {
        eprintln!("{}", style(format!("{} failures", failures)).on_red());
    }
    assert!(encode_ok, "encoding passes");
    assert!(decode_ok, "decoding passes");
}

pub fn test_definitions_same(left: Definition, right: Definition) {
    let sep = style(":").dim();
    assert_eq!(left.meta, right.meta, "meta is equal");
    assert_eq!(left.config, right.config, "config is equal");
    assert_eq!(left.specials.len(), right.specials.len(), "specials lengths are equal");
    if left.specials != right.specials {
        let line = style("specials mismatch").on_red();
        let diff_from = left
            .specials
            .iter()
            .zip(right.specials.iter())
            .position(|(a, b)| a != b)
            .unwrap();
        let diff_to = left.specials[diff_from..]
            .iter()
            .zip(right.specials[diff_from..].iter())
            .position(|(a, b)| a == b)
            .unwrap_or(left.specials.len() - diff_from)
            + diff_from;
        eprintln!(
            "{}{}\n\t{}{} {:?}\n\t{}{} {:?}",
            line,
            sep,
            style("left").bold().magenta(),
            sep,
            &left.model.vocab()[diff_from..diff_to],
            style("right").bold().magenta(),
            sep,
            &right.model.vocab()[diff_from..diff_to]
        );
        let line =
            style(format!("specials mismatch from index {} to {}", diff_from, diff_to)).red();
        eprintln!("{}", line);
    }
    assert_eq!(left.model.vocab().len(), right.model.vocab().len(), "vocab lengths are equal");
    if left.model.vocab() != right.model.vocab() {
        let line = style("vocab mismatch").on_red();
        let diff_from = left
            .model
            .vocab()
            .iter()
            .zip(right.model.vocab().iter())
            .position(|(a, b)| a != b)
            .unwrap();
        let diff_to = left.model.vocab()[diff_from..]
            .iter()
            .zip(right.model.vocab()[diff_from..].iter())
            .position(|(a, b)| a == b)
            .unwrap_or(left.model.vocab().len() - diff_from)
            + diff_from;
        eprintln!(
            "{}{}\n\t{}{} {:?}\n\t{}{} {:?}",
            line,
            sep,
            style("left").bold().magenta(),
            sep,
            &left.model.vocab()[diff_from..diff_to],
            style("right").bold().magenta(),
            sep,
            &right.model.vocab()[diff_from..diff_to]
        );
        let line = style(format!("vocab mismatch from index {} to {}", diff_from, diff_to)).red();
        eprintln!("{}", line);
    }
    if let (
        Model::Unigram {
            scores: scores_left,
            ..
        },
        Model::Unigram {
            scores: scores_right,
            ..
        },
    ) = (&left.model, &right.model)
    {
        assert_eq!(scores_left.len(), scores_right.len(), "scores lengths are equal");
        if scores_left != scores_right {
            let line = style("scores mismatch").on_red();
            let diff_from =
                scores_left.iter().zip(scores_right.iter()).position(|(a, b)| a != b).unwrap();
            let diff_to = scores_left[diff_from..]
                .iter()
                .zip(scores_right[diff_from..].iter())
                .position(|(a, b)| a == b)
                .unwrap_or(scores_left.len() - diff_from)
                + diff_from;
            eprintln!(
                "{}{}\n\t{}{} {:?}\n\t{}{} {:?}",
                line,
                sep,
                style("left").bold().magenta(),
                sep,
                &left.model.vocab()[diff_from..diff_to],
                style("right").bold().magenta(),
                sep,
                &right.model.vocab()[diff_from..diff_to]
            );
            let line =
                style(format!("scores mismatch from index {} to {}", diff_from, diff_to)).red();
            eprintln!("{}", line);
        }
    }
    assert_eq!(left, right, "definitions are equal");
}
