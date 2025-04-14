# kitoken

[![Crates.io](https://img.shields.io/crates/v/kitoken)](https://crates.io/crates/kitoken)
[![NPM](https://img.shields.io/npm/v/kitoken)](https://www.npmjs.com/package/kitoken)
[![PyPI](https://img.shields.io/pypi/v/kitoken)](https://pypi.org/project/kitoken)
[![Tests & Checks](https://img.shields.io/github/actions/workflow/status/Systemcluster/kitoken/tests.yml?label=tests%20%26%20checks)](https://github.com/Systemcluster/kitoken/actions/workflows/tests.yml)

**Tokenizer for language models.**

<sup>**Tokenize text for Llama, Gemini, GPT-4, DeepSeek, Mistral and many others; in the web, on the client and any platform.**</sup>

```py
from kitoken import Kitoken

encoder = Kitoken.from_file("models/llama4.model")

tokens = encoder.encode("hello world!", True)
string = encoder.decode(tokens).decode("utf-8")

assert string == "hello world!"
```

## Overview

Kitoken is a fast and versatile tokenizer for language models compatible with [SentencePiece](https://github.com/google/sentencepiece), [HuggingFace Tokenizers](https://github.com/huggingface/tokenizers), [OpenAI Tiktoken](https://github.com/openai/tiktoken) and [Mistral Tekken](https://docs.mistral.ai/guides/tokenization), supporting BPE, Unigram and WordPiece tokenization.

- **Fast and efficient tokenization**\
  Faster than most other tokenizers in both common and uncommon scenarios; see the [benchmarks](//github.com/Systemcluster/kitoken#benchmarks) for comparisons with different datasets.
- **Runs in all environments**\
  Native in Rust and with bindings for [Web](./packages/javascript), [Node](./packages/javascript) and [Python](./packages/python); see [kitoken.dev](https://kitoken.dev) for a web demo.
- **Supports input and output processing**\
  Including unicode-aware normalization, pre-tokenization and post-decoding options.
- **Compact data format**\
  Definitions are stored in an efficient binary format and without merge list.

See the main [README](//github.com/Systemcluster/kitoken) for more information.
