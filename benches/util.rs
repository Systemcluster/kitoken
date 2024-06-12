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


pub fn bench_models_path() -> std::path::PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/models")
}

pub fn bench_data_path() -> std::path::PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("benches/data")
}

pub fn read_data_lines(path: impl Into<PathBuf>) -> Vec<String> {
    let path = bench_data_path().join(path.into());
    let lines = std::fs::read_to_string(path).unwrap();
    lines
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| line.to_owned().replace("\\n", "\n").replace("\\s", " "))
        .collect()
}
pub fn read_data_file(path: impl Into<PathBuf>) -> String {
    let path = bench_data_path().join(path.into());
    std::fs::read_to_string(path).unwrap()
}
