//! Policy evaluation engine implementations

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

use crate::datastore::{DataStore, DataStoreResult};
use crate::models::Node;
use crate::policy::{
    EvaluationContext, PolicyEvaluator, PolicyExecutionContext, PolicyExecutionResult,
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
    ///
    /// # Errors
    ///
    /// Returns `PolicyError` if context creation fails due to invalid node data
    fn create_evaluation_context(&self, node: &Node) -> PolicyResult<EvaluationContext>;

    /// Stores policy execution results
    async fn store_results(
        &self,
        datastore: &dyn DataStore,
        node_id: &Uuid,
        results: &[PolicyExecutionResult],
    ) -> DataStoreResult<()>;
}

/// Default implementation of `PolicyEvaluationEngine`
pub struct DefaultPolicyEvaluationEngine;

impl DefaultPolicyEvaluationEngine {
    /// Creates a new policy evaluation engine
    #[must_use]
    pub const fn new() -> Self {
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
            let exec_ctx = PolicyExecutionContext::new(&context, datastore, &node.id);
            match PolicyEvaluator::execute_rule(policy, &exec_ctx).await {
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
                        policy.clone(),
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
                        vec![PolicyExecutionResult::new_error_with_id(
                            Some("evaluation".to_string()),
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
                message: format!("Failed to serialize node data: {e}"),
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
                .store_policy_result(node_id, result.rule_id().map_or("unknown", |v| v), result)
                .await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::network;
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
            management_ip: Some(
                network::parse_ip_addr("192.168.1.1").expect("Test IP address should be valid"),
            ),
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
    fn test_create_evaluation_context_no_management_ip() {
        let engine = DefaultPolicyEvaluationEngine::new();
        let mut node = create_test_node();
        node.management_ip = None;

        let context = engine.create_evaluation_context(&node).unwrap();
        let context_value = &context.node_data;

        if let Some(node_data) = context_value.get("node") {
            assert_eq!(node_data.get("has_management_ip").unwrap(), false);
        }
    }

    #[test]
    fn test_create_evaluation_context_with_location() {
        let engine = DefaultPolicyEvaluationEngine::new();
        let mut node = create_test_node();
        node.location_id = Some(Uuid::new_v4());

        let context = engine.create_evaluation_context(&node).unwrap();
        let context_value = &context.node_data;

        if let Some(node_data) = context_value.get("node") {
            assert_eq!(node_data.get("has_location").unwrap(), true);
        }
    }

    #[test]
    fn test_default_policy_evaluation_engine_creation() {
        let engine1 = DefaultPolicyEvaluationEngine::new();
        let engine2 = DefaultPolicyEvaluationEngine;

        // Both creation methods should work (they create equivalent instances)
        let node = create_test_node();
        let context1 = engine1.create_evaluation_context(&node).unwrap();
        let context2 = engine2.create_evaluation_context(&node).unwrap();

        // Both should create valid contexts
        assert!(context1.node_data.get("node").is_some());
        assert!(context2.node_data.get("node").is_some());
    }

    #[test]
    fn test_create_evaluation_context_fqdn_field() {
        let engine = DefaultPolicyEvaluationEngine::new();
        let node = create_test_node();

        let context = engine.create_evaluation_context(&node).unwrap();
        let context_value = &context.node_data;

        if let Some(node_data) = context_value.get("node") {
            // Should have fqdn field added explicitly
            assert_eq!(node_data.get("fqdn").unwrap(), "test-node.example.com");
        }
    }

    #[test]
    fn test_create_evaluation_context_computed_fields() {
        let engine = DefaultPolicyEvaluationEngine::new();
        let node = create_test_node();

        let context = engine.create_evaluation_context(&node).unwrap();
        let context_value = &context.node_data;

        if let Some(node_data) = context_value.get("node") {
            // Test all computed fields are present
            assert!(node_data.get("fqdn").is_some());
            assert!(node_data.get("has_management_ip").is_some());
            assert!(node_data.get("has_location").is_some());

            // Test their values
            assert_eq!(
                node_data.get("fqdn").unwrap().as_str().unwrap(),
                "test-node.example.com"
            );
            assert!(
                node_data
                    .get("has_management_ip")
                    .unwrap()
                    .as_bool()
                    .unwrap()
            );
            assert!(!node_data.get("has_location").unwrap().as_bool().unwrap());
        }
    }

    // The async methods are difficult to test without a full datastore implementation,
    // so we focus on the synchronous methods and leave integration tests to handle
    // the async functionality with real datastore instances.
}
