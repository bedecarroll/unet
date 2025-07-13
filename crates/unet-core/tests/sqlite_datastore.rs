// Integration test for SQLite datastore
// Moved from root test_sqlite_datastore.rs, converted into proper cargo integration test.

use uuid::Uuid;
use unet_core::datastore::{sqlite::SqliteStore, DataStore};
use unet_core::models::{Node, Vendor, DeviceRole, Lifecycle};

/// End-to-end integration test for SQLite-based DataStore
#[tokio::test]
async fn sqlite_datastore_integration() {
    // Configure a test SQLite database in the crate directory
    let database_url = "sqlite:./test_sqlite_integration.db";
    let store = SqliteStore::new(database_url)
        .await
        .expect("Failed to connect to SQLite database");

    // Health check
    store.health_check().await.expect("Health check failed");

    // Create a test node
    let mut test_node = Node::new(
        "test-router-01".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );
    test_node.model = "ISR4331".to_string();

    // Create
    let created_node = store.create_node(&test_node)
        .await
        .expect("Failed to create node");
    assert_eq!(created_node.id, test_node.id);

    // Retrieve
    let retrieved = store.get_node(&test_node.id)
        .await
        .expect("Error fetching node");
    let node = retrieved.expect("Node not found");
    assert_eq!(node.name, test_node.name);
    assert_eq!(node.vendor, test_node.vendor);
    assert_eq!(node.role, test_node.role);

    // List
    let list = store.list_nodes(&Default::default())
        .await
        .expect("Failed to list nodes");
    assert!(list.items.iter().any(|n| n.id == test_node.id));

    // Update
    test_node.lifecycle = Lifecycle::Live;
    let updated = store.update_node(&test_node)
        .await
        .expect("Failed to update node");
    assert_eq!(updated.lifecycle, Lifecycle::Live);

    // Delete
    store.delete_node(&test_node.id)
        .await
        .expect("Failed to delete node");
    let after_delete = store.get_node(&test_node.id)
        .await
        .expect("Error checking deletion");
    assert!(after_delete.is_none(), "Node was not deleted");
}
