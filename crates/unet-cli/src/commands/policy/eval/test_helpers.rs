//! Shared test helper functions for policy evaluation tests

use migration::{Migrator, MigratorTrait};
use unet_core::{
    datastore::{DataStore, sqlite::SqliteStore},
    models::*,
    policy::{Action, ComparisonOperator, Condition, FieldRef, PolicyRule, Value},
};

pub async fn setup_test_datastore() -> SqliteStore {
    let store = SqliteStore::new("sqlite::memory:").await.unwrap();
    Migrator::up(store.connection(), None).await.unwrap();
    store
}

pub async fn create_test_node(datastore: &SqliteStore) -> Node {
    let mut node = Node::new(
        "test-eval-node".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );
    node.model = "ASR1000".to_string();
    node.lifecycle = Lifecycle::Live;
    datastore.create_node(&node).await.unwrap()
}

pub fn create_test_policy_rule() -> PolicyRule {
    PolicyRule {
        id: Some("test-rule".to_string()),
        condition: Condition::Comparison {
            field: FieldRef {
                path: vec!["vendor".to_string()],
            },
            operator: ComparisonOperator::Equal,
            value: Value::String("cisco".to_string()),
        },
        action: Action::Assert {
            field: FieldRef {
                path: vec!["model".to_string()],
            },
            expected: Value::String("ASR1000".to_string()),
        },
    }
}