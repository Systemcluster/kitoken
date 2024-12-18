# kitoken

**Tokenizer for language models.**

```js
import { Kitoken } from "kitoken/node"

const model = fs.readFileSync("models/llama3.3.model")
const encoder = new Kitoken(model)

const tokens = encoder.encode("hello world!", true)
const string = TextDecoder().decode(encoder.decode(tokens))
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

The JavaScript package provides multiple exports:

| Export          | Description                                                                                           |
|-----------------|-------------------------------------------------------------------------------------------------------|
| `kitoken`       | The default export, importing the WebAssembly file directly. Usable with Webpack and other bundlers.  |
| `kitoken/node`  | Uses Node.js functions to read the WebAssembly file from the file system. Provides support for additional split strategies and regex optimizations. |
| `kitoken/web`   | Usable with web browsers, uses `new URL(..., import.meta.url)` to load the WebAssembly file.           |
| `kitoken/minimal`| Smallest file size. Similar to the default export, but only supports initialization from `.kit` definitions. |
| `kitoken/full`  | Largest file size. Similar to the default export, but provides support for additional split strategies and regex optimizations. |

See the main [README](//github.com/Systemcluster/kitoken) for more information.
