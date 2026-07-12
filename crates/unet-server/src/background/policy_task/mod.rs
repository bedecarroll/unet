//! Policy evaluation background task
//!
//! This module provides background task functionality for periodic policy evaluation
//! on network nodes, including task execution, node processing, and result handling.

use std::sync::Arc;
use unet_core::{datastore::DataStore, policy_integration::PolicyService};

pub use self::execution::TaskExecutor;

mod execution;
pub mod node_processor;
pub mod result_handler;

#[cfg(test)]
mod tests;

/// Background task for periodic policy evaluation
pub struct PolicyEvaluationTask {
    pub executor: TaskExecutor,
}

impl PolicyEvaluationTask {
    /// Create a new policy evaluation task
    pub fn new(
        datastore: Arc<dyn DataStore + Send + Sync>,
        policy_service: PolicyService,
        interval_seconds: u64,
    ) -> Self {
        Self {
            executor: TaskExecutor::new(datastore, policy_service, interval_seconds),
        }
    }

    /// Run the policy evaluation task
    pub async fn run(&mut self) {
        self.executor.run().await;
    }
}
