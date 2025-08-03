//! Policy evaluation engine trait definition

use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;

use crate::datastore::{DataStore, DataStoreResult};
use crate::models::Node;
use crate::policy::{EvaluationContext, PolicyExecutionResult, PolicyResult, PolicyRule};

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
