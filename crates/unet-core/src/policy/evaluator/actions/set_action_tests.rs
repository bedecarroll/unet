//! Tests for set action error paths and edge cases

use crate::datastore::{DataStore, sqlite::SqliteStore};
use crate::models::{DeviceRole, Node, Vendor};
use crate::policy::PolicyError;
use crate::policy::ast::{FieldRef, Value};
use crate::policy::evaluator::actions::ActionExecutor;
use crate::policy::evaluator::context::{EvaluationContext, PolicyExecutionContext};
use migration::Migrator;
use sea_orm_migration::MigratorTrait;
use serde_json::json;
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
        "metrics": {
            "cpu_usage": 45.5,
            "memory_usage": 67.2,
            "temperature": 32.1
        }
    }))
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

    assert!(result.is_err());
    match result.unwrap_err() {
        PolicyError::ValidationError { message } => {
            assert!(message.contains("Only custom_data fields can be modified"));
        }
        other => panic!("Expected ValidationError, got {other:?}"),
    }
}

#[tokio::test]
async fn test_execute_set_action_with_empty_field_path() {
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;
    let context = create_test_context();
    let exec_ctx = PolicyExecutionContext::new(&context, &datastore, &node.id);

    let field = FieldRef { path: vec![] }; // Empty path
    let value = Value::String("any_value".to_string());

    let result = ActionExecutor::execute_set_action_with_rollback(&field, &value, &exec_ctx).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        PolicyError::ValidationError { message } => {
            assert!(message.contains("Field path cannot be empty"));
        }
        other => panic!("Expected ValidationError, got {other:?}"),
    }
}

#[tokio::test]
async fn test_execute_set_action_with_nonexistent_node() {
    let datastore = setup_test_datastore().await;
    let nonexistent_node_id = Uuid::new_v4();
    let context = create_test_context();
    let exec_ctx = PolicyExecutionContext::new(&context, &datastore, &nonexistent_node_id);

    let field = FieldRef {
        path: vec!["custom_data".to_string(), "test_key".to_string()],
    };
    let value = Value::String("test_value".to_string());

    let result = ActionExecutor::execute_set_action_with_rollback(&field, &value, &exec_ctx).await;

    assert!(result.is_err());
    // The exact error type depends on datastore implementation
    // but it should be an error since the node doesn't exist
}
