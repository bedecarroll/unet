//! Tests for nested field manipulation error paths and edge cases

use crate::policy::PolicyError;
use crate::policy::evaluator::actions::ActionExecutor;
use serde_json::json;

#[test]
fn test_set_nested_field_with_empty_path() {
    let mut data = json!({"existing": "value"});
    let path = vec![];
    let value = json!("new_value");

    let result = ActionExecutor::set_nested_field(&mut data, &path, value);

    assert!(result.is_err());
    match result.unwrap_err() {
        PolicyError::ValidationError { message } => {
            assert!(message.contains("Field path cannot be empty"));
        }
        other => panic!("Expected ValidationError, got {other:?}"),
    }
}

#[test]
fn test_set_nested_field_on_non_object_converts_to_object() {
    let mut data = json!("string_value");
    let path = vec!["new_field".to_string()];
    let value = json!("field_value");

    let result = ActionExecutor::set_nested_field(&mut data, &path, value);

    assert!(result.is_ok());
    assert_eq!(data, json!({"new_field": "field_value"}));
}

#[test]
fn test_set_nested_field_deep_path_creates_nested_structure() {
    let mut data = json!({});
    let path = vec![
        "level1".to_string(),
        "level2".to_string(),
        "level3".to_string(),
    ];
    let value = json!("deep_value");

    let result = ActionExecutor::set_nested_field(&mut data, &path, value);

    assert!(result.is_ok());
    assert_eq!(
        data,
        json!({"level1": {"level2": {"level3": "deep_value"}}})
    );
}

#[test]
fn test_get_nested_field_with_empty_path() {
    let data = json!({"field": "value"});
    let path = vec![];

    let result = ActionExecutor::get_nested_field(&data, &path);

    assert!(result.is_some());
    // With empty path, it should return the root data
    assert_eq!(result.unwrap(), json!({"field": "value"}));
}

#[test]
fn test_get_nested_field_path_not_found() {
    let data = json!({"existing": "value"});
    let path = vec!["nonexistent".to_string()];

    let result = ActionExecutor::get_nested_field(&data, &path);

    assert!(result.is_none());
    // Field not found should return None
}

#[test]
fn test_get_nested_field_intermediate_path_not_object() {
    let data = json!({"field": "string_value"});
    let path = vec!["field".to_string(), "nested".to_string()];

    let result = ActionExecutor::get_nested_field(&data, &path);

    assert!(result.is_none());
    // Trying to access nested field on non-object should return None
}
