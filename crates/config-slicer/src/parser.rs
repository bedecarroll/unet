//! Match-expression parsing for `config-slicer`.

use crate::error::{ConfigSlicerError, Result};
use regex::Regex;

/// Pre-compiled representation of a match expression.
#[derive(Debug, Clone)]
pub struct MatchSpec {
    expression: String,
    levels: Vec<String>,
    regexes: Vec<Regex>,
}

impl MatchSpec {
    /// Return the original expression.
    #[must_use]
    pub fn expression(&self) -> &str {
        &self.expression
    }

    /// Return the raw levels from the expression.
    #[must_use]
    pub fn levels(&self) -> &[String] {
        &self.levels
    }

    #[must_use]
    pub(crate) fn depth(&self) -> usize {
        self.regexes.len()
    }

    pub(crate) fn matches_path_prefix(&self, path: &[String]) -> bool {
        self.regexes.len() <= path.len()
            && self
                .regexes
                .iter()
                .zip(path.iter())
                .all(|(regex, segment)| regex.is_match(segment))
    }
}

/// Parse a match expression into a reusable `MatchSpec`.
///
/// # Errors
/// Returns an error when the expression is empty or contains invalid `regex`
/// syntax at any level.
pub fn parse_match(expression: &str) -> Result<MatchSpec> {
    let expression = expression.trim();
    if expression.is_empty() {
        return Err(ConfigSlicerError::Parse(
            "match expression cannot be empty".to_string(),
        ));
    }

    let levels = expression
        .split("||")
        .map(str::trim)
        .map(str::to_string)
        .collect::<Vec<_>>();

    if levels.iter().any(String::is_empty) {
        return Err(ConfigSlicerError::Parse(
            "match expression cannot contain empty levels".to_string(),
        ));
    }

    let regexes = levels
        .iter()
        .map(|level| {
            let pattern = if level == "*" {
                ".*".to_string()
            } else {
                level.clone()
            };
            Regex::new(&format!("^(?:{pattern})$"))
        })
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(MatchSpec {
        expression: expression.to_string(),
        levels,
        regexes,
    })
}
