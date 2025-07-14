//! Error types for policy parsing

use std::fmt;

/// Errors that can occur during parsing
#[derive(Debug)]
pub struct ParseError {
    /// Error message describing what went wrong
    pub message: String,
    /// Optional location in source where error occurred (line, column)
    pub location: Option<(usize, usize)>,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.location {
            Some((line, col)) => write!(
                f,
                "Parse error at line {}, column {}: {}",
                line, col, self.message
            ),
            None => write!(f, "Parse error: {}", self.message),
        }
    }
}

impl std::error::Error for ParseError {}
