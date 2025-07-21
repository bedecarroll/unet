//! Policy evaluation background task

use crate::background::scheduler::EvaluationStats;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{interval, sleep};
use tracing::{debug, error, info, warn};
use unet_core::{
    datastore::DataStore,
    models::Node,
    policy::{PolicyExecutionResult, PolicyRule},
    policy_integration::PolicyService,
};

/// Background task for periodic policy evaluation
pub struct PolicyEvaluationTask {
    datastore: Arc<dyn DataStore + Send + Sync>,
    policy_service: PolicyService,
    interval_seconds: u64,
}

impl PolicyEvaluationTask {
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

        let stats = self.evaluate_nodes(&nodes).await;
        Self::log_evaluation_results(&nodes, &stats, start_time.elapsed());

        Ok(())
    }

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

    pub async fn evaluate_nodes(&self, nodes: &[Node]) -> super::scheduler::EvaluationStats {
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
        policy_service: &mut PolicyService,
        node: &Node,
        stats: &mut super::scheduler::EvaluationStats,
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
        policy_service: &PolicyService,
        node: &Node,
        results: &[PolicyExecutionResult],
    ) {
        if let Err(e) = policy_service
            .store_results(&*self.datastore, &node.id, results)
            .await
        {
            warn!("Failed to store policy results for node {}: {}", node.id, e);
        }
    }

    pub fn log_evaluation_results(
        nodes: &[Node],
        stats: &super::scheduler::EvaluationStats,
        duration: std::time::Duration,
    ) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::background::scheduler::EvaluationStats;
    use migration::{Migrator, MigratorTrait};
    use std::sync::Arc;
    use unet_core::{
        datastore::sqlite::SqliteStore,
        models::*,
        policy::{Action, ActionExecutionResult, ActionResult, EvaluationResult, PolicyRule},
        policy_integration::PolicyService,
    };

    async fn setup_test_datastore() -> SqliteStore {
        let store = SqliteStore::new("sqlite::memory:").await.unwrap();
        Migrator::up(store.connection(), None).await.unwrap();
        store
    }

    fn create_test_node() -> Node {
        let mut node = Node::new(
            "test-node".to_string(),
            "example.com".to_string(),
            Vendor::Cisco,
            DeviceRole::Router,
        );
        node.model = "ASR1000".to_string();
        node
    }

    #[tokio::test]
    async fn test_policy_evaluation_task_run_empty_nodes() {
        let datastore = setup_test_datastore().await;
        let policy_service = PolicyService::with_local_dir("/tmp");

        let task = PolicyEvaluationTask::new(Arc::new(datastore), policy_service, 1);

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

        let task = PolicyEvaluationTask::new(Arc::new(datastore), policy_service, 1);

        let result = task.get_nodes_for_evaluation().await;
        assert!(result.is_ok());
        let nodes = result.unwrap();
        assert_eq!(nodes.len(), 1);
    }

    #[tokio::test]
    async fn test_load_policies_for_evaluation() {
        let datastore = setup_test_datastore().await;
        let policy_service = PolicyService::with_local_dir("/tmp");

        let mut task = PolicyEvaluationTask::new(Arc::new(datastore), policy_service, 1);

        let result = task.load_policies_for_evaluation();
        assert!(result.is_ok());
        let _policies = result.unwrap();
    }

    #[tokio::test]
    async fn test_evaluate_nodes_empty() {
        let datastore = setup_test_datastore().await;
        let policy_service = PolicyService::with_local_dir("/tmp");

        let task = PolicyEvaluationTask::new(Arc::new(datastore), policy_service, 1);

        let stats = task.evaluate_nodes(&[]).await;
        assert_eq!(stats.total_results(), 0);
        assert_eq!(stats.successful_evaluations(), 0);
        assert_eq!(stats.failed_evaluations(), 0);
    }

    #[tokio::test]
    async fn test_evaluate_nodes_with_data() {
        let datastore = setup_test_datastore().await;
        let policy_service = PolicyService::with_local_dir("/tmp");
        let nodes = vec![create_test_node()];

        let task = PolicyEvaluationTask::new(Arc::new(datastore), policy_service, 1);

        let stats = task.evaluate_nodes(&nodes).await;
        assert_eq!(stats.successful_evaluations(), 1);
        assert_eq!(stats.failed_evaluations(), 0);
    }

    #[tokio::test]
    async fn test_run_policy_evaluation_cycle() {
        let datastore = setup_test_datastore().await;
        let policy_service = PolicyService::with_local_dir("/tmp");

        let mut task = PolicyEvaluationTask::new(Arc::new(datastore), policy_service, 1);

        task.run_policy_evaluation_cycle().await;
    }

    #[tokio::test]
    async fn test_evaluate_all_policies_no_nodes() {
        let datastore = setup_test_datastore().await;
        let policy_service = PolicyService::with_local_dir("/tmp");

        let mut task = PolicyEvaluationTask::new(Arc::new(datastore), policy_service, 1);

        let result = task.evaluate_all_policies().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_evaluate_all_policies_with_node() {
        let datastore = setup_test_datastore().await;
        let node = create_test_node();
        let _stored_node = datastore.create_node(&node).await.unwrap();

        let policy_service = PolicyService::with_local_dir("/tmp");

        let mut task = PolicyEvaluationTask::new(Arc::new(datastore), policy_service, 1);

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

        let task = PolicyEvaluationTask::new(Arc::new(datastore), policy_service.clone(), 1);

        task.store_evaluation_results(&policy_service, &node, &results)
            .await;
    }
}
