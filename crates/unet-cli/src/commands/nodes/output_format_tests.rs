//! Tests for different output formats in node update functionality

use super::super::update::update_node;
use crate::OutputFormat;
use crate::commands::nodes::types::UpdateNodeArgs;

use super::update_test_helpers::{create_test_node, setup_test_datastore};

#[tokio::test]
async fn test_update_node_yaml_output() {
    // Test with YAML output format
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;

    let args = UpdateNodeArgs {
        id: node.id,
        name: Some("yaml-output-node".to_string()),
        domain: None,
        vendor: None,
        model: None,
        role: None,
        lifecycle: None,
        location_id: None,
        management_ip: None,
        custom_data: None,
    };

    let result = update_node(args, &datastore, OutputFormat::Yaml).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_node_table_output() {
    // Test with Table output format
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;

    let args = UpdateNodeArgs {
        id: node.id,
        name: Some("table-output-node".to_string()),
        domain: None,
        vendor: None,
        model: None,
        role: None,
        lifecycle: None,
        location_id: None,
        management_ip: None,
        custom_data: None,
    };

    let result = update_node(args, &datastore, OutputFormat::Table).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_node_args_structure() {
    // Test that UpdateNodeArgs structure works correctly
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;

    // Test with all None values (should be valid structure)
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

    // Verify that args can be constructed and used
    assert_eq!(args.id, node.id);
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
async fn test_update_node_args_all_none() {
    // Test UpdateNodeArgs with all None values works with update function
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
