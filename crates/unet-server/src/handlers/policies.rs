//! Policy management HTTP handlers

use axum::{
    Json,
    extract::{Query, State},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    error::{ServerError, ServerResult},
    server::AppState,
};
use unet_core::policy::{PolicyExecutionResult, PolicyRule};
use unet_core::prelude::{DataStore, Node, PolicyService};

/// Request to evaluate policies against a node
#[derive(Debug, Deserialize)]
pub struct PolicyEvaluationRequest {
    /// Optional node IDs to evaluate (if empty, evaluates all nodes)
    pub node_ids: Option<Vec<Uuid>>,
    /// Optional policy rules to use (if empty, loads from configured source)
    pub policies: Option<Vec<PolicyRule>>,
    /// Whether to store results in the database
    pub store_results: Option<bool>,
}

/// Response for policy evaluation
#[derive(Debug, Serialize)]
pub struct PolicyEvaluationResponse {
    /// Results by node ID
    pub results: HashMap<Uuid, Vec<PolicyExecutionResult>>,
    /// Number of nodes evaluated
    pub nodes_evaluated: usize,
    /// Number of policies evaluated per node
    pub policies_evaluated: usize,
    /// Total evaluation time in milliseconds
    pub evaluation_time_ms: u64,
    /// Summary of results
    pub summary: PolicyEvaluationSummary,
}

/// Summary of policy evaluation results
#[derive(Debug, Serialize)]
pub struct PolicyEvaluationSummary {
    /// Total number of policy rules executed
    pub total_rules: usize,
    /// Number of satisfied rules
    pub satisfied_rules: usize,
    /// Number of unsatisfied rules
    pub unsatisfied_rules: usize,
    /// Number of rules that failed with errors
    pub error_rules: usize,
    /// Number of compliance failures
    pub compliance_failures: usize,
}

/// Query parameters for policy results
#[derive(Debug, Deserialize)]
pub struct PolicyResultsQuery {
    /// Filter by node ID
    pub node_id: Option<Uuid>,
    /// Limit number of results
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
}

/// Response for policy results
#[derive(Debug, Serialize)]
pub struct PolicyResultsResponse {
    /// Policy execution results
    pub results: Vec<PolicyExecutionResult>,
    /// Total number of results available
    pub total_count: usize,
    /// Number of results returned
    pub returned_count: usize,
}

/// Get nodes for policy evaluation based on request
async fn get_nodes_for_evaluation(
    datastore: &dyn DataStore,
    node_ids: Option<&Vec<Uuid>>,
) -> Result<Vec<Node>, ServerError> {
    if let Some(node_ids) = node_ids {
        let mut nodes = Vec::new();
        for node_id in node_ids {
            match datastore.get_node(node_id).await {
                Ok(Some(node)) => nodes.push(node),
                Ok(None) => {
                    warn!("Node {} not found for policy evaluation", node_id);
                }
                Err(e) => {
                    error!("Failed to get node {}: {}", node_id, e);
                    return Err(ServerError::Internal(format!(
                        "Failed to get node {node_id}: {e}"
                    )));
                }
            }
        }
        Ok(nodes)
    } else {
        // Evaluate all nodes
        match datastore.get_nodes_for_policy_evaluation().await {
            Ok(nodes) => Ok(nodes),
            Err(e) => {
                error!("Failed to get nodes for policy evaluation: {}", e);
                Err(ServerError::Internal(format!(
                    "Failed to get nodes for policy evaluation: {e}"
                )))
            }
        }
    }
}

/// Process evaluation results for a single node
async fn process_node_evaluation(
    policy_service: &mut PolicyService,
    datastore: &dyn DataStore,
    node: &Node,
    store_results: bool,
    summary: &mut PolicyEvaluationSummary,
    all_results: &mut HashMap<Uuid, Vec<PolicyExecutionResult>>,
) {
    match policy_service.evaluate_node(datastore, node).await {
        Ok(results) => {
            // Update summary statistics
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

            // Store results if requested
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
            // Create error result for this node
            let error_result = PolicyExecutionResult::new_error(
                "evaluation",
                format!("Failed to evaluate policies: {e}"),
            );
            all_results.insert(node.id, vec![error_result]);
            summary.error_rules += 1;
        }
    }
}

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
