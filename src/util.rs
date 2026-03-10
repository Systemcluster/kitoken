use alloc::string::{String, ToString};

pub fn parse_url(url: &str) -> String {
    if url.starts_with("hf:") {
        let repo = url.strip_prefix("hf:").unwrap();
        [
            "https://huggingface.co",
            repo,
            "resolve/main/tokenizer.json",
        ]
        .join("/")
        .to_string()
    } else {
        url.to_string()
    }
}
