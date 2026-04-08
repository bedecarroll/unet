//! Tests for shared `DataStore` test helpers.

use super::{RecordingTransaction, SeededDataStore, TransactionTracker};
use crate::datastore::{DataStore, DataStoreError, QueryOptions, Transaction};
use crate::models::{DeviceRole, Node, Vendor};

fn test_node() -> Node {
    Node::new(
        "seed-node".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    )
}

#[tokio::test]
async fn seeded_datastore_returns_descriptive_error_for_unconfigured_write() {
    let store = SeededDataStore::new();

    let error = store
        .create_node(&test_node())
        .await
        .expect_err("unconfigured create_node should fail");

    match error {
        DataStoreError::UnsupportedOperation { operation } => {
            assert!(operation.contains("SeededDataStore"));
            assert!(operation.contains("create_node"));
        }
        other => panic!("Expected UnsupportedOperation, got {other:?}"),
    }
}

#[tokio::test]
async fn seeded_datastore_lists_seeded_nodes() {
    let node = test_node();
    let store = SeededDataStore::new().with_node(node.clone());

    let result = store
        .list_nodes(&QueryOptions::default())
        .await
        .expect("seeded list_nodes should succeed");

    assert_eq!(result.items, vec![node]);
    assert_eq!(result.total_count, 1);
}

#[tokio::test]
async fn recording_transaction_tracks_commit_and_rollback() {
    let commit_tracker = TransactionTracker::default();
    let commit_tx = RecordingTransaction::successful(commit_tracker.clone());
    Box::new(commit_tx)
        .commit()
        .await
        .expect("commit should succeed");
    assert!(commit_tracker.snapshot().committed);
    assert!(!commit_tracker.snapshot().rolled_back);

    let rollback_tracker = TransactionTracker::default();
    let rollback_tx = RecordingTransaction::successful(rollback_tracker.clone());
    Box::new(rollback_tx)
        .rollback()
        .await
        .expect("rollback should succeed");
    assert!(!rollback_tracker.snapshot().committed);
    assert!(rollback_tracker.snapshot().rolled_back);
}
