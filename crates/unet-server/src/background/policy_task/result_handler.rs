//! Result handling and logging for policy evaluation

use crate::background::scheduler::EvaluationStats;
use std::time::Duration;
use tracing::info;
use unet_core::models::Node;

/// Handler for policy evaluation results and logging
pub struct ResultHandler;

impl ResultHandler {
    /// Log the results of policy evaluation
    pub fn log_evaluation_results(nodes: &[Node], stats: &EvaluationStats, duration: Duration) {
        info!(
            "Policy evaluation completed: {} nodes processed, {} successful, {} failed, {} total results, took {:?}",
            nodes.len(),
            stats.successful_evaluations(),
            stats.failed_evaluations(),
            stats.total_results(),
            duration
        );
    }
}
