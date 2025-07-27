//! Tests for field reference parsing functionality

use super::super::value_parsing::parse_field_ref;
use crate::policy::grammar::{PolicyGrammar, Rule};
use pest::{Parser, iterators::Pair};

// Helper to create test pairs
fn create_test_pair(rule_type: Rule, content: &str) -> Pair<Rule> {
    let mut pairs = PolicyGrammar::parse(rule_type, content)
        .unwrap_or_else(|_| panic!("Failed to parse: {content}"));
    pairs.next().unwrap()
}

#[test]
fn test_parse_field_ref_simple() {
    let pair = create_test_pair(Rule::field_ref, "vendor");
    let result = parse_field_ref(&pair);
    assert_eq!(result.path, vec!["vendor"]);
}

#[test]
fn test_parse_field_ref_nested() {
    let pair = create_test_pair(Rule::field_ref, "custom_data.vlan");
    let result = parse_field_ref(&pair);
    assert_eq!(result.path, vec!["custom_data", "vlan"]);
}

#[test]
fn test_parse_field_ref_with_numbers() {
    let pair = create_test_pair(Rule::field_ref, "interface_0.status");
    let result = parse_field_ref(&pair);
    assert_eq!(result.path, vec!["interface_0", "status"]);
}

#[test]
fn test_parse_field_ref_single_character() {
    let pair = create_test_pair(Rule::field_ref, "a");
    let result = parse_field_ref(&pair);
    assert_eq!(result.path, vec!["a"]);
}

#[test]
fn test_parse_field_ref_edge_cases() {
    // Test deeply nested field reference
    let pair = create_test_pair(Rule::field_ref, "level1.level2.level3.level4.field");
    let result = parse_field_ref(&pair);
    let expected_path = vec!["level1", "level2", "level3", "level4", "field"];
    assert_eq!(result.path, expected_path);

    // Test field with underscores and numbers
    let pair2 = create_test_pair(Rule::field_ref, "test_field_123.sub_field_456");
    let result2 = parse_field_ref(&pair2);
    assert_eq!(result2.path, vec!["test_field_123", "sub_field_456"]);

    // Test single level field with numbers
    let pair3 = create_test_pair(Rule::field_ref, "field123");
    let result3 = parse_field_ref(&pair3);
    assert_eq!(result3.path, vec!["field123"]);
}
