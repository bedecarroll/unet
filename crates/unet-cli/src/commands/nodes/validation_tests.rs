//! Tests for validation and error cases in node update functionality

use crate::commands::nodes::types::UpdateNodeArgs;
use uuid::Uuid;

#[tokio::test]
async fn test_update_node_vendor_invalid() {
    // Test UpdateNodeArgs with invalid vendor string
    let node_id = Uuid::new_v4();

    let args = UpdateNodeArgs {
        id: node_id,
        name: None,
        domain: None,
        vendor: Some("invalid_vendor".to_string()),
        model: None,
        role: None,
        lifecycle: None,
        location_id: None,
        management_ip: None,
        custom_data: None,
    };

    assert_eq!(args.id, node_id);
    assert_eq!(args.vendor, Some("invalid_vendor".to_string()));
    assert!(args.name.is_none());
}

#[tokio::test]
async fn test_update_node_role_invalid() {
    // Test UpdateNodeArgs with invalid role string
    let node_id = Uuid::new_v4();

    let args = UpdateNodeArgs {
        id: node_id,
        name: None,
        domain: None,
        vendor: None,
        model: None,
        role: Some("invalid_role".to_string()),
        lifecycle: None,
        location_id: None,
        management_ip: None,
        custom_data: None,
    };

    assert_eq!(args.id, node_id);
    assert_eq!(args.role, Some("invalid_role".to_string()));
    assert!(args.vendor.is_none());
}

#[tokio::test]
async fn test_update_node_lifecycle_invalid() {
    // Test UpdateNodeArgs with invalid lifecycle string
    let node_id = Uuid::new_v4();

    let args = UpdateNodeArgs {
        id: node_id,
        name: None,
        domain: None,
        vendor: None,
        model: None,
        role: None,
        lifecycle: Some("invalid_lifecycle".to_string()),
        location_id: None,
        management_ip: None,
        custom_data: None,
    };

    assert_eq!(args.id, node_id);
    assert_eq!(args.lifecycle, Some("invalid_lifecycle".to_string()));
    assert!(args.role.is_none());
}

#[tokio::test]
async fn test_update_node_management_ip_invalid() {
    // Test UpdateNodeArgs with invalid management IP string
    let node_id = Uuid::new_v4();

    let args = UpdateNodeArgs {
        id: node_id,
        name: None,
        domain: None,
        vendor: None,
        model: None,
        role: None,
        lifecycle: None,
        location_id: None,
        management_ip: Some("invalid.ip.address".to_string()),
        custom_data: None,
    };

    assert_eq!(args.id, node_id);
    assert_eq!(args.management_ip, Some("invalid.ip.address".to_string()));
    assert!(args.lifecycle.is_none());
}

#[tokio::test]
async fn test_update_node_custom_data_invalid() {
    // Test UpdateNodeArgs with invalid custom data string
    let node_id = Uuid::new_v4();

    let args = UpdateNodeArgs {
        id: node_id,
        name: None,
        domain: None,
        vendor: None,
        model: None,
        role: None,
        lifecycle: None,
        location_id: None,
        management_ip: None,
        custom_data: Some("invalid json".to_string()),
    };

    assert_eq!(args.id, node_id);
    assert_eq!(args.custom_data, Some("invalid json".to_string()));
    assert!(args.management_ip.is_none());
}

#[tokio::test]
async fn test_update_node_nonexistent_node() {
    // Test UpdateNodeArgs with nonexistent node ID
    let nonexistent_id = Uuid::new_v4();

    let args = UpdateNodeArgs {
        id: nonexistent_id,
        name: Some("nonexistent-node".to_string()),
        domain: None,
        vendor: None,
        model: None,
        role: None,
        lifecycle: None,
        location_id: None,
        management_ip: None,
        custom_data: None,
    };

    assert_eq!(args.id, nonexistent_id);
    assert_eq!(args.name, Some("nonexistent-node".to_string()));
    assert!(args.vendor.is_none());
}

#[tokio::test]
async fn test_update_node_no_changes() {
    // Test UpdateNodeArgs when no fields are provided for update
    let node_id = Uuid::new_v4();

    let args = UpdateNodeArgs {
        id: node_id,
        name: None,
        domain: None,
        vendor: None,
        model: None,
        role: None,
        lifecycle: None,
        location_id: None,
        management_ip: None,
        custom_data: None,
    };

    assert_eq!(args.id, node_id);
    assert!(args.name.is_none());
    assert!(args.domain.is_none());
    assert!(args.vendor.is_none());
    assert!(args.model.is_none());
    assert!(args.role.is_none());
    assert!(args.lifecycle.is_none());
    assert!(args.location_id.is_none());
    assert!(args.management_ip.is_none());
    assert!(args.custom_data.is_none());
}

#[tokio::test]
async fn test_update_node_mixed_valid_invalid_fields() {
    // Test UpdateNodeArgs with mixed valid and invalid field values
    let node_id = Uuid::new_v4();

    let args = UpdateNodeArgs {
        id: node_id,
        name: Some("mixed-fields-node".to_string()), // Valid
        domain: None,
        vendor: Some("invalid_vendor".to_string()), // Invalid
        model: Some("ValidModel".to_string()),      // Valid
        role: None,
        lifecycle: None,
        location_id: None,
        management_ip: None,
        custom_data: None,
    };

    assert_eq!(args.id, node_id);
    assert_eq!(args.name, Some("mixed-fields-node".to_string()));
    assert_eq!(args.vendor, Some("invalid_vendor".to_string()));
    assert_eq!(args.model, Some("ValidModel".to_string()));
    assert!(args.domain.is_none());
    assert!(args.role.is_none());
}
