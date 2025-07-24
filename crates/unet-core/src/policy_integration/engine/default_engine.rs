//! Default implementation of `PolicyEvaluationEngine`

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

use super::trait_definition::PolicyEvaluationEngine;

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
