//! Post-tokenization output processing.

use alloc::vec::Vec;

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

use crate::TokenId;

/// Processing direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub enum ProcessingDirection {
    Left,
    Right,
}

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
    /// Pad to a fixed length.
    Pad {
        id:        TokenId,
        length:    u32,
        stride:    u32,
        direction: ProcessingDirection,
    },
    /// Truncate to a fixed length.
    Truncate {
        length:    u32,
        stride:    u32,
        direction: ProcessingDirection,
    },
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
            Pad {
                id,
                length,
                stride,
                direction,
            } => {
                process_pad(tokens, *id, *length as _, *stride as _, *direction);
            }
            Truncate {
                length,
                stride,
                direction,
            } => {
                process_truncate(tokens, *length as _, *stride as _, *direction);
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

#[inline(never)]
fn process_pad(
    tokens: &mut Vec<TokenId>, id: TokenId, length: usize, stride: usize,
    direction: ProcessingDirection,
) {
    let len = tokens.len();
    if len >= length {
        return;
    }
    let amount = if stride > 0 && (length - len) % stride > 0 {
        (length - len) + (stride - (length - len) % stride)
    } else {
        length - len
    };
    if amount > 0 {
        let padding = core::iter::repeat_n(id, amount).collect::<Vec<_>>();
        match direction {
            ProcessingDirection::Left => {
                tokens.splice(0..0, padding);
            }
            ProcessingDirection::Right => {
                tokens.extend(padding);
            }
        }
    }
}

#[inline(never)]
fn process_truncate(
    tokens: &mut Vec<TokenId>, length: usize, stride: usize, direction: ProcessingDirection,
) {
    let len = tokens.len();
    if len <= length {
        return;
    }
    let amount = if stride > 0 && (len - length) % stride > 0 {
        (len - length) + (stride - (len - length) % stride)
    } else {
        len - length
    };
    if len > length {
        match direction {
            ProcessingDirection::Left => {
                tokens.drain(0..amount);
            }
            ProcessingDirection::Right => {
                tokens.truncate(len - amount);
            }
        }
    }
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

    #[test]
    fn test_processing_pad() {
        let mut tokens = Vec::from([1, 2, 3]);
        let processing = Processing::Pad {
            id:        0,
            length:    5,
            stride:    2,
            direction: ProcessingDirection::Left,
        };
        processing.process(&mut tokens);
        assert_eq!(tokens, Vec::from([0, 0, 1, 2, 3]));
        let mut tokens = Vec::from([1, 2, 3]);
        let processing = Processing::Pad {
            id:        0,
            length:    5,
            stride:    2,
            direction: ProcessingDirection::Right,
        };
        processing.process(&mut tokens);
        assert_eq!(tokens, Vec::from([1, 2, 3, 0, 0]));
    }

    #[test]
    fn test_processing_truncate() {
        let mut tokens = Vec::from([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let processing = Processing::Truncate {
            length:    5,
            stride:    2,
            direction: ProcessingDirection::Left,
        };
        processing.process(&mut tokens);
        assert_eq!(tokens, Vec::from([7, 8, 9, 10]));
        let mut tokens = Vec::from([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let processing = Processing::Truncate {
            length:    5,
            stride:    2,
            direction: ProcessingDirection::Right,
        };
        processing.process(&mut tokens);
        assert_eq!(tokens, Vec::from([1, 2, 3, 4]));
    }
}
