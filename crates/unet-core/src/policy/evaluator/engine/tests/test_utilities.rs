//! Test utilities for policy evaluation engine tests

use crate::datastore::{DataStore, sqlite::SqliteStore};
use crate::models::{DeviceRole, Node, Vendor};
use crate::policy::ast::*;
use crate::policy::evaluator::context::EvaluationContext;
use migration::Migrator;
use sea_orm_migration::MigratorTrait;
use serde_json::json;

/// Set up a test datastore for testing
pub async fn setup_test_datastore() -> SqliteStore {
    let store = SqliteStore::new("sqlite::memory:").await.unwrap();
    Migrator::up(store.connection(), None).await.unwrap();
    store
}

/// Create a test node in the datastore
pub async fn create_test_node(datastore: &SqliteStore) -> Node {
    let node = Node::new(
        "test-node".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );

    datastore.create_node(&node).await.unwrap();
    node
}

/// Create a simple test policy rule that always evaluates to true
pub fn create_always_true_rule() -> PolicyRule {
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
pub fn create_always_false_rule() -> PolicyRule {
    PolicyRule {
        id: Some("always_false".to_string()),
        condition: Condition::False,
        action: Action::Assert {
            field: FieldRef {
                path: vec!["vendor".to_string()],
            },
            expected: Value::String("cisco".to_string()),
        },
    }
}

/// Create a test policy rule with a comparison condition
pub fn create_comparison_rule() -> PolicyRule {
    PolicyRule {
        id: Some("vendor_comparison".to_string()),
        condition: Condition::Comparison {
            field: FieldRef {
                path: vec!["vendor".to_string()],
            },
            operator: ComparisonOperator::Equal,
            value: Value::String("cisco".to_string()),
        },
        action: Action::Set {
            field: FieldRef {
                path: vec!["model".to_string()],
            },
            value: Value::String("ISR4431".to_string()),
        },
    }
}

/// Create test evaluation context
pub fn create_test_context() -> EvaluationContext {
    EvaluationContext::new(json!({
        "vendor": "cisco",
        "model": "2960",
        "role": "switch"
    }))
}
