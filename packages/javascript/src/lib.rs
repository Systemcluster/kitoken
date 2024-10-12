#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(target_arch = "wasm32", not(target_feature = "atomics")))]
#[global_allocator]
static ALLOCATOR: rlsf::SmallGlobalTlsf = rlsf::SmallGlobalTlsf::new();

#[macro_use]
extern crate alloc;

use alloc::rc::Rc;
use alloc::vec::Vec;

use core::fmt::Debug;

use wasm_bindgen::prelude::*;

use ::kitoken::Kitoken as Inner;

#[wasm_bindgen]
#[derive(Debug, Clone)]
/// Kitoken tokenizer.
/// A fast and versatile tokenizer for language models.
pub struct Kitoken {
    inner: Rc<Inner>,
}
#[wasm_bindgen]
#[allow(non_snake_case)]
impl Kitoken {
    #[wasm_bindgen(constructor)]
    /// Initializes the tokenizer from a serialized `kitoken` definition.
    pub fn new(data: &[u8]) -> Result<Kitoken, JsValue> {
        Ok(Kitoken {
            inner: Rc::new(Inner::from_slice(data).map_err(convert_error)?),
        })
    }

    /// Encodes the given text into a sequence of tokens.
    ///
    /// If `encode_specials` is `true`, the text is first split around special tokens which are separately encoded with the special encoder.
    ///
    /// Returns a list of tokens, or an error if no token for a part exists in the encoder and no unknown token id is set in the configuration.
    pub fn encode(&self, text: &str, encode_specials: Option<bool>) -> Result<Vec<u32>, JsValue> {
        self.inner.encode(text, encode_specials.unwrap_or(false)).map_err(convert_error)
    }

    /// Decodes the given sequence of tokens into text.
    ///
    /// Returns a list of bytes, or an error if no byte sequence for a token exists in the decoder and no unknown token is set in the configuration.
    pub fn decode(
        &self, tokens: &[u32], decode_specials: Option<bool>,
    ) -> Result<Vec<u8>, JsValue> {
        self.inner
            .decode(tokens, decode_specials.unwrap_or(false))
            .map_err(convert_error)
    }

    /// Initializes the tokenizer from a serialized `sentencepiece` model.
    #[cfg(feature = "convert")]
    pub fn from_sentencepiece(data: &[u8]) -> Result<Kitoken, JsValue> {
        Ok(Kitoken {
            inner: Rc::new(Inner::from_sentencepiece_slice(data).map_err(convert_error)?),
        })
    }

    /// Initializes the tokenizer from a serialized `tiktoken` model.
    #[cfg(feature = "convert")]
    pub fn from_tiktoken(data: &[u8]) -> Result<Kitoken, JsValue> {
        Ok(Kitoken {
            inner: Rc::new(Inner::from_tiktoken_slice(data).map_err(convert_error)?),
        })
    }

    /// Initializes the tokenizer from a serialized `tokenizers` model.
    #[cfg(feature = "convert")]
    pub fn from_tokenizers(data: &[u8]) -> Result<Kitoken, JsValue> {
        Ok(Kitoken {
            inner: Rc::new(Inner::from_tokenizers_slice(data).map_err(convert_error)?),
        })
    }

    /// Initializes the tokenizer from a serialized `tokenizers` model.
    #[cfg(feature = "convert")]
    pub fn from_tekken(data: &[u8]) -> Result<Kitoken, JsValue> {
        Ok(Kitoken {
            inner: Rc::new(Inner::from_tekken_slice(data).map_err(convert_error)?),
        })
    }
}

#[inline(never)]
fn convert_error(e: impl Debug) -> JsValue {
    JsValue::from_str(&format!("{:?}", e))
}
