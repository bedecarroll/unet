//! Response creation and logging utilities for policy evaluation

use crate::{error::ServerResult, server::AppState};
use axum::{Json, extract::State};
use std::collections::HashMap;
use tracing::info;
use unet_core::policy::PolicyRule;
use unet_core::prelude::Node;

use super::{
    node_fetching::get_nodes_for_evaluation,
    policy_execution::{evaluate_nodes_against_policies, load_policies_for_request},
    types::{PolicyEvaluationRequest, PolicyEvaluationResponse, PolicyEvaluationSummary},
};

/// Evaluate policies against nodes
///
/// # Errors
/// Returns an error if request parsing fails or datastore operations fail.
pub async fn evaluate_policies(
    State(state): State<AppState>,
    Json(request): Json<PolicyEvaluationRequest>,
) -> ServerResult<Json<PolicyEvaluationResponse>> {
    info!(
        "Evaluating policies for {} nodes",
        request.node_ids.as_ref().map_or(0, std::vec::Vec::len)
    );

    let start_time = std::time::Instant::now();
    let mut policy_service = state.policy_service.clone();

    let nodes = get_nodes_for_evaluation(&*state.datastore, request.node_ids.as_ref()).await?;
    if nodes.is_empty() {
        return Ok(Json(create_empty_response(start_time)));
    }

    let policies = load_policies_for_request(&mut policy_service, &request)?;
    if policies.is_empty() {
        return Ok(Json(create_empty_response(start_time)));
    }

    let (all_results, summary) = evaluate_nodes_against_policies(
        &mut policy_service,
        &*state.datastore,
        &nodes,
        request.store_results.unwrap_or(true),
    )
    .await;

    let evaluation_time = start_time.elapsed();
    log_evaluation_completion(&nodes, &policies, evaluation_time);

    Ok(Json(PolicyEvaluationResponse {
        results: all_results,
        nodes_evaluated: nodes.len(),
        policies_evaluated: policies.len(),
        evaluation_time_ms: u64::try_from(evaluation_time.as_millis()).unwrap_or(0),
        summary,
    }))
}

pub fn create_empty_response(start_time: std::time::Instant) -> PolicyEvaluationResponse {
    PolicyEvaluationResponse {
        results: HashMap::new(),
        nodes_evaluated: 0,
        policies_evaluated: 0,
        evaluation_time_ms: u64::try_from(start_time.elapsed().as_millis()).unwrap_or(0),
        summary: PolicyEvaluationSummary {
            total_rules: 0,
            satisfied_rules: 0,
            unsatisfied_rules: 0,
            error_rules: 0,
            compliance_failures: 0,
        },
    }
}

pub fn log_evaluation_completion(
    nodes: &[Node],
    policies: &[PolicyRule],
    evaluation_time: std::time::Duration,
) {
    info!(
        "Policy evaluation completed: {} nodes, {} policies, {:?}",
        nodes.len(),
        policies.len(),
        evaluation_time
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Json, extract::State};
    use std::fs;
    use std::sync::Arc;
    use tempfile::TempDir;
    use unet_core::{
        datastore::{DataStore, sqlite::SqliteStore},
        models::*,
        policy::{Action, ComparisonOperator, Condition, FieldRef, PolicyRule, Value},
        policy_integration::PolicyService,
    };
    use test_support::sqlite::sqlite_store;

    async fn setup_test_datastore() -> SqliteStore { sqlite_store().await }

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
    async fn test_evaluate_policies_no_nodes() {
        let datastore = setup_test_datastore().await;
        let temp_dir = TempDir::new().unwrap();
        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir(temp_dir.path().to_str().unwrap()),
        };

        let request = PolicyEvaluationRequest {
            node_ids: Some(vec![]),
            policies: None,
            store_results: Some(false),
        };

        let result = evaluate_policies(State(app_state), Json(request)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert_eq!(response.nodes_evaluated, 0);
        assert!(response.results.is_empty());
    }

    #[tokio::test]
    async fn test_evaluate_policies_with_nodes() {
        test_support::sqlite::with_savepoint("pol_eval_nodes", |store| async move {
            let node = create_test_node(&store).await;
            let temp_dir = TempDir::new().unwrap();
            let policy_content = r#"# Test policy file
WHEN node.vendor == "cisco" THEN ASSERT node.version IS "15.1"
"#;
            let policy_file = temp_dir.path().join("test.policy");
            fs::write(&policy_file, policy_content).unwrap();
            let app_state = AppState {
                datastore: Arc::new(store),
                policy_service: PolicyService::with_local_dir(temp_dir.path().to_str().unwrap()),
            };

            let request = PolicyEvaluationRequest {
                node_ids: Some(vec![node.id]),
                policies: None,
                store_results: Some(false),
            };

            let result = evaluate_policies(State(app_state), Json(request)).await;
            assert!(result.is_ok());

            let response = result.unwrap().0;
            assert_eq!(response.nodes_evaluated, 1);
        }).await;
    }

    #[tokio::test]
    async fn test_evaluate_policies_with_custom_policies() {
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;
        let temp_dir = TempDir::new().unwrap();
        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir(temp_dir.path().to_str().unwrap()),
        };

        let policies = vec![create_test_policy_rule()];
        let request = PolicyEvaluationRequest {
            node_ids: Some(vec![node.id]),
            policies: Some(policies),
            store_results: Some(true),
        };

        let result = evaluate_policies(State(app_state), Json(request)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert_eq!(response.nodes_evaluated, 1);
        assert_eq!(response.policies_evaluated, 1);
    }

    #[tokio::test]
    async fn test_evaluate_policies_all_nodes() {
        let datastore = setup_test_datastore().await;
        let _node = create_test_node(&datastore).await;
        let temp_dir = TempDir::new().unwrap();
        let policy_content = r#"# Test policy file
WHEN node.vendor == "cisco" THEN ASSERT node.version IS "15.1"
"#;
        let policy_file = temp_dir.path().join("test.policy");
        fs::write(&policy_file, policy_content).unwrap();
        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir(temp_dir.path().to_str().unwrap()),
        };

        let request = PolicyEvaluationRequest {
            node_ids: None,
            policies: None,
            store_results: Some(false),
        };

        let result = evaluate_policies(State(app_state), Json(request)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert_eq!(response.nodes_evaluated, 1);
    }

    #[tokio::test]
    async fn test_create_empty_response() {
        let start_time = std::time::Instant::now();
        let response = create_empty_response(start_time);

        assert_eq!(response.nodes_evaluated, 0);
        assert_eq!(response.policies_evaluated, 0);
        assert!(response.results.is_empty());
        assert_eq!(response.summary.total_rules, 0);
    }

    #[tokio::test]
    async fn test_log_evaluation_completion_helper() {
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;
        let nodes = vec![node];
        let policies = vec![create_test_policy_rule()];
        let duration = std::time::Duration::from_millis(100);

        log_evaluation_completion(&nodes, &policies, duration);
    }

    #[tokio::test]
    async fn test_evaluate_policies_no_policies_warning() {
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;
        let temp_dir = TempDir::new().unwrap();
        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir(temp_dir.path().to_str().unwrap()),
        };

        let request = PolicyEvaluationRequest {
            node_ids: Some(vec![node.id]),
            policies: Some(vec![]),
            store_results: Some(false),
        };

        let result = evaluate_policies(State(app_state), Json(request)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert_eq!(response.nodes_evaluated, 0);
        assert_eq!(response.policies_evaluated, 0);
    }

    #[tokio::test]
    async fn test_policy_evaluation_timing() {
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;
        let temp_dir = TempDir::new().unwrap();
        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir(temp_dir.path().to_str().unwrap()),
        };

        let request = PolicyEvaluationRequest {
            node_ids: Some(vec![node.id]),
            policies: None,
            store_results: Some(false),
        };

        let result = evaluate_policies(State(app_state), Json(request)).await;
        assert!(result.is_ok());

        let _response = result.unwrap().0;
    }

    #[tokio::test]
    async fn test_evaluate_policies_store_results_default() {
        test_support::sqlite::with_savepoint("pol_eval_store_default", |store| async move {
            let node = create_test_node(&store).await;
            let temp_dir = TempDir::new().unwrap();
            let policy_content = r#"# Test policy file
WHEN node.vendor == "cisco" THEN ASSERT node.version IS "15.1"
"#;
            let policy_file = temp_dir.path().join("test.policy");
            fs::write(&policy_file, policy_content).unwrap();
            let app_state = AppState {
                datastore: Arc::new(store),
                policy_service: PolicyService::with_local_dir(temp_dir.path().to_str().unwrap()),
            };

            let request = PolicyEvaluationRequest {
                node_ids: Some(vec![node.id]),
                policies: None,
                store_results: None, // Should default to true
            };

            let result = evaluate_policies(State(app_state), Json(request)).await;
            assert!(result.is_ok());

            let response = result.unwrap().0;
            assert_eq!(response.nodes_evaluated, 1);
        }).await;
    }

    #[tokio::test]
    async fn test_create_empty_response_timing() {
        use std::thread;
        use std::time::Duration;

        let start_time = std::time::Instant::now();
        thread::sleep(Duration::from_millis(1)); // Small delay to test timing
        let response = create_empty_response(start_time);

        assert_eq!(response.nodes_evaluated, 0);
        assert_eq!(response.policies_evaluated, 0);
        assert!(response.results.is_empty());
        assert!(response.evaluation_time_ms > 0);
        assert_eq!(response.summary.total_rules, 0);
        assert_eq!(response.summary.satisfied_rules, 0);
        assert_eq!(response.summary.unsatisfied_rules, 0);
        assert_eq!(response.summary.error_rules, 0);
        assert_eq!(response.summary.compliance_failures, 0);
    }

    #[tokio::test]
    async fn test_log_evaluation_completion_empty() {
        let nodes = vec![];
        let policies = vec![];
        let duration = std::time::Duration::from_millis(0);

        // Should not panic with empty vectors
        log_evaluation_completion(&nodes, &policies, duration);
    }
}
