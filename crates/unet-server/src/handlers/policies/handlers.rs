//! HTTP handler functions for policy endpoints

use axum::{
    Json,
    extract::{Query, State},
};
use std::collections::HashMap;
use tracing::{error, info, warn};
use unet_core::policy::PolicyRule;

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
#[allow(clippy::cognitive_complexity)]
pub async fn evaluate_policies(
    State(state): State<AppState>,
    Json(request): Json<PolicyEvaluationRequest>,
) -> ServerResult<Json<PolicyEvaluationResponse>> {
    info!(
        "Evaluating policies for {} nodes",
        request.node_ids.as_ref().map_or(0, std::vec::Vec::len)
    );

    let start_time = std::time::Instant::now();

    // Get policy service from app state
    let mut policy_service = state.policy_service.clone();

    // Determine which nodes to evaluate
    let nodes = get_nodes_for_evaluation(&*state.datastore, request.node_ids.as_ref()).await?;

    if nodes.is_empty() {
        return Ok(Json(PolicyEvaluationResponse {
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
        }));
    }

    // Load policies (either from request or from configured source)
    let policies = if let Some(policies) = request.policies {
        policies
    } else {
        match policy_service.load_policies() {
            Ok(policies) => policies,
            Err(e) => {
                error!("Failed to load policies: {}", e);
                return Err(ServerError::Internal(format!(
                    "Failed to load policies: {e}"
                )));
            }
        }
    };

    if policies.is_empty() {
        warn!("No policies found for evaluation");
        return Ok(Json(PolicyEvaluationResponse {
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
        }));
    }

    let mut all_results = HashMap::new();
    let mut summary = PolicyEvaluationSummary {
        total_rules: 0,
        satisfied_rules: 0,
        unsatisfied_rules: 0,
        error_rules: 0,
        compliance_failures: 0,
    };

    // Evaluate policies for each node
    for node in &nodes {
        process_node_evaluation(
            &mut policy_service,
            &*state.datastore,
            node,
            request.store_results.unwrap_or(true),
            &mut summary,
            &mut all_results,
        )
        .await;
    }

    let evaluation_time = start_time.elapsed();

    info!(
        "Policy evaluation completed: {} nodes, {} policies, {:?}",
        nodes.len(),
        policies.len(),
        evaluation_time
    );

    Ok(Json(PolicyEvaluationResponse {
        results: all_results,
        nodes_evaluated: nodes.len(),
        policies_evaluated: policies.len(),
        evaluation_time_ms: u64::try_from(evaluation_time.as_millis()).unwrap_or(0),
        summary,
    }))
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
