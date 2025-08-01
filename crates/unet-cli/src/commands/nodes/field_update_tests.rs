//! Tests for individual field updates in node update functionality

use super::super::update::update_node;
use crate::OutputFormat;
use crate::commands::nodes::types::UpdateNodeArgs;
use uuid::Uuid;

use super::update_test_helpers::{create_test_node, setup_test_datastore};

#[tokio::test]
async fn test_update_node_name_only() {
    // Test lines 9-14, 17-19, 64-69 (name update path)
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;

    let args = UpdateNodeArgs {
        id: node.id,
        name: Some("updated-node-name".to_string()),
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
async fn test_update_node_domain_with_fqdn_update() {
    // Test lines 9-14, 21-25, 64-69 (domain update path with FQDN update)
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;

    let args = UpdateNodeArgs {
        id: node.id,
        name: None,
        domain: Some("newdomain.com".to_string()),
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
async fn test_update_node_vendor_valid() {
    // Test lines 9-14, 26-30, 64-69 (vendor update path)
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;

    let args = UpdateNodeArgs {
        id: node.id,
        name: None,
        domain: None,
        vendor: Some("juniper".to_string()),
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
async fn test_update_node_model() {
    // Test lines 9-14, 32-34, 64-69 (model update path)
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;

    let args = UpdateNodeArgs {
        id: node.id,
        name: None,
        domain: None,
        vendor: None,
        model: Some("ASR9000".to_string()),
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
async fn test_update_node_role_valid() {
    // Test lines 9-14, 36-40, 64-69 (role update path)
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;

    let args = UpdateNodeArgs {
        id: node.id,
        name: None,
        domain: None,
        vendor: None,
        model: None,
        role: Some("switch".to_string()),
        lifecycle: None,
        location_id: None,
        management_ip: None,
        custom_data: None,
    };

    let result = update_node(args, &datastore, OutputFormat::Json).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_node_lifecycle_valid() {
    // Test lines 9-14, 42-46, 64-69 (lifecycle update path)
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;

    let args = UpdateNodeArgs {
        id: node.id,
        name: None,
        domain: None,
        vendor: None,
        model: None,
        role: None,
        lifecycle: Some("decommissioned".to_string()),
        location_id: None,
        management_ip: None,
        custom_data: None,
    };

    let result = update_node(args, &datastore, OutputFormat::Json).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_node_location_id() {
    // Test lines 9-14, 48-50, 64-69 (location_id update path)
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;
    let new_location_id = Uuid::new_v4();

    let args = UpdateNodeArgs {
        id: node.id,
        name: None,
        domain: None,
        vendor: None,
        model: None,
        role: None,
        lifecycle: None,
        location_id: Some(new_location_id),
        management_ip: None,
        custom_data: None,
    };

    let result = update_node(args, &datastore, OutputFormat::Json).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_node_management_ip_valid() {
    // Test lines 9-14, 52-57, 64-69 (management_ip update path)
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
        management_ip: Some("10.0.0.1".to_string()),
        custom_data: None,
    };

    let result = update_node(args, &datastore, OutputFormat::Json).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_node_custom_data_valid() {
    // Test lines 9-14, 59-62, 64-69 (custom_data update path)
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
        custom_data: Some(r#"{"key": "value"}"#.to_string()),
    };

    let result = update_node(args, &datastore, OutputFormat::Json).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_node_all_fields() {
    // Test updating all fields at once
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;
    let new_location_id = Uuid::new_v4();

    let args = UpdateNodeArgs {
        id: node.id,
        name: Some("all-fields-node".to_string()),
        domain: Some("allfieldsdomain.com".to_string()),
        vendor: Some("juniper".to_string()),
        model: Some("EX4300".to_string()),
        role: Some("switch".to_string()),
        lifecycle: Some("decommissioned".to_string()),
        location_id: Some(new_location_id),
        management_ip: Some("172.16.0.1".to_string()),
        custom_data: Some(r#"{"environment": "test"}"#.to_string()),
    };

    let result = update_node(args, &datastore, OutputFormat::Json).await;
    assert!(result.is_ok());
}
