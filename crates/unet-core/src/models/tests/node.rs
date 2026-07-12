//! Tests for Node model

use crate::models::*;
use serde_json;
use std::net::{IpAddr, Ipv4Addr};
use uuid::Uuid;

#[test]
fn test_node_new() {
    let node = Node::new(
        "router1".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );

    assert_eq!(node.name, "router1");
    assert_eq!(node.domain, "example.com");
    assert_eq!(node.fqdn, "router1.example.com");
    assert_eq!(node.vendor, Vendor::Cisco);
    assert_eq!(node.role, DeviceRole::Router);
    assert_eq!(node.lifecycle, Lifecycle::Planned);
    assert!(node.model.is_empty());
    assert!(node.custom_data.is_null());
}

#[test]
fn test_node_validation_success() {
    let mut node = Node::new(
        "router1".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );
    node.model = "ISR4331".to_string();

    assert!(node.validate().is_ok());
}

#[test]
fn test_node_validation_empty_name() {
    let mut node = Node::new(
        String::new(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );
    node.model = "ISR4331".to_string();

    assert!(node.validate().is_err());
    assert!(
        node.validate()
            .unwrap_err()
            .contains("name cannot be empty")
    );
}

#[test]
fn test_node_builder_success() {
    let node = NodeBuilder::new()
        .name("router1".to_string())
        .domain("example.com".to_string())
        .vendor(Vendor::Cisco)
        .role(DeviceRole::Router)
        .model("ISR4331".to_string())
        .lifecycle(Lifecycle::Live)
        .management_ip(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)))
        .location_id(Uuid::new_v4())
        .build()
        .unwrap();

    assert_eq!(node.name, "router1");
    assert_eq!(node.domain, "example.com");
    assert_eq!(node.fqdn, "router1.example.com");
    assert_eq!(node.vendor, Vendor::Cisco);
    assert_eq!(node.role, DeviceRole::Router);
    assert_eq!(node.model, "ISR4331");
    assert_eq!(node.lifecycle, Lifecycle::Live);
    assert!(node.management_ip.is_some());
    assert!(node.location_id.is_some());
}

#[test]
fn test_node_builder_missing_required_fields() {
    let result = NodeBuilder::new().build();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Name is required"));
}

#[test]
fn test_node_serde() {
    let node = Node::new(
        "router1".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );

    let serialized = serde_json::to_string(&node).unwrap();
    let deserialized: Node = serde_json::from_str(&serialized).unwrap();

    assert_eq!(node.id, deserialized.id);
    assert_eq!(node.name, deserialized.name);
    assert_eq!(node.domain, deserialized.domain);
    assert_eq!(node.fqdn, deserialized.fqdn);
    assert_eq!(node.vendor, deserialized.vendor);
    assert_eq!(node.role, deserialized.role);
}
