//! Node data conversion and edge case tests

use super::super::setup::setup_test_db;
use crate::datastore::sqlite::nodes::*;
use crate::models::{DeviceRole, Lifecycle, Node, Vendor};
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn test_node_conversion_edge_cases() {
    let test_db = setup_test_db().await;

    // Create a node with all optional fields as None
    let minimal_node = Node {
        id: Uuid::new_v4(),
        name: "minimal-node".to_string(),
        domain: "test.local".to_string(),
        fqdn: "minimal-node.test.local".to_string(),
        vendor: Vendor::Generic,
        model: "Minimal Device".to_string(),
        role: DeviceRole::Other,
        lifecycle: Lifecycle::Decommissioned,
        management_ip: None,
        location_id: None,
        platform: None,
        version: None,
        serial_number: None,
        asset_tag: None,
        purchase_date: None,
        warranty_expires: None,
        custom_data: json!(null), // Test null JSON
    };

    let result = create_node(&test_db.store, &minimal_node).await;

    assert!(result.is_ok());
    let created_node = result.unwrap();
    assert_eq!(created_node.id, minimal_node.id);
    assert_eq!(created_node.name, minimal_node.name);
    assert!(created_node.management_ip.is_none());
    assert!(created_node.location_id.is_none());
}
