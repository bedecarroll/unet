//! Tests for `PolicyExecutionResult` construction

use crate::policy::ast::{Action, Condition, FieldRef, PolicyRule, Value};
use crate::policy::evaluator::context::{
    ActionExecutionResult, ActionResult, EvaluationResult, PolicyExecutionResult,
};

#[test]
fn test_policy_execution_result_new() {
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

    let evaluation_result = EvaluationResult::Satisfied {
        action: rule.action.clone(),
    };

    let action_result = ActionExecutionResult {
        result: ActionResult::Success {
            message: "Check passed".to_string(),
        },
        rollback_data: None,
    };

    let result = PolicyExecutionResult::new(rule, evaluation_result, Some(action_result));

    assert_eq!(result.rule.id, Some("test_rule".to_string()));
    assert!(matches!(
        result.evaluation_result,
        EvaluationResult::Satisfied { .. }
    ));
    assert!(result.action_result.is_some());
}

#[test]
fn test_policy_execution_result_new_error() {
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

    let error_result = PolicyExecutionResult::new_error(rule.clone(), "Test error".to_string());

    assert_eq!(error_result.rule, rule);
    match &error_result.evaluation_result {
        EvaluationResult::Error { message } => {
            assert_eq!(message, "Test error");
        }
        _ => panic!("Expected Error evaluation result"),
    }
    assert!(error_result.action_result.is_none());
    assert!(error_result.is_error());
    assert!(!error_result.is_successful());
}

#[test]
fn test_policy_execution_result_new_error_with_id() {
    let error_result = PolicyExecutionResult::new_error_with_id(
        Some("test_rule_id".to_string()),
        "Test error message".to_string(),
    );

    assert_eq!(error_result.rule.id, Some("test_rule_id".to_string()));
    match &error_result.evaluation_result {
        EvaluationResult::Error { message } => {
            assert_eq!(message, "Test error message");
        }
        _ => panic!("Expected Error evaluation result"),
    }
    assert!(error_result.action_result.is_none());
    assert!(error_result.is_error());
    assert!(!error_result.is_successful());

    // Test with None rule ID
    let error_result_no_id =
        PolicyExecutionResult::new_error_with_id(None, "Error without ID".to_string());
    assert_eq!(error_result_no_id.rule.id, None);
}
