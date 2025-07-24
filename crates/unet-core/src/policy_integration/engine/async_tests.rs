//! Async tests for policy evaluation engine

#[cfg(test)]
mod tests {
    use super::super::default_engine::DefaultPolicyEvaluationEngine;
    use super::super::mock_datastores::mocks::{EmptyMockDataStore, FailingMockDataStore};
    use super::super::tests::{
        create_invalid_policy_rule, create_test_node, create_test_policy_rule,
    };
    use super::super::trait_definition::PolicyEvaluationEngine;
    use crate::datastore::DataStoreError;
    use uuid::Uuid;

    /// Test evaluate_node_policies handles PolicyEvaluator::execute_rule failures
    #[tokio::test]
    async fn test_evaluate_node_policies_handles_policy_execution_errors() {
        let engine = DefaultPolicyEvaluationEngine::new();
        let datastore = EmptyMockDataStore;
        let node = create_test_node();

        // Create a policy rule that will cause evaluation errors due to missing field
        let invalid_rule = create_invalid_policy_rule();
        let policies = vec![invalid_rule];

        // This should not fail - the engine should handle the error gracefully
        let result = engine
            .evaluate_node_policies(&datastore, &node, &policies)
            .await;

        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);

        // The result should be an error result, not a failure of the whole evaluation
        let execution_result = &results[0];
        assert!(execution_result.is_error());
        assert!(execution_result.get_error_message().is_some());
    }

    /// Test evaluate_all_policies handles DataStore get_nodes_for_policy_evaluation failures
    #[tokio::test]
    async fn test_evaluate_all_policies_handles_datastore_query_failures() {
        let engine = DefaultPolicyEvaluationEngine::new();
        let failing_datastore = FailingMockDataStore;
        let policies = vec![create_test_policy_rule()];

        let result = engine
            .evaluate_all_policies(&failing_datastore, &policies)
            .await;

        // Should return an error due to DataStore failure
        assert!(result.is_err());

        // Should be a PolicyError::DataStoreError
        match result.unwrap_err() {
            crate::policy::PolicyError::DataStoreError { message } => {
                assert!(message.contains("Mock policy evaluation error"));
            }
            other => panic!("Expected DataStoreError, got {other:?}"),
        }
    }

    /// Test store_results handles DataStore storage failures
    #[tokio::test]
    async fn test_store_results_handles_datastore_storage_failures() {
        let engine = DefaultPolicyEvaluationEngine::new();
        let failing_datastore = FailingMockDataStore;
        let node_id = Uuid::new_v4();

        // Create a test result to store
        let test_result = crate::policy::PolicyExecutionResult::new_error(
            create_test_policy_rule(),
            "Test error".to_string(),
        );
        let results = vec![test_result];

        let store_result = engine
            .store_results(&failing_datastore, &node_id, &results)
            .await;

        // Should return an error due to DataStore failure
        assert!(store_result.is_err());

        // Should be a DataStoreError
        match store_result.unwrap_err() {
            DataStoreError::InternalError { message } => {
                assert!(message.contains("Mock policy result storage error"));
            }
            other => panic!("Expected InternalError error, got {other:?}"),
        }
    }

    /// Test store_results with successful storage
    #[tokio::test]
    async fn test_store_results_successful_storage() {
        let engine = DefaultPolicyEvaluationEngine::new();
        let empty_datastore = EmptyMockDataStore;
        let node_id = Uuid::new_v4();

        // Create test results to store
        let test_result = crate::policy::PolicyExecutionResult::new_error(
            create_test_policy_rule(),
            "Test error".to_string(),
        );
        let results = vec![test_result];

        let store_result = engine
            .store_results(&empty_datastore, &node_id, &results)
            .await;

        // Should succeed with empty mock datastore
        assert!(store_result.is_ok());
    }
}
