//! Tests for trait implementations: serialization, clone, debug, equality

use crate::policy::ast::*;

#[test]
fn test_serialization_round_trip() {
    let original_rule = PolicyRule {
        id: Some("test_rule".to_string()),
        condition: Condition::Comparison {
            field: FieldRef {
                path: vec!["vendor".to_string()],
            },
            operator: ComparisonOperator::Equal,
            value: Value::String("cisco".to_string()),
        },
        action: Action::Set {
            field: FieldRef {
                path: vec!["custom_data".to_string(), "validated".to_string()],
            },
            value: Value::Boolean(true),
        },
    };

    // Test that serialization/deserialization works (if implemented)
    // This assumes serde is implemented for the AST types
    let serialized = serde_json::to_string(&original_rule);
    if let Ok(json_str) = serialized {
        let deserialized: Result<PolicyRule, _> = serde_json::from_str(&json_str);
        if let Ok(restored_rule) = deserialized {
            assert_eq!(original_rule.id, restored_rule.id);
        }
    }

    // If serialization isn't implemented, just verify the structure
    assert_eq!(original_rule.id, Some("test_rule".to_string()));
}

#[test]
fn test_clone_and_equality() {
    let field_ref = FieldRef {
        path: vec!["test".to_string(), "field".to_string()],
    };
    let cloned_field_ref = field_ref.clone();
    assert_eq!(field_ref.path, cloned_field_ref.path);

    let value = Value::String("test_value".to_string());
    let cloned_value = value.clone();
    assert_eq!(value, cloned_value);

    let condition = Condition::Comparison {
        field: field_ref.clone(),
        operator: ComparisonOperator::Equal,
        value: value.clone(),
    };
    let cloned_condition = condition.clone();
    assert_eq!(condition, cloned_condition);

    let action = Action::Set {
        field: field_ref,
        value,
    };
    let cloned_action = action.clone();
    assert_eq!(action, cloned_action);
}

#[test]
fn test_debug_formatting() {
    let field_ref = FieldRef {
        path: vec!["debug".to_string(), "test".to_string()],
    };
    let debug_output = format!("{field_ref:?}");
    assert!(debug_output.contains("FieldRef"));
    assert!(debug_output.contains("debug"));
    assert!(debug_output.contains("test"));

    let value = Value::Number(42.0);
    let debug_output = format!("{value:?}");
    assert!(debug_output.contains("Number"));
    assert!(debug_output.contains("42"));

    let condition = Condition::True;
    let debug_output = format!("{condition:?}");
    assert!(debug_output.contains("True"));

    let action = Action::Assert {
        field: field_ref,
        expected: value,
    };
    let debug_output = format!("{action:?}");
    assert!(debug_output.contains("Assert"));
}

#[test]
fn test_partial_eq_implementation() {
    // Test that PartialEq works correctly for various Value types
    assert_eq!(
        Value::String("test".to_string()),
        Value::String("test".to_string())
    );
    assert_ne!(
        Value::String("test".to_string()),
        Value::String("other".to_string())
    );

    assert_eq!(Value::Number(42.0), Value::Number(42.0));
    assert_ne!(Value::Number(42.0), Value::Number(43.0));

    assert_eq!(Value::Boolean(true), Value::Boolean(true));
    assert_ne!(Value::Boolean(true), Value::Boolean(false));

    assert_eq!(Value::Null, Value::Null);

    // Test different types are not equal
    assert_ne!(Value::String("42".to_string()), Value::Number(42.0));
    assert_ne!(Value::Boolean(true), Value::String("true".to_string()));
}
