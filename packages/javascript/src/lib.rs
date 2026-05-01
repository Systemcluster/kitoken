#![no_std]

#[cfg(feature = "std")]
extern crate std;

#[cfg(all(target_arch = "wasm32", not(target_feature = "atomics")))]
#[global_allocator]
static ALLOCATOR: rlsf::SmallGlobalTlsf = rlsf::SmallGlobalTlsf::new();

#[macro_use]
extern crate alloc;

use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;

use either::Either;
use kitoken::SpecialTokenKind;
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
    /// `encode_specials` specifies which special tokens are tokenized with the special vocabulary instead of the regular vocabulary.
    /// Accepted are arrays of strings "control", "priority", "unknown", and boolean values `true` and `false`.
    /// When `true`, all special token categories from the special vocabulary are used.
    ///
    /// Returns a list of tokens, or an error if no token for a part exists in the encoder and no unknown token id is set in the configuration.
    pub fn encode(&self, text: &str, encode_specials: JsValue) -> Result<Vec<u32>, JsValue> {
        let specials = convert_special_kinds(encode_specials)?;
        match specials {
            Either::Left(b) => self.inner.encode(text, b),
            Either::Right(v) => self.inner.encode(text, v),
        }
        .map_err(convert_error)
    }

    /// Encodes the given texts into sequences of tokens.
    ///
    /// `encode_specials` specifies which special tokens are tokenized with the special vocabulary instead of the regular vocabulary.
    /// Accepted are arrays of strings "control", "priority", "unknown", and boolean values `true` and `false`.
    /// When `true`, all special token categories from the special vocabulary are used.
    ///
    /// Returns a list of lists of tokens, or an error if no token for a part exists in the encoder and no unknown token id is set in the configuration.
    pub fn encode_all(
        &self, text: Vec<String>, encode_specials: JsValue,
    ) -> Result<Vec<JsValue>, JsValue> {
        let specials = convert_special_kinds(encode_specials)?;
        match specials {
            Either::Left(b) => text
                .iter()
                .map(|text| self.inner.encode(text, b))
                .map(|result| result.map(JsValue::from))
                .collect::<Result<_, _>>(),
            Either::Right(v) => text
                .iter()
                .map(|text| self.inner.encode(text, v.as_slice()))
                .map(|result| result.map(JsValue::from))
                .collect::<Result<_, _>>(),
        }
        .map_err(convert_error)
    }

    /// Decodes the given sequence of tokens into text.
    ///
    /// `decode_specials` specifies which tokens from the special vocabulary are included in the output.
    /// Accepted are arrays of strings "control", "priority", "unknown", and boolean values `true` and `false`.
    ///
    /// Returns a list of bytes, or an error if no byte sequence for a token exists in the decoder and no unknown token is set in the configuration.
    pub fn decode(&self, tokens: &[u32], decode_specials: JsValue) -> Result<Vec<u8>, JsValue> {
        let specials = convert_special_kinds(decode_specials)?;
        match specials {
            Either::Left(b) => self.inner.decode(tokens, b),
            Either::Right(v) => self.inner.decode(tokens, v),
        }
        .map_err(convert_error)
    }

    /// Decodes the given sequences of tokens into texts.
    ///
    /// `decode_specials` specifies which tokens from the special vocabulary are included in the output.
    /// Accepted are arrays of strings "control", "priority", "unknown", and boolean values `true` and `false`.
    ///
    /// Returns a list of lists of bytes, or an error if no byte sequence for a token exists in the decoder and no unknown token is set in the configuration.
    pub fn decode_all(
        &self, tokens: Vec<JsValue>, decode_specials: JsValue,
    ) -> Result<Vec<JsValue>, JsValue> {
        let tokens = tokens
            .into_iter()
            .map(serde_wasm_bindgen::from_value)
            .collect::<Result<Vec<Vec<u32>>, _>>()
            .map_err(|_| JsValue::from_str("expected an array of arrays of tokens"))?;
        let specials = convert_special_kinds(decode_specials)?;
        match specials {
            Either::Left(b) => tokens
                .iter()
                .map(|tokens| self.inner.decode(tokens, b))
                .map(|result| result.map(JsValue::from))
                .collect::<Result<Vec<_>, _>>(),
            Either::Right(v) => tokens
                .iter()
                .map(|tokens| self.inner.decode(tokens, v.as_slice()))
                .map(|result| result.map(JsValue::from))
                .collect::<Result<Vec<_>, _>>(),
        }
        .map_err(convert_error)
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
    pub fn config(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.config()).unwrap()
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

    /// Returns the metadata of the tokenizer.
    pub fn meta(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.meta()).unwrap()
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
fn convert_error(e: impl core::fmt::Display) -> JsValue {
    JsValue::from_str(&format!("{}", e))
}

#[inline(never)]
fn convert_special_kinds(s: JsValue) -> Result<Either<bool, Vec<SpecialTokenKind>>, JsValue> {
    if s.is_array() {
        let v = serde_wasm_bindgen::from_value::<Vec<String>>(s)
            .map_err(convert_error)?
            .into_iter()
            .map(|s| match s.as_str() {
                "control" => Ok(SpecialTokenKind::Control),
                "priority" => Ok(SpecialTokenKind::Priority),
                "unknown" => Ok(SpecialTokenKind::Unknown),
                _ => Err("invalid special token kind"),
            })
            .collect::<Result<Vec<_>, &'static str>>()?;
        Ok(Either::Right(v))
    } else {
        if s.is_null_or_undefined() {
            Ok(Either::Left(false))
        } else {
            serde_wasm_bindgen::from_value::<bool>(s)
                .map(Either::Left)
                .map_err(convert_error)
        }
    }
}
