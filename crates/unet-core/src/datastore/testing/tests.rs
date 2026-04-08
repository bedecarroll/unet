//! Tests for shared `DataStore` test helpers.

use super::{RecordingTransaction, SeededDataStore, TransactionTracker};
use crate::datastore::{BatchOperation, DataStore, DataStoreError, QueryOptions, Transaction};
use crate::models::{DeviceRole, Link, Location, Node, Vendor};

fn test_node() -> Node {
    Node::new(
        "seed-node".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    )
}

fn test_link() -> Link {
    Link::new(
        "seed-link".to_string(),
        uuid::Uuid::new_v4(),
        "Ethernet1".to_string(),
        uuid::Uuid::new_v4(),
        "Ethernet2".to_string(),
    )
}

fn test_location() -> Location {
    Location::new_root("lab".to_string(), "site".to_string())
}

fn assert_unsupported_operation<T>(result: Result<T, DataStoreError>, method_name: &str) {
    match result {
        Err(DataStoreError::UnsupportedOperation { operation }) => {
            assert!(operation.contains("SeededDataStore"));
            assert!(operation.contains(method_name));
        }
        Err(other) => panic!("Expected UnsupportedOperation for {method_name}, got {other:?}"),
        Ok(_) => panic!("Expected UnsupportedOperation for {method_name}"),
    }
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

#[tokio::test]
async fn recording_transaction_supports_explicit_commit_and_rollback_results() {
    let commit_tracker = TransactionTracker::default();
    let commit_tx = RecordingTransaction::with_results(
        commit_tracker.clone(),
        Err(DataStoreError::ConnectionError {
            message: "commit failed".to_string(),
        }),
        Ok(()),
    );
    let commit_error = Box::new(commit_tx)
        .commit()
        .await
        .expect_err("commit should return configured error");
    assert!(matches!(
        commit_error,
        DataStoreError::ConnectionError { .. }
    ));
    assert!(commit_tracker.snapshot().committed);

    let rollback_tracker = TransactionTracker::default();
    let rollback_tx = RecordingTransaction::with_results(
        rollback_tracker.clone(),
        Ok(()),
        Err(DataStoreError::ConnectionError {
            message: "rollback failed".to_string(),
        }),
    );
    let rollback_error = Box::new(rollback_tx)
        .rollback()
        .await
        .expect_err("rollback should return configured error");
    assert!(matches!(
        rollback_error,
        DataStoreError::ConnectionError { .. }
    ));
    assert!(rollback_tracker.snapshot().rolled_back);
}

#[tokio::test]
async fn seeded_datastore_supports_seeded_reads_and_updates() {
    let node = test_node();
    let link = test_link();
    let location = test_location();
    let store = SeededDataStore::new()
        .with_node(node.clone())
        .with_link(link.clone())
        .with_location(location.clone());

    store
        .health_check()
        .await
        .expect("health check should succeed");
    assert_unsupported_operation(store.begin_transaction().await, "begin_transaction");

    assert_eq!(
        store
            .get_node(&node.id)
            .await
            .expect("get_node should succeed"),
        Some(node.clone())
    );
    let updated_node = store
        .update_node(&node)
        .await
        .expect("update_node should succeed");
    assert_eq!(updated_node, node);

    assert_eq!(
        store
            .get_link(&link.id)
            .await
            .expect("get_link should succeed"),
        Some(link.clone())
    );
    let listed_links = store
        .list_links(&QueryOptions::default())
        .await
        .expect("list_links should succeed");
    assert_eq!(listed_links.items, vec![link]);

    assert_eq!(
        store
            .get_location(&location.id)
            .await
            .expect("get_location should succeed"),
        Some(location.clone())
    );
    let listed_locations = store
        .list_locations(&QueryOptions::default())
        .await
        .expect("list_locations should succeed");
    assert_eq!(listed_locations.items, vec![location]);
}

#[tokio::test]
async fn seeded_datastore_reports_all_other_operations_as_unsupported() {
    let store = SeededDataStore::new();
    let node = test_node();
    let link = test_link();
    let location = test_location();
    let node_ops: Vec<BatchOperation<Node>> = Vec::new();
    let link_ops: Vec<BatchOperation<Link>> = Vec::new();
    let location_ops: Vec<BatchOperation<Location>> = Vec::new();

    assert_unsupported_operation(store.delete_node(&node.id).await, "delete_node");
    assert_unsupported_operation(
        store.get_nodes_by_location(&uuid::Uuid::new_v4()).await,
        "get_nodes_by_location",
    );
    assert_unsupported_operation(
        store.search_nodes_by_name("seed-node").await,
        "search_nodes_by_name",
    );
    assert_unsupported_operation(store.create_link(&link).await, "create_link");
    assert_unsupported_operation(store.update_link(&link).await, "update_link");
    assert_unsupported_operation(store.delete_link(&link.id).await, "delete_link");
    assert_unsupported_operation(
        store.get_links_for_node(&uuid::Uuid::new_v4()).await,
        "get_links_for_node",
    );
    assert_unsupported_operation(
        store
            .get_links_between_nodes(&uuid::Uuid::new_v4(), &uuid::Uuid::new_v4())
            .await,
        "get_links_between_nodes",
    );
    assert_unsupported_operation(store.create_location(&location).await, "create_location");
    assert_unsupported_operation(store.update_location(&location).await, "update_location");
    assert_unsupported_operation(store.delete_location(&location.id).await, "delete_location");
    assert_unsupported_operation(store.create_vendor("Cisco").await, "create_vendor");
    assert_unsupported_operation(store.list_vendors().await, "list_vendors");
    assert_unsupported_operation(store.delete_vendor("Cisco").await, "delete_vendor");
    assert_unsupported_operation(store.batch_nodes(&node_ops).await, "batch_nodes");
    assert_unsupported_operation(store.batch_links(&link_ops).await, "batch_links");
    assert_unsupported_operation(
        store.batch_locations(&location_ops).await,
        "batch_locations",
    );
    assert_unsupported_operation(store.get_entity_counts().await, "get_entity_counts");
    assert_unsupported_operation(store.get_statistics().await, "get_statistics");
}
