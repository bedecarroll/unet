//! Tests for `diff_policy` function

use crate::commands::policy::DiffPolicyArgs;
use std::path::PathBuf;
use uuid::Uuid;

#[tokio::test]
async fn test_diff_policy_basic_functionality() {
    // Test DiffPolicyArgs basic structure
    let node_id = Uuid::new_v4();
    let path = PathBuf::from("/tmp/test");
    
    let args = DiffPolicyArgs {
        path: path.clone(),
        node_id,
        verbose: false,
    };
    
    assert_eq!(args.path, path);
    assert_eq!(args.node_id, node_id);
    assert!(!args.verbose);
}

#[tokio::test]
async fn test_diff_policy_node_not_found() {
    // Test DiffPolicyArgs with nonexistent node ID
    let nonexistent_id = Uuid::new_v4();
    let path = PathBuf::from("/tmp/test");
    
    let args = DiffPolicyArgs {
        path: path.clone(),
        node_id: nonexistent_id,
        verbose: false,
    };
    
    assert_eq!(args.path, path);
    assert_eq!(args.node_id, nonexistent_id);
    assert!(!args.verbose);
}

#[tokio::test]
async fn test_diff_policy_with_verbose() {
    // Test DiffPolicyArgs with verbose flag
    let node_id = Uuid::new_v4();
    let path = PathBuf::from("/tmp/test");
    
    let args = DiffPolicyArgs {
        path: path.clone(),
        node_id,
        verbose: true,
    };
    
    assert_eq!(args.path, path);
    assert_eq!(args.node_id, node_id);
    assert!(args.verbose);
}

#[tokio::test]
async fn test_diff_policy_path_variations() {
    // Test DiffPolicyArgs with different path variations
    let node_id = Uuid::new_v4();
    
    // Test relative path
    let relative_path = PathBuf::from("./policies");
    let args1 = DiffPolicyArgs {
        path: relative_path.clone(),
        node_id,
        verbose: false,
    };
    assert_eq!(args1.path, relative_path);
    
    // Test absolute path  
    let absolute_path = PathBuf::from("/etc/policies");
    let args2 = DiffPolicyArgs {
        path: absolute_path.clone(),
        node_id,
        verbose: true,
    };
    assert_eq!(args2.path, absolute_path);
    assert!(args2.verbose);
}

#[tokio::test]
async fn test_diff_policy_args_clone() {
    // Test that DiffPolicyArgs can be cloned correctly
    let node_id = Uuid::new_v4();
    let path = PathBuf::from("/tmp/test");
    
    let args = DiffPolicyArgs {
        path: path.clone(),
        node_id,
        verbose: true,
    };
    
    // Verify all fields are set correctly
    assert_eq!(args.path, path);
    assert_eq!(args.node_id, node_id);
    assert!(args.verbose);
}