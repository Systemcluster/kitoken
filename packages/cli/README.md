# kitoken-cli

[![Crates.io](https://img.shields.io/crates/v/kitoken)](https://crates.io/crates/kitoken)
[![NPM](https://img.shields.io/npm/v/kitoken)](https://www.npmjs.com/package/kitoken)
[![PyPI](https://img.shields.io/pypi/v/kitoken)](https://pypi.org/project/kitoken)
[![Tests & Checks](https://img.shields.io/github/actions/workflow/status/Systemcluster/kitoken/tests.yml?label=tests%20%26%20checks)](https://github.com/Systemcluster/kitoken/actions/workflows/tests.yml)

**Tokenizer for language models.**

<sup>**Tokenize text for Llama, Gemini, GPT-4, DeepSeek, Mistral and many others in the command line.**</sup>

```bash
# Encode
kitoken encode deepseek.kit ./texts.txt
# Decode
kitoken decode deepseek.kit ./tokens.txt
```

```bash
# Compare
kitoken compare llama4.json llama4.model
# Convert
kitoken convert llama4.model
# Inspect
kitoken inspect llama4.kit
```

Install `kitoken-cli` with Cargo:

```bash
cargo install --git https://github.com/Systemcluster/kitoken
```

See the main [README](//github.com/Systemcluster/kitoken) for more information.
