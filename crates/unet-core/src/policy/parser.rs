//! Parser implementation that converts Pest parse trees to AST
//!
//! This module implements the conversion from Pest's parse tree to our
//! strongly-typed AST structures.

use crate::policy::ast::{Action, ComparisonOperator, Condition, FieldRef, PolicyRule, Value};
use crate::policy::grammar::{PolicyGrammar, Rule};
use pest::{Parser, iterators::Pair};
use std::fmt;

/// Parser for policy rules
pub struct PolicyParser;

/// Errors that can occur during parsing
#[derive(Debug)]
pub struct ParseError {
    pub message: String,
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

impl PolicyParser {
    /// Parse a single policy rule from text
    pub fn parse_rule(input: &str) -> Result<PolicyRule, ParseError> {
        let pairs = PolicyGrammar::parse(Rule::rule, input).map_err(|e| ParseError {
            message: e.to_string(),
            location: None,
        })?;

        let rule_pair = pairs.into_iter().next().ok_or_else(|| ParseError {
            message: "No rule found in input".to_string(),
            location: None,
        })?;

        Self::parse_rule_pair(rule_pair)
    }

    /// Parse multiple policy rules from a policy file
    pub fn parse_file(input: &str) -> Result<Vec<PolicyRule>, ParseError> {
        let pairs = PolicyGrammar::parse(Rule::policy_file, input).map_err(|e| ParseError {
            message: e.to_string(),
            location: None,
        })?;

        let mut rules = Vec::new();
        for pair in pairs {
            if pair.as_rule() == Rule::policy_file {
                for inner_pair in pair.into_inner() {
                    if inner_pair.as_rule() == Rule::rule {
                        rules.push(Self::parse_rule_pair(inner_pair)?);
                    }
                }
            }
        }

        Ok(rules)
    }

    fn parse_rule_pair(pair: Pair<Rule>) -> Result<PolicyRule, ParseError> {
        let mut condition = None;
        let mut action = None;

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::condition => {
                    condition = Some(Self::parse_condition(inner_pair)?);
                }
                Rule::action => {
                    action = Some(Self::parse_action(inner_pair)?);
                }
                _ => {}
            }
        }

        Ok(PolicyRule {
            id: None, // Parser doesn't set IDs - they can be set by loader
            condition: condition.ok_or_else(|| ParseError {
                message: "Missing condition in rule".to_string(),
                location: None,
            })?,
            action: action.ok_or_else(|| ParseError {
                message: "Missing action in rule".to_string(),
                location: None,
            })?,
        })
    }

    fn parse_condition(pair: Pair<Rule>) -> Result<Condition, ParseError> {
        Self::parse_or_condition(pair.into_inner().next().unwrap())
    }

    fn parse_or_condition(pair: Pair<Rule>) -> Result<Condition, ParseError> {
        let mut inner = pair.into_inner();
        let mut left = Self::parse_and_condition(inner.next().unwrap())?;

        for right_pair in inner {
            let right = Self::parse_and_condition(right_pair)?;
            left = Condition::Or(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_and_condition(pair: Pair<Rule>) -> Result<Condition, ParseError> {
        let mut inner = pair.into_inner();
        let mut left = Self::parse_not_condition(inner.next().unwrap())?;

        for right_pair in inner {
            let right = Self::parse_not_condition(right_pair)?;
            left = Condition::And(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_not_condition(pair: Pair<Rule>) -> Result<Condition, ParseError> {
        let mut inner = pair.into_inner();
        let first = inner.next().unwrap();

        if first.as_str() == "NOT" {
            let condition = Self::parse_primary_condition(inner.next().unwrap())?;
            Ok(Condition::Not(Box::new(condition)))
        } else {
            Self::parse_primary_condition(first)
        }
    }

    fn parse_primary_condition(pair: Pair<Rule>) -> Result<Condition, ParseError> {
        match pair.as_rule() {
            Rule::condition => Self::parse_condition(pair),
            Rule::comparison => Self::parse_comparison(pair),
            Rule::existence_check => Self::parse_existence_check(pair),
            Rule::primary_condition => {
                // Unwrap the inner rule
                let inner = pair.into_inner().next().unwrap();
                Self::parse_primary_condition(inner)
            }
            _ => Err(ParseError {
                message: format!("Unexpected rule in primary condition: {:?}", pair.as_rule()),
                location: None,
            }),
        }
    }

    fn parse_comparison(pair: Pair<Rule>) -> Result<Condition, ParseError> {
        let mut inner = pair.into_inner();
        let field = Self::parse_field_ref(inner.next().unwrap())?;
        let operator = Self::parse_operator(inner.next().unwrap())?;
        let value = Self::parse_value(inner.next().unwrap())?;

        Ok(Condition::Comparison {
            field,
            operator,
            value,
        })
    }

    fn parse_existence_check(pair: Pair<Rule>) -> Result<Condition, ParseError> {
        let mut inner = pair.into_inner();
        let field = Self::parse_field_ref(inner.next().unwrap())?;

        // Skip "IS" token - it's implicit in the grammar
        let mut is_null = true; // Default to IS NULL

        // Check if we have NOT NULL
        if let Some(next_token) = inner.next() {
            match next_token.as_str() {
                "NULL" => is_null = true,
                "NOT" => {
                    // This should be followed by NULL
                    if let Some(null_token) = inner.next() {
                        if null_token.as_str() == "NULL" {
                            is_null = false;
                        } else {
                            return Err(ParseError {
                                message: format!(
                                    "Expected NULL after NOT, got: {}",
                                    null_token.as_str()
                                ),
                                location: None,
                            });
                        }
                    } else {
                        return Err(ParseError {
                            message: "Expected NULL after NOT".to_string(),
                            location: None,
                        });
                    }
                }
                _ => {
                    return Err(ParseError {
                        message: format!(
                            "Unexpected token in existence check: {}",
                            next_token.as_str()
                        ),
                        location: None,
                    });
                }
            }
        }

        Ok(Condition::Existence { field, is_null })
    }

    fn parse_operator(pair: Pair<Rule>) -> Result<ComparisonOperator, ParseError> {
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

    fn parse_field_ref(pair: Pair<Rule>) -> Result<FieldRef, ParseError> {
        let path = pair.into_inner().map(|p| p.as_str().to_string()).collect();
        Ok(FieldRef { path })
    }

    fn parse_value(pair: Pair<Rule>) -> Result<Value, ParseError> {
        let inner = pair.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::string_literal => {
                // Check if there's an inner string content
                if let Some(content_pair) = inner.into_inner().next() {
                    Ok(Value::String(content_pair.as_str().to_string()))
                } else {
                    // Empty string
                    Ok(Value::String(String::new()))
                }
            }
            Rule::number_literal => {
                let num = inner.as_str().parse::<f64>().map_err(|_| ParseError {
                    message: format!("Invalid number: {}", inner.as_str()),
                    location: None,
                })?;
                Ok(Value::Number(num))
            }
            Rule::regex_literal => {
                let mut inner_parts = inner.into_inner();
                let pattern = inner_parts.next().unwrap().as_str();
                Ok(Value::Regex(pattern.to_string()))
            }
            Rule::boolean_literal => {
                let value = inner.as_str() == "true";
                Ok(Value::Boolean(value))
            }
            Rule::null_literal => Ok(Value::Null),
            Rule::field_ref => {
                let field_ref = Self::parse_field_ref(inner)?;
                Ok(Value::FieldRef(field_ref))
            }
            _ => Err(ParseError {
                message: format!("Unknown value type: {:?}", inner.as_rule()),
                location: None,
            }),
        }
    }

    fn parse_action(pair: Pair<Rule>) -> Result<Action, ParseError> {
        let inner = pair.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::assert_action => Self::parse_assert_action(inner),
            Rule::set_action => Self::parse_set_action(inner),
            Rule::apply_template_action => Self::parse_apply_template_action(inner),
            _ => Err(ParseError {
                message: format!("Unknown action type: {:?}", inner.as_rule()),
                location: None,
            }),
        }
    }

    fn parse_assert_action(pair: Pair<Rule>) -> Result<Action, ParseError> {
        let mut inner = pair.into_inner();
        let field = Self::parse_field_ref(inner.next().unwrap())?;
        // "IS" token is implicit in the grammar
        let expected = Self::parse_value(inner.next().unwrap())?;

        Ok(Action::Assert { field, expected })
    }

    fn parse_set_action(pair: Pair<Rule>) -> Result<Action, ParseError> {
        let mut inner = pair.into_inner();
        let field = Self::parse_field_ref(inner.next().unwrap())?;
        // "TO" token is implicit in the grammar
        let value = Self::parse_value(inner.next().unwrap())?;

        Ok(Action::Set { field, value })
    }

    fn parse_apply_template_action(pair: Pair<Rule>) -> Result<Action, ParseError> {
        let inner = pair.into_inner().next().unwrap();

        // The inner should be a string_literal
        match inner.as_rule() {
            Rule::string_literal => {
                // Extract the string content
                if let Some(content_pair) = inner.into_inner().next() {
                    Ok(Action::ApplyTemplate {
                        template_path: content_pair.as_str().to_string(),
                    })
                } else {
                    // Empty string
                    Ok(Action::ApplyTemplate {
                        template_path: String::new(),
                    })
                }
            }
            _ => Err(ParseError {
                message: format!(
                    "Expected string literal for template path, got: {:?}",
                    inner.as_rule()
                ),
                location: None,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_rule() {
        let input = r#"WHEN node.vendor == "cisco" THEN ASSERT node.version IS "15.1""#;
        let result = PolicyParser::parse_rule(input);
        assert!(result.is_ok(), "Failed to parse simple rule: {:?}", result);

        let rule = result.unwrap();
        match rule.condition {
            Condition::Comparison {
                field,
                operator,
                value,
            } => {
                assert_eq!(field.path, vec!["node", "vendor"]);
                assert_eq!(operator, ComparisonOperator::Equal);
                assert_eq!(value, Value::String("cisco".to_string()));
            }
            _ => panic!("Expected comparison condition"),
        }
    }

    #[test]
    fn test_parse_complex_condition() {
        let input = r#"WHEN node.vendor == "juniper" AND node.model CONTAINS "qfx" THEN SET custom_data.priority TO "high""#;
        let result = PolicyParser::parse_rule(input);
        assert!(
            result.is_ok(),
            "Failed to parse complex condition: {:?}",
            result
        );
    }

    #[test]
    fn test_parse_boolean_operators() {
        let input = r#"WHEN (node.vendor == "cisco" OR node.vendor == "juniper") AND NOT node.lifecycle == "decommissioned" THEN ASSERT node.snmp_enabled IS true"#;
        let result = PolicyParser::parse_rule(input);
        assert!(
            result.is_ok(),
            "Failed to parse boolean operators: {:?}",
            result
        );
    }

    #[test]
    fn test_parse_null_check() {
        let input = r#"WHEN custom_data.location IS NOT NULL THEN SET node.location_id TO custom_data.location"#;
        let result = PolicyParser::parse_rule(input);
        assert!(result.is_ok(), "Failed to parse null check: {:?}", result);
    }

    #[test]
    fn test_parse_regex_literal() {
        let input = r#"WHEN node.hostname MATCHES /^dist-\d+$/ THEN APPLY "dist-template.jinja""#;
        let result = PolicyParser::parse_rule(input);
        assert!(
            result.is_ok(),
            "Failed to parse regex literal: {:?}",
            result
        );
    }

    #[test]
    fn test_parse_policy_file() {
        let input = r#"
            WHEN node.vendor == "cisco" THEN ASSERT node.os_version IS "15.1"
            WHEN node.role == "router" AND node.location.region == "west" THEN SET custom_data.backup_priority TO "high"
        "#;
        let result = PolicyParser::parse_file(input);
        assert!(result.is_ok(), "Failed to parse policy file: {:?}", result);

        let rules = result.unwrap();
        assert_eq!(rules.len(), 2);
    }
}
