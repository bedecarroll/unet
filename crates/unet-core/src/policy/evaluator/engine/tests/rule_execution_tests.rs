//! Rule execution and action tests

use super::test_utilities::*;
use crate::policy::evaluator::PolicyEvaluator;
use crate::policy::evaluator::context::{EvaluationResult, PolicyExecutionContext};

#[cfg(test)]
mod tests {
    use super::*;

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
}
