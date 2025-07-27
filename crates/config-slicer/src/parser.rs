//! Configuration parsing utilities

use regex::Regex;

use crate::{ConfigSlicerError, Result};

/// Pre-compiled representation of a match expression.
#[derive(Debug, Clone)]
pub struct MatchSpec(pub Vec<Regex>);

/// Parse a match expression like `"interface||.*"` into a [`MatchSpec`].
///
/// # Errors
///
/// Returns [`ConfigSlicerError::Parse`] if the expression is empty or
/// contains an invalid regular expression.
pub fn parse_match(expr: &str) -> Result<MatchSpec> {
    if expr.trim().is_empty() {
        return Err(ConfigSlicerError::Parse("empty match expression".into()));
    }
    let parts = expr.split("||");
    let mut regexes = Vec::new();
    for part in parts {
        let pattern = format!("^{part}");
        let re = Regex::new(&pattern).map_err(|e| ConfigSlicerError::Parse(e.to_string()))?;
        regexes.push(re);
    }
    Ok(MatchSpec(regexes))
}

/// Single config line with depth metadata.
#[derive(Debug, Clone)]
pub struct Token<'a> {
    /// Leading whitespace count
    pub depth: usize,
    /// Full line text
    pub text: &'a str,
}

/// Tokenize a Cisco-style flat configuration using indentation for depth.
#[must_use]
pub fn tokenize_flat(text: &str) -> Vec<Token<'_>> {
    text.lines()
        .map(|line| {
            let depth = line.chars().take_while(|c| c.is_whitespace()).count();
            Token { depth, text: line }
        })
        .collect()
}
