//! Tests for validation and error cases in node update functionality

use super::super::update::update_node;
use crate::OutputFormat;
use crate::commands::nodes::types::UpdateNodeArgs;
use uuid::Uuid;

use super::update_test_helpers::{create_test_node, setup_test_datastore};

#[tokio::test]
async fn test_update_node_vendor_invalid() {
    // Test lines 9-14, 26-30 (vendor update error path)
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;

    let args = UpdateNodeArgs {
        id: node.id,
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

    let result = update_node(args, &datastore, OutputFormat::Json).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_update_node_role_invalid() {
    // Test lines 9-14, 36-40 (role update error path)
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;

    let args = UpdateNodeArgs {
        id: node.id,
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

    let result = update_node(args, &datastore, OutputFormat::Json).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_update_node_lifecycle_invalid() {
    // Test lines 9-14, 42-46 (lifecycle update error path)
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;

    let args = UpdateNodeArgs {
        id: node.id,
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

    let result = update_node(args, &datastore, OutputFormat::Json).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_update_node_management_ip_invalid() {
    // Test lines 9-14, 52-57 (management_ip update error path)
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;

    let args = UpdateNodeArgs {
        id: node.id,
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

    let result = update_node(args, &datastore, OutputFormat::Json).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_update_node_custom_data_invalid() {
    // Test lines 9-14, 59-62 (custom_data update error path)
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;

    let args = UpdateNodeArgs {
        id: node.id,
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

    let result = update_node(args, &datastore, OutputFormat::Json).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_update_node_nonexistent_node() {
    // Test error when trying to update non-existent node
    let datastore = setup_test_datastore().await;
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

    let result = update_node(args, &datastore, OutputFormat::Json).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_update_node_no_changes() {
    // Test when no fields are provided for update
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;

    let args = UpdateNodeArgs {
        id: node.id,
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

    let result = update_node(args, &datastore, OutputFormat::Json).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_node_mixed_valid_invalid_fields() {
    // Test updating multiple fields where some are valid and some are invalid
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;

    let args = UpdateNodeArgs {
        id: node.id,
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

    let result = update_node(args, &datastore, OutputFormat::Json).await;
    assert!(result.is_err()); // Should fail due to invalid vendor
}
