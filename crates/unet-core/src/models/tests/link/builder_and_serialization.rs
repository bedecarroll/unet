//! Builder pattern and serialization tests for `Link` model

use crate::models::*;
use serde_json;
use uuid::Uuid;

#[test]
fn test_link_builder_success() {
    let source_node_id = Uuid::new_v4();
    let dest_node_id = Uuid::new_v4();

    let link = LinkBuilder::new()
        .name("core-link")
        .source_node_id(source_node_id)
        .node_a_interface("GigabitEthernet0/0/1")
        .dest_node_id(dest_node_id)
        .node_z_interface("GigabitEthernet0/0/2")
        .description("Core network link")
        .bandwidth(1_000_000_000) // 1 Gbps
        .link_type("fiber")
        .build()
        .unwrap();

    assert_eq!(link.name, "core-link");
    assert_eq!(link.source_node_id, source_node_id);
    assert_eq!(link.node_a_interface, "GigabitEthernet0/0/1");
    assert_eq!(link.dest_node_id, Some(dest_node_id));
    assert_eq!(
        link.node_z_interface,
        Some("GigabitEthernet0/0/2".to_string())
    );
    assert_eq!(link.description, Some("Core network link".to_string()));
    assert_eq!(link.bandwidth, Some(1_000_000_000));
    assert_eq!(link.link_type, Some("fiber".to_string()));
    assert!(!link.is_internet_circuit);
}

#[test]
fn test_link_builder_internet_circuit() {
    let source_node_id = Uuid::new_v4();

    let link = LinkBuilder::new()
        .name("internet-circuit")
        .source_node_id(source_node_id)
        .node_a_interface("eth0")
        .is_internet_circuit(true)
        .bandwidth(100_000_000) // 100 Mbps
        .build()
        .unwrap();

    assert_eq!(link.name, "internet-circuit");
    assert_eq!(link.source_node_id, source_node_id);
    assert_eq!(link.node_a_interface, "eth0");
    assert_eq!(link.dest_node_id, None);
    assert_eq!(link.node_z_interface, None);
    assert!(link.is_internet_circuit);
    assert_eq!(link.bandwidth, Some(100_000_000));
}

#[test]
fn test_link_builder_missing_required_fields() {
    let result = LinkBuilder::new()
        .name("incomplete-link")
        // Missing source_node_id and node_a_interface
        .build();

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Node A ID is required"));
}

#[test]
fn test_link_builder_validation_failure() {
    let source_node_id = Uuid::new_v4();

    let result = LinkBuilder::new()
        .name("") // Invalid empty name
        .source_node_id(source_node_id)
        .node_a_interface("eth0")
        .build();

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("name cannot be empty"));
}

#[test]
fn test_link_serde() {
    let source_node_id = Uuid::new_v4();
    let dest_node_id = Uuid::new_v4();

    let link = Link::new(
        "link1".to_string(),
        source_node_id,
        "eth0".to_string(),
        dest_node_id,
        "eth1".to_string(),
    );

    let json = serde_json::to_string(&link).unwrap();
    let deserialized: Link = serde_json::from_str(&json).unwrap();

    assert_eq!(link.name, deserialized.name);
    assert_eq!(link.source_node_id, deserialized.source_node_id);
    assert_eq!(link.node_a_interface, deserialized.node_a_interface);
    assert_eq!(link.dest_node_id, deserialized.dest_node_id);
    assert_eq!(link.node_z_interface, deserialized.node_z_interface);
    assert_eq!(link.is_internet_circuit, deserialized.is_internet_circuit);
}
