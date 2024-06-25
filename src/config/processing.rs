//! Post-tokenization output processing.

use alloc::vec::Vec;

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

use crate::TokenId;

/// Post-tokenization output processing configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub enum Processing {
    /// Strip a token from the beginning and end.
    Strip {
        id:    TokenId,
        left:  u32,
        right: u32,
    },
    /// Collapse repeated tokens.
    Collapse { id: TokenId },
}

impl Processing {
    #[inline(never)]
    pub fn process(&self, tokens: &mut Vec<TokenId>) {
        use Processing::*;
        match self {
            Strip { id, left, right } => {
                process_strip(tokens, *id, *left, *right);
            }
            Collapse { id } => {
                process_collapse(tokens, *id);
            }
        }
    }
}

#[inline(never)]
fn process_strip(tokens: &mut Vec<TokenId>, id: TokenId, mut left: u32, mut right: u32) {
    let mut slice_start = 0;
    let mut slice_end = 0;
    if left > 0 {
        for &c in tokens.iter() {
            if c != id || left == 0 {
                break;
            }
            slice_start += 1;
            left -= 1;
        }
    }
    if right > 0 {
        for &c in tokens.iter().rev() {
            if c != id || right == 0 {
                break;
            }
            slice_end += 1;
            right -= 1;
        }
    }
    if slice_start > 0 {
        tokens.drain(..slice_start);
    }
    if slice_end > 0 {
        let len = tokens.len();
        tokens.drain(len - slice_end..);
    }
}

#[inline(never)]
fn process_collapse(tokens: &mut Vec<TokenId>, id: TokenId) {
    let mut last = None;
    tokens.retain(|&token| {
        let keep = last != Some(token) || token != id;
        last = Some(token);
        keep
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    #[test]
    fn test_processing_strip() {
        let mut tokens = Vec::from([1, 1, 2, 2, 3, 3, 3, 4, 4, 4, 1]);
        let processing = Processing::Strip {
            id:    1,
            left:  1,
            right: 1,
        };
        processing.process(&mut tokens);
        assert_eq!(tokens, Vec::from([1, 2, 2, 3, 3, 3, 4, 4, 4]));
    }

    #[test]
    fn test_processing_collapse() {
        let mut tokens = Vec::from([1, 1, 2, 2, 3, 3, 3, 4, 4, 4, 4]);
        let processing = Processing::Collapse { id: 3 };
        processing.process(&mut tokens);
        assert_eq!(tokens, Vec::from([1, 1, 2, 2, 3, 4, 4, 4, 4]));
    }
}
