//! Utilities and depinitions for normalizing inputs and outputs.

use alloc::string::String;
use alloc::vec::Vec;

use bstr::ByteSlice;

use crate::UnicodeNormalization;

/// Normalizes text according to the NMT normalization scheme defined by `sentencepiece`.
#[cfg(feature = "unicode-normalization")]
#[allow(unused_imports)]
fn nmt(mut text: String) -> String {
    use crate::regex::Regex;
    use alloc::borrow::ToOwned;
    use alloc::boxed::Box;
    use once_cell::race::OnceBox;
    text.retain(|c| !matches!(c, '\u{1}'..='\u{8}' | '\u{e}'..='\u{1f}' | '\u{b}' | '\u{7f}' | '\u{8f}' | '\u{9f}'));
    static NMT_REGEX_SPACE: OnceBox<Regex> = OnceBox::new();
    let replacer_space =
        NMT_REGEX_SPACE.get_or_init(|| Box::new(Regex::new(r"[\u{0}\u{a}\u{c}\u{d}\u{1680}\u{200B}-\u{200F}\u{2028}\u{2029}\u{2581}\u{feff}\u{fffd}]").unwrap()));
    let mut text = text.to_owned();
    text = replacer_space.replace_all(&text, " ");
    text
}

/// Normalizes text according to the given normalization scheme.
#[cfg(feature = "unicode-normalization")]
pub(crate) fn normalize_unicode_in_place(
    text: &mut String, normalization: UnicodeNormalization,
) -> Option<()> {
    use unicode_normalization::UnicodeNormalization as _;
    let normalized = match normalization {
        UnicodeNormalization::None => return Some(()),
        UnicodeNormalization::NFC => text.nfc().collect::<String>(),
        UnicodeNormalization::NFKC => text.nfkc().collect::<String>(),
        UnicodeNormalization::NFD => text.nfd().collect::<String>(),
        UnicodeNormalization::NFKD => text.nfkd().collect::<String>(),
        UnicodeNormalization::NFKCCF => text.nfkc().collect::<String>().to_lowercase(),
        UnicodeNormalization::NFKCNMT => nmt(text.nfkc().collect::<String>()),
        UnicodeNormalization::NFKCNMTCF => nmt(text.nfkc().collect::<String>()).to_lowercase(),
    };
    *text = normalized;
    Some(())
}
/// Normalizes text according to the given normalization scheme.
#[cfg(not(feature = "unicode-normalization"))]
pub(crate) fn normalize_unicode_in_place(
    _text: &mut String, normalization: UnicodeNormalization,
) -> Option<()> {
    match normalization {
        UnicodeNormalization::None => Some(()),
        _ => None,
    }
}

/// Collapses whitespace in the given text.
pub(crate) fn collapse_whitespace_in_place(text: &mut String) {
    let mut last = None;
    text.retain(|c| {
        let keep = last != Some(c) || !c.is_whitespace();
        last = Some(c);
        keep
    });
}

/// Trims whitespace from the beginning and end of the given text.
pub(crate) fn trim_whitespace_in_place(data: &mut Vec<u8>, until: Option<&[u8]>) {
    let mut slice_start = 0;
    let mut slice_end = 0;
    for c in data[..].chars() {
        let starts_with_until = until
            .as_ref()
            .map(|until| data[slice_start..].starts_with(until))
            .unwrap_or(false);
        if !starts_with_until && c.is_whitespace() {
            slice_start += c.len_utf8();
        } else {
            break;
        }
    }
    for c in data[slice_start..].chars().rev() {
        let ends_with_until = until
            .as_ref()
            .map(|until| data[..data.len() - slice_end].ends_with(until))
            .unwrap_or(false);
        if !ends_with_until && c.is_whitespace() {
            slice_end += c.len_utf8();
        } else {
            break;
        }
    }
    if slice_start > 0 {
        data.drain(..slice_start);
    }
    if slice_end > 0 {
        data.drain(data.len() - slice_end..);
    }
}

/// Collapses tokens in the given list of tokens.
pub(crate) fn collapse_tokens_in_place(tokens: &mut Vec<u32>, collapse: u32) {
    let mut last = None;
    tokens.retain(|&token| {
        let keep = last != Some(token) || token != collapse;
        last = Some(token);
        keep
    });
}
