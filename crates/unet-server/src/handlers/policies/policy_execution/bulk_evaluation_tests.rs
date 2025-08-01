//! Tests for bulk evaluation functionality

use std::fs;
use tempfile::TempDir;
use unet_core::prelude::PolicyService;

use super::test_helpers::{create_test_node, setup_test_datastore};
use crate::handlers::policies::policy_execution::bulk_evaluator;

#[tokio::test]
async fn test_evaluate_nodes_against_policies_helper() {
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;
    let temp_dir = TempDir::new().unwrap();
    let policy_content = r#"# Test policy file
WHEN node.vendor == "cisco" THEN ASSERT node.version IS "15.1"
"#;
    let policy_file = temp_dir.path().join("test.policy");
    fs::write(&policy_file, policy_content).unwrap();
    let mut policy_service = PolicyService::with_local_dir(temp_dir.path().to_str().unwrap());
    let nodes = vec![node];

    let (results, _summary) = bulk_evaluator::evaluate_nodes_against_policies(
        &mut policy_service,
        &datastore,
        &nodes,
        false,
    )
    .await;

    assert!(results.contains_key(&nodes[0].id));
}

#[tokio::test]
async fn test_evaluate_nodes_against_policies_with_store_results() {
    let datastore = setup_test_datastore().await;
    let node = create_test_node(&datastore).await;
    let temp_dir = TempDir::new().unwrap();
    let policy_content = r#"# Test policy file
WHEN node.vendor == "cisco" THEN ASSERT node.version IS "15.1"
"#;
    let policy_file = temp_dir.path().join("test.policy");
    fs::write(&policy_file, policy_content).unwrap();
    let mut policy_service = PolicyService::with_local_dir(temp_dir.path().to_str().unwrap());
    let nodes = vec![node];

    let (results, summary) = bulk_evaluator::evaluate_nodes_against_policies(
        &mut policy_service,
        &datastore,
        &nodes,
        true,
    )
    .await;

    assert!(results.contains_key(&nodes[0].id));
    assert!(summary.total_rules > 0);
}

#[tokio::test]
async fn test_evaluate_nodes_against_policies_empty_nodes() {
    let datastore = setup_test_datastore().await;
    let temp_dir = TempDir::new().unwrap();
    let mut policy_service = PolicyService::with_local_dir(temp_dir.path().to_str().unwrap());
    let nodes = vec![];

    let (results, summary) = bulk_evaluator::evaluate_nodes_against_policies(
        &mut policy_service,
        &datastore,
        &nodes,
        false,
    )
    .await;

    assert!(results.is_empty());
    assert_eq!(summary.total_rules, 0);
    assert_eq!(summary.satisfied_rules, 0);
    assert_eq!(summary.unsatisfied_rules, 0);
    assert_eq!(summary.error_rules, 0);
    assert_eq!(summary.compliance_failures, 0);
}
