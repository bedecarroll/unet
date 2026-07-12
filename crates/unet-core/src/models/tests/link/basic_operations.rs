//! Basic operations tests for `Link` model

use crate::models::*;
use serde_json;
use uuid::Uuid;

#[test]
fn test_link_new() {
    let source_node_id = Uuid::new_v4();
    let dest_node_id = Uuid::new_v4();

    let link = Link::new(
        "link1".to_string(),
        source_node_id,
        "eth0".to_string(),
        dest_node_id,
        "eth1".to_string(),
    );

    assert_eq!(link.name, "link1");
    assert_eq!(link.source_node_id, source_node_id);
    assert_eq!(link.node_a_interface, "eth0");
    assert_eq!(link.dest_node_id, Some(dest_node_id));
    assert_eq!(link.node_z_interface, Some("eth1".to_string()));
    assert!(!link.is_internet_circuit);
    assert!(link.custom_data.is_null());
}

#[test]
fn test_link_new_internet_circuit() {
    let source_node_id = Uuid::new_v4();

    let link = Link::new_internet_circuit(
        "internet-link".to_string(),
        source_node_id,
        "eth0".to_string(),
    );

    assert_eq!(link.name, "internet-link");
    assert_eq!(link.source_node_id, source_node_id);
    assert_eq!(link.node_a_interface, "eth0");
    assert_eq!(link.dest_node_id, None);
    assert_eq!(link.node_z_interface, None);
    assert!(link.is_internet_circuit);
}

#[test]
fn test_link_validation_success() {
    let source_node_id = Uuid::new_v4();
    let dest_node_id = Uuid::new_v4();

    let link = Link::new(
        "link1".to_string(),
        source_node_id,
        "eth0".to_string(),
        dest_node_id,
        "eth1".to_string(),
    );

    assert!(link.validate().is_ok());
}

#[test]
fn test_link_validation_internet_circuit_success() {
    let source_node_id = Uuid::new_v4();

    let link = Link::new_internet_circuit(
        "internet-link".to_string(),
        source_node_id,
        "eth0".to_string(),
    );

    assert!(link.validate().is_ok());
}

#[test]
fn test_link_validation_empty_name() {
    let source_node_id = Uuid::new_v4();
    let dest_node_id = Uuid::new_v4();

    let link = Link::new(
        String::new(),
        source_node_id,
        "eth0".to_string(),
        dest_node_id,
        "eth1".to_string(),
    );

    assert!(link.validate().is_err());
    assert!(
        link.validate()
            .unwrap_err()
            .contains("name cannot be empty")
    );
}

#[test]
fn test_link_validation_empty_interface() {
    let source_node_id = Uuid::new_v4();
    let dest_node_id = Uuid::new_v4();

    let link = Link::new(
        "link1".to_string(),
        source_node_id,
        String::new(),
        dest_node_id,
        "eth1".to_string(),
    );

    assert!(link.validate().is_err());
    assert!(
        link.validate()
            .unwrap_err()
            .contains("Node A interface cannot be empty")
    );
}

#[test]
fn test_link_validation_invalid_interface() {
    let source_node_id = Uuid::new_v4();
    let dest_node_id = Uuid::new_v4();

    let link = Link::new(
        "link1".to_string(),
        source_node_id,
        "invalid@interface".to_string(),
        dest_node_id,
        "eth1".to_string(),
    );

    assert!(link.validate().is_err());
    assert!(
        link.validate()
            .unwrap_err()
            .contains("Invalid node A interface name format")
    );
}

#[test]
fn test_link_validation_self_link() {
    let node_id = Uuid::new_v4();

    let link = Link::new(
        "self-link".to_string(),
        node_id,
        "eth0".to_string(),
        node_id, // Same node!
        "eth1".to_string(),
    );

    assert!(link.validate().is_err());
    assert!(
        link.validate()
            .unwrap_err()
            .contains("cannot connect a node to itself")
    );
}

#[test]
fn test_link_validation_internet_circuit_with_node_z() {
    let source_node_id = Uuid::new_v4();
    let dest_node_id = Uuid::new_v4();

    let mut link = Link::new_internet_circuit(
        "internet-link".to_string(),
        source_node_id,
        "eth0".to_string(),
    );

    // Manually set dest_node_id (invalid for internet circuit)
    link.dest_node_id = Some(dest_node_id);

    assert!(link.validate().is_err());
    assert!(
        link.validate()
            .unwrap_err()
            .contains("Internet circuits cannot have node Z")
    );
}

#[test]
fn test_link_validation_regular_link_missing_node_z() {
    let source_node_id = Uuid::new_v4();

    let mut link = Link::new_internet_circuit(
        "regular-link".to_string(),
        source_node_id,
        "eth0".to_string(),
    );

    // Make it not an internet circuit but leave dest_node_id as None
    link.is_internet_circuit = false;

    assert!(link.validate().is_err());
    assert!(
        link.validate()
            .unwrap_err()
            .contains("Regular links must have node Z")
    );
}

#[test]
fn test_link_get_other_node_id() {
    let source_node_id = Uuid::new_v4();
    let dest_node_id = Uuid::new_v4();
    let other_node_id = Uuid::new_v4();

    let link = Link::new(
        "link1".to_string(),
        source_node_id,
        "eth0".to_string(),
        dest_node_id,
        "eth1".to_string(),
    );

    assert_eq!(link.get_other_node_id(source_node_id), Some(dest_node_id));
    assert_eq!(link.get_other_node_id(dest_node_id), Some(source_node_id));
    assert_eq!(link.get_other_node_id(other_node_id), None);
}

#[test]
fn test_link_get_interface_for_node() {
    let source_node_id = Uuid::new_v4();
    let dest_node_id = Uuid::new_v4();
    let other_node_id = Uuid::new_v4();

    let link = Link::new(
        "link1".to_string(),
        source_node_id,
        "eth0".to_string(),
        dest_node_id,
        "eth1".to_string(),
    );

    assert_eq!(link.get_interface_for_node(source_node_id), Some("eth0"));
    assert_eq!(link.get_interface_for_node(dest_node_id), Some("eth1"));
    assert_eq!(link.get_interface_for_node(other_node_id), None);
}

#[test]
fn test_link_connects_nodes() {
    let source_node_id = Uuid::new_v4();
    let dest_node_id = Uuid::new_v4();
    let other_node_id = Uuid::new_v4();

    let link = Link::new(
        "link1".to_string(),
        source_node_id,
        "eth0".to_string(),
        dest_node_id,
        "eth1".to_string(),
    );

    assert!(link.connects_nodes(source_node_id, dest_node_id));
    assert!(link.connects_nodes(dest_node_id, source_node_id));
    assert!(!link.connects_nodes(source_node_id, other_node_id));
    assert!(!link.connects_nodes(other_node_id, dest_node_id));
}

#[test]
fn test_link_involves_node() {
    let source_node_id = Uuid::new_v4();
    let dest_node_id = Uuid::new_v4();
    let other_node_id = Uuid::new_v4();

    let link = Link::new(
        "link1".to_string(),
        source_node_id,
        "eth0".to_string(),
        dest_node_id,
        "eth1".to_string(),
    );

    assert!(link.involves_node(source_node_id));
    assert!(link.involves_node(dest_node_id));
    assert!(!link.involves_node(other_node_id));
}

#[test]
fn test_link_custom_data() {
    let source_node_id = Uuid::new_v4();
    let dest_node_id = Uuid::new_v4();

    let mut link = Link::new(
        "link1".to_string(),
        source_node_id,
        "eth0".to_string(),
        dest_node_id,
        "eth1".to_string(),
    );

    // Set custom data
    let value = serde_json::json!("1Gbps");
    assert!(link.set_custom_data("qos.bandwidth", value.clone()).is_ok());

    // Get custom data
    let retrieved = link.get_custom_data("qos.bandwidth");
    assert_eq!(retrieved, Some(&value));

    // Test non-existent path
    let missing = link.get_custom_data("nonexistent.path");
    assert_eq!(missing, None);
}
