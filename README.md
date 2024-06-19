# kitoken

**Tokenizer for language models.**

```rust
use kitoken::Kitoken;
let encoder = Kitoken::from_file("models/llama2.kit")?;

let tokens = encoder.encode("Your future belongs to me.", true)?;
let string = String::from_utf8(encoder.decode(&tokens, true)?)?;

assert!(string == "Your future belongs to me.");
```

## Features

- **Fast encoding and decoding**\
  Faster than most other tokenizers in both common and uncommon scenarios.
- **Support for a wide variety of tokenizer formats and tokenization strategies**\
  Including support for Tokenizers, SentencePiece, Tiktoken and more.
- **Compact definition format**\
  Definitions are stored in an efficient binary format and without merge list.
- **Support for normalization and pre-tokenization**\
  Including unicode normalization, whitespace normalization, and many others.

## Overview

Kitoken is a fast and versatile tokenizer for language models with support for BPE and Unigram tokenization.

Multiple tokenization strategies are supported:

- **BytePair**: A variation of the original BPE algorithm that merges inputs starting from individual bytes.
- **CharPair**: A variation of the modified BPE algorithm that merges inputs starting from individual characters.
- **Unigram**: The Unigram subword algorithm.

Kitoken is [compatible](#compatibility) with many existing tokenizer formats,
including [SentencePiece](https://github.com/google/sentencepiece), [HuggingFace Tokenizers](https://github.com/huggingface/tokenizers) and [OpenAI Tiktoken](https://github.com/openai/tiktoken),
while outperforming them in many scenarios. See the [benchmarks](#benchmarks) for comparisons with different datasets.

## Performance

Kitoken uses merge-list-free variations of the BPE algorithm and a reversed variation of the Unigram algorithm. The basis for the merge-list-free BPE algorithm was inspired by [Tiktoken](https://github.com/openai/tiktoken), which has similarly good performance characteristics with common tokenization inputs. However, Kitoken can be much faster with inputs that fail to split during pre-tokenization by falling back to a priority-queue-based implementation when optimal.

Kitoken also avoids memory allocations and copying of data to great extent. Most operations are performed in-place and buffers are reused where possible.

### Benchmarks

Here be graphs.

#### Datasets

##### Pride and Prejudice

A text document containing "Pride and Prejudice" by Jane Austen. This dataset is a good representation for common English-language inputs containing a mix of short and long paragraphs.

##### UTF-8 Sequence

A text document containing a single-line UTF-8 sequence. This dataset is a good representation of inputs that might fail to split during pre-tokenization.

##### Wagahai

A text document containing "Wagahai wa Neko de Aru" by Natsume Sōseki. This dataset is a good representation for Japanese-language inputs containing many long paragraphs.

## Compatibility

Kitoken can convert and initialize with many existing tokenizer formats. Every supported format is [tested](./tests) against the original implementation across a variety of inputs to ensure compatibility and correctness.

### SentencePiece

```rust
let encoder = Kitoken::from_sentencepiece_file("models/mistral.model")?;
```

Kitoken can convert and initialize with SentencePiece models in `BPE` and `Unigram` format.

- `BPE` models are converted to `CharPair` definitions. A merge list is generated and sorted using the token scores, which is then used to sort the vocabulary by merge priority. The scores and the merge list are then discarded.
- `Unigram` models are converted to `Unigram` definitions retaining the token scores.

If the model does not contain a trainer definition, `Unigram` will be used as the default encoding mode.

Normalization options and the unicode normalization scheme are taken from the contained normalizer definition.

Kitoken differs from SentencePiece in the tokenization of one character:

- SentencePiece uses [different `nfkc` normalization rules in the `nmt_nfkc` and `nmt_nfkc_cf` schemes](https://github.com/google/sentencepiece/blob/master/doc/normalization.md) than during regular `nfkc` normalization. This difference is not entirely additive and prevents the normalization of `～` to `~`. Kitoken uses the regular `nfkc` normalization rules for `nmt_nfkc` and `nmt_nfkc_cf` and normalizes `～` to `~`.

### Tokenizers

```rust
let encoder = Kitoken::from_tokenizers_file("models/llama2.json")?;
```

Kitoken can convert and initialize with HuggingFace Tokenizers definitions in `BPE` and `Unigram` format.

- `BPE` models are converted to `CharPair` or `BytePair` definitions. The included merge list is used to sort the vocabulary by merge priority and is then discarded.
- `Unigram` models are converted to `Unigram` definitions retaining the token scores.

Normalization, pre-tokenization, post-processing and decoding options contained in the definition are converted to the respective Kitoken configurations. Kitoken has full compatibility with almost all available Tokenizers options, except for the following subset:

- `WordPiece` decoding is used only for `WordPiece` models and is not implemented.
- `UnicodeScripts` pre-tokenization is not implemented.
- `Replace` decoding with regex patterns is not supported as Kitoken allows non-unicode-compatible decoding output.
- `TemplateProcessing`, `RobertaProcessing` and `Bert` post-processing is ignored as Kitoken does not support output templates.

Some normalization, post-processing and decoding options used by Tokenizers are used for converting alternative token-byte representations during encoding and decoding. Kitoken always stores and operates on tokens as byte sequences, and will use these options to pre-normalize the vocabulary during conversion.

### Tiktoken

```rust
let encoder = Kitoken::from_tiktoken_file("models/cl100k_base.tiktoken")?;
```

Tiktoken is a `BPE` tokenizer with a custom definition format used by OpenAI for GPT-3 and newer models.

Tiktoken definitions contain a sorted vocabulary of base64 encoded bytes and corresponding token ids without any additional metadata. Special tokens and the split regex are expected to be provided separately, but will be inferred from the data for common models including GPT-3, GPT-4 and GPT-4o.
For other models, or depending on the data and requirements, these values can be adjusted manually.

Tiktoken definitions are converted to `BytePair` definitions.
