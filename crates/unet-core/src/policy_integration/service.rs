//! Policy service that orchestrates policy loading, evaluation, and result storage

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use uuid::Uuid;

use crate::config::GitConfig;
use crate::datastore::{DataStore, DataStoreResult};
use crate::models::Node;
use crate::policy::{
    OrchestrationConfig, PolicyExecutionResult, PolicyLoader, PolicyOrchestrator, PolicyResult,
    PolicyRule,
};

use super::engine::{DefaultPolicyEvaluationEngine, PolicyEvaluationEngine};

/// Policy service that orchestrates policy loading, evaluation, and result storage
#[derive(Clone)]
pub struct PolicyService {
    loader: PolicyLoader,
    engine: Arc<dyn PolicyEvaluationEngine>,
    orchestrator: PolicyOrchestrator,
    runtime_status: Arc<PolicyRuntimeStatus>,
}

#[derive(Debug)]
struct PolicyRuntimeStatus {
    evaluation_interval_seconds: u64,
    last_evaluation: RwLock<Option<SystemTime>>,
}

impl PolicyRuntimeStatus {
    const fn new(evaluation_interval_seconds: u64) -> Self {
        Self {
            evaluation_interval_seconds,
            last_evaluation: RwLock::new(None),
        }
    }
}

impl PolicyService {
    /// Creates a new policy service with Git configuration
    #[must_use]
    pub fn new(git_config: GitConfig) -> Self {
        let runtime_status = Arc::new(PolicyRuntimeStatus::new(git_config.sync_interval));
        let loader = PolicyLoader::new(git_config);
        let engine = Arc::new(DefaultPolicyEvaluationEngine::new());
        let orchestrator = PolicyOrchestrator::new(OrchestrationConfig::default());

        Self {
            loader,
            engine,
            orchestrator,
            runtime_status,
        }
    }

    /// Creates a new policy service with local directory
    #[must_use]
    pub fn with_local_dir(policies_directory: &str) -> Self {
        let git_config = GitConfig {
            repository_url: None,
            local_directory: Some(policies_directory.to_string()),
            branch: "main".to_string(),
            auth_token: None,
            sync_interval: 300,
            policies_repo: None,
            templates_repo: None,
        };
        let loader = PolicyLoader::new(git_config).with_local_dir(policies_directory);
        let engine = Arc::new(DefaultPolicyEvaluationEngine::new());
        let orchestrator = PolicyOrchestrator::new(OrchestrationConfig::default());
        let runtime_status = Arc::new(PolicyRuntimeStatus::new(300));

        Self {
            loader,
            engine,
            orchestrator,
            runtime_status,
        }
    }

    /// Creates a new policy service with custom evaluation engine
    pub fn with_engine(git_config: GitConfig, engine: Arc<dyn PolicyEvaluationEngine>) -> Self {
        let runtime_status = Arc::new(PolicyRuntimeStatus::new(git_config.sync_interval));
        let loader = PolicyLoader::new(git_config);
        let orchestrator = PolicyOrchestrator::new(OrchestrationConfig::default());

        Self {
            loader,
            engine,
            orchestrator,
            runtime_status,
        }
    }

    /// Loads policies from the configured directory
    ///
    /// # Errors
    ///
    /// Returns `PolicyError` if policies cannot be loaded or parsed
    pub fn load_policies(&mut self) -> PolicyResult<Vec<PolicyRule>> {
        let result = self.loader.load_policies()?;
        // Flatten all rules from all loaded files
        Ok(result
            .loaded
            .into_iter()
            .flat_map(|file| file.rules)
            .collect())
    }

    /// Loads policies from the configured directory (async version)
    ///
    /// # Errors
    ///
    /// Returns `PolicyError` if policies cannot be loaded or parsed
    pub async fn load_policies_async(&mut self) -> PolicyResult<Vec<PolicyRule>> {
        let result = self.loader.load_policies_async().await?;
        // Flatten all rules from all loaded files
        Ok(result
            .loaded
            .into_iter()
            .flat_map(|file| file.rules)
            .collect())
    }

    /// Evaluates policies against a single node
    ///
    /// # Errors
    ///
    /// Returns `PolicyError` if evaluation fails or policies cannot be loaded
    pub async fn evaluate_node(
        &mut self,
        datastore: &dyn DataStore,
        node: &Node,
    ) -> PolicyResult<Vec<PolicyExecutionResult>> {
        let policies = self.load_policies_async().await?;
        self.engine
            .evaluate_node_policies(datastore, node, &policies)
            .await
    }

    /// Evaluates policies against all nodes
    ///
    /// # Errors
    ///
    /// Returns `PolicyError` if evaluation fails or policies cannot be loaded
    pub async fn evaluate_all_nodes(
        &mut self,
        datastore: &dyn DataStore,
    ) -> PolicyResult<HashMap<Uuid, Vec<PolicyExecutionResult>>> {
        let policies = self.load_policies_async().await?;
        self.engine
            .evaluate_all_policies(datastore, &policies)
            .await
    }

    /// Evaluates policies with orchestration (priority, batching, etc.)
    ///
    /// # Errors
    ///
    /// Returns `PolicyError` if evaluation fails or policies cannot be loaded
    pub async fn evaluate_with_orchestration(
        &mut self,
        datastore: &dyn DataStore,
    ) -> PolicyResult<HashMap<Uuid, Vec<PolicyExecutionResult>>> {
        // For now, use simple evaluation without orchestration
        // TODO: Implement proper orchestration when the interface is clearer
        self.evaluate_all_nodes(datastore).await
    }

    /// Stores evaluation results for a node
    ///
    /// # Errors
    ///
    /// Returns `DataStoreError` if results cannot be stored
    pub async fn store_results(
        &self,
        datastore: &dyn DataStore,
        node_id: &Uuid,
        results: &[PolicyExecutionResult],
    ) -> DataStoreResult<()> {
        self.engine.store_results(datastore, node_id, results).await
    }

    /// Gets the policy loader (for accessing cached policies, etc.)
    #[must_use]
    pub const fn loader(&self) -> &PolicyLoader {
        &self.loader
    }

    /// Gets the policy orchestrator (for configuration, etc.)
    #[must_use]
    pub const fn orchestrator(&self) -> &PolicyOrchestrator {
        &self.orchestrator
    }

    /// Records the completion of a policy evaluation cycle.
    pub fn record_evaluation_run(&self) {
        let mut last_evaluation = self
            .runtime_status
            .last_evaluation
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *last_evaluation = Some(SystemTime::now());
    }

    /// Returns the configured background evaluation interval in seconds.
    #[must_use]
    pub fn evaluation_interval_seconds(&self) -> u64 {
        self.runtime_status.evaluation_interval_seconds
    }

    /// Returns the most recent policy evaluation timestamp, if one has run.
    #[must_use]
    pub fn last_evaluation(&self) -> Option<SystemTime> {
        *self
            .runtime_status
            .last_evaluation
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}
