//! Tests for string value parsing functionality

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
fn test_parse_value_string_double_quotes() {
    let pair = create_test_pair(Rule::value, r#""cisco""#);
    let result = parse_value(pair);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("cisco".to_string()));
}

#[test]
fn test_parse_value_string_single_quotes() {
    let pair = create_test_pair(Rule::value, "'cisco'");
    let result = parse_value(pair);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("cisco".to_string()));
}

#[test]
fn test_parse_value_string_empty() {
    let pair = create_test_pair(Rule::value, r#""""#);
    let result = parse_value(pair);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String(String::new()));
}

#[test]
fn test_parse_value_string_special_characters() {
    let pair = create_test_pair(Rule::value, r#""Hello, World! @#$%^&*()""#);
    let result = parse_value(pair);
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Value::String("Hello, World! @#$%^&*()".to_string())
    );
}

#[test]
fn test_parse_value_string_with_spaces() {
    let pair = create_test_pair(Rule::value, r#""  spaced  content  ""#);
    let result = parse_value(pair);
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Value::String("  spaced  content  ".to_string())
    );

    // Test string with tabs and newlines (if supported by grammar)
    let pair2 = create_test_pair(Rule::value, r#""tab\there""#);
    let result2 = parse_value(pair2);
    assert!(result2.is_ok());
    // The exact behavior depends on how the grammar handles escape sequences
}
