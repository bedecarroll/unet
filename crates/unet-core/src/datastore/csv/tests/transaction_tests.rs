//! Tests for CSV transaction operations

use super::super::store::{CsvData, CsvStore};
use super::super::transaction::CsvTransaction;
use crate::datastore::DataStore;
use crate::datastore::types::{DataStoreError, Transaction};
use crate::models::{DeviceRole, Lifecycle, Link, Location, Node, NodeBuilder, Vendor};
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::Mutex;
use uuid::Uuid;

async fn setup_test_store() -> (CsvStore, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let store = CsvStore::new(temp_dir.path().to_path_buf()).await.unwrap();
    (store, temp_dir)
}

fn create_test_node() -> Node {
    NodeBuilder::new()
        .name("test-node".to_string())
        .domain("example.com".to_string())
        .vendor(Vendor::Cisco)
        .model("Test Model".to_string())
        .role(DeviceRole::Router)
        .lifecycle(Lifecycle::Live)
        .build()
        .unwrap()
}

fn create_test_location() -> Location {
    Location::new_root("Test Location".to_string(), "Building".to_string())
}

fn create_test_link(source_id: Uuid, target_id: Uuid) -> Link {
    Link::new(
        "test-link".to_string(),
        source_id,
        "eth0".to_string(),
        target_id,
        "eth1".to_string(),
    )
}

#[tokio::test]
async fn test_transaction_commit_nodes() {
    let (store, _temp_dir) = setup_test_store().await;
    let store_arc = Arc::new(store);

    let node = create_test_node();
    let node_id = node.id;

    // Create transaction with changes
    let mut changes = CsvData::default();
    changes.nodes.insert(node_id, node.clone());

    let transaction = CsvTransaction {
        store: store_arc.clone(),
        changes: Mutex::new(changes),
        committed: Mutex::new(false),
    };

    // Commit transaction
    let result = Box::new(transaction).commit().await;
    assert!(result.is_ok());

    // Verify node was added to store
    let stored_node = store_arc.get_node(&node_id).await.unwrap();
    assert!(stored_node.is_some());
    assert_eq!(stored_node.unwrap().name, "test-node");
}

#[tokio::test]
async fn test_transaction_commit_links() {
    let (store, _temp_dir) = setup_test_store().await;
    let store_arc = Arc::new(store);

    let source_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();
    let link = create_test_link(source_id, target_id);
    let link_id = link.id;

    // Create transaction with changes
    let mut changes = CsvData::default();
    changes.links.insert(link_id, link.clone());

    let transaction = CsvTransaction {
        store: store_arc.clone(),
        changes: Mutex::new(changes),
        committed: Mutex::new(false),
    };

    // Commit transaction
    let result = Box::new(transaction).commit().await;
    assert!(result.is_ok());

    // Verify link was added to store
    let stored_link = store_arc.get_link(&link_id).await.unwrap();
    assert!(stored_link.is_some());
    assert_eq!(stored_link.unwrap().source_node_id, source_id);
}

#[tokio::test]
async fn test_transaction_commit_locations() {
    let (store, _temp_dir) = setup_test_store().await;
    let store_arc = Arc::new(store);

    let location = create_test_location();
    let location_id = location.id;

    // Create transaction with changes
    let mut changes = CsvData::default();
    changes.locations.insert(location_id, location.clone());

    let transaction = CsvTransaction {
        store: store_arc.clone(),
        changes: Mutex::new(changes),
        committed: Mutex::new(false),
    };

    // Commit transaction
    let result = Box::new(transaction).commit().await;
    assert!(result.is_ok());

    // Verify location was added to store
    let stored_location = store_arc.get_location(&location_id).await.unwrap();
    assert!(stored_location.is_some());
    assert_eq!(stored_location.unwrap().name, "Test Location");
}

#[tokio::test]
async fn test_transaction_commit_mixed_entities() {
    let (store, _temp_dir) = setup_test_store().await;
    let store_arc = Arc::new(store);

    let node = create_test_node();
    let location = create_test_location();
    let link = create_test_link(node.id, Uuid::new_v4());

    // Create transaction with mixed changes
    let mut changes = CsvData::default();
    changes.nodes.insert(node.id, node.clone());
    changes.locations.insert(location.id, location.clone());
    changes.links.insert(link.id, link.clone());

    let transaction = CsvTransaction {
        store: store_arc.clone(),
        changes: Mutex::new(changes),
        committed: Mutex::new(false),
    };

    // Commit transaction
    let result = Box::new(transaction).commit().await;
    assert!(result.is_ok());

    // Verify all entities were added
    assert!(store_arc.get_node(&node.id).await.unwrap().is_some());
    assert!(
        store_arc
            .get_location(&location.id)
            .await
            .unwrap()
            .is_some()
    );
    assert!(store_arc.get_link(&link.id).await.unwrap().is_some());
}

#[tokio::test]
async fn test_transaction_commit_already_committed() {
    let (store, _temp_dir) = setup_test_store().await;
    let store_arc = Arc::new(store);

    let transaction = CsvTransaction {
        store: store_arc,
        changes: Mutex::new(CsvData::default()),
        committed: Mutex::new(true), // Already committed
    };

    // Attempt to commit already committed transaction
    let result = Box::new(transaction).commit().await;
    assert!(result.is_err());

    match result.unwrap_err() {
        DataStoreError::TransactionError { message } => {
            assert!(message.contains("already committed or rolled back"));
        }
        _ => panic!("Expected TransactionError"),
    }
}

#[tokio::test]
async fn test_transaction_rollback() {
    let (store, _temp_dir) = setup_test_store().await;
    let store_arc = Arc::new(store);

    let node = create_test_node();
    let mut changes = CsvData::default();
    changes.nodes.insert(node.id, node);

    let transaction = CsvTransaction {
        store: store_arc.clone(),
        changes: Mutex::new(changes),
        committed: Mutex::new(false),
    };

    // Rollback transaction
    let result = Box::new(transaction).rollback().await;
    assert!(result.is_ok());

    // Verify no changes were applied to store
    assert!(store_arc.data.lock().await.nodes.is_empty());
}

#[tokio::test]
async fn test_transaction_rollback_already_committed() {
    let (store, _temp_dir) = setup_test_store().await;
    let store_arc = Arc::new(store);

    let transaction = CsvTransaction {
        store: store_arc,
        changes: Mutex::new(CsvData::default()),
        committed: Mutex::new(true), // Already committed
    };

    // Attempt to rollback already committed transaction
    let result = Box::new(transaction).rollback().await;
    assert!(result.is_err());

    match result.unwrap_err() {
        DataStoreError::TransactionError { message } => {
            assert!(message.contains("already committed or rolled back"));
        }
        _ => panic!("Expected TransactionError"),
    }
}

#[tokio::test]
async fn test_transaction_commit_overwrites_existing() {
    let (store, _temp_dir) = setup_test_store().await;
    let store_arc = Arc::new(store);

    // First, add a node directly to the store
    let original_node = create_test_node();
    let node_id = original_node.id;
    {
        let mut data = store_arc.data.lock().await;
        data.nodes.insert(node_id, original_node.clone());
    }

    // Create a modified version in transaction
    let mut modified_node = original_node;
    modified_node.name = "modified-node".to_string();

    let mut changes = CsvData::default();
    changes.nodes.insert(node_id, modified_node.clone());

    let transaction = CsvTransaction {
        store: store_arc.clone(),
        changes: Mutex::new(changes),
        committed: Mutex::new(false),
    };

    // Commit transaction
    let result = Box::new(transaction).commit().await;
    assert!(result.is_ok());

    // Verify the node was updated
    let stored_node = store_arc.get_node(&node_id).await.unwrap().unwrap();
    assert_eq!(stored_node.name, "modified-node");
}

#[tokio::test]
async fn test_transaction_empty_changes() {
    let (store, _temp_dir) = setup_test_store().await;
    let store_arc = Arc::new(store);

    let transaction = CsvTransaction {
        store: store_arc.clone(),
        changes: Mutex::new(CsvData::default()),
        committed: Mutex::new(false),
    };

    // Commit transaction with no changes
    let result = Box::new(transaction).commit().await;
    assert!(result.is_ok());

    // Store should remain empty
    let data = store_arc.data.lock().await;
    assert!(data.nodes.is_empty());
    assert!(data.links.is_empty());
    assert!(data.locations.is_empty());
    drop(data);
}
