/// Test script for SQLite datastore integration
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Import what we need
    use unet_core::datastore::{sqlite::SqliteStore, DataStore};
    use unet_core::models::{Node, Vendor, DeviceRole, Lifecycle};
    use uuid::Uuid;

    // Create SQLite connection
    let database_url = "sqlite:./test_sqlite_integration.db";
    let store = SqliteStore::new(database_url).await?;

    println!("✅ Connected to SQLite database");

    // Test health check
    store.health_check().await?;
    println!("✅ Health check passed");

    // Create a test node
    let test_node = Node::new(
        "test-router-01".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );

    // Update the model field to pass validation
    let mut test_node = test_node;
    test_node.model = "ISR4331".to_string();

    println!("Creating test node: {}", test_node.name);

    // Test create node
    let created_node = store.create_node(&test_node).await?;
    println!("✅ Node created successfully: {}", created_node.id);

    // Test get node
    let retrieved_node = store.get_node(&test_node.id).await?;
    match retrieved_node {
        Some(node) => {
            println!("✅ Node retrieved successfully: {}", node.name);
            assert_eq!(node.name, test_node.name);
            assert_eq!(node.vendor, test_node.vendor);
            assert_eq!(node.role, test_node.role);
        }
        None => {
            return Err("Failed to retrieve created node".into());
        }
    }

    // Test list nodes
    use unet_core::datastore::QueryOptions;
    let list_result = store.list_nodes(&QueryOptions::default()).await?;
    println!("✅ Listed {} nodes", list_result.items.len());
    assert!(!list_result.items.is_empty());

    // Test update node
    let mut updated_node = test_node.clone();
    updated_node.lifecycle = Lifecycle::Live;
    let updated_result = store.update_node(&updated_node).await?;
    println!("✅ Node updated successfully");

    // Test delete node
    store.delete_node(&test_node.id).await?;
    println!("✅ Node deleted successfully");

    // Verify deletion
    let deleted_check = store.get_node(&test_node.id).await?;
    if deleted_check.is_none() {
        println!("✅ Node deletion verified");
    } else {
        return Err("Node was not actually deleted".into());
    }

    println!("\n🎉 All SQLite datastore tests passed!");
    Ok(())
}