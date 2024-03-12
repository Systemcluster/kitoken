#[macro_use]
extern crate alloc;

use wasm_bindgen::prelude::*;

use ::kitoken::Kitoken as Inner;

#[wasm_bindgen]
#[derive(Debug)]
/// Kitoken tokenizer.
/// A fast and versatile tokenizer for language models.
pub struct Kitoken {
    inner: Inner,
}
#[wasm_bindgen]
impl Kitoken {
    #[wasm_bindgen(constructor)]
    /// Initializes the tokenizer from a serialized `kitoken` model.
    pub fn new(data: &[u8]) -> Result<Kitoken, JsValue> {
        Ok(Kitoken {
            inner: Inner::from_slice(data).map_err(|e| JsValue::from_str(&format!("{:?}", e)))?,
        })
    }

    /// Encodes the given text into a sequence of tokens.
    ///
    /// If `encode_specials` is `true`, the text is first split around special tokens which are separately encoded with the special encoder.
    ///
    /// Returns a list of tokens, or an error if no token for a part exists in the encoder and no unknown token id is set in the configuration.
    pub fn encode(&self, text: &str, encode_specials: Option<bool>) -> Result<Vec<u32>, JsValue> {
        self.inner
            .encode(text, encode_specials.unwrap_or(false))
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))
    }

    /// Decodes the given sequence of tokens into text.
    ///
    /// Returns a list of bytes, or an error if no byte sequence for a token exists in the decoder and no unknown token is set in the configuration.
    pub fn decode(&self, tokens: &[u32]) -> Result<Vec<u8>, JsValue> {
        self.inner.decode(tokens).map_err(|e| JsValue::from_str(&format!("{:?}", e)))
    }

    /// Initializes the tokenizer from a serialized `sentencepiece` model.
    pub fn from_sentencepiece(data: &[u8]) -> Result<Kitoken, JsValue> {
        Ok(Kitoken {
            inner: Inner::from_sentencepiece_slice(data)
                .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?,
        })
    }

    /// Initializes the tokenizer from a serialized `tiktoken` model.
    pub fn from_tiktoken(data: &[u8]) -> Result<Kitoken, JsValue> {
        Ok(Kitoken {
            inner: Inner::from_tiktoken_slice(data)
                .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?,
        })
    }
}
