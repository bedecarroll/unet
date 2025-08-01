//! Tests for individual policy rule evaluation

use crate::policy::ast::*;
use crate::policy::evaluator::PolicyEvaluator;
use crate::policy::evaluator::context::{EvaluationContext, EvaluationResult};
use serde_json::json;

/// Create test evaluation context with various field types
fn create_test_context() -> EvaluationContext {
    EvaluationContext::new(json!({
        "vendor": "cisco",
        "model": "2960",
        "role": "switch",
        "custom_data": {
            "config": {
                "vlan": 100,
                "name": "test_vlan"
            }
        }
    }))
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
            expected: Value::String("juniper".to_string()),
        },
    }
}

/// Create a test policy rule with a comparison condition
fn create_comparison_rule() -> PolicyRule {
    PolicyRule {
        id: Some("vendor_cisco".to_string()),
        condition: Condition::Comparison {
            field: FieldRef {
                path: vec!["vendor".to_string()],
            },
            operator: ComparisonOperator::Equal,
            value: Value::String("cisco".to_string()),
        },
        action: Action::Assert {
            field: FieldRef {
                path: vec!["model".to_string()],
            },
            expected: Value::String("2960".to_string()),
        },
    }
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
        EvaluationResult::NotSatisfied => {}
        _ => panic!("Expected NotSatisfied result"),
    }
}

#[test]
fn test_evaluate_rule_with_complex_and_condition() {
    let rule = PolicyRule {
        id: Some("complex_and".to_string()),
        condition: Condition::And(
            Box::new(Condition::And(
                Box::new(Condition::Comparison {
                    field: FieldRef {
                        path: vec!["vendor".to_string()],
                    },
                    operator: ComparisonOperator::Equal,
                    value: Value::String("cisco".to_string()),
                }),
                Box::new(Condition::Comparison {
                    field: FieldRef {
                        path: vec!["model".to_string()],
                    },
                    operator: ComparisonOperator::Equal,
                    value: Value::String("2960".to_string()),
                }),
            )),
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
                path: vec![
                    "custom_data".to_string(),
                    "config".to_string(),
                    "vlan".to_string(),
                ],
            },
            expected: Value::Number(100.0),
        },
    };
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
fn test_evaluate_rule_with_complex_or_condition() {
    let rule = PolicyRule {
        id: Some("complex_or".to_string()),
        condition: Condition::Or(
            Box::new(Condition::Or(
                Box::new(Condition::Comparison {
                    field: FieldRef {
                        path: vec!["vendor".to_string()],
                    },
                    operator: ComparisonOperator::Equal,
                    value: Value::String("juniper".to_string()),
                }),
                Box::new(Condition::Comparison {
                    field: FieldRef {
                        path: vec!["model".to_string()],
                    },
                    operator: ComparisonOperator::Equal,
                    value: Value::String("3750".to_string()),
                }),
            )),
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
                path: vec![
                    "custom_data".to_string(),
                    "config".to_string(),
                    "name".to_string(),
                ],
            },
            expected: Value::String("test_vlan".to_string()),
        },
    };
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
fn test_evaluate_rule_with_not_condition() {
    let rule = PolicyRule {
        id: Some("not_juniper".to_string()),
        condition: Condition::Not(Box::new(Condition::Comparison {
            field: FieldRef {
                path: vec!["vendor".to_string()],
            },
            operator: ComparisonOperator::Equal,
            value: Value::String("juniper".to_string()),
        })),
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
        EvaluationResult::Satisfied { action } => {
            assert_eq!(action, rule.action);
        }
        _ => panic!("Expected Satisfied result"),
    }
}
