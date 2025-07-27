//! Tests for other value types: null, regex, field references, and nested values

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
fn test_parse_value_null() {
    let pair = create_test_pair(Rule::value, "null");
    let result = parse_value(pair);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Null);
}

#[test]
fn test_parse_value_regex_simple() {
    let pair = create_test_pair(Rule::value, "/cisco/");
    let result = parse_value(pair);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Regex("cisco".to_string()));
}

#[test]
fn test_parse_value_regex_complex() {
    let pair = create_test_pair(Rule::value, r"/^[A-Z]{2,4}-\d+$/");
    let result = parse_value(pair);
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Value::Regex(r"^[A-Z]{2,4}-\d+$".to_string())
    );
}

#[test]
fn test_parse_value_regex_with_escape() {
    let pair = create_test_pair(Rule::value, r"/test\.example\.com/");
    let result = parse_value(pair);
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Value::Regex(r"test\.example\.com".to_string())
    );
}

#[test]
fn test_parse_value_regex_empty() {
    let pair = create_test_pair(Rule::value, "//");
    let result = parse_value(pair);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Regex(String::new()));
}

#[test]
fn test_parse_value_field_ref() {
    // This test assumes field references can be used as values
    let pair = create_test_pair(Rule::value, "vendor");
    let result = parse_value(pair);
    assert!(result.is_ok());

    if let Value::FieldRef(field_ref) = result.unwrap() {
        assert_eq!(field_ref.path, vec!["vendor"]);
    } else {
        panic!("Expected FieldReference value");
    }
}

#[test]
fn test_parse_value_nested_value_rule() {
    // Test that the parser can handle nested rules correctly
    let pair = create_test_pair(Rule::value, r#""nested string""#);
    let result = parse_value(pair);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("nested string".to_string()));
}
