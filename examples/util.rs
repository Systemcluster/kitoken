#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::Once;

static INIT_ENV: Once = Once::new();

pub fn init_env() {
    INIT_ENV.call_once(|| {
        simple_logger::SimpleLogger::new()
            .with_level(log::Level::Info.to_level_filter())
            .env()
            .init()
            .unwrap();
    });
}

pub fn test_models_path() -> std::path::PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/models")
}

pub fn test_data_path() -> std::path::PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("benches/data")
}
