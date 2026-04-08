use super::condition_parsing::{parse_comparison, parse_existence_check, parse_not_condition};
use crate::policy::ast::Condition;
use crate::policy::grammar::{PolicyGrammar, Rule};
use pest::{Parser, iterators::Pair};

fn create_test_pair(rule_type: Rule, content: &str) -> Pair<'_, Rule> {
    let mut pairs = PolicyGrammar::parse(rule_type, content)
        .unwrap_or_else(|_| panic!("Failed to parse: {content}"));
    pairs.next().unwrap()
}

#[test]
fn test_parse_not_condition_returns_error_for_pair_without_inner_conditions() {
    let pair = create_test_pair(Rule::identifier, "vendor");
    let result = parse_not_condition(pair);

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .message
            .contains("Expected not condition expression")
    );
}

#[test]
fn test_parse_comparison_returns_error_when_operator_is_missing() {
    let pair = create_test_pair(Rule::field_ref, "node");
    let result = parse_comparison(pair);

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .message
            .contains("Expected comparison operator")
    );
}

#[test]
fn test_parse_existence_check_returns_error_when_null_keyword_is_missing() {
    let pair = create_test_pair(Rule::field_ref, "custom_data");
    let result = parse_existence_check(pair);

    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("Expected NULL or NOT"));
}

#[test]
fn test_parse_not_condition_wraps_primary_condition() {
    let pair = create_test_pair(Rule::not_condition, r#"NOT node.vendor == "cisco""#);
    let result = parse_not_condition(pair);

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), Condition::Not(_)));
}

#[test]
fn test_parse_not_condition_passthrough_without_not_keyword() {
    let pair = create_test_pair(Rule::not_condition, r#"node.vendor == "cisco""#);
    let result = parse_not_condition(pair);

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), Condition::Comparison { .. }));
}

#[test]
fn test_parse_existence_check_parses_is_not_null() {
    let pair = create_test_pair(Rule::existence_check, "custom_data.location IS NOT NULL");
    let result = parse_existence_check(pair);

    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Condition::Existence {
            field: crate::policy::ast::FieldRef {
                path: vec!["custom_data".to_string(), "location".to_string()],
            },
            is_null: false,
        }
    );
}

#[test]
fn test_parse_existence_check_parses_is_null() {
    let pair = create_test_pair(Rule::existence_check, "custom_data.location IS NULL");
    let result = parse_existence_check(pair);

    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Condition::Existence {
            field: crate::policy::ast::FieldRef {
                path: vec!["custom_data".to_string(), "location".to_string()],
            },
            is_null: true,
        }
    );
}
