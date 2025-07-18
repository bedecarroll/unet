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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_creation() {
        let error = ParseError {
            message: "Test error".to_string(),
            location: None,
        };
        assert_eq!(error.message, "Test error");
        assert!(error.location.is_none());
    }

    #[test]
    fn test_parse_error_with_location() {
        let error = ParseError {
            message: "Syntax error".to_string(),
            location: Some((10, 5)),
        };
        assert_eq!(error.message, "Syntax error");
        assert_eq!(error.location, Some((10, 5)));
    }

    #[test]
    fn test_parse_error_display_without_location() {
        let error = ParseError {
            message: "Missing semicolon".to_string(),
            location: None,
        };
        let display_str = format!("{error}");
        assert_eq!(display_str, "Parse error: Missing semicolon");
    }

    #[test]
    fn test_parse_error_display_with_location() {
        let error = ParseError {
            message: "Unexpected token".to_string(),
            location: Some((42, 15)),
        };
        let display_str = format!("{error}");
        assert_eq!(
            display_str,
            "Parse error at line 42, column 15: Unexpected token"
        );
    }

    #[test]
    fn test_parse_error_debug() {
        let error = ParseError {
            message: "Debug test".to_string(),
            location: Some((1, 1)),
        };
        let debug_str = format!("{error:?}");
        assert!(debug_str.contains("ParseError"));
        assert!(debug_str.contains("Debug test"));
        assert!(debug_str.contains("1, 1"));
    }

    #[test]
    fn test_parse_error_implements_error_trait() {
        use std::error::Error;

        let error = ParseError {
            message: "Error trait test".to_string(),
            location: None,
        };

        // Test that it implements std::error::Error
        let _: &dyn std::error::Error = &error;

        // Test the Error trait methods
        assert!(error.source().is_none()); // Default implementation
    }

    #[test]
    fn test_parse_error_location_edge_cases() {
        // Test with zero values
        let error = ParseError {
            message: "Start of file".to_string(),
            location: Some((0, 0)),
        };
        let display_str = format!("{error}");
        assert_eq!(
            display_str,
            "Parse error at line 0, column 0: Start of file"
        );

        // Test with large values
        let error = ParseError {
            message: "Large numbers".to_string(),
            location: Some((999_999, 888_888)),
        };
        let display_str = format!("{error}");
        assert_eq!(
            display_str,
            "Parse error at line 999999, column 888888: Large numbers"
        );
    }
}
