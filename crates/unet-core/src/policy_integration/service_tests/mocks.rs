//! Mock implementations for policy service testing

use std::collections::HashMap;
use uuid::Uuid;

use crate::config::GitConfig;
use crate::datastore::testing::SeededDataStore;
use crate::datastore::{DataStore, DataStoreResult};
use crate::models::{DeviceRole, Node, Vendor};
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

/// Mock datastore for testing.
pub type MockDataStore = SeededDataStore;

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
