//! Tests for `Node` model and `NodeBuilder`

use crate::models::*;
use serde_json;
use std::net::{IpAddr, Ipv4Addr};

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
fn test_node_new_no_domain() {
    let node = Node::new(
        "router1".to_string(),
        String::new(),
        Vendor::Cisco,
        DeviceRole::Router,
    );

    assert_eq!(node.name, "router1");
    assert_eq!(node.domain, "");
    assert_eq!(node.fqdn, "router1");
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
fn test_node_validation_invalid_name() {
    let mut node = Node::new(
        "router@1".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );
    node.model = "ISR4331".to_string();
    node.update_fqdn();

    assert!(node.validate().is_err());
    assert!(node.validate().unwrap_err().contains("alphanumeric"));
}

#[test]
fn test_node_validation_invalid_domain() {
    let mut node = Node::new(
        "router1".to_string(),
        "invalid..domain".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );
    node.model = "ISR4331".to_string();
    node.update_fqdn();

    assert!(node.validate().is_err());
    assert!(
        node.validate()
            .unwrap_err()
            .contains("Invalid domain format")
    );
}

#[test]
fn test_node_validation_empty_model() {
    let node = Node::new(
        "router1".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );

    assert!(node.validate().is_err());
    assert!(
        node.validate()
            .unwrap_err()
            .contains("Model cannot be empty")
    );
}

#[test]
fn test_node_validation_fqdn_mismatch() {
    let mut node = Node::new(
        "router1".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );
    node.model = "ISR4331".to_string();
    node.fqdn = "wrong.fqdn.com".to_string();

    assert!(node.validate().is_err());
    assert!(node.validate().unwrap_err().contains("FQDN must match"));
}

#[test]
fn test_node_update_fqdn() {
    let mut node = Node::new(
        "router1".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );

    node.name = "router2".to_string();
    node.domain = "newdomain.com".to_string();
    node.update_fqdn();

    assert_eq!(node.fqdn, "router2.newdomain.com");
}

#[test]
fn test_node_custom_data() {
    let mut node = Node::new(
        "router1".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );

    // Set custom data
    let value = serde_json::json!("test_value");
    assert!(node.set_custom_data("config.vlan", value.clone()).is_ok());

    // Get custom data
    let retrieved = node.get_custom_data("config.vlan");
    assert_eq!(retrieved, Some(&value));

    // Test nested path
    let nested_value = serde_json::json!(42);
    assert!(
        node.set_custom_data("config.ports.count", nested_value.clone())
            .is_ok()
    );

    let retrieved_nested = node.get_custom_data("config.ports.count");
    assert_eq!(retrieved_nested, Some(&nested_value));

    // Test non-existent path
    let missing = node.get_custom_data("nonexistent.path");
    assert_eq!(missing, None);
}

#[test]
fn test_node_builder_success() {
    let node = NodeBuilder::new()
        .name("router1")
        .domain("example.com")
        .vendor(Vendor::Cisco)
        .model("ISR4331")
        .role(DeviceRole::Router)
        .lifecycle(Lifecycle::Live)
        .management_ip(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)))
        .platform("IOS XE")
        .version("16.12.04")
        .build()
        .unwrap();

    assert_eq!(node.name, "router1");
    assert_eq!(node.domain, "example.com");
    assert_eq!(node.fqdn, "router1.example.com");
    assert_eq!(node.vendor, Vendor::Cisco);
    assert_eq!(node.model, "ISR4331");
    assert_eq!(node.role, DeviceRole::Router);
    assert_eq!(node.lifecycle, Lifecycle::Live);
    assert_eq!(
        node.management_ip,
        Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)))
    );
    assert_eq!(node.platform, Some("IOS XE".to_string()));
    assert_eq!(node.version, Some("16.12.04".to_string()));
}

#[test]
fn test_node_builder_missing_required_fields() {
    let result = NodeBuilder::new()
        .name("router1")
        // Missing domain, vendor, model, role
        .build();

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Vendor is required"));
}

#[test]
fn test_node_builder_validation_failure() {
    let result = NodeBuilder::new()
        .name("") // Invalid empty name
        .domain("example.com")
        .vendor(Vendor::Cisco)
        .model("ISR4331")
        .role(DeviceRole::Router)
        .build();

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("name cannot be empty"));
}

#[test]
fn test_node_builder_custom_data() {
    let custom_data = serde_json::json!({
        "config": {
            "vlans": [10, 20, 30]
        }
    });

    let node = NodeBuilder::new()
        .name("switch1")
        .domain("example.com")
        .vendor(Vendor::Cisco)
        .model("Catalyst 9300")
        .role(DeviceRole::Switch)
        .custom_data(custom_data.clone())
        .build()
        .unwrap();

    assert_eq!(node.custom_data, custom_data);
    assert_eq!(
        node.get_custom_data("config.vlans"),
        Some(&serde_json::json!([10, 20, 30]))
    );
}

#[test]
fn test_node_serde() {
    let node = NodeBuilder::new()
        .name("router1")
        .domain("example.com")
        .vendor(Vendor::Cisco)
        .model("ISR4331")
        .role(DeviceRole::Router)
        .build()
        .unwrap();

    let json = serde_json::to_string(&node).unwrap();
    let deserialized: Node = serde_json::from_str(&json).unwrap();

    assert_eq!(node.name, deserialized.name);
    assert_eq!(node.domain, deserialized.domain);
    assert_eq!(node.fqdn, deserialized.fqdn);
    assert_eq!(node.vendor, deserialized.vendor);
    assert_eq!(node.model, deserialized.model);
    assert_eq!(node.role, deserialized.role);
}
