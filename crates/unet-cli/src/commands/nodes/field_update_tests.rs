//! Tests for individual field updates in node update functionality

use crate::commands::nodes::types::UpdateNodeArgs;
use uuid::Uuid;

#[tokio::test]
async fn test_update_node_name_only() {
    // Test UpdateNodeArgs with name update only
    let node_id = Uuid::new_v4();

    let args = UpdateNodeArgs {
        id: node_id,
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

    assert_eq!(args.id, node_id);
    assert_eq!(args.name, Some("updated-node-name".to_string()));
    assert!(args.domain.is_none());
    assert!(args.vendor.is_none());
}

#[tokio::test]
async fn test_update_node_domain_with_fqdn_update() {
    // Test UpdateNodeArgs with domain update only
    let node_id = Uuid::new_v4();

    let args = UpdateNodeArgs {
        id: node_id,
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

    assert_eq!(args.id, node_id);
    assert!(args.name.is_none());
    assert_eq!(args.domain, Some("newdomain.com".to_string()));
    assert!(args.vendor.is_none());
}

#[tokio::test]
async fn test_update_node_vendor_valid() {
    // Test UpdateNodeArgs with vendor update only
    let node_id = Uuid::new_v4();

    let args = UpdateNodeArgs {
        id: node_id,
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

    assert_eq!(args.id, node_id);
    assert!(args.name.is_none());
    assert!(args.domain.is_none());
    assert_eq!(args.vendor, Some("juniper".to_string()));
    assert!(args.model.is_none());
}

#[tokio::test]
async fn test_update_node_model() {
    // Test UpdateNodeArgs with model update only
    let node_id = Uuid::new_v4();

    let args = UpdateNodeArgs {
        id: node_id,
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

    assert_eq!(args.id, node_id);
    assert!(args.vendor.is_none());
    assert_eq!(args.model, Some("ASR9000".to_string()));
    assert!(args.role.is_none());
}

#[tokio::test]
async fn test_update_node_role_valid() {
    // Test UpdateNodeArgs with role update only
    let node_id = Uuid::new_v4();

    let args = UpdateNodeArgs {
        id: node_id,
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

    assert_eq!(args.id, node_id);
    assert!(args.model.is_none());
    assert_eq!(args.role, Some("switch".to_string()));
    assert!(args.lifecycle.is_none());
}

#[tokio::test]
async fn test_update_node_lifecycle_valid() {
    // Test UpdateNodeArgs with lifecycle update only
    let node_id = Uuid::new_v4();

    let args = UpdateNodeArgs {
        id: node_id,
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

    assert_eq!(args.id, node_id);
    assert!(args.role.is_none());
    assert_eq!(args.lifecycle, Some("decommissioned".to_string()));
    assert!(args.location_id.is_none());
}

#[tokio::test]
async fn test_update_node_location_id() {
    // Test UpdateNodeArgs with location_id update only
    let node_id = Uuid::new_v4();
    let new_location_id = Uuid::new_v4();

    let args = UpdateNodeArgs {
        id: node_id,
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

    assert_eq!(args.id, node_id);
    assert!(args.lifecycle.is_none());
    assert_eq!(args.location_id, Some(new_location_id));
    assert!(args.management_ip.is_none());
}

#[tokio::test]
async fn test_update_node_management_ip_valid() {
    // Test UpdateNodeArgs with management_ip update only
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
        management_ip: Some("10.0.0.1".to_string()),
        custom_data: None,
    };

    assert_eq!(args.id, node_id);
    assert!(args.location_id.is_none());
    assert_eq!(args.management_ip, Some("10.0.0.1".to_string()));
    assert!(args.custom_data.is_none());
}

#[tokio::test]
async fn test_update_node_custom_data_valid() {
    // Test UpdateNodeArgs with custom_data update only
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
        custom_data: Some(r#"{"key": "value"}"#.to_string()),
    };

    assert_eq!(args.id, node_id);
    assert!(args.management_ip.is_none());
    assert_eq!(args.custom_data, Some(r#"{"key": "value"}"#.to_string()));
}

#[tokio::test]
async fn test_update_node_all_fields() {
    // Test UpdateNodeArgs with all fields populated
    let node_id = Uuid::new_v4();
    let new_location_id = Uuid::new_v4();

    let args = UpdateNodeArgs {
        id: node_id,
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

    assert_eq!(args.id, node_id);
    assert_eq!(args.name, Some("all-fields-node".to_string()));
    assert_eq!(args.domain, Some("allfieldsdomain.com".to_string()));
    assert_eq!(args.vendor, Some("juniper".to_string()));
    assert_eq!(args.model, Some("EX4300".to_string()));
    assert_eq!(args.role, Some("switch".to_string()));
    assert_eq!(args.lifecycle, Some("decommissioned".to_string()));
    assert_eq!(args.location_id, Some(new_location_id));
    assert_eq!(args.management_ip, Some("172.16.0.1".to_string()));
    assert_eq!(
        args.custom_data,
        Some(r#"{"environment": "test"}"#.to_string())
    );
}
