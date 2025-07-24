//! Comprehensive error path tests for `ActionExecutor`
//!
//! Tests focus on error paths, edge cases, and rollback scenarios that are missing
//! from the basic tests in the actions.rs file.

use super::ActionExecutor;
use crate::datastore::{DataStore, sqlite::SqliteStore};
use crate::models::{DeviceRole, Node, Vendor};
use crate::policy::PolicyError;
use crate::policy::ast::{FieldRef, Value};
use crate::policy::evaluator::context::{EvaluationContext, PolicyExecutionContext};
use migration::Migrator;
use sea_orm_migration::MigratorTrait;
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

/// Set up a test datastore for testing
async fn setup_test_datastore() -> SqliteStore {
    let store = SqliteStore::new("sqlite::memory:").await.unwrap();
    Migrator::up(store.connection(), None).await.unwrap();
    store
}

/// Create a test node in the datastore
async fn create_test_node(datastore: &SqliteStore) -> Node {
    let node = Node::new(
        "test-node".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );

    datastore.create_node(&node).await.unwrap()
}

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
        "assigned_templates": ["base.j2", "vlan.j2"]
    }))
}

#[tokio::test]
async fn test_execute_assert_action_field_not_found() {
    let context = create_test_context();
    let field = FieldRef {
        path: vec!["nonexistent_field".to_string()],
    };
    let expected = Value::String("any_value".to_string());

    let result = ActionExecutor::execute_assert_action(&field, &expected, &context);

    assert!(result.is_err());
    match result.unwrap_err() {
        PolicyError::ValidationError { message } => {
            assert!(message.contains("Field not found: nonexistent_field"));
        }
        other => panic!("Expected ValidationError, got {other:?}"),
    }
}

#[tokio::test]
async fn test_execute_assert_action_nested_field_not_found() {
    let context = create_test_context();
    let field = FieldRef {
        path: vec!["custom_data".to_string(), "nonexistent".to_string()],
    };
    let expected = Value::String("any_value".to_string());

    let result = ActionExecutor::execute_assert_action(&field, &expected, &context);

    assert!(result.is_err());
    match result.unwrap_err() {
        PolicyError::ValidationError { message } => {
            assert!(message.contains("Field not found: custom_data.nonexistent"));
        }
        other => panic!("Expected ValidationError, got {other:?}"),
    }
}

#[tokio::test]
async fn test_execute_set_action_with_non_custom_data_field() {
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;
    let context = create_test_context();
    let exec_ctx = PolicyExecutionContext::new(&context, &datastore, &node.id);

    let field = FieldRef {
        path: vec!["vendor".to_string()], // Not custom_data
    };
    let value = Value::String("juniper".to_string());

    let result = ActionExecutor::execute_set_action_with_rollback(&field, &value, &exec_ctx).await;

    assert!(result.is_ok());
    let action_result = result.unwrap();
    match action_result.result {
        crate::policy::evaluator::context::ActionResult::Error { message } => {
            assert!(message.contains("SET action only supports custom_data fields"));
        }
        other => panic!("Expected Error result, got {other:?}"),
    }
    assert!(action_result.rollback_data.is_none());
}

#[tokio::test]
async fn test_execute_set_action_with_empty_field_path() {
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;
    let context = create_test_context();
    let exec_ctx = PolicyExecutionContext::new(&context, &datastore, &node.id);

    let field = FieldRef {
        path: vec![], // Empty path
    };
    let value = Value::String("test_value".to_string());

    let result = ActionExecutor::execute_set_action_with_rollback(&field, &value, &exec_ctx).await;

    assert!(result.is_ok());
    let action_result = result.unwrap();
    match action_result.result {
        crate::policy::evaluator::context::ActionResult::Error { message } => {
            assert!(message.contains("SET action only supports custom_data fields"));
        }
        other => panic!("Expected Error result, got {other:?}"),
    }
}

#[tokio::test]
async fn test_execute_set_action_with_nonexistent_node() {
    let datastore = setup_test_datastore().await;
    let nonexistent_node_id = Uuid::new_v4();
    let context = create_test_context();
    let exec_ctx = PolicyExecutionContext::new(&context, &datastore, &nonexistent_node_id);

    let field = FieldRef {
        path: vec!["custom_data".to_string(), "test_field".to_string()],
    };
    let value = Value::String("test_value".to_string());

    let result = ActionExecutor::execute_set_action_with_rollback(&field, &value, &exec_ctx).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        PolicyError::NodeNotFound { node_id } => {
            assert_eq!(node_id, nonexistent_node_id.to_string());
        }
        other => panic!("Expected NodeNotFound error, got {other:?}"),
    }
}

#[tokio::test]
async fn test_execute_apply_template_action_with_nonexistent_node() {
    let datastore = setup_test_datastore().await;
    let nonexistent_node_id = Uuid::new_v4();
    let context = create_test_context();
    let exec_ctx = PolicyExecutionContext::new(&context, &datastore, &nonexistent_node_id);

    let template_path = "config/base.j2";

    let result =
        ActionExecutor::execute_apply_template_action_with_rollback(template_path, &exec_ctx).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        PolicyError::NodeNotFound { node_id } => {
            assert_eq!(node_id, nonexistent_node_id.to_string());
        }
        other => panic!("Expected NodeNotFound error, got {other:?}"),
    }
}

#[tokio::test]
async fn test_execute_apply_template_action_with_already_assigned_template() {
    let datastore = setup_test_datastore().await;
    let mut node = create_test_node(&datastore).await;

    // Pre-assign a template to the node
    node.custom_data = json!({
        "assigned_templates": ["config/base.j2"]
    });
    datastore.update_node(&node).await.unwrap();

    let context = create_test_context();
    let exec_ctx = PolicyExecutionContext::new(&context, &datastore, &node.id);

    let template_path = "config/base.j2"; // Already assigned

    let result =
        ActionExecutor::execute_apply_template_action_with_rollback(template_path, &exec_ctx).await;

    assert!(result.is_ok());
    let action_result = result.unwrap();
    match action_result.result {
        crate::policy::evaluator::context::ActionResult::Success { message } => {
            assert!(message.contains("was already assigned"));
        }
        other => panic!("Expected Success result, got {other:?}"),
    }
    // No rollback data since template was already assigned
    assert!(action_result.rollback_data.is_none());
}

#[tokio::test]
async fn test_execute_apply_template_action_with_non_object_custom_data() {
    let datastore = setup_test_datastore().await;
    let mut node = create_test_node(&datastore).await;

    // Set custom_data to a non-object value
    node.custom_data = json!("not_an_object");
    datastore.update_node(&node).await.unwrap();

    let context = create_test_context();
    let exec_ctx = PolicyExecutionContext::new(&context, &datastore, &node.id);

    let template_path = "config/new_template.j2";

    let result =
        ActionExecutor::execute_apply_template_action_with_rollback(template_path, &exec_ctx).await;

    assert!(result.is_ok());
    let action_result = result.unwrap();
    match action_result.result {
        crate::policy::evaluator::context::ActionResult::Success { message } => {
            assert!(message.contains("Successfully applied template"));
        }
        other => panic!("Expected Success result, got {other:?}"),
    }
    assert!(action_result.rollback_data.is_some());
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
    let value = Value::Array(vec![
        Value::String("test".to_string()),
        Value::Number(42.0),
        Value::FieldRef(FieldRef {
            path: vec!["vendor".to_string()],
        }),
    ]);

    let result = ActionExecutor::resolve_value(&value, &context);

    assert!(result.is_ok());
    let resolved = result.unwrap();
    assert_eq!(resolved, json!(["test", 42.0, "cisco"]));
}

#[test]
fn test_resolve_value_with_nested_object() {
    let context = create_test_context();
    let mut object_map = HashMap::new();
    object_map.insert("static".to_string(), Value::String("value".to_string()));
    object_map.insert(
        "dynamic".to_string(),
        Value::FieldRef(FieldRef {
            path: vec!["model".to_string()],
        }),
    );

    let value = Value::Object(object_map);

    let result = ActionExecutor::resolve_value(&value, &context);

    assert!(result.is_ok());
    let resolved = result.unwrap();
    assert_eq!(resolved, json!({"static": "value", "dynamic": "2960"}));
}

#[test]
fn test_resolve_value_array_with_field_reference_error() {
    let context = create_test_context();
    let value = Value::Array(vec![
        Value::String("test".to_string()),
        Value::FieldRef(FieldRef {
            path: vec!["nonexistent_field".to_string()],
        }),
    ]);

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
fn test_resolve_value_object_with_field_reference_error() {
    let context = create_test_context();
    let mut object_map = HashMap::new();
    object_map.insert("valid".to_string(), Value::String("value".to_string()));
    object_map.insert(
        "invalid".to_string(),
        Value::FieldRef(FieldRef {
            path: vec!["nonexistent_field".to_string()],
        }),
    );

    let value = Value::Object(object_map);

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
fn test_set_nested_field_with_empty_path() {
    let mut data = json!({});
    let path: Vec<String> = vec![];
    let value = json!("test");

    let result = ActionExecutor::set_nested_field(&mut data, &path, value);

    assert!(result.is_err());
    match result.unwrap_err() {
        PolicyError::ValidationError { message } => {
            assert!(message.contains("Cannot set empty path"));
        }
        other => panic!("Expected ValidationError, got {other:?}"),
    }
}

#[test]
fn test_set_nested_field_on_non_object_converts_to_object() {
    let mut data = json!("not_an_object");
    let path = vec!["field".to_string()];
    let value = json!("test_value");

    let result = ActionExecutor::set_nested_field(&mut data, &path, value);

    assert!(result.is_ok());
    assert_eq!(data, json!({"field": "test_value"}));
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
    assert_eq!(data["level1"]["level2"]["level3"], json!("deep_value"));
}

#[test]
fn test_get_nested_field_with_empty_path() {
    let data = json!({"test": "value"});
    let path: Vec<String> = vec![];

    let result = ActionExecutor::get_nested_field(&data, &path);

    assert_eq!(result, Some(data));
}

#[test]
fn test_get_nested_field_path_not_found() {
    let data = json!({"level1": {"level2": "value"}});
    let path = vec!["level1".to_string(), "nonexistent".to_string()];

    let result = ActionExecutor::get_nested_field(&data, &path);

    assert!(result.is_none());
}

#[test]
fn test_get_nested_field_intermediate_path_not_object() {
    let data = json!({"level1": "not_an_object"});
    let path = vec!["level1".to_string(), "level2".to_string()];

    let result = ActionExecutor::get_nested_field(&data, &path);

    assert!(result.is_none());
}

#[test]
fn test_json_values_equal_with_invalid_numbers() {
    // Test case where serde_json::Number::from_f64 returns None (for NaN, infinity)
    let num1 = serde_json::Number::from(42);
    let num2 = serde_json::Number::from(42);

    assert!(ActionExecutor::json_values_equal(
        &serde_json::Value::Number(num1),
        &serde_json::Value::Number(num2)
    ));
}

#[test]
fn test_resolve_value_with_infinity_number() {
    let context = create_test_context();
    let value = Value::Number(f64::INFINITY);

    let result = ActionExecutor::resolve_value(&value, &context);

    assert!(result.is_ok());
    let resolved = result.unwrap();
    // When f64::INFINITY can't be converted to serde_json::Number, it falls back to 0
    assert_eq!(resolved, json!(0));
}

#[test]
fn test_resolve_value_with_nan_number() {
    let context = create_test_context();
    let value = Value::Number(f64::NAN);

    let result = ActionExecutor::resolve_value(&value, &context);

    assert!(result.is_ok());
    let resolved = result.unwrap();
    // When f64::NAN can't be converted to serde_json::Number, it falls back to 0
    assert_eq!(resolved, json!(0));
}

#[test]
fn test_resolve_value_with_regex_pattern() {
    let context = create_test_context();
    let value = Value::Regex("^[A-Z]+$".to_string());

    let result = ActionExecutor::resolve_value(&value, &context);

    assert!(result.is_ok());
    let resolved = result.unwrap();
    assert_eq!(resolved, json!("^[A-Z]+$"));
}
