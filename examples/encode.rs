//! Example for encoding benchmarks.

use std::hint::black_box;
use std::time::Instant;

use kitoken::Kitoken;

mod util;
use util::*;

#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
#[global_allocator]
static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    init_env();

    let model = std::env::var("MODEL").unwrap_or("tokenizers/llama4.json".to_string());
    let dataset = std::env::var("DATASET").unwrap_or("pride_and_prejudice.txt".to_string());
    let iters = std::env::var("ITERS").ok().and_then(|i| i.parse().ok()).unwrap_or(20usize);
    let specials = std::env::var("SPECIALS")
        .map(|s| matches!(s.to_lowercase().as_str(), "1" | "true"))
        .unwrap_or(true);

    let dataset_path = test_data_path().join(&dataset);
    let model_path = test_models_path().join(&model);

    eprintln!("model path:    {}", model_path.display());
    eprintln!("dataset path:    {}", dataset_path.display());


    let text = std::fs::read_to_string(dataset_path).unwrap();

    eprintln!("model:    {}", model);
    eprintln!("dataset:  {} ({}b)", dataset, text.len());
    eprintln!("iters:    {}", iters);
    eprintln!("specials: {}", specials);

    let tokenizer = Kitoken::from_file(model_path).unwrap();

    let start = Instant::now();
    let mut tokens = 0usize;
    for _ in 0..iters {
        for _ in 0..10 {
            let result = tokenizer.encode(black_box(&text), specials).unwrap();
            tokens += result.len();
            black_box(result);
        }
    }
    let elapsed = start.elapsed();

    eprintln!(
        "encoded {} tokens in {:.3?} ({:.0} tokens/s)",
        tokens,
        elapsed,
        tokens as f64 / elapsed.as_secs_f64()
    );
}
