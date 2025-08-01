/// Tests for node CRUD arguments structures
use uuid::Uuid;

use crate::commands::nodes::types::{
    AddNodeArgs, DeleteNodeArgs, ListNodeArgs, ShowNodeArgs, UpdateNodeArgs,
};

#[tokio::test]
async fn test_add_node_args_creation() {
    let location_id = Uuid::new_v4();

    let args = AddNodeArgs {
        name: "test-router".to_string(),
        domain: "example.com".to_string(),
        vendor: "cisco".to_string(),
        model: "ISR4321".to_string(),
        role: "router".to_string(),
        lifecycle: "live".to_string(),
        location_id: Some(location_id),
        management_ip: Some("192.168.1.1".to_string()),
        custom_data: Some(r#"{"rack": "A1"}"#.to_string()),
    };

    assert_eq!(args.name, "test-router");
    assert_eq!(args.domain, "example.com");
    assert_eq!(args.vendor, "cisco");
    assert_eq!(args.model, "ISR4321");
    assert_eq!(args.role, "router");
    assert_eq!(args.lifecycle, "live");
    assert_eq!(args.location_id, Some(location_id));
    assert_eq!(args.management_ip, Some("192.168.1.1".to_string()));
    assert!(args.custom_data.is_some());
}

#[tokio::test]
async fn test_add_node_args_minimal() {
    let args = AddNodeArgs {
        name: "minimal-node".to_string(),
        domain: "test.local".to_string(),
        vendor: "generic".to_string(),
        model: "unknown".to_string(),
        role: "server".to_string(),
        lifecycle: "planned".to_string(),
        location_id: None,
        management_ip: None,
        custom_data: None,
    };

    assert_eq!(args.name, "minimal-node");
    assert_eq!(args.domain, "test.local");
    assert_eq!(args.vendor, "generic");
    assert_eq!(args.model, "unknown");
    assert_eq!(args.role, "server");
    assert_eq!(args.lifecycle, "planned");
    assert_eq!(args.location_id, None);
    assert_eq!(args.management_ip, None);
    assert_eq!(args.custom_data, None);
}

#[tokio::test]
async fn test_list_node_args_creation() {
    let args = ListNodeArgs {
        vendor: Some("cisco".to_string()),
        role: Some("router".to_string()),
        lifecycle: Some("live".to_string()),
        page: 2,
        per_page: 50,
    };

    assert_eq!(args.vendor, Some("cisco".to_string()));
    assert_eq!(args.role, Some("router".to_string()));
    assert_eq!(args.lifecycle, Some("live".to_string()));
    assert_eq!(args.page, 2);
    assert_eq!(args.per_page, 50);
}

#[tokio::test]
async fn test_list_node_args_default_pagination() {
    let args = ListNodeArgs {
        vendor: None,
        role: None,
        lifecycle: None,
        page: 1,
        per_page: 20,
    };

    assert_eq!(args.vendor, None);
    assert_eq!(args.role, None);
    assert_eq!(args.lifecycle, None);
    assert_eq!(args.page, 1);
    assert_eq!(args.per_page, 20);
}

#[tokio::test]
async fn test_show_node_args_creation() {
    let node_id = Uuid::new_v4();

    let args = ShowNodeArgs {
        id: node_id,
        include_status: true,
        show_interfaces: true,
        show_system_info: true,
    };

    assert_eq!(args.id, node_id);
    assert!(args.include_status);
    assert!(args.show_interfaces);
    assert!(args.show_system_info);
}

#[tokio::test]
async fn test_show_node_args_minimal() {
    let node_id = Uuid::new_v4();

    let args = ShowNodeArgs {
        id: node_id,
        include_status: false,
        show_interfaces: false,
        show_system_info: false,
    };

    assert_eq!(args.id, node_id);
    assert!(!args.include_status);
    assert!(!args.show_interfaces);
    assert!(!args.show_system_info);
}

#[tokio::test]
async fn test_update_node_args_creation() {
    let node_id = Uuid::new_v4();
    let location_id = Uuid::new_v4();

    let args = UpdateNodeArgs {
        id: node_id,
        name: Some("updated-router".to_string()),
        domain: Some("new-domain.com".to_string()),
        vendor: Some("juniper".to_string()),
        model: Some("MX960".to_string()),
        role: Some("core-router".to_string()),
        lifecycle: Some("production".to_string()),
        location_id: Some(location_id),
        management_ip: Some("10.0.0.1".to_string()),
        custom_data: Some(r#"{"updated": true}"#.to_string()),
    };

    assert_eq!(args.id, node_id);
    assert_eq!(args.name, Some("updated-router".to_string()));
    assert_eq!(args.domain, Some("new-domain.com".to_string()));
    assert_eq!(args.vendor, Some("juniper".to_string()));
    assert_eq!(args.model, Some("MX960".to_string()));
    assert_eq!(args.role, Some("core-router".to_string()));
    assert_eq!(args.lifecycle, Some("production".to_string()));
    assert_eq!(args.location_id, Some(location_id));
    assert_eq!(args.management_ip, Some("10.0.0.1".to_string()));
    assert!(args.custom_data.is_some());
}

#[tokio::test]
async fn test_update_node_args_partial() {
    let node_id = Uuid::new_v4();

    let args = UpdateNodeArgs {
        id: node_id,
        name: Some("partially-updated".to_string()),
        domain: None,
        vendor: None,
        model: None,
        role: Some("edge-router".to_string()),
        lifecycle: None,
        location_id: None,
        management_ip: None,
        custom_data: None,
    };

    assert_eq!(args.id, node_id);
    assert_eq!(args.name, Some("partially-updated".to_string()));
    assert_eq!(args.domain, None);
    assert_eq!(args.vendor, None);
    assert_eq!(args.model, None);
    assert_eq!(args.role, Some("edge-router".to_string()));
    assert_eq!(args.lifecycle, None);
    assert_eq!(args.location_id, None);
    assert_eq!(args.management_ip, None);
    assert_eq!(args.custom_data, None);
}

#[tokio::test]
async fn test_delete_node_args_creation() {
    let node_id = Uuid::new_v4();

    let args = DeleteNodeArgs {
        id: node_id,
        yes: true,
    };

    assert_eq!(args.id, node_id);
    assert!(args.yes);
}

#[tokio::test]
async fn test_delete_node_args_interactive() {
    let node_id = Uuid::new_v4();

    let args = DeleteNodeArgs {
        id: node_id,
        yes: false,
    };

    assert_eq!(args.id, node_id);
    assert!(!args.yes);
}