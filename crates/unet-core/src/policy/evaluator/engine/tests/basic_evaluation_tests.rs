//! Basic policy rule evaluation tests

use super::test_utilities::*;
use crate::policy::ast::{Action, ComparisonOperator, Condition, FieldRef, PolicyRule, Value};
use crate::policy::evaluator::PolicyEvaluator;
use crate::policy::evaluator::context::EvaluationResult;

#[cfg(test)]
mod tests {
    use super::*;

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
}
