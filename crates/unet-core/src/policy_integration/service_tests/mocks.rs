//! Mock implementations for policy service testing

use std::collections::HashMap;
use uuid::Uuid;

use crate::config::GitConfig;
use crate::datastore::{DataStore, DataStoreResult};
use crate::models::{DeviceRole, Link, Location, Node, Vendor};
use crate::policy::{
    Action, Condition, EvaluationContext, EvaluationResult, FieldRef, PolicyExecutionResult,
    PolicyResult, PolicyRule, Value,
};
use async_trait::async_trait;

use super::super::engine::PolicyEvaluationEngine;

/// Mock policy evaluation engine for testing
pub struct MockPolicyEvaluationEngine {
    should_fail: bool,
    mock_results: Vec<PolicyExecutionResult>,
}

impl MockPolicyEvaluationEngine {
    pub fn new() -> Self {
        Self {
            should_fail: false,
            mock_results: Vec::new(),
        }
    }

    pub fn with_failure() -> Self {
        Self {
            should_fail: true,
            mock_results: Vec::new(),
        }
    }

    pub fn with_results(results: Vec<PolicyExecutionResult>) -> Self {
        Self {
            should_fail: false,
            mock_results: results,
        }
    }
}

#[async_trait]
impl PolicyEvaluationEngine for MockPolicyEvaluationEngine {
    async fn evaluate_node_policies(
        &self,
        _datastore: &dyn DataStore,
        _node: &Node,
        _policies: &[PolicyRule],
    ) -> PolicyResult<Vec<PolicyExecutionResult>> {
        if self.should_fail {
            return Err(crate::policy::PolicyError::EvaluationError(
                "Mock evaluation failed".to_string(),
            ));
        }
        Ok(self.mock_results.clone())
    }

    async fn evaluate_all_policies(
        &self,
        _datastore: &dyn DataStore,
        _policies: &[PolicyRule],
    ) -> PolicyResult<HashMap<Uuid, Vec<PolicyExecutionResult>>> {
        if self.should_fail {
            return Err(crate::policy::PolicyError::EvaluationError(
                "Mock evaluation failed".to_string(),
            ));
        }
        let mut results = HashMap::new();
        let node_id = Uuid::new_v4();
        results.insert(node_id, self.mock_results.clone());
        Ok(results)
    }

    fn create_evaluation_context(&self, _node: &Node) -> PolicyResult<EvaluationContext> {
        if self.should_fail {
            return Err(crate::policy::PolicyError::EvaluationError(
                "Mock context creation failed".to_string(),
            ));
        }
        Ok(EvaluationContext::new(serde_json::json!({})))
    }

    async fn store_results(
        &self,
        _datastore: &dyn DataStore,
        _node_id: &Uuid,
        _results: &[PolicyExecutionResult],
    ) -> DataStoreResult<()> {
        if self.should_fail {
            return Err(crate::datastore::types::DataStoreError::InternalError {
                message: "Mock storage failed".to_string(),
            });
        }
        Ok(())
    }
}

/// Mock datastore for testing
pub struct MockDataStore;

#[async_trait]
impl DataStore for MockDataStore {
    fn name(&self) -> &'static str {
        "MockDataStore"
    }

    async fn health_check(&self) -> DataStoreResult<()> {
        Ok(())
    }

    async fn begin_transaction(
        &self,
    ) -> DataStoreResult<Box<dyn crate::datastore::types::Transaction>> {
        unimplemented!("Not needed for policy service tests")
    }

    async fn create_node(&self, _node: &Node) -> DataStoreResult<Node> {
        unimplemented!("Not needed for policy service tests")
    }

    async fn get_node(&self, _id: &Uuid) -> DataStoreResult<Option<Node>> {
        Ok(None)
    }

    async fn list_nodes(
        &self,
        _options: &crate::datastore::types::QueryOptions,
    ) -> DataStoreResult<crate::datastore::types::PagedResult<Node>> {
        Ok(crate::datastore::types::PagedResult {
            items: vec![],
            total_count: 0,
            page: 1,
            page_size: 10,
            total_pages: 0,
            has_next: false,
            has_previous: false,
        })
    }

    async fn update_node(&self, node: &Node) -> DataStoreResult<Node> {
        Ok(node.clone())
    }

    async fn delete_node(&self, _id: &Uuid) -> DataStoreResult<()> {
        Ok(())
    }

    async fn get_nodes_by_location(&self, _location_id: &Uuid) -> DataStoreResult<Vec<Node>> {
        Ok(vec![])
    }

    async fn search_nodes_by_name(&self, _name: &str) -> DataStoreResult<Vec<Node>> {
        Ok(vec![])
    }

    async fn create_link(&self, _link: &Link) -> DataStoreResult<Link> {
        unimplemented!("Not needed for policy service tests")
    }

    async fn get_link(&self, _id: &Uuid) -> DataStoreResult<Option<Link>> {
        Ok(None)
    }

    async fn list_links(
        &self,
        _options: &crate::datastore::types::QueryOptions,
    ) -> DataStoreResult<crate::datastore::types::PagedResult<Link>> {
        unimplemented!("Not needed for policy service tests")
    }

    async fn update_link(&self, _link: &Link) -> DataStoreResult<Link> {
        unimplemented!("Not needed for policy service tests")
    }

    async fn delete_link(&self, _id: &Uuid) -> DataStoreResult<()> {
        unimplemented!("Not needed for policy service tests")
    }

    async fn get_links_for_node(&self, _node_id: &Uuid) -> DataStoreResult<Vec<Link>> {
        Ok(vec![])
    }

    async fn get_links_between_nodes(
        &self,
        _first_node_id: &Uuid,
        _second_node_id: &Uuid,
    ) -> DataStoreResult<Vec<Link>> {
        Ok(vec![])
    }

    async fn create_location(&self, _location: &Location) -> DataStoreResult<Location> {
        unimplemented!("Not needed for policy service tests")
    }

    async fn get_location(&self, _id: &Uuid) -> DataStoreResult<Option<Location>> {
        Ok(None)
    }

    async fn list_locations(
        &self,
        _options: &crate::datastore::types::QueryOptions,
    ) -> DataStoreResult<crate::datastore::types::PagedResult<Location>> {
        unimplemented!("Not needed for policy service tests")
    }

    async fn update_location(&self, _location: &Location) -> DataStoreResult<Location> {
        unimplemented!("Not needed for policy service tests")
    }

    async fn delete_location(&self, _id: &Uuid) -> DataStoreResult<()> {
        unimplemented!("Not needed for policy service tests")
    }

    async fn create_vendor(&self, _name: &str) -> DataStoreResult<()> {
        unimplemented!("Not needed for policy service tests")
    }

    async fn list_vendors(&self) -> DataStoreResult<Vec<String>> {
        Ok(vec![])
    }

    async fn delete_vendor(&self, _name: &str) -> DataStoreResult<()> {
        unimplemented!("Not needed for policy service tests")
    }

    async fn batch_nodes(
        &self,
        _operations: &[crate::datastore::types::BatchOperation<Node>],
    ) -> DataStoreResult<crate::datastore::types::BatchResult> {
        unimplemented!("Not needed for policy service tests")
    }

    async fn batch_links(
        &self,
        _operations: &[crate::datastore::types::BatchOperation<Link>],
    ) -> DataStoreResult<crate::datastore::types::BatchResult> {
        unimplemented!("Not needed for policy service tests")
    }

    async fn batch_locations(
        &self,
        _operations: &[crate::datastore::types::BatchOperation<Location>],
    ) -> DataStoreResult<crate::datastore::types::BatchResult> {
        unimplemented!("Not needed for policy service tests")
    }

    async fn get_entity_counts(&self) -> DataStoreResult<HashMap<String, usize>> {
        Ok(HashMap::new())
    }

    async fn get_statistics(&self) -> DataStoreResult<HashMap<String, serde_json::Value>> {
        Ok(HashMap::new())
    }
}

/// Creates a test `GitConfig` for testing
pub fn create_test_git_config() -> GitConfig {
    GitConfig {
        repository_url: Some("https://github.com/test/policies.git".to_string()),
        local_directory: Some("/tmp/policies".to_string()),
        branch: "main".to_string(),
        auth_token: None,
        sync_interval: 300,
        policies_repo: None,
        templates_repo: None,
    }
}

/// Creates a test node for testing
pub fn create_test_node() -> Node {
    Node::new(
        "test-node".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    )
}

/// Creates a test policy execution result
pub fn create_test_policy_result() -> PolicyExecutionResult {
    let rule = PolicyRule {
        id: Some("test-rule".to_string()),
        condition: Condition::True,
        action: Action::Assert {
            field: FieldRef {
                path: vec!["status".to_string()],
            },
            expected: Value::String("active".to_string()),
        },
    };

    PolicyExecutionResult::new(
        rule,
        EvaluationResult::Satisfied {
            action: Action::Assert {
                field: FieldRef {
                    path: vec!["status".to_string()],
                },
                expected: Value::String("active".to_string()),
            },
        },
        None,
    )
}
