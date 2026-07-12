//! Tests for evaluating multiple policy rules

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

    // First rule should be satisfied
    match &results[0] {
        EvaluationResult::Satisfied { action } => {
            assert_eq!(*action, rules[0].action);
        }
        _ => panic!("Expected Satisfied result for first rule"),
    }

    // Second rule should not be satisfied
    match &results[1] {
        EvaluationResult::NotSatisfied => {}
        _ => panic!("Expected NotSatisfied result for second rule"),
    }

    // Third rule should be satisfied
    match &results[2] {
        EvaluationResult::Satisfied { action } => {
            assert_eq!(*action, rules[2].action);
        }
        _ => panic!("Expected Satisfied result for third rule"),
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
