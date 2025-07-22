//! Policy execution and evaluation logic

use crate::error::ServerError;
use std::collections::HashMap;
use tracing::{error, warn};
use unet_core::policy::{PolicyExecutionResult, PolicyRule};
use unet_core::prelude::{DataStore, Node, PolicyService};
use uuid::Uuid;

use super::types::{PolicyEvaluationRequest, PolicyEvaluationSummary};

/// Process evaluation results for a single node
pub async fn process_node_evaluation(
    policy_service: &mut PolicyService,
    datastore: &dyn DataStore,
    node: &Node,
    store_results: bool,
    summary: &mut PolicyEvaluationSummary,
    all_results: &mut HashMap<Uuid, Vec<PolicyExecutionResult>>,
) {
    match policy_service.evaluate_node(datastore, node).await {
        Ok(results) => {
            for result in &results {
                summary.total_rules += 1;

                if result.is_error() {
                    summary.error_rules += 1;
                } else if result.is_satisfied() {
                    summary.satisfied_rules += 1;
                } else {
                    summary.unsatisfied_rules += 1;
                }

                if result.is_compliance_failure() {
                    summary.compliance_failures += 1;
                }
            }

            if store_results {
                if let Err(e) = policy_service
                    .store_results(datastore, &node.id, &results)
                    .await
                {
                    warn!("Failed to store policy results for node {}: {}", node.id, e);
                }
            }

            all_results.insert(node.id, results);
        }
        Err(e) => {
            error!("Failed to evaluate policies for node {}: {}", node.id, e);
            let error_result = PolicyExecutionResult::new_error_with_id(
                Some("evaluation".to_string()),
                format!("Failed to evaluate policies: {e}"),
            );
            all_results.insert(node.id, vec![error_result]);
            summary.error_rules += 1;
        }
    }
}

pub fn load_policies_for_request(
    policy_service: &mut PolicyService,
    request: &PolicyEvaluationRequest,
) -> Result<Vec<PolicyRule>, ServerError> {
    request.policies.as_ref().map_or_else(
        || {
            policy_service.load_policies().map_err(|e| {
                error!("Failed to load policies: {}", e);
                ServerError::Internal(format!("Failed to load policies: {e}"))
            })
        },
        |policies| Ok(policies.clone()),
    )
}

pub async fn evaluate_nodes_against_policies(
    policy_service: &mut PolicyService,
    datastore: &dyn DataStore,
    nodes: &[Node],
    store_results: bool,
) -> (
    HashMap<Uuid, Vec<PolicyExecutionResult>>,
    PolicyEvaluationSummary,
) {
    let mut all_results = HashMap::new();
    let mut summary = PolicyEvaluationSummary {
        total_rules: 0,
        satisfied_rules: 0,
        unsatisfied_rules: 0,
        error_rules: 0,
        compliance_failures: 0,
    };

    for node in nodes {
        process_node_evaluation(
            policy_service,
            datastore,
            node,
            store_results,
            &mut summary,
            &mut all_results,
        )
        .await;
    }

    (all_results, summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use migration::{Migrator, MigratorTrait};
    use std::collections::HashMap;
    use std::fs;
    use tempfile::TempDir;
    use unet_core::{
        datastore::sqlite::SqliteStore,
        models::*,
        policy::{Action, ComparisonOperator, Condition, FieldRef, PolicyRule, Value},
        policy_integration::PolicyService,
    };

    async fn setup_test_datastore() -> SqliteStore {
        let store = SqliteStore::new("sqlite::memory:").await.unwrap();
        Migrator::up(store.connection(), None).await.unwrap();
        store
    }

    async fn create_test_node(datastore: &SqliteStore) -> Node {
        let mut node = Node::new(
            "test-node".to_string(),
            "example.com".to_string(),
            Vendor::Cisco,
            DeviceRole::Router,
        );
        node.model = "ASR1000".to_string();
        node.lifecycle = Lifecycle::Live;
        datastore.create_node(&node).await.unwrap()
    }

    fn create_test_policy_rule() -> PolicyRule {
        PolicyRule {
            id: Some("test-rule".to_string()),
            condition: Condition::Comparison {
                field: FieldRef {
                    path: vec!["vendor".to_string()],
                },
                operator: ComparisonOperator::Equal,
                value: Value::String("cisco".to_string()),
            },
            action: Action::Assert {
                field: FieldRef {
                    path: vec!["version".to_string()],
                },
                expected: Value::String("15.1".to_string()),
            },
        }
    }

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

        process_node_evaluation(
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

        process_node_evaluation(
            &mut policy_service,
            &datastore,
            &node,
            true,
            &mut summary,
            &mut all_results,
        )
        .await;

        assert!(all_results.contains_key(&node.id));
    }

    #[tokio::test]
    async fn test_load_policies_for_request_with_policies() {
        let temp_dir = TempDir::new().unwrap();
        let mut policy_service = PolicyService::with_local_dir(temp_dir.path().to_str().unwrap());
        let policies = vec![create_test_policy_rule()];
        let request = PolicyEvaluationRequest {
            node_ids: None,
            policies: Some(policies),
            store_results: None,
        };

        let result = load_policies_for_request(&mut policy_service, &request);
        assert!(result.is_ok());
        let loaded_policies = result.unwrap();
        assert_eq!(loaded_policies.len(), 1);
    }

    #[tokio::test]
    async fn test_load_policies_for_request_without_policies() {
        let temp_dir = TempDir::new().unwrap();
        let mut policy_service = PolicyService::with_local_dir(temp_dir.path().to_str().unwrap());
        let request = PolicyEvaluationRequest {
            node_ids: None,
            policies: None,
            store_results: None,
        };

        let result = load_policies_for_request(&mut policy_service, &request);
        assert!(result.is_ok());
        let _loaded_policies = result.unwrap();
    }

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

        let (results, _summary) =
            evaluate_nodes_against_policies(&mut policy_service, &datastore, &nodes, false).await;

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

        let (results, summary) =
            evaluate_nodes_against_policies(&mut policy_service, &datastore, &nodes, true).await;

        assert!(results.contains_key(&nodes[0].id));
        assert!(summary.total_rules > 0);
    }

    #[tokio::test]
    async fn test_evaluate_nodes_against_policies_empty_nodes() {
        let datastore = setup_test_datastore().await;
        let temp_dir = TempDir::new().unwrap();
        let mut policy_service = PolicyService::with_local_dir(temp_dir.path().to_str().unwrap());
        let nodes = vec![];

        let (results, summary) =
            evaluate_nodes_against_policies(&mut policy_service, &datastore, &nodes, false).await;

        assert!(results.is_empty());
        assert_eq!(summary.total_rules, 0);
        assert_eq!(summary.satisfied_rules, 0);
        assert_eq!(summary.unsatisfied_rules, 0);
        assert_eq!(summary.error_rules, 0);
        assert_eq!(summary.compliance_failures, 0);
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

        process_node_evaluation(
            &mut policy_service,
            &datastore,
            &node,
            false,
            &mut summary,
            &mut all_results,
        )
        .await;

        // Should have at least one rule processed
        assert!(summary.total_rules > 0);
        assert!(all_results.contains_key(&node.id));
        let results = all_results.get(&node.id).unwrap();
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_process_node_evaluation_compliance_failure() {
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;
        let temp_dir = TempDir::new().unwrap();
        let policy_content = r#"# Test policy file for compliance failure
WHEN node.vendor == "cisco" THEN ASSERT node.version IS "14.0"
"#;
        let policy_file = temp_dir.path().join("compliance.policy");
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

        process_node_evaluation(
            &mut policy_service,
            &datastore,
            &node,
            false,
            &mut summary,
            &mut all_results,
        )
        .await;

        assert_eq!(summary.total_rules, 1);
        assert!(all_results.contains_key(&node.id));
    }

    #[tokio::test]
    async fn test_load_policies_for_request_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        // Don't create any policy files - empty directory

        let mut policy_service = PolicyService::with_local_dir(temp_dir.path().to_str().unwrap());
        let request = PolicyEvaluationRequest {
            node_ids: None,
            policies: None, // This will trigger loading from file system
            store_results: None,
        };

        let result = load_policies_for_request(&mut policy_service, &request);
        // Should succeed but return empty policies
        assert!(result.is_ok());
        let policies = result.unwrap();
        assert!(policies.is_empty());
    }
}
