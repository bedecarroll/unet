/// Tests for policy evaluation utility functions
use std::path::PathBuf;
use unet_core::models::{DeviceRole, Node, Vendor};
use unet_core::policy::EvaluationContext;

#[tokio::test]
async fn test_node_serialization_for_evaluation() {
    let mut node = Node::new(
        "test-node".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );
    node.model = "Test Model".to_string();

    let node_json = serde_json::to_value(&node);
    assert!(node_json.is_ok());

    let json_value = node_json.unwrap();
    assert!(json_value.is_object());

    let obj = json_value.as_object().unwrap();
    assert!(obj.contains_key("name"));
    assert!(obj.contains_key("domain"));
    assert!(obj.contains_key("vendor"));
    assert!(obj.contains_key("role"));
    assert!(obj.contains_key("model"));
}

#[tokio::test]
async fn test_evaluation_context_creation() {
    let mut node = Node::new(
        "test-node".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );
    node.model = "Test Model".to_string();

    let node_json = serde_json::to_value(&node).unwrap();
    let context = EvaluationContext {
        node_data: node_json,
        derived_data: None,
    };

    assert!(context.node_data.is_object());
    assert!(context.derived_data.is_none());
}

#[tokio::test]
async fn test_pathbuf_display() {
    let path = PathBuf::from("/test/policy/file.pol");
    let display_str = format!("{}", path.display());
    assert!(display_str.contains("policy"));
    assert!(display_str.contains("file.pol"));
}