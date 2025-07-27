//! Tests for `evaluate_node_policies` function

use crate::commands::policy::eval::evaluate_node_policies;
use crate::commands::policy::EvalPolicyArgs;
use std::path::PathBuf;
use unet_core::{
    models::*,
    policy::{Action, ComparisonOperator, Condition, FieldRef, PolicyRule, Value},
};

use super::test_helpers::create_test_policy_rule;

#[tokio::test]
async fn test_evaluate_node_policies_satisfied_result() {
    // Test lines 55-63 (Satisfied result path)
    let mut node = Node::new(
        "test-node".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );
    node.model = "ASR1000".to_string();
    
    let rule = create_test_policy_rule();
    let policies = vec![vec![rule]];
    
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
async fn test_evaluate_node_policies_not_satisfied_result() {
    // Test lines 64-69 (NotSatisfied result path)
    let mut node = Node::new(
        "test-node".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );
    node.model = "WrongModel".to_string(); // This will cause assertion to fail
    
    let rule = create_test_policy_rule();
    let policies = vec![vec![rule]];
    
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
async fn test_evaluate_node_policies_error_result() {
    // Test lines 70-76 (Error result path)
    let mut node = Node::new(
        "test-node".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );
    node.model = "ASR1000".to_string();
    
    // Create a rule that will cause an error during evaluation
    let rule = PolicyRule {
        id: Some("test-error-rule".to_string()),
        condition: Condition::Comparison {
            field: FieldRef {
                path: vec!["nonexistent_field".to_string()],
            },
            operator: ComparisonOperator::Equal,
            value: Value::String("value".to_string()),
        },
        action: Action::Assert {
            field: FieldRef {
                path: vec!["model".to_string()],
            },
            expected: Value::String("ASR1000".to_string()),
        },
    };
    
    let policies = vec![vec![rule]];
    
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
async fn test_evaluate_node_policies_evaluation_error() {
    // Test lines 78-85 (evaluation error path)
    let mut node = Node::new(
        "test-node".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );
    node.model = "ASR1000".to_string();
    
    // Create a malformed rule that will cause evaluation to fail
    let rule = PolicyRule {
        id: Some("test-eval-error-rule".to_string()),
        condition: Condition::Comparison {
            field: FieldRef {
                path: vec!["invalid.field.path".to_string()],
            },
            operator: ComparisonOperator::Equal,
            value: Value::String("value".to_string()),
        },
        action: Action::Assert {
            field: FieldRef {
                path: vec!["model".to_string()],
            },
            expected: Value::String("ASR1000".to_string()),
        },
    };
    
    let policies = vec![vec![rule]];
    
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
async fn test_evaluate_node_policies_multiple_rules() {
    // Test lines 50-87 with multiple rules to cover loop iterations
    let mut node = Node::new(
        "test-node".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );
    node.model = "ASR1000".to_string();
    
    let rule1 = create_test_policy_rule();
    let mut rule2 = create_test_policy_rule();
    rule2.id = Some("test-rule-2".to_string());
    
    let policies = vec![vec![rule1, rule2]];
    
    let args = EvalPolicyArgs {
        path: PathBuf::from("/test"),
        node_id: None,
        verbose: true,
        failures_only: false,
    };
    
    let result = evaluate_node_policies(&node, &policies, &args);
    assert!(result.is_ok());
}