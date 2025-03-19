# kitoken

**Tokenizer for language models.**

<sup>**Tokenize text for Llama, Gemini, GPT-4, Mistral and many others; in the web, on the client and any platform.**</sup>

```js
import { Kitoken } from "kitoken/node"

const model = fs.readFileSync("models/llama3.3.model")
const encoder = new Kitoken(model)

const tokens = encoder.encode("hello world!", true)
const string = TextDecoder().decode(encoder.decode(tokens))
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

## Usage

The JavaScript package provides multiple exports:

| Export            | Description                                                                                           |
|-------------------|-------------------------------------------------------------------------------------------------------|
| `kitoken`         | The default export, importing the WebAssembly file directly. Usable with Webpack and other bundlers.  |
| `kitoken/node`    | Uses Node.js functions to read the WebAssembly file from the file system. Provides support for additional split strategies and regex optimizations. |
| `kitoken/web`     | Can be used in web browsers without a bundler, uses `new URL(..., import.meta.url)` to load the WebAssembly file. |
| `kitoken/minimal` | Smallest file size. Similar to the default export, but only supports initialization from `.kit` definitions. |
| `kitoken/full`    | Largest file size. Similar to the default export, but provides support for additional split strategies and regex optimizations. |

See also the [Node test](./test.js) and the [Web example](./examples/web.html).
