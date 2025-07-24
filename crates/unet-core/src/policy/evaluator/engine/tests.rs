//! Tests for the policy evaluation engine
//!
//! Contains comprehensive tests for the `PolicyEvaluator` including
//! rule evaluation, rule execution, and transaction management.

#[cfg(test)]
mod engine_tests {
    use crate::datastore::{DataStore, sqlite::SqliteStore};
    use crate::models::{DeviceRole, Node, Vendor};
    use crate::policy::ast::*;
    use crate::policy::evaluator::PolicyEvaluator;
    use crate::policy::evaluator::context::{
        EvaluationContext, EvaluationResult, PolicyExecutionContext,
    };
    use migration::Migrator;
    use sea_orm_migration::MigratorTrait;
    use serde_json::json;
    use uuid::Uuid;

    /// Set up a test datastore for testing
    async fn setup_test_datastore() -> SqliteStore {
        let store = SqliteStore::new("sqlite::memory:").await.unwrap();
        Migrator::up(store.connection(), None).await.unwrap();
        store
    }

    /// Create a test node in the datastore
    async fn create_test_node(datastore: &SqliteStore) -> Node {
        let node = Node::new(
            "test-node".to_string(),
            "example.com".to_string(),
            Vendor::Cisco,
            DeviceRole::Router,
        );

        datastore.create_node(&node).await.unwrap();
        node
    }

    /// Create a simple test policy rule that always evaluates to true
    fn create_always_true_rule() -> PolicyRule {
        PolicyRule {
            id: Some("always_true".to_string()),
            condition: Condition::True,
            action: Action::Assert {
                field: FieldRef {
                    path: vec!["vendor".to_string()],
                },
                expected: Value::String("cisco".to_string()),
            },
        }
    }

    /// Create a simple test policy rule that always evaluates to false
    fn create_always_false_rule() -> PolicyRule {
        PolicyRule {
            id: Some("always_false".to_string()),
            condition: Condition::False,
            action: Action::Assert {
                field: FieldRef {
                    path: vec!["vendor".to_string()],
                },
                expected: Value::String("cisco".to_string()),
            },
        }
    }

    /// Create a test policy rule with a comparison condition
    fn create_comparison_rule() -> PolicyRule {
        PolicyRule {
            id: Some("vendor_comparison".to_string()),
            condition: Condition::Comparison {
                field: FieldRef {
                    path: vec!["vendor".to_string()],
                },
                operator: ComparisonOperator::Equal,
                value: Value::String("cisco".to_string()),
            },
            action: Action::Set {
                field: FieldRef {
                    path: vec!["model".to_string()],
                },
                value: Value::String("ISR4431".to_string()),
            },
        }
    }

    /// Create test evaluation context
    fn create_test_context() -> EvaluationContext {
        EvaluationContext::new(json!({
            "vendor": "cisco",
            "model": "2960",
            "role": "switch"
        }))
    }

    #[test]
    fn test_evaluate_rule_with_true_condition_returns_satisfied() {
        let rule = create_always_true_rule();
        let context = create_test_context();

        let result = PolicyEvaluator::evaluate_rule(&rule, &context);

        assert!(result.is_ok());
        match result.unwrap() {
            EvaluationResult::Satisfied { action } => {
                assert_eq!(action, rule.action);
            }
            _ => panic!("Expected Satisfied result"),
        }
    }

    #[test]
    fn test_evaluate_rule_with_false_condition_returns_not_satisfied() {
        let rule = create_always_false_rule();
        let context = create_test_context();

        let result = PolicyEvaluator::evaluate_rule(&rule, &context);

        assert!(result.is_ok());
        match result.unwrap() {
            EvaluationResult::NotSatisfied => {}
            _ => panic!("Expected NotSatisfied result"),
        }
    }

    #[test]
    fn test_evaluate_rule_with_comparison_condition_matching_returns_satisfied() {
        let rule = create_comparison_rule();
        let context = create_test_context();

        let result = PolicyEvaluator::evaluate_rule(&rule, &context);

        assert!(result.is_ok());
        match result.unwrap() {
            EvaluationResult::Satisfied { action } => {
                assert_eq!(action, rule.action);
            }
            _ => panic!("Expected Satisfied result"),
        }
    }

    #[test]
    fn test_evaluate_rule_with_comparison_condition_not_matching_returns_not_satisfied() {
        let rule = PolicyRule {
            id: Some("vendor_mismatch".to_string()),
            condition: Condition::Comparison {
                field: FieldRef {
                    path: vec!["vendor".to_string()],
                },
                operator: ComparisonOperator::Equal,
                value: Value::String("juniper".to_string()),
            },
            action: Action::Set {
                field: FieldRef {
                    path: vec!["model".to_string()],
                },
                value: Value::String("EX4300".to_string()),
            },
        };
        let context = create_test_context();

        let result = PolicyEvaluator::evaluate_rule(&rule, &context);

        assert!(result.is_ok());
        match result.unwrap() {
            EvaluationResult::NotSatisfied => {}
            _ => panic!("Expected NotSatisfied result"),
        }
    }

    #[test]
    fn test_evaluate_rules_multiple_rules_returns_all_results() {
        let rules = vec![
            create_always_true_rule(),
            create_always_false_rule(),
            create_comparison_rule(),
        ];
        let context = create_test_context();

        let results = PolicyEvaluator::evaluate_rules(&rules, &context);

        assert!(results.is_ok());
        let results = results.unwrap();
        assert_eq!(results.len(), 3);

        // First rule (always true) should be satisfied
        match &results[0] {
            EvaluationResult::Satisfied { .. } => {}
            _ => panic!("Expected first result to be Satisfied"),
        }

        // Second rule (always false) should not be satisfied
        match &results[1] {
            EvaluationResult::NotSatisfied => {}
            _ => panic!("Expected second result to be NotSatisfied"),
        }

        // Third rule (comparison) should be satisfied
        match &results[2] {
            EvaluationResult::Satisfied { .. } => {}
            _ => panic!("Expected third result to be Satisfied"),
        }
    }

    #[test]
    fn test_evaluate_rules_empty_rules_returns_empty_results() {
        let rules = vec![];
        let context = create_test_context();

        let results = PolicyEvaluator::evaluate_rules(&rules, &context);

        assert!(results.is_ok());
        let results = results.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_execute_rule_with_satisfied_condition_executes_action() {
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;
        let rule = create_always_true_rule();
        let context = create_test_context();
        let exec_ctx = PolicyExecutionContext::new(&context, &datastore, &node.id);

        let result = PolicyEvaluator::execute_rule(&rule, &exec_ctx).await;

        assert!(result.is_ok());
        let result = result.unwrap();

        assert_eq!(result.rule, rule);
        match result.evaluation_result {
            EvaluationResult::Satisfied { .. } => {}
            _ => panic!("Expected Satisfied evaluation result"),
        }
        assert!(result.action_result.is_some());
    }

    #[tokio::test]
    async fn test_execute_rule_with_not_satisfied_condition_skips_action() {
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;
        let rule = create_always_false_rule();
        let context = create_test_context();
        let exec_ctx = PolicyExecutionContext::new(&context, &datastore, &node.id);

        let result = PolicyEvaluator::execute_rule(&rule, &exec_ctx).await;

        assert!(result.is_ok());
        let result = result.unwrap();

        assert_eq!(result.rule, rule);
        match result.evaluation_result {
            EvaluationResult::NotSatisfied => {}
            _ => panic!("Expected NotSatisfied evaluation result"),
        }
        assert!(result.action_result.is_none());
    }

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

    #[test]
    fn test_evaluate_rule_with_complex_and_condition() {
        let rule = PolicyRule {
            id: Some("complex_and".to_string()),
            condition: Condition::And(
                Box::new(Condition::Comparison {
                    field: FieldRef {
                        path: vec!["vendor".to_string()],
                    },
                    operator: ComparisonOperator::Equal,
                    value: Value::String("cisco".to_string()),
                }),
                Box::new(Condition::Comparison {
                    field: FieldRef {
                        path: vec!["role".to_string()],
                    },
                    operator: ComparisonOperator::Equal,
                    value: Value::String("switch".to_string()),
                }),
            ),
            action: Action::Assert {
                field: FieldRef {
                    path: vec!["model".to_string()],
                },
                expected: Value::String("2960".to_string()),
            },
        };
        let context = create_test_context();

        let result = PolicyEvaluator::evaluate_rule(&rule, &context);

        assert!(result.is_ok());
        match result.unwrap() {
            EvaluationResult::Satisfied { .. } => {}
            _ => panic!("Expected Satisfied result"),
        }
    }

    #[test]
    fn test_evaluate_rule_with_complex_or_condition() {
        let rule = PolicyRule {
            id: Some("complex_or".to_string()),
            condition: Condition::Or(
                Box::new(Condition::Comparison {
                    field: FieldRef {
                        path: vec!["vendor".to_string()],
                    },
                    operator: ComparisonOperator::Equal,
                    value: Value::String("juniper".to_string()),
                }),
                Box::new(Condition::Comparison {
                    field: FieldRef {
                        path: vec!["role".to_string()],
                    },
                    operator: ComparisonOperator::Equal,
                    value: Value::String("switch".to_string()),
                }),
            ),
            action: Action::Assert {
                field: FieldRef {
                    path: vec!["model".to_string()],
                },
                expected: Value::String("2960".to_string()),
            },
        };
        let context = create_test_context();

        let result = PolicyEvaluator::evaluate_rule(&rule, &context);

        assert!(result.is_ok());
        match result.unwrap() {
            EvaluationResult::Satisfied { .. } => {}
            _ => panic!("Expected Satisfied result"),
        }
    }

    #[test]
    fn test_evaluate_rule_with_not_condition() {
        let rule = PolicyRule {
            id: Some("not_condition".to_string()),
            condition: Condition::Not(Box::new(Condition::False)),
            action: Action::Assert {
                field: FieldRef {
                    path: vec!["vendor".to_string()],
                },
                expected: Value::String("cisco".to_string()),
            },
        };
        let context = create_test_context();

        let result = PolicyEvaluator::evaluate_rule(&rule, &context);

        assert!(result.is_ok());
        match result.unwrap() {
            EvaluationResult::Satisfied { .. } => {}
            _ => panic!("Expected Satisfied result"),
        }
    }
}
