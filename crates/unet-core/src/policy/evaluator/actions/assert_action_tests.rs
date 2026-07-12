//! Tests for assert action error paths and edge cases

use crate::policy::PolicyError;
use crate::policy::ast::{FieldRef, Value};
use crate::policy::evaluator::actions::ActionExecutor;
use crate::policy::evaluator::context::EvaluationContext;
use serde_json::json;

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
