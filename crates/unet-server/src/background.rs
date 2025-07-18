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
    #[allow(clippy::cognitive_complexity)]
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

            if let Err(e) = self.evaluate_all_policies().await {
                error!("Policy evaluation failed: {}", e);
            }
        }
    }

    /// Evaluate policies for all nodes
    #[allow(clippy::cognitive_complexity)]
    async fn evaluate_all_policies(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let start_time = std::time::Instant::now();

        // Get all nodes that need policy evaluation
        let nodes = self
            .datastore
            .get_nodes_for_policy_evaluation()
            .await
            .map_err(|e| format!("Failed to get nodes: {e}"))?;

        if nodes.is_empty() {
            debug!("No nodes found for policy evaluation");
            return Ok(());
        }

        info!("Evaluating policies for {} nodes", nodes.len());

        // Clone policy service for mutable operations
        let mut policy_service = self.policy_service.clone();

        // Load policies
        let policies = policy_service
            .load_policies()
            .map_err(|e| format!("Failed to load policies: {e}"))?;

        if policies.is_empty() {
            debug!("No policies loaded for evaluation");
            return Ok(());
        }

        info!("Loaded {} policies for evaluation", policies.len());

        let mut total_results = 0;
        let mut successful_evaluations = 0;
        let mut failed_evaluations = 0;

        // Evaluate policies for each node
        for node in &nodes {
            match policy_service.evaluate_node(&*self.datastore, node).await {
                Ok(results) => {
                    total_results += results.len();
                    successful_evaluations += 1;

                    // Store results in the database
                    if let Err(e) = policy_service
                        .store_results(&*self.datastore, &node.id, &results)
                        .await
                    {
                        warn!("Failed to store policy results for node {}: {}", node.id, e);
                    }

                    debug!(
                        "Evaluated {} policies for node {} ({})",
                        results.len(),
                        node.id,
                        node.name
                    );
                }
                Err(e) => {
                    failed_evaluations += 1;
                    error!(
                        "Failed to evaluate policies for node {} ({}): {}",
                        node.id, node.name, e
                    );
                }
            }
        }

        let duration = start_time.elapsed();

        info!(
            "Policy evaluation completed: {} nodes processed, {} successful, {} failed, {} total results, took {:?}",
            nodes.len(),
            successful_evaluations,
            failed_evaluations,
            total_results,
            duration
        );

        Ok(())
    }
}
