//! Policy evaluation logic and utilities

use crate::error::ServerError;
use std::collections::HashMap;
use tracing::{error, warn};
use unet_core::policy::PolicyExecutionResult;
use unet_core::prelude::{DataStore, Node, PolicyService};
use uuid::Uuid;

use super::types::PolicyEvaluationSummary;

/// Get nodes for policy evaluation based on request
pub async fn get_nodes_for_evaluation(
    datastore: &dyn DataStore,
    node_ids: Option<&Vec<Uuid>>,
) -> Result<Vec<Node>, ServerError> {
    match node_ids {
        Some(node_ids) => get_specific_nodes(datastore, node_ids).await,
        None => get_all_nodes_for_evaluation(datastore).await,
    }
}

async fn get_specific_nodes(
    datastore: &dyn DataStore,
    node_ids: &[Uuid],
) -> Result<Vec<Node>, ServerError> {
    let mut nodes = Vec::new();
    for node_id in node_ids {
        if let Some(node) = fetch_node_by_id(datastore, node_id).await? {
            nodes.push(node);
        } else {
            warn!("Node {} not found for policy evaluation", node_id);
        }
    }
    Ok(nodes)
}

async fn fetch_node_by_id(
    datastore: &dyn DataStore,
    node_id: &Uuid,
) -> Result<Option<Node>, ServerError> {
    datastore.get_node(node_id).await.map_err(|e| {
        error!("Failed to get node {}: {}", node_id, e);
        ServerError::Internal(format!("Failed to get node {node_id}: {e}"))
    })
}

async fn get_all_nodes_for_evaluation(datastore: &dyn DataStore) -> Result<Vec<Node>, ServerError> {
    datastore
        .get_nodes_for_policy_evaluation()
        .await
        .map_err(|e| {
            error!("Failed to get nodes for policy evaluation: {}", e);
            ServerError::Internal(format!("Failed to get nodes for policy evaluation: {e}"))
        })
}

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
            let error_result = PolicyExecutionResult::new_error_with_id(
                Some("evaluation".to_string()),
                format!("Failed to evaluate policies: {e}"),
            );
            all_results.insert(node.id, vec![error_result]);
            summary.error_rules += 1;
        }
    }
}
