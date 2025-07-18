//! JSON value comparison operations
//!
//! Contains functions for comparing JSON values, including numeric comparisons,
//! string operations, and regex matching.

use crate::policy::PolicyError;
use dashmap::DashMap;
use regex::Regex;
use serde_json::Value as JsonValue;
use std::sync::LazyLock;

/// Global cache for compiled regular expressions
static REGEX_CACHE: LazyLock<DashMap<String, Regex>> = LazyLock::new(DashMap::new);

/// Compare two JSON values using a comparison function
pub fn compare_json_values<F>(
    left: &JsonValue,
    right: &JsonValue,
    compare_fn: F,
) -> Result<bool, PolicyError>
where
    F: Fn(f64, f64) -> bool,
{
    match (left, right) {
        (JsonValue::Number(a), JsonValue::Number(b)) => {
            let a_f64 = a.as_f64().ok_or_else(|| PolicyError::ValidationError {
                message: "Invalid number in comparison".to_string(),
            })?;
            let b_f64 = b.as_f64().ok_or_else(|| PolicyError::ValidationError {
                message: "Invalid number in comparison".to_string(),
            })?;
            Ok(compare_fn(a_f64, b_f64))
        }
        (JsonValue::String(a), JsonValue::String(b)) =>
        {
            #[allow(clippy::cast_precision_loss)]
            Ok(compare_fn(a.len() as f64, b.len() as f64))
        }
        _ => Err(PolicyError::ValidationError {
            message: "Cannot compare non-numeric/non-string values".to_string(),
        }),
    }
}

/// Evaluate contains operation for JSON values
pub fn evaluate_contains_json(
    haystack: &JsonValue,
    needle: &JsonValue,
) -> Result<bool, PolicyError> {
    match (haystack, needle) {
        (JsonValue::String(s), JsonValue::String(substr)) => Ok(s.contains(substr)),
        (JsonValue::Array(arr), value) => Ok(arr.contains(value)),
        (JsonValue::Object(obj), JsonValue::String(key)) => Ok(obj.contains_key(key)),
        _ => Err(PolicyError::ValidationError {
            message: "Contains operation not supported for these types".to_string(),
        }),
    }
}

/// Evaluate regex match operation for JSON values
pub fn evaluate_regex_match_json(
    text: &JsonValue,
    pattern: &JsonValue,
) -> Result<bool, PolicyError> {
    match (text, pattern) {
        (JsonValue::String(s), JsonValue::String(regex_str)) => {
            let regex = if let Some(existing) = REGEX_CACHE.get(regex_str) {
                existing.clone()
            } else {
                let compiled = Regex::new(regex_str).map_err(|_| PolicyError::InvalidRegex {
                    pattern: regex_str.clone(),
                })?;
                REGEX_CACHE.insert(regex_str.clone(), compiled.clone());
                compiled
            };
            Ok(regex.is_match(s))
        }
        _ => Err(PolicyError::ValidationError {
            message: "Regex match requires string values".to_string(),
        }),
    }
}
