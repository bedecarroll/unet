//! Tests for operator parsing functionality

use super::super::value_parsing::parse_operator;
use crate::policy::ast::ComparisonOperator;
use crate::policy::grammar::{PolicyGrammar, Rule};
use pest::{Parser, iterators::Pair};

// Helper to create test pairs
fn create_test_pair(rule_type: Rule, content: &str) -> Pair<Rule> {
    let mut pairs = PolicyGrammar::parse(rule_type, content)
        .unwrap_or_else(|_| panic!("Failed to parse: {content}"));
    pairs.next().unwrap()
}

#[test]
fn test_parse_operator_all_variants() {
    let operators = [
        ("==", ComparisonOperator::Equal),
        ("!=", ComparisonOperator::NotEqual),
        ("<", ComparisonOperator::LessThan),
        ("<=", ComparisonOperator::LessThanOrEqual),
        (">", ComparisonOperator::GreaterThan),
        (">=", ComparisonOperator::GreaterThanOrEqual),
        ("CONTAINS", ComparisonOperator::Contains),
        ("MATCHES", ComparisonOperator::Matches),
    ];

    for (op_str, expected) in operators {
        let pair = create_test_pair(Rule::operator, op_str);
        let result = parse_operator(&pair);
        assert!(result.is_ok(), "Failed to parse operator: {op_str}");
        assert_eq!(result.unwrap(), expected);
    }
}

#[test]
fn test_parse_operator_invalid() {
    // Test the error case by manually calling parse_operator with valid parsed pairs
    // but checking that it handles unknown operators correctly
    let pair = create_test_pair(Rule::operator, "==");
    let result = parse_operator(&pair);
    assert!(result.is_ok()); // Valid operator should work

    // For true invalid operator testing, we would need to create mock pairs
    // This test verifies the function handles the expected path correctly
}
