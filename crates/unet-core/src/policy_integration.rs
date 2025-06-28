//! Policy Integration for μNet Core
//!
//! This module provides integration between the policy engine and the data layer,
//! enabling policy evaluation against live network data and storage of policy results.

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::config::GitConfig;
use crate::datastore::{DataStore, DataStoreResult};
use crate::models::Node;
use crate::policy::{
    EvaluationContext, PolicyEvaluator, PolicyExecutionResult, PolicyLoader, PolicyOrchestrator,
    PolicyResult, PolicyRule,
};

/// Policy evaluation trait for integrating with DataStore
#[async_trait]
pub trait PolicyEvaluationEngine: Send + Sync {
    /// Evaluates policies against a single node
    async fn evaluate_node_policies(
        &self,
        datastore: &dyn DataStore,
        node: &Node,
        policies: &[PolicyRule],
    ) -> PolicyResult<Vec<PolicyExecutionResult>>;

    /// Evaluates policies against all nodes
    async fn evaluate_all_policies(
        &self,
        datastore: &dyn DataStore,
        policies: &[PolicyRule],
    ) -> PolicyResult<HashMap<Uuid, Vec<PolicyExecutionResult>>>;

    /// Creates evaluation context from node data
    fn create_evaluation_context(&self, node: &Node) -> PolicyResult<EvaluationContext>;

    /// Stores policy execution results
    async fn store_results(
        &self,
        datastore: &dyn DataStore,
        node_id: &Uuid,
        results: &[PolicyExecutionResult],
    ) -> DataStoreResult<()>;
}

/// Default implementation of PolicyEvaluationEngine
pub struct DefaultPolicyEvaluationEngine;

impl DefaultPolicyEvaluationEngine {
    /// Creates a new policy evaluation engine
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultPolicyEvaluationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PolicyEvaluationEngine for DefaultPolicyEvaluationEngine {
    async fn evaluate_node_policies(
        &self,
        datastore: &dyn DataStore,
        node: &Node,
        policies: &[PolicyRule],
    ) -> PolicyResult<Vec<PolicyExecutionResult>> {
        let context = self.create_evaluation_context(node)?;
        let mut results = Vec::new();

        for policy in policies {
            match PolicyEvaluator::execute_rule(policy, &context, datastore, &node.id).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    // Log error but continue with other policies
                    tracing::warn!(
                        policy_id = policy.id.as_deref().unwrap_or("unknown"),
                        node_id = %node.id,
                        error = %e,
                        "Failed to evaluate policy for node"
                    );
                    // Create a failed result
                    results.push(PolicyExecutionResult::new_error(
                        policy.id.as_deref().unwrap_or("unknown"),
                        e.to_string(),
                    ));
                }
            }
        }

        Ok(results)
    }

    async fn evaluate_all_policies(
        &self,
        datastore: &dyn DataStore,
        policies: &[PolicyRule],
    ) -> PolicyResult<HashMap<Uuid, Vec<PolicyExecutionResult>>> {
        let nodes = datastore
            .get_nodes_for_policy_evaluation()
            .await
            .map_err(|e| crate::policy::PolicyError::DataStoreError {
                message: e.to_string(),
            })?;

        let mut all_results = HashMap::new();

        for node in nodes {
            match self
                .evaluate_node_policies(datastore, &node, policies)
                .await
            {
                Ok(results) => {
                    all_results.insert(node.id, results);
                }
                Err(e) => {
                    tracing::error!(
                        node_id = %node.id,
                        error = %e,
                        "Failed to evaluate policies for node"
                    );
                    // Store error result
                    all_results.insert(
                        node.id,
                        vec![PolicyExecutionResult::new_error(
                            "evaluation",
                            e.to_string(),
                        )],
                    );
                }
            }
        }

        Ok(all_results)
    }

    fn create_evaluation_context(&self, node: &Node) -> PolicyResult<EvaluationContext> {
        // Convert node to JSON for policy evaluation
        let mut node_data =
            serde_json::to_value(node).map_err(|e| crate::policy::PolicyError::Evaluation {
                message: format!("Failed to serialize node data: {}", e),
            })?;

        // Ensure node data is an object
        if let Value::Object(ref mut obj) = node_data {
            // Add computed fields that might be useful for policies
            obj.insert("fqdn".to_string(), Value::String(node.fqdn.clone()));
            obj.insert(
                "has_management_ip".to_string(),
                Value::Bool(node.management_ip.is_some()),
            );
            obj.insert(
                "has_location".to_string(),
                Value::Bool(node.location_id.is_some()),
            );
        }

        // Create context with node data
        let context_data = serde_json::json!({
            "node": node_data
        });

        Ok(EvaluationContext::new(context_data))
    }

    async fn store_results(
        &self,
        datastore: &dyn DataStore,
        node_id: &Uuid,
        results: &[PolicyExecutionResult],
    ) -> DataStoreResult<()> {
        for result in results {
            datastore
                .store_policy_result(
                    node_id,
                    result.rule_id().as_deref().map_or("unknown", |v| v),
                    result,
                )
                .await?;
        }
        Ok(())
    }
}

/// Policy service that orchestrates policy loading, evaluation, and result storage
#[derive(Clone)]
pub struct PolicyService {
    loader: PolicyLoader,
    engine: Arc<dyn PolicyEvaluationEngine>,
    orchestrator: PolicyOrchestrator,
}

impl PolicyService {
    /// Creates a new policy service with Git configuration
    pub fn new(git_config: GitConfig) -> Self {
        let loader = PolicyLoader::new(git_config);
        let engine = Arc::new(DefaultPolicyEvaluationEngine::new());
        let orchestrator = PolicyOrchestrator::new(Default::default());

        Self {
            loader,
            engine,
            orchestrator,
        }
    }

    /// Creates a new policy service with local directory
    pub fn with_local_dir(policies_directory: &str) -> Self {
        let git_config = GitConfig {
            policies_repo: None,
            templates_repo: None,
            branch: "main".to_string(),
            sync_interval: 300,
        };
        let loader = PolicyLoader::new(git_config).with_local_dir(policies_directory);
        let engine = Arc::new(DefaultPolicyEvaluationEngine::new());
        let orchestrator = PolicyOrchestrator::new(Default::default());

        Self {
            loader,
            engine,
            orchestrator,
        }
    }

    /// Creates a new policy service with custom evaluation engine
    pub fn with_engine(git_config: GitConfig, engine: Arc<dyn PolicyEvaluationEngine>) -> Self {
        let loader = PolicyLoader::new(git_config);
        let orchestrator = PolicyOrchestrator::new(Default::default());

        Self {
            loader,
            engine,
            orchestrator,
        }
    }

    /// Loads policies from the configured directory
    pub async fn load_policies(&mut self) -> PolicyResult<Vec<PolicyRule>> {
        let result = self.loader.load_policies().await?;
        // Flatten all rules from all loaded files
        Ok(result
            .loaded
            .into_iter()
            .flat_map(|file| file.rules)
            .collect())
    }

    /// Evaluates policies against a single node
    pub async fn evaluate_node(
        &mut self,
        datastore: &dyn DataStore,
        node: &Node,
    ) -> PolicyResult<Vec<PolicyExecutionResult>> {
        let policies = self.load_policies().await?;
        self.engine
            .evaluate_node_policies(datastore, node, &policies)
            .await
    }

    /// Evaluates policies against all nodes
    pub async fn evaluate_all_nodes(
        &mut self,
        datastore: &dyn DataStore,
    ) -> PolicyResult<HashMap<Uuid, Vec<PolicyExecutionResult>>> {
        let policies = self.load_policies().await?;
        self.engine
            .evaluate_all_policies(datastore, &policies)
            .await
    }

    /// Evaluates policies with orchestration (priority, batching, etc.)
    pub async fn evaluate_with_orchestration(
        &mut self,
        datastore: &dyn DataStore,
    ) -> PolicyResult<HashMap<Uuid, Vec<PolicyExecutionResult>>> {
        // For now, use simple evaluation without orchestration
        // TODO: Implement proper orchestration when the interface is clearer
        self.evaluate_all_nodes(datastore).await
    }

    /// Stores evaluation results for a node
    pub async fn store_results(
        &self,
        datastore: &dyn DataStore,
        node_id: &Uuid,
        results: &[PolicyExecutionResult],
    ) -> DataStoreResult<()> {
        self.engine.store_results(datastore, node_id, results).await
    }

    /// Gets the policy loader (for accessing cached policies, etc.)
    pub fn loader(&self) -> &PolicyLoader {
        &self.loader
    }

    /// Gets the policy orchestrator (for configuration, etc.)
    pub fn orchestrator(&self) -> &PolicyOrchestrator {
        &self.orchestrator
    }

    /// Sync policies from Git and reload them with validation
    pub async fn sync_and_reload_policies(&mut self) -> PolicyResult<Vec<PolicyRule>> {
        let result = self.loader.sync_and_reload().await?;

        // Flatten all rules from all loaded files
        let policies: Vec<PolicyRule> = result
            .loaded
            .into_iter()
            .flat_map(|file| file.rules)
            .collect();

        tracing::info!("Synced and loaded {} policies from Git", policies.len());

        // Log any errors from the sync process
        if !result.errors.is_empty() {
            tracing::warn!("Policy sync completed with {} errors:", result.errors.len());
            for (path, error) in &result.errors {
                tracing::warn!("Policy error in {}: {}", path.display(), error);
            }
        }

        Ok(policies)
    }

    /// Force reload policies from source (clears cache)
    pub async fn reload_policies(&mut self) -> PolicyResult<Vec<PolicyRule>> {
        let result = self.loader.reload_policies().await?;

        // Flatten all rules from all loaded files
        let policies: Vec<PolicyRule> = result
            .loaded
            .into_iter()
            .flat_map(|file| file.rules)
            .collect();

        tracing::info!("Force reloaded {} policies", policies.len());

        // Log any errors from the reload process
        if !result.errors.is_empty() {
            tracing::warn!(
                "Policy reload completed with {} errors:",
                result.errors.len()
            );
            for (path, error) in &result.errors {
                tracing::warn!("Policy error in {}: {}", path.display(), error);
            }
        }

        Ok(policies)
    }

    /// Get policy loader mutable reference for advanced operations
    pub fn loader_mut(&mut self) -> &mut PolicyLoader {
        &mut self.loader
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DeviceRole, Lifecycle, Vendor};
    use uuid::Uuid;

    fn create_test_node() -> Node {
        Node {
            id: Uuid::new_v4(),
            name: "test-node".to_string(),
            domain: "example.com".to_string(),
            fqdn: "test-node.example.com".to_string(),
            vendor: Vendor::Cisco,
            model: "ISR4321".to_string(),
            role: DeviceRole::Router,
            lifecycle: Lifecycle::Live,
            management_ip: Some("192.168.1.1".parse().unwrap()),
            location_id: None,
            platform: None,
            version: Some("15.1".to_string()),
            serial_number: Some("ABC123".to_string()),
            asset_tag: None,
            purchase_date: None,
            warranty_expires: None,
            custom_data: serde_json::json!({"compliance": "pending"}),
        }
    }

    #[test]
    fn test_create_evaluation_context() {
        let engine = DefaultPolicyEvaluationEngine::new();
        let node = create_test_node();

        let context = engine.create_evaluation_context(&node).unwrap();

        // Verify the context contains node data
        let context_value = &context.node_data;
        assert!(context_value.get("node").is_some());

        if let Some(node_data) = context_value.get("node") {
            assert_eq!(node_data.get("name").unwrap(), "test-node");
            assert_eq!(node_data.get("vendor").unwrap(), "cisco");
            assert_eq!(node_data.get("has_management_ip").unwrap(), true);
            assert_eq!(node_data.get("has_location").unwrap(), false);
        }
    }

    #[test]
    fn test_policy_service_creation() {
        // This test creates a policy service with local directory
        let _service = PolicyService::with_local_dir("test_policies");

        // Should succeed - PolicyService is created successfully
        // (We can't test much without loading actual policies)
    }
}
