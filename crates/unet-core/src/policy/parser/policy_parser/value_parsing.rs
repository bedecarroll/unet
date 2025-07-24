//! Value, field reference, and operator parsing logic

use super::super::error::ParseError;
use crate::policy::ast::{ComparisonOperator, FieldRef, Value};
use crate::policy::grammar::Rule;
use pest::iterators::Pair;

/// Parse a comparison operator
pub fn parse_operator(pair: &Pair<Rule>) -> Result<ComparisonOperator, ParseError> {
    match pair.as_str() {
        "==" => Ok(ComparisonOperator::Equal),
        "!=" => Ok(ComparisonOperator::NotEqual),
        "<" => Ok(ComparisonOperator::LessThan),
        "<=" => Ok(ComparisonOperator::LessThanOrEqual),
        ">" => Ok(ComparisonOperator::GreaterThan),
        ">=" => Ok(ComparisonOperator::GreaterThanOrEqual),
        "CONTAINS" => Ok(ComparisonOperator::Contains),
        "MATCHES" => Ok(ComparisonOperator::Matches),
        _ => Err(ParseError {
            message: format!("Unknown operator: {}", pair.as_str()),
            location: None,
        }),
    }
}

/// Parse a field reference (e.g., node.network.interfaces.eth0.status)
pub fn parse_field_ref(pair: &Pair<Rule>) -> FieldRef {
    let path: Vec<String> = pair.as_str().split('.').map(String::from).collect();
    FieldRef { path }
}

/// Parse a value (string, number, boolean, null, regex, or field reference)
pub fn parse_value(pair: Pair<Rule>) -> Result<Value, ParseError> {
    match pair.as_rule() {
        Rule::value => {
            // Handle nested value rule by unwrapping the inner rule
            let inner_pair = pair.into_inner().next().ok_or_else(|| ParseError {
                message: "Empty value rule".to_string(),
                location: None,
            })?;
            parse_value(inner_pair)
        }
        Rule::string_literal => {
            let str_content = pair.as_str();
            // Remove surrounding quotes - both single and double quotes use the same logic
            let content = if (str_content.starts_with('"') && str_content.ends_with('"'))
                || (str_content.starts_with('\'') && str_content.ends_with('\''))
            {
                &str_content[1..str_content.len() - 1]
            } else {
                str_content
            };
            Ok(Value::String(content.to_string()))
        }
        Rule::number_literal => {
            let number_str = pair.as_str();
            if number_str.contains('.') {
                number_str
                    .parse::<f64>()
                    .map(Value::Number)
                    .map_err(|_| ParseError {
                        message: format!("Invalid float: {number_str}"),
                        location: None,
                    })
            } else {
                // Parse as f64 directly to avoid precision loss warnings
                number_str
                    .parse::<f64>()
                    .map(Value::Number)
                    .map_err(|_| ParseError {
                        message: format!("Invalid number: {number_str}"),
                        location: None,
                    })
            }
        }
        Rule::boolean_literal => match pair.as_str() {
            "true" => Ok(Value::Boolean(true)),
            "false" => Ok(Value::Boolean(false)),
            _ => Err(ParseError {
                message: format!("Invalid boolean: {}", pair.as_str()),
                location: None,
            }),
        },
        Rule::null_literal => Ok(Value::Null),
        Rule::regex_literal => {
            let regex_str = pair.as_str();
            // Extract the regex pattern between the slashes
            if regex_str.starts_with('/') && regex_str.len() > 1 {
                let end_pos = regex_str.rfind('/').unwrap_or(regex_str.len());
                let pattern = &regex_str[1..end_pos];
                Ok(Value::Regex(pattern.to_string()))
            } else {
                Err(ParseError {
                    message: format!("Invalid regex literal: {regex_str}"),
                    location: None,
                })
            }
        }
        Rule::field_ref => {
            let field_ref = parse_field_ref(&pair);
            Ok(Value::FieldRef(field_ref))
        }
        _ => Err(ParseError {
            message: format!("Unexpected value rule: {:?}", pair.as_rule()),
            location: None,
        }),
    }
}
