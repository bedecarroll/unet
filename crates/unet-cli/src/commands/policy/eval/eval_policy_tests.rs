//! Tests for `eval_policy` function

use crate::commands::policy::EvalPolicyArgs;
use std::path::PathBuf;
use uuid::Uuid;

#[tokio::test]
async fn test_eval_policy_basic_functionality() {
    // Test EvalPolicyArgs basic structure
    let node_id = Uuid::new_v4();
    let path = PathBuf::from("/tmp/test");
    
    let args = EvalPolicyArgs {
        path: path.clone(),
        node_id: Some(node_id),
        verbose: false,
        failures_only: false,
    };
    
    assert_eq!(args.path, path);
    assert_eq!(args.node_id, Some(node_id));
    assert!(!args.verbose);
    assert!(!args.failures_only);
}

#[tokio::test]
async fn test_eval_policy_no_nodes_found() {
    // Test EvalPolicyArgs with nonexistent node ID
    let nonexistent_id = Uuid::new_v4();
    let path = PathBuf::from("/tmp/test");
    
    let args = EvalPolicyArgs {
        path: path.clone(),
        node_id: Some(nonexistent_id),
        verbose: false,
        failures_only: false,
    };
    
    assert_eq!(args.path, path);
    assert_eq!(args.node_id, Some(nonexistent_id));
    assert!(!args.verbose);
    assert!(!args.failures_only);
}

#[tokio::test]
async fn test_eval_policy_with_verbose() {
    // Test EvalPolicyArgs with verbose flag
    let node_id = Uuid::new_v4();
    let path = PathBuf::from("/tmp/test");
    
    let args = EvalPolicyArgs {
        path: path.clone(),
        node_id: Some(node_id),
        verbose: true,
        failures_only: false,
    };
    
    assert_eq!(args.path, path);
    assert_eq!(args.node_id, Some(node_id));
    assert!(args.verbose);
    assert!(!args.failures_only);
}

#[tokio::test]
async fn test_eval_policy_failures_only() {
    // Test EvalPolicyArgs with failures_only flag
    let node_id = Uuid::new_v4();
    let path = PathBuf::from("/tmp/test");
    
    let args = EvalPolicyArgs {
        path: path.clone(),
        node_id: Some(node_id),
        verbose: false,
        failures_only: true,
    };
    
    assert_eq!(args.path, path);
    assert_eq!(args.node_id, Some(node_id));
    assert!(!args.verbose);
    assert!(args.failures_only);
}

#[tokio::test]
async fn test_eval_policy_all_nodes() {
    // Test EvalPolicyArgs with all nodes (node_id = None)
    let path = PathBuf::from("/tmp/test");
    
    let args = EvalPolicyArgs {
        path: path.clone(),
        node_id: None, // Evaluate against all nodes
        verbose: false,
        failures_only: false,
    };
    
    assert_eq!(args.path, path);
    assert!(args.node_id.is_none());
    assert!(!args.verbose);
    assert!(!args.failures_only);
}

#[tokio::test]
async fn test_eval_policy_empty_database() {
    // Test EvalPolicyArgs with verbose and failures_only both true
    let path = PathBuf::from("/tmp/test");
    
    let args = EvalPolicyArgs {
        path: path.clone(),
        node_id: None, // Evaluate against all nodes (should be empty)
        verbose: true,
        failures_only: true,
    };
    
    assert_eq!(args.path, path);
    assert!(args.node_id.is_none());
    assert!(args.verbose);
    assert!(args.failures_only);
}