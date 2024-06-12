#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::Once;

use bstr::ByteSlice;
use console::style;

use kitoken::{Definition, Kitoken};

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
    tokenizer: &Kitoken, input: &str, tokens: &[u32],
) -> (bool, Vec<u32>, bool, Vec<u8>) {
    check_encode_decode_different(tokenizer, input, tokens, input)
}

pub fn check_encode_decode_different(
    tokenizer: &Kitoken, input: &str, tokens: &[u32], output: &str,
) -> (bool, Vec<u32>, bool, Vec<u8>) {
    let encode_result = tokenizer.encode(input, true).unwrap();
    let encode_ok = encode_result == tokens;
    let decode_result = tokenizer.decode(&encode_result, true).unwrap();
    let decode_ok = decode_result == output.as_bytes();
    (encode_ok, encode_result, decode_ok, decode_result)
}

pub fn test_encode_decode_full_same(
    tokenizer: &Kitoken, input: impl Into<PathBuf>, tokens: impl Into<PathBuf>,
) {
    let input = input.into();
    test_encode_decode_full_different(tokenizer, &input, tokens, &input);
}

pub fn test_encode_decode_full_different(
    tokenizer: &Kitoken, input: impl Into<PathBuf>, tokens: impl Into<PathBuf>,
    output: impl Into<PathBuf>,
) {
    let input = read_full(test_data_path().join(input.into()));
    let tokens = read_tokens_full(test_data_path().join(tokens.into()));
    let output = read_full(test_data_path().join(output.into()));
    eprintln!();
    let (encode_ok, _, decode_ok, _) =
        check_encode_decode_different(tokenizer, &input, &tokens, &output);
    if !encode_ok {
        let line = style("encode mismatch".to_string()).on_red();
        eprintln!("{}", line);
    }
    if !decode_ok {
        let line = style("decode mismatch".to_string()).on_red();
        eprintln!("{}", line);
    }
    assert!(encode_ok, "encoding passes");
    assert!(decode_ok, "decoding passes");
}

pub fn test_encode_decode_lines_same(
    tokenizer: &Kitoken, input: impl Into<PathBuf>, tokens: impl Into<PathBuf>, exit_on_error: bool,
) {
    let input = input.into();
    test_encode_decode_lines_different(tokenizer, &input, tokens, &input, exit_on_error);
}

pub fn test_encode_decode_lines_different(
    tokenizer: &Kitoken, input: impl Into<PathBuf>, tokens: impl Into<PathBuf>,
    output: impl Into<PathBuf>, exit_on_error: bool,
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
    eprintln!();
    let sep = style(":").dim();
    let mut results = Vec::with_capacity(input_lines.len());
    for (i, ((input, tokens), output)) in
        input_lines.iter().zip(tokens_lines.iter()).zip(output_lines.iter()).enumerate()
    {
        let (encode_ok, encode_result, decode_ok, decode_result) =
            check_encode_decode_different(tokenizer, input, tokens, output);
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
            &left.vocab[diff_from..diff_to],
            style("right").bold().magenta(),
            sep,
            &right.vocab[diff_from..diff_to]
        );
        let line =
            style(format!("specials mismatch from index {} to {}", diff_from, diff_to)).red();
        eprintln!("{}", line);
    }
    assert_eq!(left.vocab.len(), right.vocab.len(), "vocab lengths are equal");
    if left.vocab != right.vocab {
        let line = style("vocab mismatch").on_red();
        let diff_from =
            left.vocab.iter().zip(right.vocab.iter()).position(|(a, b)| a != b).unwrap();
        let diff_to = left.vocab[diff_from..]
            .iter()
            .zip(right.vocab[diff_from..].iter())
            .position(|(a, b)| a == b)
            .unwrap_or(left.vocab.len() - diff_from)
            + diff_from;
        eprintln!(
            "{}{}\n\t{}{} {:?}\n\t{}{} {:?}",
            line,
            sep,
            style("left").bold().magenta(),
            sep,
            &left.vocab[diff_from..diff_to],
            style("right").bold().magenta(),
            sep,
            &right.vocab[diff_from..diff_to]
        );
        let line = style(format!("vocab mismatch from index {} to {}", diff_from, diff_to)).red();
        eprintln!("{}", line);
    }
    assert_eq!(left.scores.len(), right.scores.len(), "scores lengths are equal");
    if left.scores != right.scores {
        let line = style("scores mismatch").on_red();
        let diff_from =
            left.scores.iter().zip(right.scores.iter()).position(|(a, b)| a != b).unwrap();
        let diff_to = left.scores[diff_from..]
            .iter()
            .zip(right.scores[diff_from..].iter())
            .position(|(a, b)| a == b)
            .unwrap_or(left.scores.len() - diff_from)
            + diff_from;
        eprintln!(
            "{}{}\n\t{}{} {:?}\n\t{}{} {:?}",
            line,
            sep,
            style("left").bold().magenta(),
            sep,
            &left.vocab[diff_from..diff_to],
            style("right").bold().magenta(),
            sep,
            &right.vocab[diff_from..diff_to]
        );
        let line = style(format!("scores mismatch from index {} to {}", diff_from, diff_to)).red();
        eprintln!("{}", line);
    }
    assert_eq!(left, right, "definitions are equal");
}
