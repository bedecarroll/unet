//! Tests for `eval_policy` function

use crate::commands::policy::eval::eval_policy;
use crate::commands::policy::EvalPolicyArgs;
use std::fs;
use tempfile::TempDir;
use uuid::Uuid;

use super::test_helpers::{setup_test_datastore, create_test_node};

#[tokio::test]
async fn test_eval_policy_basic_functionality() {
    // Test lines 9-31 (basic eval_policy execution)
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;
    let temp_dir = TempDir::new().unwrap();
    
    // Create a test policy file
    let policy_content = r#"# Test policy
WHEN node.vendor == "cisco" THEN ASSERT node.model IS "ASR1000"
"#;
    let policy_file = temp_dir.path().join("test.policy");
    fs::write(&policy_file, policy_content).unwrap();
    
    let args = EvalPolicyArgs {
        path: temp_dir.path().to_path_buf(),
        node_id: Some(node.id),
        verbose: false,
        failures_only: false,
    };
    
    let result = eval_policy(args, &datastore).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_eval_policy_no_nodes_found() {
    // Test lines 16 - get_evaluation_nodes error path when node not found
    let datastore = setup_test_datastore().await;
    let temp_dir = TempDir::new().unwrap();
    
    // Create a test policy file
    let policy_content = r#"# Test policy
WHEN node.vendor == "cisco" THEN ASSERT node.model IS "ASR1000"
"#;
    let policy_file = temp_dir.path().join("test.policy");
    fs::write(&policy_file, policy_content).unwrap();
    
    // Use non-existent node ID - this should cause get_evaluation_nodes to return an error
    let nonexistent_id = Uuid::new_v4();
    let args = EvalPolicyArgs {
        path: temp_dir.path().to_path_buf(),
        node_id: Some(nonexistent_id),
        verbose: false,
        failures_only: false,
    };
    
    let result = eval_policy(args, &datastore).await;
    assert!(result.is_err()); // Should error because node not found
}

#[tokio::test]
async fn test_eval_policy_with_verbose() {
    // Test lines 9-31 with verbose flag
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;
    let temp_dir = TempDir::new().unwrap();
    
    let policy_content = r#"# Test policy
WHEN node.vendor == "cisco" THEN ASSERT node.model IS "ASR1000"
"#;
    let policy_file = temp_dir.path().join("test.policy");
    fs::write(&policy_file, policy_content).unwrap();
    
    let args = EvalPolicyArgs {
        path: temp_dir.path().to_path_buf(),
        node_id: Some(node.id),
        verbose: true,
        failures_only: false,
    };
    
    let result = eval_policy(args, &datastore).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_eval_policy_failures_only() {
    // Test lines 9-31 with failures_only flag
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;
    let temp_dir = TempDir::new().unwrap();
    
    let policy_content = r#"# Test policy
WHEN node.vendor == "cisco" THEN ASSERT node.model IS "WrongModel"
"#;
    let policy_file = temp_dir.path().join("test.policy");
    fs::write(&policy_file, policy_content).unwrap();
    
    let args = EvalPolicyArgs {
        path: temp_dir.path().to_path_buf(),
        node_id: Some(node.id),
        verbose: false,
        failures_only: true,
    };
    
    let result = eval_policy(args, &datastore).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_eval_policy_all_nodes() {
    // Test lines 9-31 with all nodes (node_id = None)
    let datastore = setup_test_datastore().await;
    let _node = create_test_node(&datastore).await;
    let temp_dir = TempDir::new().unwrap();
    
    let policy_content = r#"# Test policy for all nodes
WHEN node.vendor == "cisco" THEN ASSERT node.lifecycle IS "live"
"#;
    let policy_file = temp_dir.path().join("all_nodes.policy");
    fs::write(&policy_file, policy_content).unwrap();
    
    let args = EvalPolicyArgs {
        path: temp_dir.path().to_path_buf(),
        node_id: None, // Evaluate against all nodes
        verbose: false,
        failures_only: false,
    };
    
    let result = eval_policy(args, &datastore).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_eval_policy_empty_database() {
    // Test lines 18-21 (no nodes found path when database is empty)
    let datastore = setup_test_datastore().await;
    // Don't create any nodes - database should be empty
    let temp_dir = TempDir::new().unwrap();
    
    let policy_content = r#"# Test policy for empty database
WHEN node.vendor == "cisco" THEN ASSERT node.lifecycle IS "live"
"#;
    let policy_file = temp_dir.path().join("empty_db.policy");
    fs::write(&policy_file, policy_content).unwrap();
    
    let args = EvalPolicyArgs {
        path: temp_dir.path().to_path_buf(),
        node_id: None, // Evaluate against all nodes (should be empty)
        verbose: false,
        failures_only: false,
    };
    
    let result = eval_policy(args, &datastore).await;
    assert!(result.is_ok()); // Should succeed but print warning about no nodes
}