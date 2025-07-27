//! Configuration slicing engine

use crate::{
    Result,
    parser::{MatchSpec, tokenize_flat},
};

/// Slice configuration text using the provided [`MatchSpec`].
///
/// # Errors
///
/// This function currently never returns an error.
pub fn slice_config(text: &str, spec: &MatchSpec) -> Result<Vec<String>> {
    let tokens = tokenize_flat(text);
    let mut result = Vec::new();
    let mut stack: Vec<String> = Vec::new();
    let mut capturing = false;

    for token in tokens {
        while stack.len() > token.depth {
            stack.pop();
            if stack.len() < spec.0.len() {
                capturing = false;
            }
        }
        stack.push(token.text.trim_start().to_string());

        if stack.len() >= spec.0.len() {
            let mut matched = true;
            for (i, re) in spec.0.iter().enumerate() {
                if !re.is_match(&stack[i]) {
                    matched = false;
                    break;
                }
            }
            if matched {
                capturing = true;
            } else if stack.len() == spec.0.len() {
                capturing = false;
            }
        }

        if capturing && stack.len() >= spec.0.len() && !token.text.trim_start().starts_with('!') {
            result.push(token.text.to_string());
        }
    }

    Ok(result)
}
