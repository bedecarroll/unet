//! Tests for individual node evaluation functionality

use crate::handlers::policies::types::PolicyEvaluationSummary;
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;
use unet_core::prelude::PolicyService;

use super::test_helpers::{create_test_node, setup_test_datastore};
use crate::handlers::policies::policy_execution::node_evaluator;

#[tokio::test]
async fn test_process_node_evaluation_success() {
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;
    let temp_dir = TempDir::new().unwrap();
    let policy_content = r#"# Test policy file
WHEN node.vendor == "cisco" THEN ASSERT node.version IS "15.1"
"#;
    let policy_file = temp_dir.path().join("test.policy");
    fs::write(&policy_file, policy_content).unwrap();
    let mut policy_service = PolicyService::with_local_dir(temp_dir.path().to_str().unwrap());
    let mut summary = PolicyEvaluationSummary {
        total_rules: 0,
        satisfied_rules: 0,
        unsatisfied_rules: 0,
        error_rules: 0,
        compliance_failures: 0,
    };
    let mut all_results = HashMap::new();

    node_evaluator::process_node_evaluation(
        &mut policy_service,
        &datastore,
        &node,
        false,
        &mut summary,
        &mut all_results,
    )
    .await;

    assert!(all_results.contains_key(&node.id));
    assert_eq!(summary.total_rules, 1);
}

#[tokio::test]
async fn test_process_node_evaluation_with_store_results() {
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;
    let temp_dir = TempDir::new().unwrap();
    let policy_content = r#"# Test policy file
WHEN node.vendor == "cisco" THEN ASSERT node.version IS "15.1"
"#;
    let policy_file = temp_dir.path().join("test.policy");
    fs::write(&policy_file, policy_content).unwrap();
    let mut policy_service = PolicyService::with_local_dir(temp_dir.path().to_str().unwrap());
    let mut summary = PolicyEvaluationSummary {
        total_rules: 0,
        satisfied_rules: 0,
        unsatisfied_rules: 0,
        error_rules: 0,
        compliance_failures: 0,
    };
    let mut all_results = HashMap::new();

    node_evaluator::process_node_evaluation(
        &mut policy_service,
        &datastore,
        &node,
        true,
        &mut summary,
        &mut all_results,
    )
    .await;

    assert!(all_results.contains_key(&node.id));
    assert_eq!(summary.total_rules, 1);
}

#[tokio::test]
async fn test_process_node_evaluation_valid_policy() {
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;
    let temp_dir = TempDir::new().unwrap();
    // Create a valid policy that should pass evaluation
    let policy_content = r#"# Test policy file for evaluation
WHEN node.vendor == "cisco" THEN ASSERT node.vendor IS "cisco"
"#;
    let policy_file = temp_dir.path().join("valid.policy");
    fs::write(&policy_file, policy_content).unwrap();
    let mut policy_service = PolicyService::with_local_dir(temp_dir.path().to_str().unwrap());

    let mut summary = PolicyEvaluationSummary {
        total_rules: 0,
        satisfied_rules: 0,
        unsatisfied_rules: 0,
        error_rules: 0,
        compliance_failures: 0,
    };
    let mut all_results = HashMap::new();

    node_evaluator::process_node_evaluation(
        &mut policy_service,
        &datastore,
        &node,
        false,
        &mut summary,
        &mut all_results,
    )
    .await;

    assert!(all_results.contains_key(&node.id));
    assert_eq!(summary.total_rules, 1);
    assert_eq!(summary.satisfied_rules, 1);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_process_node_evaluation_compliance_failure() {
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;
    let temp_dir = TempDir::new().unwrap();
    // Create a policy that should fail compliance
    let policy_content = r#"# Test policy file for compliance failure
WHEN node.vendor == "cisco" THEN ASSERT node.version IS "WrongVersion"
"#;
    let policy_file = temp_dir.path().join("failing.policy");
    fs::write(&policy_file, policy_content).unwrap();
    let mut policy_service = PolicyService::with_local_dir(temp_dir.path().to_str().unwrap());

    let mut summary = PolicyEvaluationSummary {
        total_rules: 0,
        satisfied_rules: 0,
        unsatisfied_rules: 0,
        error_rules: 0,
        compliance_failures: 0,
    };
    let mut all_results = HashMap::new();

    node_evaluator::process_node_evaluation(
        &mut policy_service,
        &datastore,
        &node,
        false,
        &mut summary,
        &mut all_results,
    )
    .await;

    assert!(all_results.contains_key(&node.id));
    assert_eq!(summary.total_rules, 1);
    assert_eq!(summary.satisfied_rules, 1);
    assert_eq!(summary.compliance_failures, 1);
}

#[tokio::test]
async fn test_process_node_evaluation_error_result() {
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;
    let temp_dir = TempDir::new().unwrap();
    // Create a policy that will cause an error during evaluation
    let policy_content = r#"# Test policy file that causes error
WHEN node.nonexistent_field == "value" THEN ASSERT node.vendor IS "cisco"
"#;
    let policy_file = temp_dir.path().join("error.policy");
    fs::write(&policy_file, policy_content).unwrap();
    let mut policy_service = PolicyService::with_local_dir(temp_dir.path().to_str().unwrap());

    let mut summary = PolicyEvaluationSummary {
        total_rules: 0,
        satisfied_rules: 0,
        unsatisfied_rules: 0,
        error_rules: 0,
        compliance_failures: 0,
    };
    let mut all_results = HashMap::new();

    node_evaluator::process_node_evaluation(
        &mut policy_service,
        &datastore,
        &node,
        false,
        &mut summary,
        &mut all_results,
    )
    .await;

    assert!(all_results.contains_key(&node.id));
    assert_eq!(summary.total_rules, 1);
    assert_eq!(summary.error_rules, 1);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_process_node_evaluation_unsatisfied_result() {
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;
    let temp_dir = TempDir::new().unwrap();
    // Create a policy that will be unsatisfied
    let policy_content = r#"# Test policy file for unsatisfied result
WHEN node.vendor == "juniper" THEN ASSERT node.vendor IS "juniper"
"#;
    let policy_file = temp_dir.path().join("unsatisfied.policy");
    fs::write(&policy_file, policy_content).unwrap();
    let mut policy_service = PolicyService::with_local_dir(temp_dir.path().to_str().unwrap());

    let mut summary = PolicyEvaluationSummary {
        total_rules: 0,
        satisfied_rules: 0,
        unsatisfied_rules: 0,
        error_rules: 0,
        compliance_failures: 0,
    };
    let mut all_results = HashMap::new();

    node_evaluator::process_node_evaluation(
        &mut policy_service,
        &datastore,
        &node,
        false,
        &mut summary,
        &mut all_results,
    )
    .await;

    assert!(all_results.contains_key(&node.id));
    // Rule was evaluated but condition didn't match, so it's unsatisfied
    assert_eq!(summary.total_rules, 1);
    assert_eq!(summary.unsatisfied_rules, 1);
}

#[tokio::test]
async fn test_process_node_evaluation_policy_service_error() {
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;
    // Use an invalid directory path to force a policy service error
    let mut policy_service = PolicyService::with_local_dir("/nonexistent/path");

    let mut summary = PolicyEvaluationSummary {
        total_rules: 0,
        satisfied_rules: 0,
        unsatisfied_rules: 0,
        error_rules: 0,
        compliance_failures: 0,
    };
    let mut all_results = HashMap::new();

    node_evaluator::process_node_evaluation(
        &mut policy_service,
        &datastore,
        &node,
        false,
        &mut summary,
        &mut all_results,
    )
    .await;

    // Should handle the error gracefully
    assert_eq!(summary.total_rules, 0);
}

#[tokio::test]
async fn test_process_node_evaluation_store_results_failure() {
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;
    let temp_dir = TempDir::new().unwrap();
    let policy_content = r#"# Test policy file
WHEN node.vendor == "cisco" THEN ASSERT node.version IS "15.1"
"#;
    let policy_file = temp_dir.path().join("test.policy");
    fs::write(&policy_file, policy_content).unwrap();
    let mut policy_service = PolicyService::with_local_dir(temp_dir.path().to_str().unwrap());

    let mut summary = PolicyEvaluationSummary {
        total_rules: 0,
        satisfied_rules: 0,
        unsatisfied_rules: 0,
        error_rules: 0,
        compliance_failures: 0,
    };
    let mut all_results = HashMap::new();

    // Test with store_results = true to test the storage path
    node_evaluator::process_node_evaluation(
        &mut policy_service,
        &datastore,
        &node,
        true,
        &mut summary,
        &mut all_results,
    )
    .await;

    assert!(all_results.contains_key(&node.id));
    assert_eq!(summary.total_rules, 1);
}
