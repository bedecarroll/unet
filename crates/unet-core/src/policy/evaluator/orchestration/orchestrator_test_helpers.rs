//! Shared test helper functions for orchestrator tests

use crate::datastore::sqlite::SqliteStore;
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

/// Create a simple test policy rule for testing
pub fn create_test_rule() -> PolicyRule {
    PolicyRule {
        id: Some("test_rule_1".to_string()),
        condition: Condition::Comparison {
            field: FieldRef {
                path: vec!["node".to_string(), "vendor".to_string()],
            },
            operator: ComparisonOperator::Equal,
            value: Value::String("cisco".to_string()),
        },
        action: Action::Assert {
            field: FieldRef {
                path: vec!["node".to_string(), "vendor".to_string()],
            },
            expected: Value::String("cisco".to_string()),
        },
    }
}

/// Create test evaluation context
pub fn create_test_context() -> EvaluationContext {
    EvaluationContext::new(json!({
        "node": {
            "vendor": "cisco",
            "model": "2960"
        }
    }))
}
