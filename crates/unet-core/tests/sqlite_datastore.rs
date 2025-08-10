// Integration test for SQLite datastore
// Moved from root test_sqlite_datastore.rs, converted into proper cargo integration test.

use sea_orm::EntityTrait;
use test_support::sqlite::sqlite_store;
use unet_core::datastore::{DataStore, QueryOptions};
use unet_core::entities::Vendors;
use unet_core::models::{DeviceRole, Lifecycle, Node, Vendor};

/// End-to-end integration test for SQLite-based DataStore
#[tokio::test]
async fn sqlite_datastore_integration() {
    // Use entity-based schema on in-memory SQLite for testing
    let store = sqlite_store().await;

    // Verify seeded vendors
    let vendor_names: Vec<String> = Vendors::find()
        .all(store.connection())
        .await
        .expect("Query vendors failed")
        .into_iter()
        .map(|v| v.name)
        .collect();
    assert!(vendor_names.contains(&"Cisco".to_string()));
    assert!(vendor_names.contains(&"Juniper".to_string()));

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
    let created_node = store
        .create_node(&test_node)
        .await
        .expect("Failed to create node");
    assert_eq!(created_node.id, test_node.id);

    // Retrieve
    let retrieved = store
        .get_node(&test_node.id)
        .await
        .expect("Error fetching node");
    let node = retrieved.expect("Node not found");
    assert_eq!(node.name, test_node.name);
    assert_eq!(node.vendor, test_node.vendor);
    assert_eq!(node.role, test_node.role);

    // List
    let list = store
        .list_nodes(&QueryOptions::default())
        .await
        .expect("Failed to list nodes");
    assert!(list.items.iter().any(|n| n.id == test_node.id));

    // Update
    test_node.lifecycle = Lifecycle::Live;
    let updated = store
        .update_node(&test_node)
        .await
        .expect("Failed to update node");
    assert_eq!(updated.lifecycle, Lifecycle::Live);

    // Delete
    store
        .delete_node(&test_node.id)
        .await
        .expect("Failed to delete node");
    let after_delete = store
        .get_node(&test_node.id)
        .await
        .expect("Error checking deletion");
    assert!(after_delete.is_none(), "Node was not deleted");
}

/// Verify vendor management through the datastore
#[tokio::test]
async fn vendor_management() {
    let store = sqlite_store().await;

    // Add vendor
    store
        .create_vendor("ExampleCorp")
        .await
        .expect("add vendor");
    let vendors = store.list_vendors().await.expect("list vendors");
    assert!(vendors.contains(&"ExampleCorp".to_string()));

    // Delete vendor
    store
        .delete_vendor("ExampleCorp")
        .await
        .expect("delete vendor");
    let vendors = store.list_vendors().await.expect("list vendors");
    assert!(!vendors.contains(&"ExampleCorp".to_string()));
}

/// Verify error handling when inserting duplicate vendors
#[tokio::test]
async fn vendor_duplicate_insertion_errors() {
    let store = sqlite_store().await;
    // First insert succeeds
    store.create_vendor("DupCorp").await.expect("add vendor");
    // Second insert should error
    let err = store.create_vendor("DupCorp").await.expect_err("expected error on duplicate");
    // Ensure error message path covered
    let msg = format!("{err}");
    assert!(msg.to_lowercase().contains("insert") || msg.to_lowercase().contains("failed"));
}

/// Ensure get_node_required returns NotFound for unknown ID
#[tokio::test]
async fn get_node_required_not_found() {
    let store = sqlite_store().await;
    let missing = uuid::Uuid::new_v4();
    let err = store.get_node_required(&missing).await.expect_err("expected not found");
    let msg = format!("{err}");
    assert!(msg.to_lowercase().contains("not found") || msg.to_lowercase().contains("node"));
}

/// Deleting a non-existent vendor returns NotFound
#[tokio::test]
async fn delete_vendor_nonexistent_errors() {
    let store = sqlite_store().await;
    let err = store.delete_vendor("does-not-exist").await.expect_err("expected not found");
    let msg = format!("{err}");
    assert!(msg.to_lowercase().contains("not found") || msg.to_lowercase().contains("vendor"));
}
