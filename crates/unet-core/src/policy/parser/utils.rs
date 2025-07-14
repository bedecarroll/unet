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
