#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(target_arch = "wasm32", not(target_feature = "atomics")))]
#[global_allocator]
static ALLOCATOR: rlsf::SmallGlobalTlsf = rlsf::SmallGlobalTlsf::new();

#[macro_use]
extern crate alloc;

use alloc::rc::Rc;
use alloc::vec::Vec;

use core::fmt::{Debug, Display};

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

    /// Encodes the given texts into sequences of tokens.
    ///
    /// If `encode_specials` is `true`, the text is first split around special tokens which are separately encoded with the special encoder.
    ///
    /// Returns a list of lists of tokens, or an error if no token for a part exists in the encoder and no unknown token id is set in the configuration.
    pub fn encode_all(
        &self, text: Vec<String>, encode_specials: Option<bool>,
    ) -> Result<Vec<JsValue>, JsValue> {
        text.iter()
            .map(|text| self.encode(text, encode_specials))
            .map(|result| result.map(JsValue::from))
            .collect::<Result<_, _>>()
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

    /// Decodes the given sequences of tokens into texts.
    ///
    /// Returns a list of lists of bytes, or an error if no byte sequence for a token exists in the decoder and no unknown token is set in the configuration.
    pub fn decode_all(
        &self, tokens: Vec<JsValue>, decode_specials: Option<bool>,
    ) -> Result<Vec<JsValue>, JsValue> {
        let tokens = tokens
            .into_iter()
            .map(serde_wasm_bindgen::from_value)
            .collect::<Result<Vec<Vec<u32>>, _>>()
            .map_err(|_| JsValue::from_str("expected an array of arrays of tokens"))?;
        let result = tokens
            .into_iter()
            .map(|tokens| self.decode(&tokens, decode_specials))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(result.into_iter().map(JsValue::from).collect())
    }

    /// Returns the definition of the tokenizer.
    #[cfg(feature = "inspect")]
    pub fn definition(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.to_definition()).unwrap()
    }

    /// Sets the definition of the tokenizer.
    ///
    /// Returns an error if the definition is invalid.
    #[cfg(feature = "inspect")]
    pub fn set_definition(&mut self, definition: JsValue) -> Result<(), JsValue> {
        let definition = serde_wasm_bindgen::from_value(definition).map_err(convert_error)?;
        self.inner = Rc::new(Inner::from_definition(definition).map_err(convert_error)?);
        Ok(())
    }

    /// Returns the configuration of the tokenizer.
    #[cfg(feature = "inspect")]
    pub fn config(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.to_definition().config).unwrap()
    }

    /// Sets the configuration of the tokenizer.
    ///
    /// Returns an error if the configuration is invalid.
    #[cfg(feature = "inspect")]
    pub fn set_config(&mut self, config: JsValue) -> Result<(), JsValue> {
        let mut definition = self.inner.to_definition();
        definition.config = serde_wasm_bindgen::from_value(config).map_err(convert_error)?;
        self.inner = Rc::new(Inner::from_definition(definition).map_err(convert_error)?);
        Ok(())
    }

    /// Creates a definition from this tokenizer and serializes it to bytes.
    #[cfg(feature = "convert")]
    pub fn to_bytes(&self) -> Vec<u8> {
        self.inner.to_vec()
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
fn convert_error(e: impl Display) -> JsValue {
    JsValue::from_str(&format!("{}", e))
}
