/// Tests for policy evaluation functions
use crate::commands::policy::{eval::evaluate_node_policies, EvalPolicyArgs};
use std::path::PathBuf;
use unet_core::models::{DeviceRole, Node, Vendor};

#[tokio::test]
async fn test_evaluate_node_policies_basic_functionality() {
    let mut node = Node::new(
        "test-node".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );
    node.model = "Test Model".to_string();

    let policies = vec![];
    let args = EvalPolicyArgs {
        path: PathBuf::from("/test"),
        node_id: None,
        verbose: false,
        failures_only: false,
    };

    let result = evaluate_node_policies(&node, &policies, &args);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_evaluate_node_policies_with_verbose() {
    let mut node = Node::new(
        "test-node".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );
    node.model = "Test Model".to_string();

    let policies = vec![];
    let args = EvalPolicyArgs {
        path: PathBuf::from("/test"),
        node_id: None,
        verbose: true,
        failures_only: false,
    };

    let result = evaluate_node_policies(&node, &policies, &args);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_evaluate_node_policies_failures_only() {
    let mut node = Node::new(
        "test-node".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );
    node.model = "Test Model".to_string();

    let policies = vec![];
    let args = EvalPolicyArgs {
        path: PathBuf::from("/test"),
        node_id: None,
        verbose: false,
        failures_only: true,
    };

    let result = evaluate_node_policies(&node, &policies, &args);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_evaluate_node_policies_verbose_and_failures_only() {
    let mut node = Node::new(
        "test-node".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );
    node.model = "Test Model".to_string();

    let policies = vec![];
    let args = EvalPolicyArgs {
        path: PathBuf::from("/test"),
        node_id: None,
        verbose: true,
        failures_only: true,
    };

    let result = evaluate_node_policies(&node, &policies, &args);
    assert!(result.is_ok());
}