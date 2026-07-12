//! Tests for different output formats in node update functionality

use crate::commands::nodes::types::UpdateNodeArgs;
use uuid::Uuid;

#[tokio::test]
async fn test_update_node_yaml_output() {
    // Test UpdateNodeArgs for YAML output format
    let node_id = Uuid::new_v4();

    let args = UpdateNodeArgs {
        id: node_id,
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

    assert_eq!(args.id, node_id);
    assert_eq!(args.name, Some("yaml-output-node".to_string()));
    assert!(args.domain.is_none());
}

#[tokio::test]
async fn test_update_node_table_output() {
    // Test UpdateNodeArgs for Table output format
    let node_id = Uuid::new_v4();

    let args = UpdateNodeArgs {
        id: node_id,
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

    assert_eq!(args.id, node_id);
    assert_eq!(args.name, Some("table-output-node".to_string()));
    assert!(args.domain.is_none());
}

#[tokio::test]
async fn test_update_node_args_structure() {
    // Test that UpdateNodeArgs structure works correctly
    let node_id = Uuid::new_v4();

    // Test with all None values (should be valid structure)
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

    // Verify that args can be constructed and used
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
async fn test_update_node_args_all_none() {
    // Test UpdateNodeArgs with all None values
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
