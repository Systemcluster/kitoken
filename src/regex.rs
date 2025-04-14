//! Regex wrapper for different regex engines with serialization support.

use core::fmt::{Debug, Display};
use core::ops::Deref;

use alloc::borrow::Cow;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

#[cfg(feature = "regex-onig")]
use alloc::sync::Arc;

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

/// Regex error type.
#[derive(thiserror::Error)]
pub struct RegexError(pub String);
impl Display for RegexError {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}
impl Debug for RegexError {
    #[inline(never)]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_tuple("RegexError").field(&self.0).finish()
    }
}

/// Regex wrapper for different regex engines with serialization support.
#[derive(Clone)]
pub struct Regex {
    pub(crate) pattern: String,
    #[cfg(feature = "regex-onig")]
    pub(crate) regex:   Arc<onig::Regex>,
    #[cfg(not(feature = "regex-onig"))]
    pub(crate) regex:   fancy_regex::Regex,
}
#[allow(dead_code)]
impl Regex {
    #[inline(always)]
    pub fn new(pattern: &str) -> Result<Self, RegexError> {
        #[cfg(feature = "regex-onig")]
        return Ok(Self {
            pattern: pattern.to_string(),
            regex:   Arc::new(onig::Regex::new(pattern).map_err(|e| RegexError(e.to_string()))?),
        });
        #[cfg(not(feature = "regex-onig"))]
        return Ok(Self {
            pattern: pattern.to_string(),
            regex:   fancy_regex::Regex::new(pattern).map_err(|e| RegexError(e.to_string()))?,
        });
    }

    #[cfg(not(feature = "regex-onig"))]
    #[inline(always)]
    pub(crate) fn find_iter(&self, text: &str) -> Vec<(usize, usize)> {
        self.regex
            .find_iter(text)
            .map(|m| m.unwrap())
            .map(|m| (m.start(), m.end()))
            .collect()
    }

    #[cfg(feature = "regex-onig")]
    #[inline(always)]
    pub(crate) fn find_iter(&self, text: &str) -> Vec<(usize, usize)> {
        self.regex.find_iter(text).collect()
    }

    #[cfg(not(feature = "regex-onig"))]
    #[inline(always)]
    pub(crate) fn find(&self, text: &str) -> Option<(usize, usize)> {
        self.regex.find(text).unwrap().map(|mat| (mat.start(), mat.end()))
    }

    #[cfg(feature = "regex-onig")]
    #[inline(always)]
    pub(crate) fn find(&self, text: &str) -> Option<(usize, usize)> {
        self.regex.find(text)
    }

    #[cfg(not(feature = "regex-onig"))]
    #[inline(always)]
    pub(crate) fn replace_all(&self, text: &str, replace: &str) -> String {
        self.regex.replace_all(text, replace).into_owned()
    }

    #[cfg(feature = "regex-onig")]
    #[inline(always)]
    pub(crate) fn replace_all(&self, text: &str, replace: &str) -> String {
        self.regex.replace_all(text, replace)
    }

    #[inline(always)]
    pub(crate) fn escape(&self) -> Cow<'_, str> {
        fancy_regex::escape(&self.pattern)
    }
}
impl PartialEq for Regex {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.pattern == other.pattern
    }
}
impl Eq for Regex {}
impl Deref for Regex {
    type Target = str;

    #[inline(always)]
    fn deref(&self) -> &str {
        &self.pattern
    }
}
impl AsRef<str> for Regex {
    #[inline(always)]
    fn as_ref(&self) -> &str {
        self.deref()
    }
}
impl Display for Regex {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        Display::fmt(&self.pattern, f)
    }
}
impl Debug for Regex {
    #[inline(never)]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_tuple("Regex").field(&self.pattern).finish()
    }
}
impl TryFrom<String> for Regex {
    type Error = RegexError;

    #[inline(always)]
    fn try_from(pattern: String) -> Result<Self, Self::Error> {
        Self::new(&pattern)
    }
}
impl TryFrom<&str> for Regex {
    type Error = RegexError;

    #[inline(always)]
    fn try_from(pattern: &str) -> Result<Self, Self::Error> {
        Self::new(pattern)
    }
}

#[cfg(feature = "serialization")]
impl Serialize for Regex {
    #[inline(always)]
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.pattern)
    }
}

#[cfg(feature = "serialization")]
impl<'de> Deserialize<'de> for Regex {
    #[inline(always)]
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let pattern = String::deserialize(deserializer)?;
        Self::new(&pattern).map_err(serde::de::Error::custom)
    }
}

pub(crate) fn escape(pattern: &'_ str) -> Cow<'_, str> {
    fancy_regex::escape(pattern)
}
