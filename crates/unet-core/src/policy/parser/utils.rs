//! Utility functions for parsing operations

use super::error::ParseError;
use crate::policy::grammar::Rule;
use pest::iterators::Pair;

/// Helper to safely extract the next pair from an iterator
///
/// This replaces the pattern `iter.next().unwrap()` with proper error handling
pub fn next_pair<'a>(
    mut iter: impl Iterator<Item = Pair<'a, Rule>>,
    context: &str,
) -> Result<Pair<'a, Rule>, ParseError> {
    iter.next().ok_or_else(|| ParseError {
        message: format!("Expected {context} in parse tree but none found"),
        location: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_pair_with_empty_iterator() {
        let empty_pairs: Vec<Pair<Rule>> = vec![];
        let result = next_pair(empty_pairs.into_iter(), "test context");

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(
            error
                .message
                .contains("Expected test context in parse tree but none found")
        );
        assert!(error.location.is_none());
    }

    #[test]
    fn test_next_pair_error_message_format() {
        let empty_pairs: Vec<Pair<Rule>> = vec![];
        let contexts = ["condition", "action", "rule body", "identifier"];

        for context in &contexts {
            let result = next_pair(empty_pairs.clone().into_iter(), context);
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(
                error
                    .message
                    .contains(&format!("Expected {context} in parse tree but none found"))
            );
        }
    }
}
