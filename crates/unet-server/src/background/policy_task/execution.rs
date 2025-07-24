//! Policy evaluation execution logic

use std::sync::Arc;
use std::time::Duration;
use tokio::time::{interval, sleep};
use tracing::{debug, error, info};
use unet_core::{
    datastore::DataStore, models::Node, policy::PolicyRule, policy_integration::PolicyService,
};

/// Task execution handler for policy evaluation
pub struct TaskExecutor {
    pub datastore: Arc<dyn DataStore + Send + Sync>,
    pub policy_service: PolicyService,
    interval_seconds: u64,
}

impl TaskExecutor {
    /// Create a new task executor
    pub fn new(
        datastore: Arc<dyn DataStore + Send + Sync>,
        policy_service: PolicyService,
        interval_seconds: u64,
    ) -> Self {
        Self {
            datastore,
            policy_service,
            interval_seconds,
        }
    }

    /// Run the policy evaluation task
    pub async fn run(&mut self) {
        info!(
            "Starting policy evaluation background task with interval: {}s",
            self.interval_seconds
        );

        sleep(Duration::from_secs(30)).await;

        let mut interval = interval(Duration::from_secs(self.interval_seconds));

        loop {
            interval.tick().await;
            debug!("Running periodic policy evaluation");
            self.run_policy_evaluation_cycle().await;
        }
    }

    /// Run a single policy evaluation cycle
    pub async fn run_policy_evaluation_cycle(&mut self) {
        if let Err(e) = self.evaluate_all_policies().await {
            error!("Policy evaluation failed: {}", e);
        }
    }

    /// Evaluate policies for all nodes
    pub async fn evaluate_all_policies(
        &mut self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let start_time = std::time::Instant::now();

        let nodes = self.get_nodes_for_evaluation().await?;
        if nodes.is_empty() {
            debug!("No nodes found for policy evaluation");
            return Ok(());
        }

        let policies = self.load_policies_for_evaluation()?;
        if policies.is_empty() {
            debug!("No policies loaded for evaluation");
            return Ok(());
        }

        let stats =
            super::node_processor::NodeProcessor::new(&self.datastore, &self.policy_service)
                .evaluate_nodes(&nodes)
                .await;

        super::result_handler::ResultHandler::log_evaluation_results(
            &nodes,
            &stats,
            start_time.elapsed(),
        );

        Ok(())
    }

    /// Get nodes for evaluation from datastore
    pub async fn get_nodes_for_evaluation(
        &self,
    ) -> Result<Vec<Node>, Box<dyn std::error::Error + Send + Sync>> {
        let nodes = self
            .datastore
            .get_nodes_for_policy_evaluation()
            .await
            .map_err(|e| format!("Failed to get nodes for evaluation: {e}"))?;

        debug!("Found {} nodes for policy evaluation", nodes.len());
        Ok(nodes)
    }

    /// Load policies for evaluation from policy service
    pub fn load_policies_for_evaluation(
        &mut self,
    ) -> Result<Vec<PolicyRule>, Box<dyn std::error::Error + Send + Sync>> {
        let policies = self
            .policy_service
            .load_policies()
            .map_err(|e| format!("Failed to load policies: {e}"))?;

        debug!("Loaded {} policies for evaluation", policies.len());
        Ok(policies)
    }
}
