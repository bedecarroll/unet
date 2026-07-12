/// Unit tests for link CRUD business logic
/// These tests focus on testable functions and validation logic
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::commands::links::types::*;
use unet_core::models::LinkBuilder;

// JSON PARSING TESTS

#[tokio::test]
async fn test_json_parsing_valid_custom_data() {
    let valid_json = r#"{"provider": "ISP", "vlan": 100, "priority": "high"}"#;
    let parsed_result = serde_json::from_str::<JsonValue>(valid_json);
    assert!(parsed_result.is_ok());

    let json_value = parsed_result.unwrap();
    assert_eq!(json_value["provider"], "ISP");
    assert_eq!(json_value["vlan"], 100);
    assert_eq!(json_value["priority"], "high");
}

#[tokio::test]
async fn test_json_parsing_invalid_custom_data() {
    let invalid_json = "invalid json content";
    let parsed_result = serde_json::from_str::<JsonValue>(invalid_json);
    assert!(parsed_result.is_err());
}

#[tokio::test]
async fn test_json_parsing_empty_object() {
    let empty_json = "{}";
    let parsed_result = serde_json::from_str::<JsonValue>(empty_json);
    assert!(parsed_result.is_ok());

    let json_value = parsed_result.unwrap();
    assert!(json_value.is_object());
    assert!(json_value.as_object().unwrap().is_empty());
}

#[tokio::test]
async fn test_json_parsing_complex_nested_data() {
    let complex_json = r#"{
        "provider": "Test ISP",
        "config": {
            "vlan": 100,
            "qos": {"priority": 7, "bandwidth": "1G"},
            "security": {"encryption": true}
        },
        "tags": ["production", "critical"],
        "metrics": {"latency_ms": 5.2}
    }"#;

    let parsed_result = serde_json::from_str::<JsonValue>(complex_json);
    assert!(parsed_result.is_ok());

    let json_value = parsed_result.unwrap();
    assert_eq!(json_value["provider"], "Test ISP");
    assert_eq!(json_value["config"]["vlan"], 100);
    assert_eq!(json_value["config"]["qos"]["priority"], 7);
    assert_eq!(json_value["tags"][0], "production");
    assert_eq!(json_value["metrics"]["latency_ms"], 5.2);
}

// LINK BUILDER INTEGRATION TESTS

#[tokio::test]
async fn test_link_builder_from_add_args_minimal() {
    let source_node_id = Uuid::new_v4();

    let args = AddLinkArgs {
        name: "test-link".to_string(),
        node_a_id: source_node_id,
        node_a_interface: "eth0".to_string(),
        node_z_id: None,
        node_z_interface: None,
        bandwidth_bps: None,
        description: None,
        custom_data: None,
    };

    // Test the builder pattern used in add_link function
    let mut builder = LinkBuilder::new()
        .name(args.name)
        .source_node_id(args.node_a_id)
        .node_a_interface(args.node_a_interface);

    // Apply optional fields like add_link does
    if let Some(node_z_id) = args.node_z_id {
        builder = builder.dest_node_id(node_z_id);
    }
    if let Some(node_z_interface) = args.node_z_interface {
        builder = builder.node_z_interface(node_z_interface);
    }
    if let Some(bandwidth_bps) = args.bandwidth_bps {
        builder = builder.bandwidth(bandwidth_bps);
    }
    if let Some(description) = args.description {
        builder = builder.description(description);
    }

    // Build the link (this needs to be an internet circuit for validation)
    let result = builder.is_internet_circuit(true).build();
    assert!(result.is_ok());

    let link = result.unwrap();
    assert_eq!(link.name, "test-link");
    assert_eq!(link.source_node_id, source_node_id);
    assert_eq!(link.node_a_interface, "eth0");
    assert!(link.is_internet_circuit);
}

#[tokio::test]
async fn test_link_builder_from_add_args_full() {
    let source_node_id = Uuid::new_v4();
    let dest_node_id = Uuid::new_v4();
    let custom_data_str = r#"{"provider": "ISP", "circuit_id": "ABC123"}"#;
    let custom_data = serde_json::from_str::<JsonValue>(custom_data_str).unwrap();

    let args = AddLinkArgs {
        name: "full-link".to_string(),
        node_a_id: source_node_id,
        node_a_interface: "GigabitEthernet0/1".to_string(),
        node_z_id: Some(dest_node_id),
        node_z_interface: Some("GigabitEthernet0/2".to_string()),
        bandwidth_bps: Some(1_000_000_000),
        description: Some("Full featured test link".to_string()),
        custom_data: Some(custom_data_str.to_string()),
    };

    // Test the builder pattern used in add_link function
    let mut builder = LinkBuilder::new()
        .name(args.name)
        .source_node_id(args.node_a_id)
        .node_a_interface(args.node_a_interface);

    // Apply optional fields like add_link does
    if let Some(node_z_id) = args.node_z_id {
        builder = builder.dest_node_id(node_z_id);
    }
    if let Some(node_z_interface) = args.node_z_interface {
        builder = builder.node_z_interface(node_z_interface);
    }
    if let Some(bandwidth_bps) = args.bandwidth_bps {
        builder = builder.bandwidth(bandwidth_bps);
    }
    if let Some(description) = args.description {
        builder = builder.description(description);
    }
    builder = builder.custom_data(custom_data);

    // Build the link (this is a node-to-node link, not internet circuit)
    let result = builder.build();
    assert!(result.is_ok());

    let link = result.unwrap();
    assert_eq!(link.name, "full-link");
    assert_eq!(link.source_node_id, source_node_id);
    assert_eq!(link.dest_node_id, Some(dest_node_id));
    assert_eq!(link.node_a_interface, "GigabitEthernet0/1");
    assert_eq!(
        link.node_z_interface,
        Some("GigabitEthernet0/2".to_string())
    );
    assert_eq!(link.bandwidth, Some(1_000_000_000));
    assert_eq!(
        link.description,
        Some("Full featured test link".to_string())
    );
    assert!(!link.is_internet_circuit); // Node-to-node link

    // Check custom data
    assert_eq!(link.custom_data["provider"], "ISP");
    assert_eq!(link.custom_data["circuit_id"], "ABC123");
}

#[tokio::test]
async fn test_link_builder_validation_failures() {
    let source_node_id = Uuid::new_v4();

    // Test empty name validation
    let result = LinkBuilder::new()
        .name(String::new()) // Empty name should fail
        .source_node_id(source_node_id)
        .node_a_interface("eth0".to_string())
        .is_internet_circuit(true)
        .build();
    assert!(result.is_err());

    // Test empty interface validation
    let result = LinkBuilder::new()
        .name("test-link".to_string())
        .source_node_id(source_node_id)
        .node_a_interface(String::new()) // Empty interface should fail
        .is_internet_circuit(true)
        .build();
    assert!(result.is_err());
}

// ARGUMENT PROCESSING TESTS

#[tokio::test]
async fn test_add_link_args_with_custom_data_parsing() {
    let source_node_id = Uuid::new_v4();
    let custom_data_str = r#"{"vlan": 100, "provider": "Test ISP"}"#;

    let args = AddLinkArgs {
        name: "test-link".to_string(),
        node_a_id: source_node_id,
        node_a_interface: "eth0".to_string(),
        node_z_id: None,
        node_z_interface: None,
        bandwidth_bps: None,
        description: None,
        custom_data: Some(custom_data_str.to_string()),
    };

    // Test parsing custom data like add_link function does
    let custom_data = args
        .custom_data
        .map(|json_str| serde_json::from_str::<JsonValue>(&json_str));

    assert!(custom_data.is_some());
    let parsed_result = custom_data.unwrap();
    assert!(parsed_result.is_ok());

    let json_value = parsed_result.unwrap();
    assert_eq!(json_value["vlan"], 100);
    assert_eq!(json_value["provider"], "Test ISP");
}

#[tokio::test]
async fn test_add_link_args_invalid_custom_data_parsing() {
    let source_node_id = Uuid::new_v4();
    let invalid_json = "not valid json";

    let args = AddLinkArgs {
        name: "test-link".to_string(),
        node_a_id: source_node_id,
        node_a_interface: "eth0".to_string(),
        node_z_id: None,
        node_z_interface: None,
        bandwidth_bps: None,
        description: None,
        custom_data: Some(invalid_json.to_string()),
    };

    // Test parsing custom data like add_link function does
    let custom_data = args
        .custom_data
        .map(|json_str| serde_json::from_str::<JsonValue>(&json_str));

    assert!(custom_data.is_some());
    let parsed_result = custom_data.unwrap();
    assert!(parsed_result.is_err()); // Should fail to parse
}

// UPDATE ARGS TESTS

#[tokio::test]
async fn test_update_link_args_partial_updates() {
    let link_id = Uuid::new_v4();

    let args = UpdateLinkArgs {
        id: link_id,
        name: Some("updated-name".to_string()),
        node_a_id: None,                    // Not updating
        node_a_interface: None,             // Not updating
        node_z_id: None,                    // Not updating
        node_z_interface: None,             // Not updating
        bandwidth_bps: Some(5_000_000_000), // Updating
        description: None,                  // Not updating
        custom_data: None,                  // Not updating
    };

    // Verify partial update pattern
    assert_eq!(args.id, link_id);
    assert_eq!(args.name, Some("updated-name".to_string()));
    assert_eq!(args.bandwidth_bps, Some(5_000_000_000));

    // Verify None fields
    assert!(args.node_a_id.is_none());
    assert!(args.node_a_interface.is_none());
    assert!(args.description.is_none());
    assert!(args.custom_data.is_none());
}

#[tokio::test]
async fn test_update_link_args_json_validation() {
    let link_id = Uuid::new_v4();
    let valid_json = r#"{"updated": true, "version": 2}"#;

    let args = UpdateLinkArgs {
        id: link_id,
        name: None,
        node_a_id: None,
        node_a_interface: None,
        node_z_id: None,
        node_z_interface: None,
        bandwidth_bps: None,
        description: None,
        custom_data: Some(valid_json.to_string()),
    };

    // Test JSON validation like update_link would do
    if let Some(custom_data_str) = args.custom_data {
        let parsed_result = serde_json::from_str::<JsonValue>(&custom_data_str);
        assert!(parsed_result.is_ok());

        let json_value = parsed_result.unwrap();
        assert_eq!(json_value["updated"], true);
        assert_eq!(json_value["version"], 2);
    }
}

// PAGINATION TESTS

#[tokio::test]
async fn test_pagination_offset_calculation() {
    // Test pagination logic like list_links uses
    let page = 2_u64;
    let per_page = 20_u64;

    let offset = (page - 1) * per_page;
    assert_eq!(offset, 20); // Page 2 should have offset 20

    let page = 1_u64;
    let offset = (page - 1) * per_page;
    assert_eq!(offset, 0); // Page 1 should have offset 0

    let page = 5_u64;
    let per_page = 10_u64;
    let offset = (page - 1) * per_page;
    assert_eq!(offset, 40); // Page 5 with per_page 10 should have offset 40
}

#[tokio::test]
async fn test_pagination_edge_cases() {
    // Test edge cases for pagination
    let page = 1_u64;
    let per_page = 1_u64;
    let offset = (page - 1) * per_page;
    assert_eq!(offset, 0);

    let page = 1000_u64;
    let per_page = 100_u64;
    let offset = (page - 1) * per_page;
    assert_eq!(offset, 99_900);
}

// BANDWIDTH VALIDATION TESTS

#[tokio::test]
async fn test_bandwidth_value_ranges() {
    // Test common bandwidth values
    let bandwidth_100_mbps = 100_000_000_u64; // 100 Mbps
    let bandwidth_one_gbps = 1_000_000_000_u64; // 1 Gbps
    let bandwidth_ten_gbps = 10_000_000_000_u64; // 10 Gbps

    // All should be valid u64 values
    assert!(bandwidth_100_mbps < u64::MAX);
    assert!(bandwidth_one_gbps < u64::MAX);
    assert!(bandwidth_ten_gbps < u64::MAX);

    // Test zero bandwidth (should be valid)
    let bandwidth_zero = 0_u64;
    assert_eq!(bandwidth_zero, 0);

    // Test maximum bandwidth
    let bandwidth_max = u64::MAX;
    assert_eq!(bandwidth_max, u64::MAX);
}

// LIST LINK ARGS TESTS

#[tokio::test]
async fn test_list_link_args_default_pagination() {
    let args = ListLinkArgs {
        node_id: None,
        min_bandwidth: None,
        page: 1,
        per_page: 20,
    };

    assert_eq!(args.page, 1);
    assert_eq!(args.per_page, 20);
    assert!(args.node_id.is_none());
    assert!(args.min_bandwidth.is_none());
}

#[tokio::test]
async fn test_list_link_args_with_filters() {
    let filter_node_id = Uuid::new_v4();
    let args = ListLinkArgs {
        node_id: Some(filter_node_id),
        min_bandwidth: Some(1_000_000_000), // 1 Gbps minimum
        page: 2,
        per_page: 50,
    };

    assert_eq!(args.node_id, Some(filter_node_id));
    assert_eq!(args.min_bandwidth, Some(1_000_000_000));
    assert_eq!(args.page, 2);
    assert_eq!(args.per_page, 50);
}

// DELETE LINK ARGS TESTS

#[tokio::test]
async fn test_delete_link_args_with_confirmation() {
    let link_id = Uuid::new_v4();

    let args = DeleteLinkArgs {
        id: link_id,
        yes: true, // Skip confirmation
    };

    assert_eq!(args.id, link_id);
    assert!(args.yes);
}

#[tokio::test]
async fn test_delete_link_args_interactive() {
    let link_id = Uuid::new_v4();

    let args = DeleteLinkArgs {
        id: link_id,
        yes: false, // Require confirmation
    };

    assert_eq!(args.id, link_id);
    assert!(!args.yes);
}
