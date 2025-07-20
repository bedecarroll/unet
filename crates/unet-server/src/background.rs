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
            self.evaluate_single_node(&mut policy_service, node, &mut stats)
                .await;
        }

        stats
    }

    async fn evaluate_single_node(
        &self,
        policy_service: &mut unet_core::policy_integration::PolicyService,
        node: &unet_core::models::Node,
        stats: &mut EvaluationStats,
    ) {
        match policy_service.evaluate_node(&*self.datastore, node).await {
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

#[cfg(test)]
mod tests {
    use super::*;
    use unet_core::{
        datastore::{DataStore, sqlite::SqliteStore},
        models::*,
        policy::PolicyExecutionResult,
        policy_integration::PolicyService,
    };

    async fn setup_test_datastore() -> SqliteStore {
        SqliteStore::new("sqlite::memory:").await.unwrap()
    }

    fn create_test_node() -> Node {
        let mut node = Node::new(
            "test-node".to_string(),
            "example.com".to_string(),
            Vendor::Cisco,
            DeviceRole::Router,
        );
        node.model = "ASR1000".to_string(); // Required field
        node
    }

    #[tokio::test]
    async fn test_background_tasks_new() {
        let datastore = setup_test_datastore().await;
        let config = Config::default();
        let policy_service = PolicyService::with_local_dir("/tmp");

        let background_tasks = BackgroundTasks::new(config, Arc::new(datastore), policy_service);

        assert_eq!(background_tasks.config.git.sync_interval, 300);
    }

    #[tokio::test]
    async fn test_policy_evaluation_task_run_empty_nodes() {
        let datastore = setup_test_datastore().await;
        let policy_service = PolicyService::with_local_dir("/tmp");

        let task = PolicyEvaluationTask {
            datastore: Arc::new(datastore),
            policy_service,
            interval_seconds: 1,
        };

        let result = task.get_nodes_for_evaluation().await;
        assert!(result.is_ok());
        let nodes = result.unwrap();
        assert!(nodes.is_empty());
    }

    #[tokio::test]
    async fn test_policy_evaluation_task_with_nodes() {
        let datastore = setup_test_datastore().await;
        let node = create_test_node();
        let _stored_node = datastore.create_node(&node).await.unwrap();

        let policy_service = PolicyService::with_local_dir("/tmp");

        let task = PolicyEvaluationTask {
            datastore: Arc::new(datastore),
            policy_service,
            interval_seconds: 1,
        };

        let result = task.get_nodes_for_evaluation().await;
        assert!(result.is_ok());
        let nodes = result.unwrap();
        assert_eq!(nodes.len(), 1);
    }

    #[tokio::test]
    async fn test_load_policies_for_evaluation() {
        let datastore = setup_test_datastore().await;
        let policy_service = PolicyService::with_local_dir("/tmp");

        let task = PolicyEvaluationTask {
            datastore: Arc::new(datastore),
            policy_service,
            interval_seconds: 1,
        };

        let result = task.load_policies_for_evaluation();
        assert!(result.is_ok());
        let _policies = result.unwrap();
        // PolicyService might have default policies even with local directory
        // No need to assert len() >= 0 since it's always true for usize
    }

    #[tokio::test]
    async fn test_evaluate_nodes_empty() {
        let datastore = setup_test_datastore().await;
        let policy_service = PolicyService::with_local_dir("/tmp");

        let task = PolicyEvaluationTask {
            datastore: Arc::new(datastore),
            policy_service,
            interval_seconds: 1,
        };

        let stats = task.evaluate_nodes(&[]).await;
        assert_eq!(stats.total_results, 0);
        assert_eq!(stats.successful_evaluations, 0);
        assert_eq!(stats.failed_evaluations, 0);
    }

    #[tokio::test]
    async fn test_evaluate_nodes_with_data() {
        let datastore = setup_test_datastore().await;
        let policy_service = PolicyService::with_local_dir("/tmp");
        let nodes = vec![create_test_node()];

        let task = PolicyEvaluationTask {
            datastore: Arc::new(datastore),
            policy_service,
            interval_seconds: 1,
        };

        let stats = task.evaluate_nodes(&nodes).await;
        assert_eq!(stats.successful_evaluations, 1);
        assert_eq!(stats.failed_evaluations, 0);
    }

    #[tokio::test]
    async fn test_evaluation_stats_new() {
        let stats = EvaluationStats::new();
        assert_eq!(stats.total_results, 0);
        assert_eq!(stats.successful_evaluations, 0);
        assert_eq!(stats.failed_evaluations, 0);
    }

    #[tokio::test]
    async fn test_evaluation_stats_record_success() {
        let mut stats = EvaluationStats::new();
        stats.record_success(5);
        assert_eq!(stats.total_results, 5);
        assert_eq!(stats.successful_evaluations, 1);
        assert_eq!(stats.failed_evaluations, 0);
    }

    #[tokio::test]
    async fn test_evaluation_stats_record_failure() {
        let mut stats = EvaluationStats::new();
        stats.record_failure();
        assert_eq!(stats.total_results, 0);
        assert_eq!(stats.successful_evaluations, 0);
        assert_eq!(stats.failed_evaluations, 1);
    }

    #[tokio::test]
    async fn test_run_policy_evaluation_cycle() {
        let datastore = setup_test_datastore().await;
        let policy_service = PolicyService::with_local_dir("/tmp");

        let task = PolicyEvaluationTask {
            datastore: Arc::new(datastore),
            policy_service,
            interval_seconds: 1,
        };

        task.run_policy_evaluation_cycle().await;
    }

    #[tokio::test]
    async fn test_evaluate_all_policies_no_nodes() {
        let datastore = setup_test_datastore().await;
        let policy_service = PolicyService::with_local_dir("/tmp");

        let task = PolicyEvaluationTask {
            datastore: Arc::new(datastore),
            policy_service,
            interval_seconds: 1,
        };

        let result = task.evaluate_all_policies().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_evaluate_all_policies_with_node() {
        let datastore = setup_test_datastore().await;
        let node = create_test_node();
        let _stored_node = datastore.create_node(&node).await.unwrap();

        let policy_service = PolicyService::with_local_dir("/tmp");

        let task = PolicyEvaluationTask {
            datastore: Arc::new(datastore),
            policy_service,
            interval_seconds: 1,
        };

        let result = task.evaluate_all_policies().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_log_evaluation_results() {
        let nodes = vec![create_test_node()];
        let mut stats = EvaluationStats::new();
        stats.record_success(3);
        stats.record_failure();

        PolicyEvaluationTask::log_evaluation_results(&nodes, &stats, Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_store_evaluation_results() {
        use unet_core::policy::{
            Action, ActionExecutionResult, ActionResult, EvaluationResult, PolicyRule,
        };

        let datastore = setup_test_datastore().await;
        let policy_service = PolicyService::with_local_dir("/tmp");
        let node = create_test_node();

        let rule = PolicyRule {
            id: Some("test-rule".to_string()),
            condition: unet_core::policy::Condition::Comparison {
                field: unet_core::policy::FieldRef {
                    path: vec!["vendor".to_string()],
                },
                operator: unet_core::policy::ComparisonOperator::Equal,
                value: unet_core::policy::Value::String("cisco".to_string()),
            },
            action: Action::Assert {
                field: unet_core::policy::FieldRef {
                    path: vec!["version".to_string()],
                },
                expected: unet_core::policy::Value::String("15.1".to_string()),
            },
        };

        let results = vec![PolicyExecutionResult::new(
            rule,
            EvaluationResult::Satisfied {
                action: Action::Assert {
                    field: unet_core::policy::FieldRef {
                        path: vec!["version".to_string()],
                    },
                    expected: unet_core::policy::Value::String("15.1".to_string()),
                },
            },
            Some(ActionExecutionResult {
                result: ActionResult::Success {
                    message: "Test passed".to_string(),
                },
                rollback_data: None,
            }),
        )];

        let task = PolicyEvaluationTask {
            datastore: Arc::new(datastore),
            policy_service: policy_service.clone(),
            interval_seconds: 1,
        };

        task.store_evaluation_results(&policy_service, &node, &results)
            .await;
    }
}
