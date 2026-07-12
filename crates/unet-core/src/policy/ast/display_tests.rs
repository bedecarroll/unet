//! Tests for Display trait implementations on AST types

use crate::policy::ast::*;
use std::collections::HashMap;

#[test]
fn test_comparison_operator_display_all_variants() {
    assert_eq!(ComparisonOperator::Equal.to_string(), "==");
    assert_eq!(ComparisonOperator::NotEqual.to_string(), "!=");
    assert_eq!(ComparisonOperator::LessThan.to_string(), "<");
    assert_eq!(ComparisonOperator::LessThanOrEqual.to_string(), "<=");
    assert_eq!(ComparisonOperator::GreaterThan.to_string(), ">");
    assert_eq!(ComparisonOperator::GreaterThanOrEqual.to_string(), ">=");
    assert_eq!(ComparisonOperator::Contains.to_string(), "CONTAINS");
    assert_eq!(ComparisonOperator::Matches.to_string(), "MATCHES");
}

#[test]
fn test_value_array_display() {
    let array = Value::Array(vec![
        Value::String("item1".to_string()),
        Value::Number(42.0),
        Value::Boolean(true),
    ]);
    assert_eq!(array.to_string(), "[\"item1\", 42, true]");
}

#[test]
fn test_value_empty_array_display() {
    let empty_array = Value::Array(vec![]);
    assert_eq!(empty_array.to_string(), "[]");
}

#[test]
fn test_value_object_display() {
    let mut obj = HashMap::new();
    obj.insert("key1".to_string(), Value::String("value1".to_string()));
    obj.insert("key2".to_string(), Value::Number(123.0));

    let object = Value::Object(obj);
    let display = object.to_string();

    // Order might vary in HashMap, so check that it contains the expected parts
    assert!(display.starts_with('{'));
    assert!(display.ends_with('}'));
    assert!(display.contains("\"key1\": \"value1\"") || display.contains("\"key2\": 123"));
}

#[test]
fn test_value_empty_object_display() {
    let empty_object = Value::Object(HashMap::new());
    assert_eq!(empty_object.to_string(), "{}");
}

#[test]
fn test_condition_and_display() {
    let condition = Condition::And(
        Box::new(Condition::Comparison {
            field: FieldRef {
                path: vec!["vendor".to_string()],
            },
            operator: ComparisonOperator::Equal,
            value: Value::String("cisco".to_string()),
        }),
        Box::new(Condition::Comparison {
            field: FieldRef {
                path: vec!["model".to_string()],
            },
            operator: ComparisonOperator::Equal,
            value: Value::String("2960".to_string()),
        }),
    );
    let display = condition.to_string();
    assert!(display.contains("AND"));
    assert!(display.contains("vendor == \"cisco\""));
    assert!(display.contains("model == \"2960\""));
}

#[test]
fn test_condition_or_display() {
    let condition = Condition::Or(
        Box::new(Condition::Comparison {
            field: FieldRef {
                path: vec!["vendor".to_string()],
            },
            operator: ComparisonOperator::Equal,
            value: Value::String("cisco".to_string()),
        }),
        Box::new(Condition::Comparison {
            field: FieldRef {
                path: vec!["vendor".to_string()],
            },
            operator: ComparisonOperator::Equal,
            value: Value::String("juniper".to_string()),
        }),
    );
    let display = condition.to_string();
    assert!(display.contains("OR"));
    assert!(display.contains("vendor == \"cisco\""));
    assert!(display.contains("vendor == \"juniper\""));
}

#[test]
fn test_condition_not_display() {
    let condition = Condition::Not(Box::new(Condition::Comparison {
        field: FieldRef {
            path: vec!["vendor".to_string()],
        },
        operator: ComparisonOperator::Equal,
        value: Value::String("cisco".to_string()),
    }));
    let display = condition.to_string();
    assert!(display.contains("NOT"));
    assert!(display.contains("vendor == \"cisco\""));
}

#[test]
fn test_condition_existence_null_display() {
    let condition = Condition::Existence {
        field: FieldRef {
            path: vec!["optional_field".to_string()],
        },
        is_null: true,
    };
    let display = condition.to_string();
    assert!(display.contains("optional_field"));
    assert!(display.contains("null") || display.contains("NULL"));
}

#[test]
fn test_condition_existence_not_null_display() {
    let condition = Condition::Existence {
        field: FieldRef {
            path: vec!["required_field".to_string()],
        },
        is_null: false,
    };
    let display = condition.to_string();
    assert!(display.contains("required_field"));
    assert!(display.contains("not null") || display.contains("NOT NULL"));
}

#[test]
fn test_condition_true_false_display() {
    assert_eq!(Condition::True.to_string(), "TRUE");
    assert_eq!(Condition::False.to_string(), "FALSE");
}

#[test]
fn test_action_set_display() {
    let action = Action::Set {
        field: FieldRef {
            path: vec!["custom_data".to_string(), "vlan".to_string()],
        },
        value: Value::Number(100.0),
    };
    let display = action.to_string();
    assert!(display.contains("SET"));
    assert!(display.contains("custom_data.vlan"));
    assert!(display.contains("100"));
}

#[test]
fn test_action_apply_template_display() {
    let action = Action::ApplyTemplate {
        template_path: "router_template".to_string(),
    };
    let display = action.to_string();
    assert!(display.contains("APPLY"));
    assert!(display.contains("router_template"));
}

#[test]
fn test_policy_rule_with_id_display() {
    let rule = PolicyRule {
        id: Some("test_rule".to_string()),
        condition: Condition::True,
        action: Action::Assert {
            field: FieldRef {
                path: vec!["vendor".to_string()],
            },
            expected: Value::String("cisco".to_string()),
        },
    };
    let display = rule.to_string();
    assert!(display.contains("WHEN"));
    assert!(display.contains("TRUE"));
    assert!(display.contains("THEN"));
    assert!(display.contains("ASSERT"));
    assert!(display.contains("vendor"));
    assert!(display.contains("cisco"));
}
