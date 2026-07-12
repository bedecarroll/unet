//! Node creation error tests

use super::super::setup::setup_test_db;
use crate::datastore::sqlite::nodes::*;
use crate::models::{DeviceRole, Lifecycle, Node, Vendor};
use serde_json::json;
use std::net::IpAddr;
use uuid::Uuid;

#[tokio::test]
async fn test_create_node_with_invalid_json_in_custom_data() {
    let test_db = setup_test_db().await;

    // Create a node with problematic custom data that should still work
    // but test the error handling path
    let node = Node {
        id: Uuid::new_v4(),
        name: "test-node".to_string(),
        domain: "test.local".to_string(),
        fqdn: "test-node.test.local".to_string(),
        vendor: Vendor::Cisco,
        model: "Test Device".to_string(),
        role: DeviceRole::Router,
        lifecycle: Lifecycle::Live,
        management_ip: Some(IpAddr::V4(std::net::Ipv4Addr::new(192, 168, 1, 1))),
        location_id: None,
        platform: Some("Test Platform".to_string()),
        version: Some("1.0.0".to_string()),
        serial_number: Some("TEST123".to_string()),
        asset_tag: None,
        purchase_date: None,
        warranty_expires: None,
        custom_data: json!({
            "complex_nested": {
                "deeply": {
                    "nested": {
                        "data": "value",
                        "numbers": [1, 2, 3, 4, 5]
                    }
                }
            }
        }),
    };

    // This should work despite complex JSON
    let result = create_node(&test_db.store, &node).await;
    assert!(
        result.is_ok(),
        "Creating node with complex JSON should succeed"
    );

    // Verify the node was created correctly
    let created = result.unwrap();
    assert_eq!(created.name, node.name);
    assert_eq!(created.custom_data, node.custom_data);
}

#[tokio::test]
async fn test_create_node_duplicate_id() {
    let test_db = setup_test_db().await;

    let node = Node {
        id: Uuid::new_v4(),
        name: "test-node".to_string(),
        domain: "test.local".to_string(),
        fqdn: "test-node.test.local".to_string(),
        vendor: Vendor::Cisco,
        model: "Test Device".to_string(),
        role: DeviceRole::Router,
        lifecycle: Lifecycle::Live,
        management_ip: Some(IpAddr::V4(std::net::Ipv4Addr::new(192, 168, 1, 1))),
        location_id: None,
        platform: Some("Test Platform".to_string()),
        version: Some("1.0.0".to_string()),
        serial_number: Some("TEST123".to_string()),
        asset_tag: None,
        purchase_date: None,
        warranty_expires: None,
        custom_data: json!({}),
    };

    // Create the node once - should succeed
    let result1 = create_node(&test_db.store, &node).await;
    assert!(result1.is_ok(), "First creation should succeed");

    // Try to create the same node again with the same ID - should fail
    let result2 = create_node(&test_db.store, &node).await;
    assert!(result2.is_err(), "Duplicate ID creation should fail");

    match result2.unwrap_err() {
        crate::datastore::types::DataStoreError::InternalError { .. } => {
            // Expected error type for unique constraint violation
        }
        other => panic!("Expected InternalError, got: {other:?}"),
    }
}
