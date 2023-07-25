//! Regex wrapper for different regex engines.

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

#[cfg(feature = "regex-onig")]
pub type RegexError = Box<onig::Error>;
#[cfg(not(feature = "regex-onig"))]
pub type RegexError = Box<fancy_regex::Error>;

#[derive(Debug)]
pub struct Regex {
    #[cfg(feature = "regex-onig")]
    regex: onig::Regex,
    #[cfg(not(feature = "regex-onig"))]
    regex: fancy_regex::Regex,
}
#[allow(dead_code)]
impl Regex {
    #[inline(always)]
    pub fn new(pattern: &str) -> Result<Self, RegexError> {
        #[cfg(feature = "regex-onig")]
        return Ok(Self {
            regex: onig::Regex::new(pattern)?,
        });
        #[cfg(not(feature = "regex-onig"))]
        return Ok(Self {
            regex: fancy_regex::Regex::new(pattern)?,
        });
    }

    #[cfg(not(feature = "regex-onig"))]
    #[inline(always)]
    pub fn find_iter(&self, text: &str) -> Vec<(usize, usize)> {
        self.regex
            .find_iter(text)
            .map(|m| m.unwrap())
            .map(|m| (m.start(), m.end()))
            .collect()
    }

    #[cfg(feature = "regex-onig")]
    #[inline(always)]
    pub fn find_iter(&self, text: &str) -> Vec<(usize, usize)> {
        self.regex.find_iter(text).collect()
    }

    #[cfg(not(feature = "regex-onig"))]
    #[inline(always)]
    pub fn find(&self, text: &str) -> Option<(usize, usize)> {
        self.regex.find(text).unwrap().map(|mat| (mat.start(), mat.end()))
    }

    #[cfg(feature = "regex-onig")]
    #[inline(always)]
    pub fn find(&self, text: &str) -> Option<(usize, usize)> {
        self.regex.find(text)
    }

    #[cfg(not(feature = "regex-onig"))]
    #[inline(always)]
    pub fn replace_all(&self, text: &str, replace: &str) -> String {
        self.regex.replace_all(text, replace).into_owned()
    }

    #[cfg(feature = "regex-onig")]
    #[inline(always)]
    pub fn replace_all(&self, text: &str, replace: &str) -> String {
        self.regex.replace_all(text, replace)
    }
}
