//! Tests for number value parsing functionality

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
fn test_parse_value_number_integer() {
    let pair = create_test_pair(Rule::value, "42");
    let result = parse_value(pair);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(42.0));
}

#[test]
fn test_parse_value_number_float() {
    let pi_string = std::f64::consts::PI.to_string();
    let pair = create_test_pair(Rule::value, &pi_string);
    let result = parse_value(pair);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(std::f64::consts::PI));
}

#[test]
fn test_parse_value_number_negative() {
    let pair = create_test_pair(Rule::value, "-123");
    let result = parse_value(pair);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(-123.0));
}

#[test]
fn test_parse_value_number_zero() {
    let pair = create_test_pair(Rule::value, "0");
    let result = parse_value(pair);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(0.0));
}

#[test]
fn test_parse_value_number_zero_float() {
    let pair = create_test_pair(Rule::value, "0.0");
    let result = parse_value(pair);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(0.0));
}

#[test]
fn test_parse_value_number_large() {
    let pair = create_test_pair(Rule::value, "999999.999999");
    let result = parse_value(pair);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(999_999.999_999));
}

#[test]
fn test_number_precision() {
    // Test various number formats and precision
    let test_cases = [
        ("1", 1.0),
        ("1.0", 1.0),
        ("1.5", 1.5),
        ("0.1", 0.1),
        ("0.01", 0.01),
        ("0.001", 0.001),
        ("123.456", 123.456),
        ("1000", 1000.0),
        ("1000.0", 1000.0),
        ("-1", -1.0),
        ("-1.5", -1.5),
        ("-0.1", -0.1),
    ];

    for (input, expected) in test_cases {
        let pair = create_test_pair(Rule::value, input);
        let result = parse_value(pair);
        assert!(result.is_ok(), "Failed to parse number: {input}");

        if let Value::Number(actual) = result.unwrap() {
            assert!(
                (actual - expected).abs() < f64::EPSILON,
                "Expected {expected}, got {actual} for input '{input}'"
            );
        } else {
            panic!("Expected Number value for input '{input}'");
        }
    }
}
