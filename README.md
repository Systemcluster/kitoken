# kitoken

**Tokenizer for language models.**

Supports BPE and Unigram tokenization. Usable in native and WASM environments.

## Features

- **Fast encoding and decoding of text and tokens**\
  Faster than many other tokenizers in both common and uncommon scenarios.
- **Support for a wide variety of commonly used tokenizer formats and strategies**\
  Including support for SentencePiece, Tiktoken and more.
- **Compact definition format**\
  Definitions are stored in an efficient binary format and without merge list.
- **Support for normalization and pre-tokenization**\
  Including NFKC normalization, whitespace normalization, and others.

## Overview

Kitoken is a fast and versatile tokenizer for language models with support for BPE and Unigram tokenization.

Kitoken is compatible with many existing tokenizer formats,
including [SentencePiece](https://github.com/google/sentencepiece) and [tiktoken](https://github.com/openai/tiktoken),
while outperforming them in many scenarios. See [benchmarks](#benchmarks) for comparisons with different datasets.

## Usage

### Loading a Kitoken definition

```rust
use kitoken::Kitoken;
let tokenizer = Kitoken::from_file("tests/models/llama2.kit")?;
```

### Loading a SentencePiece model

```rust
use kitoken::Kitoken;
let tokenizer = Kitoken::from_sentencepiece_file("tests/models/llama2.model")?;
```

### Encoding and decoding

```rust
let tokens = tokenizer.encode("Your future belongs to me <|endoftext|>", true)?;
let text = String::from_utf8(tokenizer.decode(&tokens)?)?;

assert_eq!(text, "Your future belongs to me <|endoftext|>");
```

## Benchmarks

Here be graphs.

### Datasets

#### Pride and Prejudice

A text document containing "Pride and Prejudice" by Jane Austen. This dataset is a good representation for common English-language inputs containing a mix of short and long paragraphs.

#### Wagahai wa Neko de Aru

A text document "Wagahai wa Neko de Aru" by Natsume Soseki. This dataset is a good representation for Japanese-language inputs containing many long paragraphs.

#### UTF-8 sequence

A text document containing a single UTF-8 sequence. This is a worst-case scenario dataset designed to stress-test the behavior of tokenizers with inputs that fail to split during pre-tokenization.

## Details

### Motivation

At the time of writing, there is no other tokenizer that is compatible with various tokenizer formats and tokenization algorithms while also being fast, usable in both native and WASM environments, and having a small binary and definition size.
There is also no common method or toolset for converting different tokenizer definitions.

Kitoken was created to fulfill all of these requirements and more.

### Compatibility

Kitoken can convert and initialize with many existing tokenizer formats. Every supported format is [tested](./tests) against the original implementation across a variety of inputs to ensure compatibility and correctness.

#### SentencePiece

Kitoken can convert and initialize with SentencePiece models in `BPE` and `Unigram` format.

- `BPE` models are converted to `CharPair` definitions. A merge list is generated and sorted using the token scores, which is then used to sort the vocabulary by merge priority. The scores and the merge list are then discarded.
- `Unigram` models are converted to `Unigram` definitions retaining the token scores.

If the model does not contain a trainer definition, `Unigram` will be used as the default encoding mode.

Normalization options and the unicode normalization scheme are taken from the contained normalizer definition.

Kitoken differs from SentencePiece in the tokenization of two characters:

- SentencePiece uses the `▁` character to indicate whitespace, which means that during encoding, SentencePiece treats `▁` and space as the same character. During conversion to a Kitoken definition, `▁` is replaced with a regular space in the vocabulary. This means that Kitoken treats `▁` and space as different characters during encoding.
- SentencePiece uses [different `nfkc` normalization rules in the `nmt_nfkc` and `nmt_nfkc_cf` schemes](https://github.com/google/sentencepiece/blob/master/doc/normalization.md) than during regular `nfkc` normalization. This difference is not entirely additive and prevents the normalization of `～` to `~`. Kitoken uses the regular `nfkc` normalization rules for `nmt_nfkc` and `nmt_nfkc_cf` and normalizes `～` to `~`.

#### Tiktoken

Tiktoken is a `BPE` tokenizer with a custom definition format used by OpenAI for GPT-3 and newer models.

Tiktoken definitions contain a sorted vocabulary of base64 encoded bytes and corresponding token ids without any additional metadata. Special tokens and the split regex are expected to be provided separately. Kitoken will infer defaults for both from the vocabulary, but depending on the data and requirements, these values may have to be adjusted manually.

Tiktoken definitions are converted to `BytePair` definitions.

### Performance

Kitoken avoids memory allocations and copying of data to great extent. Most operations are performed in-place and buffers are reused where possible.
Additionally Kitoken selects between multiple merge strategies depending on the input, which can greatly improve performance in different scenarios.

The basis for the merge-list-free `BytePair` merge algorithm was inspired by [tiktoken](https://github.com/openai/tiktoken), and the performance characteristics on common tokenization inputs, when using the same tiktoken definition data, are therefore similar. However, Kitoken can be much faster in many less common scenarios due to selective merge strategies, especially with inputs that are split into fewer but larger parts during pre-tokenization. See the [utf-8 sequence benchmark](#benchmarks) for a worst-case example.

### Definitions

Kitoken definitions contain the following data:

| Definition     | Description                                                                 |
| -------------- | --------------------------------------------------------------------------- |
| **Vocab**      | The encoder vocabulary without special tokens. A list of tokens and their id. Tokens are stored as bytes and can contain partial or invalid UTF-8. Sorted by merge priority. |
| **Specials**   | The special encoder vocabulary. A list of special token bytes and their id. Special tokens are stored as bytes but must contain valid UTF-8. Prioritized over the vocabulary during encoding and decoding. Sorted by split priority. |
| **Scores**     | A list of token ids and their scores. Optional and only used for the `Unigram` encoding mode. Must be the same length as the vocabulary and must contain scores for all tokens if present. |
| **Config**     | The tokenizer configuration. Contains the split regex, encoding mode, normalization options and other settings. |

Kitoken definitions don't contain a merge list. Instead, the vocabulary is sorted by merge priority. This allows use of more efficient merge strategies and reduces the size of the definition files.

Definitions are stored using the [Postcard Wire Format](https://postcard.jamesmunns.com/wire-format).

#### Configuration

The tokenizer configuration contains the following data:

| Configuration     | Description                                                                 |
| ----------------- | --------------------------------------------------------------------------- |
| **Split Regex**   | A regex used to split the input into parts during pre-tokenization. |
| **Encoding Mode** | The encoding mode used by the tokenizer. |
| **Normalization** | Normalization configuration for encoding and decoding. |
| **Special Tokens** | Optional tokens and token ids for `unk`, `pad`, `bos`, `eos`, `sep` and `mask`. |

#### Encoding Modes

Kitoken supports multiple encoding modes:

| Mode            | Description                                                                 |
| --------------- | --------------------------------------------------------------------------- |
| **BytePair**    | The original BPE algorithm, which merges pairs of bytes. |
| **CharPair**    | A variant of the BPE algorithm, which merges pairs of characters first before falling back to `BytePair`. |
| **Unigram**     | The Unigram subword algorithm. |
