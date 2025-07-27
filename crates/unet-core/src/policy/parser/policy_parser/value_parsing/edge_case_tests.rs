//! Tests for edge cases and error handling in value parsing

use super::super::value_parsing::parse_value;
use crate::policy::ast::{FieldRef, Value};
use crate::policy::grammar::{PolicyGrammar, Rule};
use pest::{Parser, iterators::Pair};

// Helper to create test pairs
fn create_test_pair(rule_type: Rule, content: &str) -> Pair<Rule> {
    let mut pairs = PolicyGrammar::parse(rule_type, content)
        .unwrap_or_else(|_| panic!("Failed to parse: {content}"));
    pairs.next().unwrap()
}

#[test]
fn test_parse_value_error_cases() {
    // Test that the parser handles various edge cases correctly

    // This test is mainly for documentation - actual error cases
    // would typically not parse successfully with the grammar

    // Test valid edge cases that might cause issues
    let edge_cases = [
        ("0", Value::Number(0.0)),
        (r#""""#, Value::String(String::new())),
        ("null", Value::Null),
        ("//", Value::Regex(String::new())),
    ];

    for (input, expected) in edge_cases {
        let pair = create_test_pair(Rule::value, input);
        let result = parse_value(pair);
        assert!(result.is_ok(), "Failed to parse edge case: {input}");
        assert_eq!(result.unwrap(), expected);
    }
}

#[test]
fn test_parse_regex_edge_cases() {
    // Test various regex patterns that might be problematic
    let regex_patterns = [
        ("/simple/", "simple"),
        ("//", ""),
        (r"/.*\//", r".*\/"),
        (r"/\d+/", r"\d+"),
        (r"/^test$/", r"^test$"),
        (r"/[a-zA-Z0-9]+/", r"[a-zA-Z0-9]+"),
    ];

    for (input, expected_pattern) in regex_patterns {
        let pair = create_test_pair(Rule::value, input);
        let result = parse_value(pair);
        assert!(result.is_ok(), "Failed to parse regex: {input}");

        if let Value::Regex(pattern) = result.unwrap() {
            assert_eq!(pattern, expected_pattern);
        } else {
            panic!("Expected Regex for input: {input}");
        }
    }
}

#[test]
fn test_string_without_quotes() {
    // Test behavior when parsing values that might be ambiguous
    // This tests the grammar's ability to distinguish between different value types

    // These should parse as their respective types, not as strings
    let test_cases = [
        ("42", Value::Number(42.0)),
        ("true", Value::Boolean(true)),
        ("false", Value::Boolean(false)),
        ("null", Value::Null),
    ];

    for (input, expected) in test_cases {
        let pair = create_test_pair(Rule::value, input);
        let result = parse_value(pair);
        assert!(result.is_ok(), "Failed to parse: {input}");
        assert_eq!(result.unwrap(), expected);
    }
}

#[test]
fn test_field_ref_creation() {
    // Test that field references are created correctly with proper structure
    let pair = create_test_pair(Rule::value, "complex.nested.field");
    let result = parse_value(pair);
    assert!(result.is_ok());

    if let Value::FieldRef(field_ref) = result.unwrap() {
        let expected_path = vec![
            "complex".to_string(),
            "nested".to_string(),
            "field".to_string(),
        ];
        assert_eq!(field_ref.path, expected_path);

        // Test that FieldRef struct is properly constructed
        let manual_field_ref = FieldRef {
            path: expected_path,
        };
        assert_eq!(field_ref.path, manual_field_ref.path);
    } else {
        panic!("Expected FieldReference value");
    }
}

#[test]
fn test_value_variants_coverage() {
    // Test that all Value enum variants can be parsed
    let test_cases = [
        (r#""string""#, "String"),
        ("42", "Number"),
        ("true", "Boolean"),
        ("null", "Null"),
        ("/regex/", "Regex"),
        ("field", "FieldRef"),
    ];

    for (input, variant_name) in test_cases {
        let pair = create_test_pair(Rule::value, input);
        let result = parse_value(pair);
        assert!(result.is_ok(), "Failed to parse {variant_name}: {input}");

        // Verify the correct variant is created
        match result.unwrap() {
            Value::String(_) => assert_eq!(variant_name, "String"),
            Value::Number(_) => assert_eq!(variant_name, "Number"),
            Value::Boolean(_) => assert_eq!(variant_name, "Boolean"),
            Value::Null => assert_eq!(variant_name, "Null"),
            Value::Regex(_) => assert_eq!(variant_name, "Regex"),
            Value::FieldRef(_) => assert_eq!(variant_name, "FieldRef"),
            Value::Array(_) => assert_eq!(variant_name, "Array"),
            Value::Object(_) => assert_eq!(variant_name, "Object"),
        }
    }
}
