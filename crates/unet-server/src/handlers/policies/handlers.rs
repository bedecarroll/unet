//! HTTP handler functions for policy endpoints

use axum::{
    Json,
    extract::{Query, State},
};
use std::collections::HashMap;
use tracing::{error, info, warn};
use unet_core::datastore::DataStore;
use unet_core::models::Node;
use unet_core::policy::{PolicyExecutionResult, PolicyRule};
use unet_core::policy_integration::PolicyService;
use uuid::Uuid;

use crate::{
    error::{ServerError, ServerResult},
    server::AppState,
};

use super::{
    evaluation::{get_nodes_for_evaluation, process_node_evaluation},
    types::{
        PolicyEvaluationRequest, PolicyEvaluationResponse, PolicyEvaluationSummary,
        PolicyResultsQuery, PolicyResultsResponse,
    },
};

/// Evaluate policies against nodes
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
        warn!("No policies found for evaluation");
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

fn create_empty_response(start_time: std::time::Instant) -> PolicyEvaluationResponse {
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

fn load_policies_for_request(
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

async fn evaluate_nodes_against_policies(
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

fn log_evaluation_completion(
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

/// Get policy evaluation results
pub async fn get_policy_results(
    State(state): State<AppState>,
    Query(query): Query<PolicyResultsQuery>,
) -> ServerResult<Json<PolicyResultsResponse>> {
    info!("Getting policy results with filter: {:?}", query);

    // For now, return results for a specific node if requested
    if let Some(node_id) = query.node_id {
        match state.datastore.get_policy_results(&node_id).await {
            Ok(results) => {
                let total_count = results.len();
                let offset = query.offset.unwrap_or(0);
                let limit = query.limit.unwrap_or(100);

                let paginated_results: Vec<_> =
                    results.into_iter().skip(offset).take(limit).collect();

                let returned_count = paginated_results.len();

                Ok(Json(PolicyResultsResponse {
                    results: paginated_results,
                    total_count,
                    returned_count,
                }))
            }
            Err(e) => {
                error!("Failed to get policy results for node {}: {}", node_id, e);
                Err(ServerError::Internal(format!(
                    "Failed to get policy results: {e}"
                )))
            }
        }
    } else {
        // Return empty results for now - implementing cross-node queries requires more complex logic
        Ok(Json(PolicyResultsResponse {
            results: Vec::new(),
            total_count: 0,
            returned_count: 0,
        }))
    }
}

/// Validate policy rules
pub async fn validate_policies(
    State(_state): State<AppState>,
    Json(policies): Json<Vec<PolicyRule>>,
) -> ServerResult<Json<serde_json::Value>> {
    info!("Validating {} policy rules", policies.len());

    let mut validation_results = Vec::new();
    let mut valid_count = 0;
    let mut error_count = 0;

    for (index, policy) in policies.iter().enumerate() {
        // For now, just check if the policy has basic required fields
        // In a full implementation, we'd parse and validate the policy syntax
        let is_valid =
            !policy.condition.to_string().is_empty() && !policy.action.to_string().is_empty();

        if is_valid {
            valid_count += 1;
            validation_results.push(serde_json::json!({
                "index": index,
                "valid": true,
                "message": "Policy rule is valid"
            }));
        } else {
            error_count += 1;
            validation_results.push(serde_json::json!({
                "index": index,
                "valid": false,
                "message": "Policy rule is missing required fields"
            }));
        }
    }

    Ok(Json(serde_json::json!({
        "total_policies": policies.len(),
        "valid_policies": valid_count,
        "invalid_policies": error_count,
        "validation_results": validation_results
    })))
}

/// Get policy engine status
pub async fn get_policy_status(
    State(state): State<AppState>,
) -> ServerResult<Json<serde_json::Value>> {
    info!("Getting policy engine status");

    // Get some basic statistics
    let nodes_count = match state.datastore.get_nodes_for_policy_evaluation().await {
        Ok(nodes) => nodes.len(),
        Err(e) => {
            warn!("Failed to get nodes count: {}", e);
            0
        }
    };

    let mut policy_service = state.policy_service.clone();
    let policies_count = match policy_service.load_policies() {
        Ok(policies) => policies.len(),
        Err(e) => {
            warn!("Failed to load policies for status: {}", e);
            0
        }
    };

    Ok(Json(serde_json::json!({
        "policy_engine_enabled": true,
        "nodes_available": nodes_count,
        "policies_available": policies_count,
        "last_evaluation": null, // TODO: Track last evaluation time
        "evaluation_frequency": "on-demand" // TODO: Configure scheduled evaluations
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::AppState;
    use axum::{Json, extract::State};
    use std::sync::Arc;
    use unet_core::{
        datastore::sqlite::SqliteStore,
        models::*,
        policy::{Action, ComparisonOperator, Condition, FieldRef, PolicyRule, Value},
        policy_integration::PolicyService,
    };

    async fn setup_test_datastore() -> SqliteStore {
        SqliteStore::new("sqlite::memory:").await.unwrap()
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
    async fn test_evaluate_policies_no_nodes() {
        let datastore = setup_test_datastore().await;
        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
        };

        let request = PolicyEvaluationRequest {
            node_ids: Some(vec![]), // Empty list
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
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;
        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
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
    }

    #[tokio::test]
    async fn test_evaluate_policies_with_custom_policies() {
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;
        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
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
        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
        };

        let request = PolicyEvaluationRequest {
            node_ids: None, // Evaluate all nodes
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
    async fn test_load_policies_for_request_with_policies() {
        let mut policy_service = PolicyService::with_local_dir("/tmp");
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
        let mut policy_service = PolicyService::with_local_dir("/tmp");
        let request = PolicyEvaluationRequest {
            node_ids: None,
            policies: None,
            store_results: None,
        };

        let result = load_policies_for_request(&mut policy_service, &request);
        assert!(result.is_ok());
        let _loaded_policies = result.unwrap();
        // PolicyService might return default/example policies even when no custom policies exist
        // Just verify we got a result without panicking
        // No need to assert len() >= 0 since it's always true for usize
    }

    #[tokio::test]
    async fn test_evaluate_nodes_against_policies() {
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;
        let mut policy_service = PolicyService::with_local_dir("/tmp");
        let nodes = vec![node];

        let (results, _summary) =
            evaluate_nodes_against_policies(&mut policy_service, &datastore, &nodes, false).await;

        assert!(results.contains_key(&nodes[0].id));
        // Removed useless comparison - total_rules is always >= 0
    }

    #[tokio::test]
    async fn test_log_evaluation_completion() {
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;
        let nodes = vec![node];
        let policies = vec![create_test_policy_rule()];
        let duration = std::time::Duration::from_millis(100);

        // This should not panic
        log_evaluation_completion(&nodes, &policies, duration);
    }

    #[tokio::test]
    async fn test_get_policy_results_with_node_id() {
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;
        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
        };

        let query = PolicyResultsQuery {
            node_id: Some(node.id),
            offset: None,
            limit: None,
        };

        let result = get_policy_results(State(app_state), axum::extract::Query(query)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        // CSV datastore should return empty results
        assert_eq!(response.total_count, 0);
        assert_eq!(response.returned_count, 0);
    }

    #[tokio::test]
    async fn test_get_policy_results_without_node_id() {
        let datastore = setup_test_datastore().await;
        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
        };

        let query = PolicyResultsQuery {
            node_id: None,
            offset: None,
            limit: None,
        };

        let result = get_policy_results(State(app_state), axum::extract::Query(query)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert_eq!(response.total_count, 0);
        assert_eq!(response.returned_count, 0);
    }

    #[tokio::test]
    async fn test_get_policy_results_with_pagination() {
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;
        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
        };

        let query = PolicyResultsQuery {
            node_id: Some(node.id),
            offset: Some(0),
            limit: Some(10),
        };

        let result = get_policy_results(State(app_state), axum::extract::Query(query)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert_eq!(response.total_count, 0);
        assert_eq!(response.returned_count, 0);
    }

    #[tokio::test]
    async fn test_validate_policies_valid() {
        let datastore = setup_test_datastore().await;
        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
        };

        let policies = vec![create_test_policy_rule()];

        let result = validate_policies(State(app_state), Json(policies)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert_eq!(response["total_policies"], 1);
        assert_eq!(response["valid_policies"], 1);
        assert_eq!(response["invalid_policies"], 0);
    }

    #[tokio::test]
    async fn test_validate_policies_empty() {
        let datastore = setup_test_datastore().await;
        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
        };

        let policies = vec![];

        let result = validate_policies(State(app_state), Json(policies)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert_eq!(response["total_policies"], 0);
        assert_eq!(response["valid_policies"], 0);
        assert_eq!(response["invalid_policies"], 0);
    }

    #[tokio::test]
    async fn test_get_policy_status() {
        let datastore = setup_test_datastore().await;
        let _node = create_test_node(&datastore).await;
        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
        };

        let result = get_policy_status(State(app_state)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert_eq!(response["policy_engine_enabled"], true);
        assert_eq!(response["nodes_available"], 1);
        // Policy service might have default policies loaded
        // Removed useless comparison - u64 is always >= 0
    }

    #[tokio::test]
    async fn test_evaluate_policies_no_policies_warning() {
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;
        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
        };

        let request = PolicyEvaluationRequest {
            node_ids: Some(vec![node.id]),
            policies: Some(vec![]), // Empty policies list
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
        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
        };

        let request = PolicyEvaluationRequest {
            node_ids: Some(vec![node.id]),
            policies: None,
            store_results: Some(false),
        };

        let result = evaluate_policies(State(app_state), Json(request)).await;
        assert!(result.is_ok());

        let _response = result.unwrap().0;
        // Should have recorded evaluation time
        // Removed useless comparison - evaluation_time_ms is always >= 0
    }
}
