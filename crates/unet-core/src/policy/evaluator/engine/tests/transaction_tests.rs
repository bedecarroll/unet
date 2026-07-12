//! Transaction management tests for policy execution

use super::test_utilities::*;
use crate::policy::evaluator::PolicyEvaluator;
use crate::policy::evaluator::context::PolicyExecutionContext;
use uuid::Uuid;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_rules_with_transaction_creates_transaction() {
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;
        let rules = vec![create_always_true_rule(), create_always_false_rule()];
        let context = create_test_context();
        let exec_ctx = PolicyExecutionContext::new(&context, &datastore, &node.id);

        let result = PolicyEvaluator::execute_rules_with_transaction(&rules, &exec_ctx).await;

        assert!(result.is_ok());
        let (results, transaction) = result.unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(transaction.node_id, node.id);
        assert!(transaction.transaction_id.starts_with("tx_"));
        assert!(transaction.original_node_state.is_some());
    }

    #[tokio::test]
    async fn test_execute_rules_with_transaction_processes_all_rules() {
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;
        let rules = vec![
            create_always_true_rule(),
            create_always_false_rule(),
            create_comparison_rule(),
        ];
        let context = create_test_context();
        let exec_ctx = PolicyExecutionContext::new(&context, &datastore, &node.id);

        let result = PolicyEvaluator::execute_rules_with_transaction(&rules, &exec_ctx).await;

        assert!(result.is_ok());
        let (results, _transaction) = result.unwrap();

        assert_eq!(results.len(), 3);

        // Check that each result corresponds to the correct rule
        for (i, result) in results.iter().enumerate() {
            assert_eq!(result.rule, rules[i]);
        }
    }

    #[tokio::test]
    async fn test_execute_rules_with_transaction_captures_original_node_state() {
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;
        let rules = vec![create_always_true_rule()];
        let context = create_test_context();
        let exec_ctx = PolicyExecutionContext::new(&context, &datastore, &node.id);

        let result = PolicyEvaluator::execute_rules_with_transaction(&rules, &exec_ctx).await;

        assert!(result.is_ok());
        let (_results, transaction) = result.unwrap();

        assert!(transaction.original_node_state.is_some());
        let original_state = transaction.original_node_state.unwrap();

        // Verify that the captured state contains node information
        assert!(original_state.get("name").is_some());
        assert!(original_state.get("vendor").is_some());
    }

    #[tokio::test]
    async fn test_execute_rules_with_transaction_empty_rules_succeeds() {
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;
        let rules = vec![];
        let context = create_test_context();
        let exec_ctx = PolicyExecutionContext::new(&context, &datastore, &node.id);

        let result = PolicyEvaluator::execute_rules_with_transaction(&rules, &exec_ctx).await;

        assert!(result.is_ok());
        let (results, transaction) = result.unwrap();

        assert_eq!(results.len(), 0);
        assert_eq!(transaction.node_id, node.id);
        assert!(transaction.original_node_state.is_some());
    }

    #[tokio::test]
    async fn test_execute_rules_with_transaction_node_not_found_fails() {
        let datastore = setup_test_datastore().await;
        let non_existent_node_id = Uuid::new_v4();
        let rules = vec![create_always_true_rule()];
        let context = create_test_context();
        let exec_ctx = PolicyExecutionContext::new(&context, &datastore, &non_existent_node_id);

        let result = PolicyEvaluator::execute_rules_with_transaction(&rules, &exec_ctx).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            crate::policy::PolicyError::NodeNotFound { node_id } => {
                assert_eq!(node_id, non_existent_node_id.to_string());
            }
            _ => panic!("Expected NodeNotFound error"),
        }
    }
}
