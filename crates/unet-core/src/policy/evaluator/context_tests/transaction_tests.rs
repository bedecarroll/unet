//! Tests for `PolicyTransaction` and rollback functionality

use super::super::context::*;
use crate::policy::ast::FieldRef;
use serde_json::json;
use std::time::Duration;
use uuid::Uuid;

#[test]
fn test_policy_transaction_new() {
    let transaction_id = "tx_123456".to_string();
    let node_id = Uuid::new_v4();

    let transaction = PolicyTransaction::new(transaction_id.clone(), node_id);

    assert_eq!(transaction.transaction_id, transaction_id);
    assert_eq!(transaction.node_id, node_id);
    assert!(transaction.rollback_stack.is_empty());
    assert!(transaction.original_node_state.is_none());
    // Check that started_at is recent (within 1 second)
    assert!(transaction.started_at.elapsed() < Duration::from_secs(1));
}

#[test]
fn test_policy_transaction_add_rollback() {
    let mut transaction = PolicyTransaction::new("tx_123".to_string(), Uuid::new_v4());

    let rollback1 = RollbackData::SetRollback {
        field: FieldRef {
            path: vec!["version".to_string()],
        },
        previous_value: Some(json!("14.2")),
    };

    let rollback2 = RollbackData::ApplyRollback {
        template_path: "/templates/config.j2".to_string(),
    };

    transaction.add_rollback(rollback1);
    transaction.add_rollback(rollback2);

    assert_eq!(transaction.rollback_stack.len(), 2);

    // Check order (should be in reverse order for rollback)
    match &transaction.rollback_stack[0] {
        RollbackData::SetRollback { .. } => {}
        _ => panic!("Expected SetRollback as first item"),
    }

    match &transaction.rollback_stack[1] {
        RollbackData::ApplyRollback { .. } => {}
        _ => panic!("Expected ApplyRollback as second item"),
    }
}

#[test]
fn test_policy_transaction_set_original_state() {
    let mut transaction = PolicyTransaction::new("tx_123".to_string(), Uuid::new_v4());

    let original_state = json!({
        "version": "14.2",
        "config": {
            "vlan": 10,
            "port_config": "auto"
        }
    });

    transaction.set_original_state(original_state.clone());

    assert_eq!(transaction.original_node_state, Some(original_state));
}
