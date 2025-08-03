//! Tests for value resolution error paths and edge cases

use crate::policy::PolicyError;
use crate::policy::ast::{FieldRef, Value};
use crate::policy::evaluator::actions::ActionExecutor;
use crate::policy::evaluator::context::EvaluationContext;
use serde_json::json;
use std::collections::HashMap;

/// Create test evaluation context with various field types
fn create_test_context() -> EvaluationContext {
    EvaluationContext::new(json!({
        "vendor": "cisco",
        "model": "2960",
        "role": "switch",
        "custom_data": {
            "config": {
                "vlan": 100,
                "name": "test_vlan"
            },
            "interfaces": [
                {"name": "GigabitEthernet0/1", "status": "up"},
                {"name": "GigabitEthernet0/2", "status": "down"}
            ]
        },
        "metrics": {
            "cpu_usage": 45.5,
            "memory_usage": 67.2,
            "temperature": 32.1
        }
    }))
}

#[test]
fn test_resolve_value_with_field_reference_not_found() {
    let context = create_test_context();
    let field_ref = FieldRef {
        path: vec!["nonexistent_field".to_string()],
    };
    let value = Value::FieldRef(field_ref);

    let result = ActionExecutor::resolve_value(&value, &context);

    assert!(result.is_err());
    match result.unwrap_err() {
        PolicyError::ValidationError { message } => {
            assert!(message.contains("Field not found: nonexistent_field"));
        }
        other => panic!("Expected ValidationError, got {other:?}"),
    }
}

#[test]
fn test_resolve_value_with_nested_array() {
    let context = create_test_context();
    let array_values = vec![
        Value::String("test1".to_string()),
        Value::FieldRef(FieldRef {
            path: vec!["vendor".to_string()],
        }),
        Value::Number(42.0),
    ];
    let value = Value::Array(array_values);

    let result = ActionExecutor::resolve_value(&value, &context);

    assert!(result.is_ok());
    let resolved = result.unwrap();
    assert!(resolved.is_array());
    let array = resolved.as_array().unwrap();
    assert_eq!(array.len(), 3);
    assert_eq!(array[0], json!("test1"));
    assert_eq!(array[1], json!("cisco"));
    assert_eq!(array[2], json!(42.0));
}

#[test]
fn test_resolve_value_with_nested_object() {
    let context = create_test_context();
    let mut object_values = HashMap::new();
    object_values.insert("static".to_string(), Value::String("value".to_string()));
    object_values.insert(
        "dynamic".to_string(),
        Value::FieldRef(FieldRef {
            path: vec!["model".to_string()],
        }),
    );
    let value = Value::Object(object_values);

    let result = ActionExecutor::resolve_value(&value, &context);

    assert!(result.is_ok());
    let resolved = result.unwrap();
    assert!(resolved.is_object());
    let obj = resolved.as_object().unwrap();
    assert_eq!(obj["static"], json!("value"));
    assert_eq!(obj["dynamic"], json!("2960"));
}

#[test]
fn test_resolve_value_array_with_field_reference_error() {
    let context = create_test_context();
    let array_values = vec![
        Value::String("test1".to_string()),
        Value::FieldRef(FieldRef {
            path: vec!["nonexistent".to_string()],
        }),
    ];
    let value = Value::Array(array_values);

    let result = ActionExecutor::resolve_value(&value, &context);

    assert!(result.is_err());
    match result.unwrap_err() {
        PolicyError::ValidationError { message } => {
            assert!(message.contains("Field not found: nonexistent"));
        }
        other => panic!("Expected ValidationError, got {other:?}"),
    }
}

#[test]
fn test_resolve_value_object_with_field_reference_error() {
    let context = create_test_context();
    let mut object_values = HashMap::new();
    object_values.insert("static".to_string(), Value::String("value".to_string()));
    object_values.insert(
        "invalid".to_string(),
        Value::FieldRef(FieldRef {
            path: vec!["nonexistent".to_string()],
        }),
    );
    let value = Value::Object(object_values);

    let result = ActionExecutor::resolve_value(&value, &context);

    assert!(result.is_err());
    match result.unwrap_err() {
        PolicyError::ValidationError { message } => {
            assert!(message.contains("Field not found: nonexistent"));
        }
        other => panic!("Expected ValidationError, got {other:?}"),
    }
}

#[test]
fn test_resolve_value_with_infinity_number() {
    let context = create_test_context();
    let value = Value::Number(f64::INFINITY);

    let result = ActionExecutor::resolve_value(&value, &context);

    assert!(result.is_err());
    match result.unwrap_err() {
        PolicyError::ValidationError { message } => {
            assert!(message.contains("Number values cannot be infinite or NaN"));
        }
        other => panic!("Expected ValidationError, got {other:?}"),
    }
}

#[test]
fn test_resolve_value_with_nan_number() {
    let context = create_test_context();
    let value = Value::Number(f64::NAN);

    let result = ActionExecutor::resolve_value(&value, &context);

    assert!(result.is_err());
    match result.unwrap_err() {
        PolicyError::ValidationError { message } => {
            assert!(message.contains("Number values cannot be infinite or NaN"));
        }
        other => panic!("Expected ValidationError, got {other:?}"),
    }
}

#[test]
fn test_resolve_value_with_regex_pattern() {
    let context = create_test_context();
    let value = Value::Regex("invalid[regex".to_string());

    let result = ActionExecutor::resolve_value(&value, &context);

    assert!(result.is_err());
    // The exact error message depends on the regex implementation
    assert!(result.unwrap_err().to_string().contains("regex"));
}
