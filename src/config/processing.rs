use alloc::vec::Vec;

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

/// Post-tokenization processing configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub enum Processing {
    /// Collapse repeated tokens.
    Collapse { id: u32 },
}

impl Processing {
    #[inline(never)]
    pub fn process(&self, tokens: &mut Vec<u32>) {
        use Processing::*;
        match self {
            Collapse { id } => {
                let mut last = None;
                tokens.retain(|&token| {
                    let keep = last != Some(token) || token != *id;
                    last = Some(token);
                    keep
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    #[test]
    fn test_processing_collapse() {
        let mut tokens = Vec::from([1, 1, 2, 2, 3, 3, 3, 4, 4, 4, 4]);
        let processing = Processing::Collapse { id: 3 };
        processing.process(&mut tokens);
        assert_eq!(tokens, Vec::from([1, 1, 2, 2, 3, 4, 4, 4, 4]));
    }
}
