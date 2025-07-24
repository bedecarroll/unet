//! Bulk node evaluation logic for policy execution

use std::collections::HashMap;
use unet_core::policy::PolicyExecutionResult;
use unet_core::prelude::{DataStore, Node, PolicyService};
use uuid::Uuid;

use super::node_evaluator::process_node_evaluation;
use crate::handlers::policies::types::PolicyEvaluationSummary;

/// Evaluate all nodes against policies and return results and summary
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
