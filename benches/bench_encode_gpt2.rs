use std::hint::black_box;
use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};

use kitoken::Kitoken;

mod util;
use util::*;

static MODEL_PATH: &str = "tokenizers/gpt2.json";

fn bench_convert(b: &mut Criterion) {
    let data = std::fs::read(bench_models_path().join(MODEL_PATH)).unwrap();
    b.bench_function("gpt2: convert tokenizers", |b| {
        b.iter(|| {
            Kitoken::from_tokenizers_slice(black_box(&data)).unwrap();
        })
    });
}

fn bench_encode_pride_and_prejudice(b: &mut Criterion) {
    init_env();
    let text = read_data_file("pride_and_prejudice.txt");
    let mut g = b.benchmark_group("gpt2: encode pride_and_prejudice");
    g.sampling_mode(criterion::SamplingMode::Flat);
    g.bench_function("full", |b| {
        let tokenizer =
            Kitoken::from_tokenizers_file(bench_models_path().join(MODEL_PATH)).unwrap();
        b.iter(|| {
            for _ in 0..10 {
                black_box(tokenizer.encode(black_box(&text), true).unwrap());
            }
        })
    });
    let lines = read_data_lines("pride_and_prejudice.txt");
    g.bench_function("lines", |b| {
        let tokenizer =
            Kitoken::from_tokenizers_file(bench_models_path().join(MODEL_PATH)).unwrap();
        b.iter(|| {
            for _ in 0..10 {
                for line in &lines {
                    black_box(tokenizer.encode(black_box(line), true).unwrap());
                }
            }
        })
    });
    g.finish();
}

fn bench_encode_utf8_sequence_0x10ffff(b: &mut Criterion) {
    init_env();
    let text = read_data_file("utf8_sequence_0x10ffff.txt");
    let mut g = b.benchmark_group("gpt2: encode utf8_sequence_0x10ffff");
    g.sampling_mode(criterion::SamplingMode::Flat);
    g.bench_function("full", |b| {
        let tokenizer =
            Kitoken::from_tokenizers_file(bench_models_path().join(MODEL_PATH)).unwrap();
        b.iter(|| {
            for _ in 0..10 {
                black_box(tokenizer.encode(black_box(&text), true).unwrap());
            }
        })
    });
    g.finish();
}

fn bench_encode_wagahai(b: &mut Criterion) {
    init_env();
    let text = read_data_file("wagahai.txt");
    let mut g = b.benchmark_group("gpt2: encode wagahai");
    g.sampling_mode(criterion::SamplingMode::Flat);
    g.bench_function("full", |b| {
        let tokenizer =
            Kitoken::from_tokenizers_file(bench_models_path().join(MODEL_PATH)).unwrap();
        b.iter(|| {
            for _ in 0..10 {
                black_box(tokenizer.encode(black_box(&text), true).unwrap());
            }
        })
    });
    let lines = read_data_lines("wagahai.txt");
    g.bench_function("lines", |b| {
        let tokenizer =
            Kitoken::from_tokenizers_file(bench_models_path().join(MODEL_PATH)).unwrap();
        b.iter(|| {
            for _ in 0..10 {
                for line in &lines {
                    black_box(tokenizer.encode(black_box(line), true).unwrap());
                }
            }
        })
    });
    g.finish();
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
        .measurement_time(Duration::from_secs(10))
        .sample_size(10);
    targets = bench_encode_pride_and_prejudice, bench_encode_utf8_sequence_0x10ffff, bench_encode_wagahai
}
criterion_main!(encode, convert);
