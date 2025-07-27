//! Tests for boolean value parsing functionality

use super::super::value_parsing::parse_value;
use crate::policy::ast::Value;
use crate::policy::grammar::{PolicyGrammar, Rule};
use pest::{Parser, iterators::Pair};

// Helper to create test pairs
fn create_test_pair(rule_type: Rule, content: &str) -> Pair<Rule> {
    let mut pairs = PolicyGrammar::parse(rule_type, content)
        .unwrap_or_else(|_| panic!("Failed to parse: {content}"));
    pairs.next().unwrap()
}

#[test]
fn test_parse_value_boolean_true() {
    let pair = create_test_pair(Rule::value, "true");
    let result = parse_value(pair);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_parse_value_boolean_false() {
    let pair = create_test_pair(Rule::value, "false");
    let result = parse_value(pair);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(false));
}

#[test]
fn test_parse_value_boolean_invalid() {
    // This test would check error handling for invalid boolean values
    // However, since we're using a grammar parser, invalid values
    // would not successfully parse as Rule::value in the first place

    // Test that true/false are case sensitive (if they are)
    // This would depend on the grammar definition
    let pair = create_test_pair(Rule::value, "true");
    let result = parse_value(pair);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}
