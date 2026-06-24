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
#[path = "response_handling_tests.rs"]
mod response_handling_tests;
