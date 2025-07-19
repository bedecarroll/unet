//! Helper functions for link tests

use crate::models::Link;
use serde_json::json;
use uuid::Uuid;

/// Helper function to create a test link
pub fn create_test_link(name: &str, source_node_id: Uuid, dest_node_id: Option<Uuid>) -> Link {
    Link {
        id: Uuid::new_v4(),
        name: name.to_string(),
        source_node_id,
        node_a_interface: "eth0".to_string(),
        dest_node_id,
        node_z_interface: dest_node_id.map(|_| "eth1".to_string()),
        description: Some("Test link".to_string()),
        bandwidth: Some(1_000_000_000), // 1 Gbps
        link_type: Some("ethernet".to_string()),
        is_internet_circuit: false,
        custom_data: json!({"test": "data"}),
    }
}

/// Helper function to create an internet circuit link
pub fn create_internet_circuit_link(name: &str, source_node_id: Uuid) -> Link {
    Link {
        id: Uuid::new_v4(),
        name: name.to_string(),
        source_node_id,
        node_a_interface: "wan0".to_string(),
        dest_node_id: None,
        node_z_interface: None,
        description: Some("Internet circuit".to_string()),
        bandwidth: Some(100_000_000), // 100 Mbps
        link_type: Some("fiber".to_string()),
        is_internet_circuit: true,
        custom_data: json!({"provider": "ISP1"}),
    }
}
