//! Tests for apply template action error paths and edge cases

use crate::datastore::{DataStore, sqlite::SqliteStore};
use crate::models::{DeviceRole, Node, Vendor};
use crate::policy::PolicyError;
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
async fn test_execute_apply_template_action_with_nonexistent_node() {
    let datastore = setup_test_datastore().await;
    let nonexistent_node_id = Uuid::new_v4();
    let context = create_test_context();
    let exec_ctx = PolicyExecutionContext::new(&context, &datastore, &nonexistent_node_id);

    let template_path = "test_template";

    let result =
        ActionExecutor::execute_apply_template_action_with_rollback(template_path, &exec_ctx).await;

    assert!(result.is_err());
    // The exact error type depends on datastore implementation
    // but it should be an error since the node doesn't exist
}

#[tokio::test]
async fn test_execute_apply_template_action_with_already_assigned_template() {
    let datastore = setup_test_datastore().await;
    let mut node = create_test_node(&datastore).await;

    // Set custom_data with a template already assigned
    node.custom_data = json!({
        "template": "existing_template",
        "other_data": "value"
    });
    datastore.update_node(&node).await.unwrap();

    let context = create_test_context();
    let exec_ctx = PolicyExecutionContext::new(&context, &datastore, &node.id);

    let template_path = "new_template";

    let result =
        ActionExecutor::execute_apply_template_action_with_rollback(template_path, &exec_ctx).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        PolicyError::ValidationError { message } => {
            assert!(message.contains("Template already assigned to node"));
        }
        other => panic!("Expected ValidationError, got {other:?}"),
    }
}

#[tokio::test]
async fn test_execute_apply_template_action_with_non_object_custom_data() {
    let datastore = setup_test_datastore().await;
    let mut node = create_test_node(&datastore).await;

    // Set custom_data to a non-object value
    node.custom_data = json!("invalid_custom_data");
    datastore.update_node(&node).await.unwrap();

    let context = create_test_context();
    let exec_ctx = PolicyExecutionContext::new(&context, &datastore, &node.id);

    let template_path = "test_template";

    let result =
        ActionExecutor::execute_apply_template_action_with_rollback(template_path, &exec_ctx).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        PolicyError::ValidationError { message } => {
            assert!(message.contains("custom_data must be an object"));
        }
        other => panic!("Expected ValidationError, got {other:?}"),
    }
}
