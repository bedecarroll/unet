// Integration test for SQLite datastore
// Moved from root test_sqlite_datastore.rs, converted into proper cargo integration test.

use migration::{Migrator, MigratorTrait};
use sea_orm::EntityTrait;
use unet_core::datastore::{DataStore, QueryOptions, sqlite::SqliteStore};
use unet_core::entities::Vendors;
use unet_core::models::{DeviceRole, Lifecycle, Node, Vendor};

/// End-to-end integration test for SQLite-based DataStore
#[tokio::test]
async fn sqlite_datastore_integration() {
    // Use in-memory SQLite database for testing
    let database_url = "sqlite::memory:";
    let store = SqliteStore::new(database_url)
        .await
        .expect("Failed to connect to SQLite database");

    // Run database migrations to create tables
    Migrator::up(store.connection(), None)
        .await
        .expect("Failed to run database migrations");

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
    let store = SqliteStore::new("sqlite::memory:")
        .await
        .expect("Failed to init DB");
    Migrator::up(store.connection(), None)
        .await
        .expect("migrations");

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
