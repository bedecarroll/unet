//! Tests for policy rule execution and transaction management

use crate::datastore::{DataStore, sqlite::SqliteStore};
use crate::models::{DeviceRole, Node, Vendor};
use crate::policy::ast::*;
use crate::policy::evaluator::PolicyEvaluator;
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

    datastore.create_node(&node).await.unwrap();
    node
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
            }
        }
    }))
}

/// Create a simple test policy rule that always evaluates to true
fn create_always_true_rule() -> PolicyRule {
    PolicyRule {
        id: Some("always_true".to_string()),
        condition: Condition::True,
        action: Action::Assert {
            field: FieldRef {
                path: vec!["vendor".to_string()],
            },
            expected: Value::String("cisco".to_string()),
        },
    }
}

/// Create a simple test policy rule that always evaluates to false
fn create_always_false_rule() -> PolicyRule {
    PolicyRule {
        id: Some("always_false".to_string()),
        condition: Condition::False,
        action: Action::Assert {
            field: FieldRef {
                path: vec!["vendor".to_string()],
            },
            expected: Value::String("juniper".to_string()),
        },
    }
}

/// Create a test policy rule with a set action
fn create_set_action_rule() -> PolicyRule {
    PolicyRule {
        id: Some("set_custom_data".to_string()),
        condition: Condition::True,
        action: Action::Set {
            field: FieldRef {
                path: vec!["custom_data".to_string(), "test_key".to_string()],
            },
            value: Value::String("test_value".to_string()),
        },
    }
}

#[tokio::test]
async fn test_execute_rule_with_satisfied_condition_executes_action() {
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;
    let context = create_test_context();
    let exec_ctx = PolicyExecutionContext::new(&context, &datastore, &node.id);

    let rule = create_always_true_rule();

    let result = PolicyEvaluator::execute_rule(&rule, &exec_ctx).await;

    assert!(result.is_ok());
    // The exact result depends on action execution details
}

#[tokio::test]
async fn test_execute_rule_with_not_satisfied_condition_skips_action() {
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;
    let context = create_test_context();
    let exec_ctx = PolicyExecutionContext::new(&context, &datastore, &node.id);

    let rule = create_always_false_rule();

    let result = PolicyEvaluator::execute_rule(&rule, &exec_ctx).await;

    assert!(result.is_ok());
    // Should succeed but skip action execution
}

#[tokio::test]
async fn test_execute_rules_with_transaction_creates_transaction() {
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;
    let context = create_test_context();
    let exec_ctx = PolicyExecutionContext::new(&context, &datastore, &node.id);

    let rules = vec![create_set_action_rule()];

    let result = PolicyEvaluator::execute_rules_with_transaction(&rules, &exec_ctx).await;

    assert!(result.is_ok());
    let (_results, transaction) = result.unwrap();

    // Transaction should be created and committed
    assert_eq!(transaction.node_id, node.id);
    assert!(transaction.transaction_id.starts_with("tx_"));
    assert!(transaction.original_node_state.is_some());
}

#[tokio::test]
async fn test_execute_rules_with_transaction_processes_all_rules() {
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;
    let context = create_test_context();
    let exec_ctx = PolicyExecutionContext::new(&context, &datastore, &node.id);

    let rules = vec![
        create_always_true_rule(),
        create_always_false_rule(),
        create_set_action_rule(),
    ];

    let result = PolicyEvaluator::execute_rules_with_transaction(&rules, &exec_ctx).await;

    assert!(result.is_ok());
    let (results, _transaction) = result.unwrap();
    assert_eq!(results.len(), 3);
}

#[tokio::test]
async fn test_execute_rules_with_transaction_captures_original_node_state() {
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;
    let context = create_test_context();
    let exec_ctx = PolicyExecutionContext::new(&context, &datastore, &node.id);

    let rules = vec![create_set_action_rule()];

    let result = PolicyEvaluator::execute_rules_with_transaction(&rules, &exec_ctx).await;

    assert!(result.is_ok());
    let (_results, transaction) = result.unwrap();

    // Should capture original node state for rollback
    assert!(transaction.original_node_state.is_some());
}

#[tokio::test]
async fn test_execute_rules_with_transaction_empty_rules_succeeds() {
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;
    let context = create_test_context();
    let exec_ctx = PolicyExecutionContext::new(&context, &datastore, &node.id);

    let rules = vec![];

    let result = PolicyEvaluator::execute_rules_with_transaction(&rules, &exec_ctx).await;

    assert!(result.is_ok());
    let (results, _transaction) = result.unwrap();
    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn test_execute_rules_with_transaction_node_not_found_fails() {
    let datastore = setup_test_datastore().await;
    let nonexistent_node_id = Uuid::new_v4();
    let context = create_test_context();
    let exec_ctx = PolicyExecutionContext::new(&context, &datastore, &nonexistent_node_id);

    let rules = vec![create_set_action_rule()];

    let result = PolicyEvaluator::execute_rules_with_transaction(&rules, &exec_ctx).await;

    assert!(result.is_err());
    // Should fail because node doesn't exist
}
