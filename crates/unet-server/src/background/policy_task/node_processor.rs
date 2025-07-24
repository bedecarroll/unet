//! Node processing logic for policy evaluation

use crate::background::scheduler::EvaluationStats;
use std::sync::Arc;
use tracing::{debug, error};
use unet_core::{
    datastore::DataStore, models::Node, policy::PolicyExecutionResult,
    policy_integration::PolicyService,
};

/// Node processor for evaluating policies on individual nodes
pub struct NodeProcessor<'a> {
    datastore: &'a Arc<dyn DataStore + Send + Sync>,
    policy_service: &'a PolicyService,
}

impl<'a> NodeProcessor<'a> {
    /// Create a new node processor
    pub fn new(
        datastore: &'a Arc<dyn DataStore + Send + Sync>,
        policy_service: &'a PolicyService,
    ) -> Self {
        Self {
            datastore,
            policy_service,
        }
    }

    /// Evaluate policies on all provided nodes
    pub async fn evaluate_nodes(&self, nodes: &[Node]) -> EvaluationStats {
        let mut stats = EvaluationStats::new();
        let mut policy_service = self.policy_service.clone();

        for node in nodes {
            self.evaluate_single_node(&mut policy_service, node, &mut stats)
                .await;
        }

        stats
    }

    /// Evaluate policies on a single node
    async fn evaluate_single_node(
        &self,
        policy_service: &mut PolicyService,
        node: &Node,
        stats: &mut EvaluationStats,
    ) {
        match policy_service.evaluate_node(&**self.datastore, node).await {
            Ok(results) => {
                stats.record_success(results.len());
                self.store_evaluation_results(policy_service, node, &results)
                    .await;
                debug!(
                    "Evaluated {} policies for node {} ({})",
                    results.len(),
                    node.id,
                    node.name
                );
            }
            Err(e) => {
                stats.record_failure();
                error!(
                    "Failed to evaluate policies for node {} ({}): {}",
                    node.id, node.name, e
                );
            }
        }
    }

    /// Store evaluation results for a node
    pub async fn store_evaluation_results(
        &self,
        policy_service: &PolicyService,
        node: &Node,
        results: &[PolicyExecutionResult],
    ) {
        if let Err(e) = policy_service
            .store_results(&**self.datastore, &node.id, results)
            .await
        {
            tracing::warn!("Failed to store policy results for node {}: {}", node.id, e);
        }
    }
}
