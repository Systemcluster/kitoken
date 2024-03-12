use std::hint::black_box;
use std::path::PathBuf;
use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};

use kitoken::convert::*;
use kitoken::Kitoken;

pub fn bench_models_path() -> std::path::PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/models")
}

pub fn bench_data_path() -> std::path::PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("benches/data")
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

fn init_kitoken() -> Kitoken {
    let data = std::fs::read(bench_models_path().join("mistral.model")).unwrap();
    let definition = convert_sentencepiece(data).unwrap();
    definition.try_into().unwrap()
}

fn bench_convert(b: &mut Criterion) {
    let data = std::fs::read(bench_models_path().join("mistral.model")).unwrap();
    let definition = convert_sentencepiece(data).unwrap();
    b.bench_function("mistral: convert", |b| {
        b.iter(|| {
            Kitoken::try_from(black_box(definition.clone())).unwrap();
        })
    });
}

fn bench_encode_pride_and_prejudice(b: &mut Criterion) {
    let tokenizer = init_kitoken();
    let text = std::fs::read_to_string(bench_data_path().join("pride_and_prejudice.txt")).unwrap();
    b.bench_function("mistral: encode pride_and_prejudice", |b| {
        b.iter(|| {
            tokenizer.encode(black_box(&text), true).unwrap();
        })
    });
}

fn bench_encode_utf8_sequence_0x10ffff(b: &mut Criterion) {
    let tokenizer = init_kitoken();
    let text =
        std::fs::read_to_string(bench_data_path().join("utf8_sequence_0x10ffff.txt")).unwrap();
    b.bench_function("mistral: encode utf8_sequence_0x10ffff", |b| {
        b.iter(|| {
            tokenizer.encode(black_box(&text), true).unwrap();
        })
    });
}

fn bench_encode_wagahai(b: &mut Criterion) {
    let tokenizer = init_kitoken();
    let text = std::fs::read_to_string(bench_data_path().join("wagahai.txt")).unwrap();
    b.bench_function("mistral: encode wagahai", |b| {
        b.iter(|| {
            tokenizer.encode(black_box(&text), true).unwrap();
        })
    });
}

criterion_group! {
    name = convert;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(10);
    targets = bench_convert
}
criterion_group! {
    name = encode;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(20))
        .sample_size(20);
    targets = bench_encode_pride_and_prejudice, bench_encode_utf8_sequence_0x10ffff, bench_encode_wagahai
}
criterion_main!(convert, encode);
