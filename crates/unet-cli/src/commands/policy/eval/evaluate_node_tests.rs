//! Tests for `EvalPolicyArgs` structure

use crate::commands::policy::EvalPolicyArgs;
use std::path::PathBuf;
use uuid::Uuid;

#[tokio::test]
async fn test_eval_policy_args_basic_structure() {
    // Test EvalPolicyArgs basic structure
    let path = PathBuf::from("/tmp/policies");
    let node_id = Uuid::new_v4();
    
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
async fn test_eval_policy_args_with_flags() {
    // Test EvalPolicyArgs with various flag combinations
    let path = PathBuf::from("/etc/policies");
    
    // Test verbose flag
    let args1 = EvalPolicyArgs {
        path: path.clone(),
        node_id: None,
        verbose: true,
        failures_only: false,
    };
    
    assert_eq!(args1.path, path);
    assert!(args1.node_id.is_none());
    assert!(args1.verbose);
    assert!(!args1.failures_only);
    
    // Test failures_only flag
    let args2 = EvalPolicyArgs {
        path: path.clone(),
        node_id: None,
        verbose: false,
        failures_only: true,
    };
    
    assert_eq!(args2.path, path);
    assert!(args2.node_id.is_none());
    assert!(!args2.verbose);
    assert!(args2.failures_only);
}

#[tokio::test]
async fn test_eval_policy_args_both_flags() {
    // Test EvalPolicyArgs with both verbose and failures_only true
    let path = PathBuf::from("./policies");
    let node_id = Uuid::new_v4();
    
    let args = EvalPolicyArgs {
        path: path.clone(),
        node_id: Some(node_id),
        verbose: true,
        failures_only: true,
    };
    
    assert_eq!(args.path, path);
    assert_eq!(args.node_id, Some(node_id));
    assert!(args.verbose);
    assert!(args.failures_only);
}

#[tokio::test]
async fn test_eval_policy_args_path_variations() {
    // Test EvalPolicyArgs with different path types
    let node_id = Uuid::new_v4();
    
    // Relative path
    let relative_path = PathBuf::from("./policies");
    let args1 = EvalPolicyArgs {
        path: relative_path.clone(),
        node_id: Some(node_id),
        verbose: false,
        failures_only: false,
    };
    assert_eq!(args1.path, relative_path);
    
    // Absolute path
    let absolute_path = PathBuf::from("/home/user/policies");
    let args2 = EvalPolicyArgs {
        path: absolute_path.clone(),
        node_id: Some(node_id),
        verbose: true,
        failures_only: true,
    };
    assert_eq!(args2.path, absolute_path);
}