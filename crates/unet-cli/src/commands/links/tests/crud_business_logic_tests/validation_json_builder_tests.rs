use crate::commands::links::types::AddLinkArgs;
use crate::commands::test_support::{expect_json_object, expect_json_parse_error};
use unet_core::models::LinkBuilder;
use uuid::Uuid;

#[tokio::test]
async fn test_add_link_args_validation_empty_name() {
    let node_a_id = Uuid::new_v4();
    let args = AddLinkArgs {
        name: String::new(),
        node_a_id,
        node_a_interface: "eth0".to_string(),
        node_z_id: None,
        node_z_interface: None,
        bandwidth_bps: None,
        description: None,
        custom_data: None,
    };

    let result = LinkBuilder::new()
        .name(args.name)
        .source_node_id(args.node_a_id)
        .node_a_interface(args.node_a_interface)
        .build();

    assert!(result.is_err());
}

#[tokio::test]
async fn test_add_link_args_validation_empty_interface() {
    let node_a_id = Uuid::new_v4();
    let args = AddLinkArgs {
        name: "test-link".to_string(),
        node_a_id,
        node_a_interface: String::new(),
        node_z_id: None,
        node_z_interface: None,
        bandwidth_bps: None,
        description: None,
        custom_data: None,
    };

    let result = LinkBuilder::new()
        .name(args.name)
        .source_node_id(args.node_a_id)
        .node_a_interface(args.node_a_interface)
        .build();

    assert!(result.is_err());
}

#[tokio::test]
async fn test_add_link_args_validation_valid_minimum() {
    let node_a_id = Uuid::new_v4();
    let args = AddLinkArgs {
        name: "test-link".to_string(),
        node_a_id,
        node_a_interface: "eth0".to_string(),
        node_z_id: None,
        node_z_interface: None,
        bandwidth_bps: None,
        description: None,
        custom_data: None,
    };

    let result = LinkBuilder::new()
        .name(args.name)
        .source_node_id(args.node_a_id)
        .node_a_interface(args.node_a_interface)
        .is_internet_circuit(true)
        .build();

    assert!(result.is_ok());
    let link = result.unwrap();
    assert!(link.is_internet_circuit);
    assert_eq!(link.dest_node_id, None);
}

#[tokio::test]
async fn test_custom_data_json_parsing_valid() {
    let value = expect_json_object(
        r#"{"provider": "ISP", "vlan": 100, "priority": "high"}"#,
    );

    assert_eq!(value["provider"], "ISP");
    assert_eq!(value["vlan"], 100);
    assert_eq!(value["priority"], "high");
}

#[tokio::test]
async fn test_custom_data_json_parsing_invalid() {
    expect_json_parse_error(r#"{"provider": "ISP", "vlan": }"#);
}

#[tokio::test]
async fn test_custom_data_json_parsing_empty_object() {
    let value = expect_json_object("{}");
    assert!(value.is_object());
}

#[tokio::test]
async fn test_custom_data_json_parsing_complex() {
    let value = expect_json_object(
        r#"{
            "nested": {
                "array": [1, 2, 3],
                "object": {"key": "value"},
                "boolean": true,
                "null_value": null
            },
            "notes": "!@#$%^&*()_+-={}[]"
        }"#,
    );

    assert!(value["nested"]["array"].is_array());
    assert!(value["nested"]["boolean"].as_bool().unwrap());
    assert!(value["nested"]["null_value"].is_null());
}

#[tokio::test]
async fn test_custom_data_json_parsing_not_json_string() {
    expect_json_parse_error("this is not json");
}

#[tokio::test]
async fn test_link_builder_with_all_fields() {
    let source_node_id = Uuid::new_v4();
    let dest_node_id = Uuid::new_v4();
    let custom_data = serde_json::json!({"provider": "ISP", "vlan": 100});

    let result = LinkBuilder::new()
        .name("comprehensive-link".to_string())
        .source_node_id(source_node_id)
        .node_a_interface("GigabitEthernet0/1".to_string())
        .dest_node_id(dest_node_id)
        .node_z_interface("GigabitEthernet0/2".to_string())
        .bandwidth(1_000_000_000)
        .description("Full featured link".to_string())
        .custom_data(custom_data)
        .build();

    assert!(result.is_ok());
    let link = result.unwrap();
    assert_eq!(link.dest_node_id, Some(dest_node_id));
    assert_eq!(link.bandwidth, Some(1_000_000_000));
    assert_eq!(link.description, Some("Full featured link".to_string()));
}

#[tokio::test]
async fn test_link_builder_minimal_fields() {
    let source_node_id = Uuid::new_v4();
    let result = LinkBuilder::new()
        .name("minimal-link".to_string())
        .source_node_id(source_node_id)
        .node_a_interface("eth0".to_string())
        .is_internet_circuit(true)
        .build();

    assert!(result.is_ok());
    let link = result.unwrap();
    assert_eq!(link.dest_node_id, None);
    assert_eq!(link.node_z_interface, None);
    assert_eq!(link.bandwidth, None);
    assert_eq!(link.description, None);
    assert!(link.is_internet_circuit);
}

#[tokio::test]
async fn test_link_builder_validation_failures() {
    let source_node_id = Uuid::new_v4();

    let result = LinkBuilder::new()
        .name(String::new())
        .source_node_id(source_node_id)
        .node_a_interface("eth0".to_string())
        .build();
    assert!(result.is_err());

    let result = LinkBuilder::new()
        .name("test-link".to_string())
        .source_node_id(source_node_id)
        .node_a_interface(String::new())
        .build();
    assert!(result.is_err());
}
