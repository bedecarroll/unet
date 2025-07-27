//! Tests for `PolicyExecutionResult` utility methods

use crate::policy::ast::{Action, Condition, FieldRef, PolicyRule, Value};
use crate::policy::evaluator::context::{
    ActionExecutionResult, ActionResult, EvaluationResult, PolicyExecutionResult,
};

#[test]
fn test_policy_execution_result_get_error_message() {
    let rule = PolicyRule {
        id: Some("test_rule".to_string()),
        condition: Condition::True,
        action: Action::Assert {
            field: FieldRef {
                path: vec!["status".to_string()],
            },
            expected: Value::String("active".to_string()),
        },
    };

    // Test evaluation error
    let eval_error_result = PolicyExecutionResult::new(
        rule.clone(),
        EvaluationResult::Error {
            message: "Evaluation failed".to_string(),
        },
        None,
    );
    assert_eq!(
        eval_error_result.get_error_message(),
        Some("Evaluation failed")
    );

    // Test action error
    let action_error_result = PolicyExecutionResult::new(
        rule.clone(),
        EvaluationResult::Satisfied {
            action: rule.action.clone(),
        },
        Some(ActionExecutionResult {
            result: ActionResult::Error {
                message: "Action failed".to_string(),
            },
            rollback_data: None,
        }),
    );
    assert_eq!(
        action_error_result.get_error_message(),
        Some("Action failed")
    );

    // Test success case
    let success_result = PolicyExecutionResult::new(
        rule.clone(),
        EvaluationResult::Satisfied {
            action: rule.action.clone(),
        },
        Some(ActionExecutionResult {
            result: ActionResult::Success {
                message: "Success".to_string(),
            },
            rollback_data: None,
        }),
    );
    assert_eq!(success_result.get_error_message(), None);

    // Test not satisfied case
    let not_satisfied_result =
        PolicyExecutionResult::new(rule, EvaluationResult::NotSatisfied, None);
    assert_eq!(not_satisfied_result.get_error_message(), None);
}

#[test]
fn test_policy_execution_result_rule_id() {
    let rule_with_id = PolicyRule {
        id: Some("my_rule".to_string()),
        condition: Condition::True,
        action: Action::Assert {
            field: FieldRef {
                path: vec!["status".to_string()],
            },
            expected: Value::String("active".to_string()),
        },
    };

    let rule_without_id = PolicyRule {
        id: None,
        condition: Condition::True,
        action: Action::Assert {
            field: FieldRef {
                path: vec!["status".to_string()],
            },
            expected: Value::String("active".to_string()),
        },
    };

    let result_with_id =
        PolicyExecutionResult::new(rule_with_id, EvaluationResult::NotSatisfied, None);
    assert_eq!(result_with_id.rule_id(), Some("my_rule"));

    let result_without_id =
        PolicyExecutionResult::new(rule_without_id, EvaluationResult::NotSatisfied, None);
    assert_eq!(result_without_id.rule_id(), None);
}
