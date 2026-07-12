//! Node mutation operation error tests

use super::super::setup::{create_test_node, setup_test_db};
use crate::datastore::sqlite::nodes::*;
use crate::datastore::types::DataStoreError;
use crate::models::{DeviceRole, Lifecycle, Node, Vendor};
use std::net::IpAddr;
use uuid::Uuid;

#[tokio::test]
async fn test_update_node_not_found() {
    let test_db = setup_test_db().await;

    // Try to update a node that doesn't exist
    let non_existent_node = Node {
        id: Uuid::new_v4(),
        name: "non-existent".to_string(),
        domain: "test.local".to_string(),
        fqdn: "non-existent.test.local".to_string(),
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
        custom_data: serde_json::json!({}),
    };

    let result = update_node(&test_db.store, &non_existent_node).await;

    // Should return NotFound error
    assert!(result.is_err());
    match result.unwrap_err() {
        DataStoreError::NotFound { entity_type, id } => {
            assert_eq!(entity_type, "Node");
            assert_eq!(id, non_existent_node.id.to_string());
        }
        other => panic!("Expected InternalError or NotFound error, got {other:?}"),
    }
}

#[tokio::test]
async fn test_delete_node_not_found() {
    let test_db = setup_test_db().await;

    let non_existent_id = Uuid::new_v4();
    let result = delete_node(&test_db.store, &non_existent_id).await;

    // Should return NotFound error
    assert!(result.is_err());
    match result.unwrap_err() {
        DataStoreError::NotFound { entity_type, id } => {
            assert_eq!(entity_type, "Node");
            assert_eq!(id, non_existent_id.to_string());
        }
        other => panic!("Expected NotFound error, got {other:?}"),
    }
}

#[tokio::test]
async fn test_delete_node_success() {
    let test_db = setup_test_db().await;

    // Create a node first
    let node_id = Uuid::new_v4();
    create_test_node(&test_db.store, node_id, "test-node")
        .await
        .unwrap();

    // Verify node exists
    let node = get_node(&test_db.store, &node_id).await.unwrap();
    assert!(node.is_some());

    // Delete the node
    let result = delete_node(&test_db.store, &node_id).await;
    assert!(result.is_ok());

    // Verify node no longer exists
    let node = get_node(&test_db.store, &node_id).await.unwrap();
    assert!(node.is_none());
}
