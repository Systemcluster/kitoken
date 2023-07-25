#![allow(dead_code)]

use std::path::PathBuf;

use bstr::ByteSlice;
use console::style;

use kitoken::Kitoken;

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
        .map(|line| line.to_owned().replace("\\n", "\n").replace("\\s", " "))
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
    let decode_result = tokenizer.decode(&encode_result).unwrap();
    let decode_ok = decode_result == output.as_bytes();
    (encode_ok, encode_result, decode_ok, decode_result)
}

pub fn test_encode_decode_full_same(
    tokenizer: &Kitoken, input: impl Into<PathBuf>, tokens: impl Into<PathBuf>,
) {
    let input = input.into();
    let tokens = tokens.into();
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
    let tokens = tokens.into();
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
    let mut results = Vec::with_capacity(input_lines.len());
    for (i, ((input, tokens), output)) in
        input_lines.iter().zip(tokens_lines.iter()).zip(output_lines.iter()).enumerate()
    {
        let (encode_ok, encode_result, decode_ok, decode_result) =
            check_encode_decode_different(tokenizer, input, tokens, output);
        if !encode_ok {
            let line = style(format!("encode mismatch #{}", i + 1)).on_red();
            eprintln!("{}: {:?}", line, encode_result);
            let line = style(format!("expected:       #{}", i + 1)).on_yellow();
            eprintln!("{}: {:?}", line, tokens);
        }
        if !decode_ok {
            let line = style(format!("decode mismatch #{}", i + 1)).on_red();
            eprintln!("{}: {:?}", line, decode_result.as_bstr());
            let line = style(format!("expected:       #{}", i + 1)).on_yellow();
            eprintln!("{}: {:?}", line, output);
        }
        if encode_ok && decode_ok {
            let line = style(format!("ok #{}", i + 1)).on_green();
            eprintln!("{}: {:?}", line, input);
        } else if exit_on_error {
            std::process::exit(1);
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
