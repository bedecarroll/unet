//! Tests for `PolicyExecutionResult` and execution flow

use super::super::context::*;
use crate::policy::ast::{Action, Condition, FieldRef, PolicyRule, Value};
use serde_json::json;

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
fn test_policy_execution_result_is_successful() {
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

    // Test successful case
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
    assert!(success_result.is_successful());

    // Test not satisfied case (should be considered successful - no error)
    let not_satisfied_result =
        PolicyExecutionResult::new(rule.clone(), EvaluationResult::NotSatisfied, None);
    assert!(not_satisfied_result.is_successful());

    // Test evaluation error case
    let eval_error_result = PolicyExecutionResult::new(
        rule.clone(),
        EvaluationResult::Error {
            message: "Eval error".to_string(),
        },
        None,
    );
    assert!(!eval_error_result.is_successful());

    // Test action error case
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
    assert!(!action_error_result.is_successful());

    // Test compliance failure case (should not be successful)
    let compliance_failure_result = PolicyExecutionResult::new(
        rule.clone(),
        EvaluationResult::Satisfied {
            action: rule.action,
        },
        Some(ActionExecutionResult {
            result: ActionResult::ComplianceFailure {
                field: "version".to_string(),
                expected: json!("15.1"),
                actual: json!("14.2"),
            },
            rollback_data: None,
        }),
    );
    assert!(!compliance_failure_result.is_successful());
}

#[test]
fn test_policy_execution_result_is_satisfied() {
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

    let satisfied_result = PolicyExecutionResult::new(
        rule.clone(),
        EvaluationResult::Satisfied {
            action: rule.action.clone(),
        },
        None,
    );
    assert!(satisfied_result.is_satisfied());

    let not_satisfied_result =
        PolicyExecutionResult::new(rule.clone(), EvaluationResult::NotSatisfied, None);
    assert!(!not_satisfied_result.is_satisfied());

    let error_result = PolicyExecutionResult::new(
        rule,
        EvaluationResult::Error {
            message: "Error".to_string(),
        },
        None,
    );
    assert!(!error_result.is_satisfied());
}

#[test]
fn test_policy_execution_result_is_error() {
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
            message: "Eval error".to_string(),
        },
        None,
    );
    assert!(eval_error_result.is_error());

    // Test action error
    let action_error_result = PolicyExecutionResult::new(
        rule.clone(),
        EvaluationResult::Satisfied {
            action: rule.action.clone(),
        },
        Some(ActionExecutionResult {
            result: ActionResult::Error {
                message: "Action error".to_string(),
            },
            rollback_data: None,
        }),
    );
    assert!(action_error_result.is_error());

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
    assert!(!success_result.is_error());

    // Test not satisfied case
    let not_satisfied_result =
        PolicyExecutionResult::new(rule, EvaluationResult::NotSatisfied, None);
    assert!(!not_satisfied_result.is_error());
}

#[test]
fn test_policy_execution_result_is_compliance_failure() {
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

    let compliance_failure_result = PolicyExecutionResult::new(
        rule.clone(),
        EvaluationResult::Satisfied {
            action: rule.action.clone(),
        },
        Some(ActionExecutionResult {
            result: ActionResult::ComplianceFailure {
                field: "version".to_string(),
                expected: json!("15.1"),
                actual: json!("14.2"),
            },
            rollback_data: None,
        }),
    );
    assert!(compliance_failure_result.is_compliance_failure());

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
    assert!(!success_result.is_compliance_failure());

    let no_action_result = PolicyExecutionResult::new(rule, EvaluationResult::NotSatisfied, None);
    assert!(!no_action_result.is_compliance_failure());
}

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
