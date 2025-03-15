# kitoken

**Tokenizer for language models.**

<sup>**Tokenize text for Llama, Gemini, GPT-4, Mistral and many others; in the web, on the client and any platform.**</sup>

```py
from kitoken import Kitoken

encoder = Kitoken.from_file("models/llama3.3.model")

tokens = encoder.encode("hello world!", True)
string = encoder.decode(tokens).decode("utf-8")

assert string == "hello world!"
```

## Overview

Kitoken is a fast and versatile tokenizer for language models compatible with [SentencePiece](https://github.com/google/sentencepiece), [HuggingFace Tokenizers](https://github.com/huggingface/tokenizers), [OpenAI Tiktoken](https://github.com/openai/tiktoken) and [Mistral Tekken](https://docs.mistral.ai/guides/tokenization), supporting BPE, Unigram and WordPiece tokenization.

- **Fast and efficient tokenization**\
  Faster than most other tokenizers in both common and uncommon scenarios; see the [benchmarks](//github.com/Systemcluster/kitoken#benchmarks) for comparisons with different datasets.
- **Runs in all environments**\
  Native in Rust and with bindings for Web, Node and Python; see [kitoken.dev](https://kitoken.dev) for a web demo.
- **Support for normalization and pre-tokenization**\
  Including unicode-aware normalization, pre-tokenization and post-processing options.
- **Compact data format**\
  Definitions are stored in an efficient binary format and without merge list.

See the main [README](//github.com/Systemcluster/kitoken) for more information.
