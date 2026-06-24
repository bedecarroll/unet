use std::net::IpAddr;

use crate::commands::nodes::types::AddNodeArgs;
use crate::commands::test_support::expect_json_object;
use unet_core::models::{DeviceRole, Lifecycle, NodeBuilder, Vendor};
use uuid::Uuid;

#[tokio::test]
async fn test_add_node_args_validation_empty_name() {
    let args = AddNodeArgs {
        name: String::new(),
        domain: "example.com".to_string(),
        vendor: "cisco".to_string(),
        model: "ISR4321".to_string(),
        role: "router".to_string(),
        lifecycle: "live".to_string(),
        location_id: None,
        management_ip: None,
        custom_data: None,
    };

    let vendor = args.vendor.parse::<Vendor>().unwrap();
    let role = args.role.parse::<DeviceRole>().unwrap();
    let lifecycle = args.lifecycle.parse::<Lifecycle>().unwrap();

    let result = NodeBuilder::new()
        .name(args.name)
        .domain(args.domain)
        .vendor(vendor)
        .model(args.model)
        .role(role)
        .lifecycle(lifecycle)
        .build();

    assert!(result.is_err());
}

#[tokio::test]
async fn test_add_node_args_validation_empty_domain_is_allowed() {
    let args = AddNodeArgs {
        name: "test-router".to_string(),
        domain: String::new(),
        vendor: "cisco".to_string(),
        model: "ISR4321".to_string(),
        role: "router".to_string(),
        lifecycle: "live".to_string(),
        location_id: None,
        management_ip: None,
        custom_data: None,
    };

    let vendor = args.vendor.parse::<Vendor>().unwrap();
    let role = args.role.parse::<DeviceRole>().unwrap();
    let lifecycle = args.lifecycle.parse::<Lifecycle>().unwrap();

    let result = NodeBuilder::new()
        .name(args.name)
        .domain(args.domain)
        .vendor(vendor)
        .model(args.model)
        .role(role)
        .lifecycle(lifecycle)
        .build();

    assert!(result.is_ok());
    let node = result.unwrap();
    assert_eq!(node.domain, "");
    assert_eq!(node.fqdn, "test-router");
}

#[tokio::test]
async fn test_add_node_args_validation_empty_model() {
    let args = AddNodeArgs {
        name: "test-router".to_string(),
        domain: "example.com".to_string(),
        vendor: "cisco".to_string(),
        model: String::new(),
        role: "router".to_string(),
        lifecycle: "live".to_string(),
        location_id: None,
        management_ip: None,
        custom_data: None,
    };

    let vendor = args.vendor.parse::<Vendor>().unwrap();
    let role = args.role.parse::<DeviceRole>().unwrap();
    let lifecycle = args.lifecycle.parse::<Lifecycle>().unwrap();

    let result = NodeBuilder::new()
        .name(args.name)
        .domain(args.domain)
        .vendor(vendor)
        .model(args.model)
        .role(role)
        .lifecycle(lifecycle)
        .build();

    assert!(result.is_err());
}

#[tokio::test]
async fn test_add_node_args_validation_invalid_vendor() {
    let args = AddNodeArgs {
        name: "test-router".to_string(),
        domain: "example.com".to_string(),
        vendor: "invalid-vendor".to_string(),
        model: "ISR4321".to_string(),
        role: "router".to_string(),
        lifecycle: "live".to_string(),
        location_id: None,
        management_ip: None,
        custom_data: None,
    };

    let result = args.vendor.parse::<Vendor>();
    assert!(result.is_err());
}

#[tokio::test]
async fn test_add_node_args_validation_invalid_role() {
    let args = AddNodeArgs {
        name: "test-router".to_string(),
        domain: "example.com".to_string(),
        vendor: "cisco".to_string(),
        model: "ISR4321".to_string(),
        role: "invalid-role".to_string(),
        lifecycle: "live".to_string(),
        location_id: None,
        management_ip: None,
        custom_data: None,
    };

    let result = args.role.parse::<DeviceRole>();
    assert!(result.is_err());
}

#[tokio::test]
async fn test_add_node_args_validation_invalid_lifecycle() {
    let args = AddNodeArgs {
        name: "test-router".to_string(),
        domain: "example.com".to_string(),
        vendor: "cisco".to_string(),
        model: "ISR4321".to_string(),
        role: "router".to_string(),
        lifecycle: "invalid-lifecycle".to_string(),
        location_id: None,
        management_ip: None,
        custom_data: None,
    };

    let result = args.lifecycle.parse::<Lifecycle>();
    assert!(result.is_err());
}

#[tokio::test]
async fn test_add_node_args_validation_valid_minimum() {
    let args = AddNodeArgs {
        name: "test-router".to_string(),
        domain: "example.com".to_string(),
        vendor: "cisco".to_string(),
        model: "ISR4321".to_string(),
        role: "router".to_string(),
        lifecycle: "live".to_string(),
        location_id: None,
        management_ip: None,
        custom_data: None,
    };

    let vendor = args.vendor.parse::<Vendor>().unwrap();
    let role = args.role.parse::<DeviceRole>().unwrap();
    let lifecycle = args.lifecycle.parse::<Lifecycle>().unwrap();

    let result = NodeBuilder::new()
        .name(args.name)
        .domain(args.domain)
        .vendor(vendor)
        .model(args.model)
        .role(role)
        .lifecycle(lifecycle)
        .build();

    assert!(result.is_ok());
    let node = result.unwrap();
    assert_eq!(node.name, "test-router");
    assert_eq!(node.domain, "example.com");
    assert_eq!(node.vendor, Vendor::Cisco);
    assert_eq!(node.model, "ISR4321");
    assert_eq!(node.role, DeviceRole::Router);
    assert_eq!(node.lifecycle, Lifecycle::Live);
    assert_eq!(node.location_id, None);
    assert_eq!(node.management_ip, None);
}

#[tokio::test]
async fn test_add_node_args_validation_with_optional_fields() {
    let location_id = Uuid::new_v4();
    let args = AddNodeArgs {
        name: "full-router".to_string(),
        domain: "example.com".to_string(),
        vendor: "cisco".to_string(),
        model: "ISR4321".to_string(),
        role: "router".to_string(),
        lifecycle: "live".to_string(),
        location_id: Some(location_id),
        management_ip: Some("192.168.1.1".to_string()),
        custom_data: Some(r#"{"rack": "A1"}"#.to_string()),
    };

    let vendor = args.vendor.parse::<Vendor>().unwrap();
    let role = args.role.parse::<DeviceRole>().unwrap();
    let lifecycle = args.lifecycle.parse::<Lifecycle>().unwrap();
    let management_ip: IpAddr = args.management_ip.unwrap().parse().unwrap();
    let custom_data = expect_json_object(&args.custom_data.unwrap());

    let result = NodeBuilder::new()
        .name(args.name)
        .domain(args.domain)
        .vendor(vendor)
        .model(args.model)
        .role(role)
        .lifecycle(lifecycle)
        .location_id(location_id)
        .management_ip(management_ip)
        .custom_data(custom_data)
        .build();

    assert!(result.is_ok());
    let node = result.unwrap();
    assert_eq!(node.location_id, Some(location_id));
    assert_eq!(node.management_ip, Some(management_ip));
    assert!(node.custom_data.is_object());
}
