//! Complex condition tests (AND, OR, NOT) for policy evaluation

use super::test_utilities::*;
use crate::policy::ast::{Action, ComparisonOperator, Condition, FieldRef, PolicyRule, Value};
use crate::policy::evaluator::PolicyEvaluator;
use crate::policy::evaluator::context::EvaluationResult;

#[cfg(test)]
mod tests {
    use super::*;

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
