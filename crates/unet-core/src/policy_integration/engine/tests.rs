//! Tests for policy integration engine

use super::default_engine::DefaultPolicyEvaluationEngine;
use super::trait_definition::PolicyEvaluationEngine;
use crate::config::network;
use crate::models::{DeviceRole, Lifecycle, Node, Vendor};
use uuid::Uuid;

pub fn create_test_node() -> Node {
    Node {
        id: Uuid::new_v4(),
        name: "test-node".to_string(),
        domain: "example.com".to_string(),
        fqdn: "test-node.example.com".to_string(),
        vendor: Vendor::Cisco,
        model: "ISR4321".to_string(),
        role: DeviceRole::Router,
        lifecycle: Lifecycle::Live,
        management_ip: Some(
            network::parse_ip_addr("192.168.1.1").expect("Test IP address should be valid"),
        ),
        location_id: None,
        platform: None,
        version: Some("15.1".to_string()),
        serial_number: Some("ABC123".to_string()),
        asset_tag: None,
        purchase_date: None,
        warranty_expires: None,
        custom_data: serde_json::json!({"compliance": "pending"}),
    }
}

#[test]
fn test_create_evaluation_context() {
    let engine = DefaultPolicyEvaluationEngine::new();
    let node = create_test_node();

    let context = engine.create_evaluation_context(&node).unwrap();

    // Verify the context contains node data
    let context_value = &context.node_data;
    assert!(context_value.get("node").is_some());

    if let Some(node_data) = context_value.get("node") {
        assert_eq!(node_data.get("name").unwrap(), "test-node");
        assert_eq!(node_data.get("vendor").unwrap(), "cisco");
        assert_eq!(node_data.get("has_management_ip").unwrap(), true);
        assert_eq!(node_data.get("has_location").unwrap(), false);
    }
}

#[test]
fn test_create_evaluation_context_no_management_ip() {
    let engine = DefaultPolicyEvaluationEngine::new();
    let mut node = create_test_node();
    node.management_ip = None;

    let context = engine.create_evaluation_context(&node).unwrap();
    let context_value = &context.node_data;

    if let Some(node_data) = context_value.get("node") {
        assert_eq!(node_data.get("has_management_ip").unwrap(), false);
    }
}

#[test]
fn test_create_evaluation_context_with_location() {
    let engine = DefaultPolicyEvaluationEngine::new();
    let mut node = create_test_node();
    node.location_id = Some(Uuid::new_v4());

    let context = engine.create_evaluation_context(&node).unwrap();
    let context_value = &context.node_data;

    if let Some(node_data) = context_value.get("node") {
        assert_eq!(node_data.get("has_location").unwrap(), true);
    }
}

#[test]
fn test_default_policy_evaluation_engine_creation() {
    let engine1 = DefaultPolicyEvaluationEngine::new();
    let engine2 = DefaultPolicyEvaluationEngine;

    // Both creation methods should work (they create equivalent instances)
    let node = create_test_node();
    let context1 = engine1.create_evaluation_context(&node).unwrap();
    let context2 = engine2.create_evaluation_context(&node).unwrap();

    // Both should create valid contexts
    assert!(context1.node_data.get("node").is_some());
    assert!(context2.node_data.get("node").is_some());
}

#[test]
fn test_create_evaluation_context_fqdn_field() {
    let engine = DefaultPolicyEvaluationEngine::new();
    let node = create_test_node();

    let context = engine.create_evaluation_context(&node).unwrap();
    let context_value = &context.node_data;

    if let Some(node_data) = context_value.get("node") {
        // Should have fqdn field added explicitly
        assert_eq!(node_data.get("fqdn").unwrap(), "test-node.example.com");
    }
}

#[test]
fn test_create_evaluation_context_computed_fields() {
    let engine = DefaultPolicyEvaluationEngine::new();
    let node = create_test_node();

    let context = engine.create_evaluation_context(&node).unwrap();
    let context_value = &context.node_data;

    if let Some(node_data) = context_value.get("node") {
        // Test all computed fields are present
        assert!(node_data.get("fqdn").is_some());
        assert!(node_data.get("has_management_ip").is_some());
        assert!(node_data.get("has_location").is_some());

        // Test their values
        assert_eq!(
            node_data.get("fqdn").unwrap().as_str().unwrap(),
            "test-node.example.com"
        );
        assert!(
            node_data
                .get("has_management_ip")
                .unwrap()
                .as_bool()
                .unwrap()
        );
        assert!(!node_data.get("has_location").unwrap().as_bool().unwrap());
    }
}

/// Test that Default implementation returns the same as `new()`
#[test]
fn test_default_trait_implementation() {
    let engine_new = DefaultPolicyEvaluationEngine::new();
    let engine_default = DefaultPolicyEvaluationEngine;

    // Both should create equivalent functionality
    let node = create_test_node();
    let context_new = engine_new.create_evaluation_context(&node).unwrap();
    let context_default = engine_default.create_evaluation_context(&node).unwrap();

    // Should produce identical contexts
    assert_eq!(context_new.node_data, context_default.node_data);
}

/// Test `create_evaluation_context` with edge case node data
#[test]
fn test_create_evaluation_context_edge_cases() {
    let engine = DefaultPolicyEvaluationEngine::new();

    // Test with minimal node data
    let mut minimal_node = create_test_node();
    minimal_node.version = None;
    minimal_node.serial_number = None;
    minimal_node.asset_tag = None;
    minimal_node.purchase_date = None;
    minimal_node.warranty_expires = None;
    minimal_node.platform = None;
    minimal_node.custom_data = serde_json::json!({});

    let context = engine.create_evaluation_context(&minimal_node);
    assert!(context.is_ok());

    let context = context.unwrap();
    let node_data = context.node_data.get("node").unwrap();

    // Should still have computed fields
    assert!(node_data.get("fqdn").is_some());
    assert!(node_data.get("has_management_ip").is_some());
    assert!(node_data.get("has_location").is_some());
}

// Helper function to create a test policy rule
pub fn create_test_policy_rule() -> crate::policy::PolicyRule {
    use crate::policy::{Action, Condition, FieldRef, Value};
    crate::policy::PolicyRule {
        id: Some("test_rule".to_string()),
        condition: Condition::True,
        action: Action::Assert {
            field: FieldRef {
                path: vec!["vendor".to_string()],
            },
            expected: Value::String("cisco".to_string()),
        },
    }
}

// Helper function to create an invalid test policy rule for triggering evaluation errors
pub fn create_invalid_policy_rule() -> crate::policy::PolicyRule {
    use crate::policy::{Action, ComparisonOperator, Condition, FieldRef, Value};
    crate::policy::PolicyRule {
        id: Some("invalid_rule".to_string()),
        condition: Condition::Comparison {
            field: FieldRef {
                path: vec!["nonexistent_field".to_string()],
            },
            operator: ComparisonOperator::Equal,
            value: Value::String("invalid".to_string()),
        },
        action: Action::Assert {
            field: FieldRef {
                path: vec!["vendor".to_string()],
            },
            expected: Value::String("cisco".to_string()),
        },
    }
}

// Mock implementations and async tests will be moved to separate files
// to keep this file under 300 lines
