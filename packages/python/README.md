# kitoken

**Tokenizer for language models.**

```py
from kitoken import Kitoken

encoder = Kitoken.from_file("models/llama3.3.model")

tokens = encoder.encode("hello world!", True)
string = encoder.decode(tokens).decode("utf-8")

assert string == "hello world!"
```

## Features

- **Fast encoding and decoding**\
  Faster than most other tokenizers in both common and uncommon scenarios.
- **Support for a wide variety of tokenizer formats and tokenization strategies**\
  Including support for Tokenizers, SentencePiece, Tiktoken and more.
- **Compatible with many systems and platforms**\
  Runs on Windows, Linux, macOS and embedded, and comes with bindings for Web, Node and Python.
- **Compact data format**\
  Definitions are stored in an efficient binary format and without merge list.
- **Support for normalization and pre-tokenization**\
  Including unicode normalization, whitespace normalization, and many others.

## Overview

Kitoken is a fast and versatile tokenizer for language models with support for multiple tokenization algorithms:

- **BytePair**: A variation of the BPE algorithm, merging byte or character pairs.
- **Unigram**: The Unigram subword algorithm.
- **WordPiece**: The WordPiece subword algorithm.

Kitoken is compatible with many existing tokenizers,
including [SentencePiece](https://github.com/google/sentencepiece), [HuggingFace Tokenizers](https://github.com/huggingface/tokenizers), [OpenAI Tiktoken](https://github.com/openai/tiktoken) and [Mistral Tekken](https://docs.mistral.ai/guides/tokenization).

See the main [README](//github.com/Systemcluster/kitoken) for more information.
