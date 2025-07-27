//! Tests for `diff_policy` function

use crate::commands::policy::eval::diff_policy;
use crate::commands::policy::DiffPolicyArgs;
use std::fs;
use tempfile::TempDir;
use uuid::Uuid;

use super::test_helpers::{setup_test_datastore, create_test_node};

#[tokio::test]
async fn test_diff_policy_basic_functionality() {
    // Test lines 94-175 (basic diff_policy execution)
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;
    let temp_dir = TempDir::new().unwrap();
    
    let policy_content = r#"# Test compliance policy
WHEN node.vendor == "cisco" THEN ASSERT node.model IS "ASR1000"
"#;
    let policy_file = temp_dir.path().join("compliance.policy");
    fs::write(&policy_file, policy_content).unwrap();
    
    let args = DiffPolicyArgs {
        path: temp_dir.path().to_path_buf(),
        node_id: node.id,
        verbose: false,
    };
    
    let result = diff_policy(args, &datastore).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_diff_policy_node_not_found() {
    // Test lines 98-102 (node not found path)
    let datastore = setup_test_datastore().await;
    let temp_dir = TempDir::new().unwrap();
    
    let policy_content = r#"# Test policy
WHEN node.vendor == "cisco" THEN ASSERT node.model IS "ASR1000"
"#;
    let policy_file = temp_dir.path().join("test.policy");
    fs::write(&policy_file, policy_content).unwrap();
    
    let nonexistent_id = Uuid::new_v4();
    let args = DiffPolicyArgs {
        path: temp_dir.path().to_path_buf(),
        node_id: nonexistent_id,
        verbose: false,
    };
    
    let result = diff_policy(args, &datastore).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_diff_policy_with_verbose() {
    // Test lines 94-175 with verbose flag
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;
    let temp_dir = TempDir::new().unwrap();
    
    let policy_content = r#"# Test compliance policy
WHEN node.vendor == "cisco" THEN ASSERT node.model IS "ASR1000"
"#;
    let policy_file = temp_dir.path().join("compliance.policy");
    fs::write(&policy_file, policy_content).unwrap();
    
    let args = DiffPolicyArgs {
        path: temp_dir.path().to_path_buf(),
        node_id: node.id,
        verbose: true,
    };
    
    let result = diff_policy(args, &datastore).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_diff_policy_compliance_passed() {
    // Test lines 127-134 (compliance passed path)
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;
    let temp_dir = TempDir::new().unwrap();
    
    let policy_content = r#"# Test compliance policy that should pass
WHEN node.vendor == "cisco" THEN ASSERT node.vendor IS "cisco"
"#;
    let policy_file = temp_dir.path().join("passing.policy");
    fs::write(&policy_file, policy_content).unwrap();
    
    let args = DiffPolicyArgs {
        path: temp_dir.path().to_path_buf(),
        node_id: node.id,
        verbose: true,
    };
    
    let result = diff_policy(args, &datastore).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_diff_policy_compliance_failed() {
    // Test lines 135-143 (compliance failed path)
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;
    let temp_dir = TempDir::new().unwrap();
    
    let policy_content = r#"# Test compliance policy that should fail
WHEN node.vendor == "cisco" THEN ASSERT node.model IS "WrongModel"
"#;
    let policy_file = temp_dir.path().join("failing.policy");
    fs::write(&policy_file, policy_content).unwrap();
    
    let args = DiffPolicyArgs {
        path: temp_dir.path().to_path_buf(),
        node_id: node.id,
        verbose: true,
    };
    
    let result = diff_policy(args, &datastore).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_diff_policy_compliance_error() {
    // Test lines 144-148 (compliance error result path)
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;
    let temp_dir = TempDir::new().unwrap();
    
    let policy_content = r#"# Test compliance policy with error
WHEN node.nonexistent_field == "value" THEN ASSERT node.model IS "ASR1000"
"#;
    let policy_file = temp_dir.path().join("error.policy");
    fs::write(&policy_file, policy_content).unwrap();
    
    let args = DiffPolicyArgs {
        path: temp_dir.path().to_path_buf(),
        node_id: node.id,
        verbose: false,
    };
    
    let result = diff_policy(args, &datastore).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_diff_policy_evaluation_error() {
    // Test lines 150-155 (evaluation error path)
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;
    let temp_dir = TempDir::new().unwrap();
    
    // Create a malformed policy that will cause evaluation error
    let policy_content = r#"# Test policy with malformed syntax that causes evaluation error
WHEN node.invalid.deep.field == "value" THEN ASSERT node.model IS "ASR1000"
"#;
    let policy_file = temp_dir.path().join("eval_error.policy");
    fs::write(&policy_file, policy_content).unwrap();
    
    let args = DiffPolicyArgs {
        path: temp_dir.path().to_path_buf(),
        node_id: node.id,
        verbose: false,
    };
    
    let result = diff_policy(args, &datastore).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_diff_policy_no_compliance_rules() {
    // Test lines 160-162 (no compliance rules found path)
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;
    let temp_dir = TempDir::new().unwrap();
    
    // Create an empty policy file (no rules at all)
    let policy_content = r"# Test policy with no rules
# This file contains no policy rules
";
    let policy_file = temp_dir.path().join("no_rules.policy");
    fs::write(&policy_file, policy_content).unwrap();
    
    let args = DiffPolicyArgs {
        path: temp_dir.path().to_path_buf(),
        node_id: node.id,
        verbose: false,
    };
    
    let result = diff_policy(args, &datastore).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_diff_policy_fully_compliant() {
    // Test lines 167-169 (fully compliant path)
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;
    let temp_dir = TempDir::new().unwrap();
    
    let policy_content = r#"# Test compliance policy that passes
WHEN node.vendor == "cisco" THEN ASSERT node.vendor IS "cisco"
"#;
    let policy_file = temp_dir.path().join("compliant.policy");
    fs::write(&policy_file, policy_content).unwrap();
    
    let args = DiffPolicyArgs {
        path: temp_dir.path().to_path_buf(),
        node_id: node.id,
        verbose: false,
    };
    
    let result = diff_policy(args, &datastore).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_diff_policy_compliance_violations() {
    // Test lines 170-171 (compliance violations path)
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;
    let temp_dir = TempDir::new().unwrap();
    
    let policy_content = r#"# Test compliance policy that fails
WHEN node.vendor == "cisco" THEN ASSERT node.model IS "WrongModel"
"#;
    let policy_file = temp_dir.path().join("violations.policy");
    fs::write(&policy_file, policy_content).unwrap();
    
    let args = DiffPolicyArgs {
        path: temp_dir.path().to_path_buf(),
        node_id: node.id,
        verbose: false,
    };
    
    let result = diff_policy(args, &datastore).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_diff_policy_multiple_compliance_rules() {
    // Test lines 120-158 with multiple compliance rules
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;
    let temp_dir = TempDir::new().unwrap();
    
    let policy_content = r#"# Multiple compliance rules
WHEN node.vendor == "cisco" THEN ASSERT node.vendor IS "cisco"
WHEN node.lifecycle == "live" THEN ASSERT node.lifecycle IS "live"
"#;
    let policy_file = temp_dir.path().join("multiple.policy");
    fs::write(&policy_file, policy_content).unwrap();
    
    let args = DiffPolicyArgs {
        path: temp_dir.path().to_path_buf(),
        node_id: node.id,
        verbose: true,
    };
    
    let result = diff_policy(args, &datastore).await;
    assert!(result.is_ok());
}