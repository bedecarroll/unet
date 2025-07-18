//! Background tasks for the μNet server

use std::sync::Arc;
use std::time::Duration;
use tokio::time::{interval, sleep};
use tracing::{debug, error, info, warn};

use unet_core::{config::Config, datastore::DataStore, policy_integration::PolicyService};

/// Background task manager
pub struct BackgroundTasks {
    config: Config,
    datastore: Arc<dyn DataStore + Send + Sync>,
    policy_service: PolicyService,
}

impl BackgroundTasks {
    /// Create a new background task manager
    pub fn new(
        config: Config,
        datastore: Arc<dyn DataStore + Send + Sync>,
        policy_service: PolicyService,
    ) -> Self {
        Self {
            config,
            datastore,
            policy_service,
        }
    }

    /// Start all background tasks
    pub fn start(&self) {
        info!("Starting background tasks");

        // Start policy evaluation task
        let policy_task = PolicyEvaluationTask {
            datastore: self.datastore.clone(),
            policy_service: self.policy_service.clone(),
            interval_seconds: self.config.git.sync_interval,
        };

        tokio::spawn(async move {
            policy_task.run().await;
        });

        info!("Background tasks started");
    }
}

/// Background task for periodic policy evaluation
struct PolicyEvaluationTask {
    datastore: Arc<dyn DataStore + Send + Sync>,
    policy_service: PolicyService,
    interval_seconds: u64,
}

impl PolicyEvaluationTask {
    /// Run the policy evaluation task
    async fn run(&self) {
        info!(
            "Starting policy evaluation background task with interval: {}s",
            self.interval_seconds
        );

        // Wait a bit before starting the first evaluation
        sleep(Duration::from_secs(30)).await;

        let mut interval = interval(Duration::from_secs(self.interval_seconds));

        loop {
            interval.tick().await;
            debug!("Running periodic policy evaluation");
            self.run_policy_evaluation_cycle().await;
        }
    }

    async fn run_policy_evaluation_cycle(&self) {
        if let Err(e) = self.evaluate_all_policies().await {
            error!("Policy evaluation failed: {}", e);
        }
    }

    /// Evaluate policies for all nodes
    async fn evaluate_all_policies(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

        let stats = self.evaluate_nodes(&nodes).await;
        Self::log_evaluation_results(&nodes, &stats, start_time.elapsed());

        Ok(())
    }

    async fn get_nodes_for_evaluation(
        &self,
    ) -> Result<Vec<unet_core::models::Node>, Box<dyn std::error::Error + Send + Sync>> {
        self.datastore
            .get_nodes_for_policy_evaluation()
            .await
            .map_err(|e| format!("Failed to get nodes: {e}").into())
    }

    fn load_policies_for_evaluation(
        &self,
    ) -> Result<Vec<unet_core::policy::PolicyRule>, Box<dyn std::error::Error + Send + Sync>> {
        let mut policy_service = self.policy_service.clone();
        let policies = policy_service
            .load_policies()
            .map_err(|e| format!("Failed to load policies: {e}"))?;

        info!("Loaded {} policies for evaluation", policies.len());
        Ok(policies)
    }

    async fn evaluate_nodes(&self, nodes: &[unet_core::models::Node]) -> EvaluationStats {
        info!("Evaluating policies for {} nodes", nodes.len());

        let mut stats = EvaluationStats::new();
        let mut policy_service = self.policy_service.clone();

        for node in nodes {
            match policy_service.evaluate_node(&*self.datastore, node).await {
                Ok(results) => {
                    stats.record_success(results.len());
                    self.store_evaluation_results(&policy_service, node, &results)
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

        stats
    }

    async fn store_evaluation_results(
        &self,
        policy_service: &unet_core::policy_integration::PolicyService,
        node: &unet_core::models::Node,
        results: &[unet_core::policy::PolicyExecutionResult],
    ) {
        if let Err(e) = policy_service
            .store_results(&*self.datastore, &node.id, results)
            .await
        {
            warn!("Failed to store policy results for node {}: {}", node.id, e);
        }
    }

    fn log_evaluation_results(
        nodes: &[unet_core::models::Node],
        stats: &EvaluationStats,
        duration: std::time::Duration,
    ) {
        info!(
            "Policy evaluation completed: {} nodes processed, {} successful, {} failed, {} total results, took {:?}",
            nodes.len(),
            stats.successful_evaluations,
            stats.failed_evaluations,
            stats.total_results,
            duration
        );
    }
}

struct EvaluationStats {
    total_results: usize,
    successful_evaluations: usize,
    failed_evaluations: usize,
}

impl EvaluationStats {
    const fn new() -> Self {
        Self {
            total_results: 0,
            successful_evaluations: 0,
            failed_evaluations: 0,
        }
    }

    const fn record_success(&mut self, result_count: usize) {
        self.total_results += result_count;
        self.successful_evaluations += 1;
    }

    const fn record_failure(&mut self) {
        self.failed_evaluations += 1;
    }
}
