//! Tests for `PolicyExecutionResult` status checking methods

use crate::policy::ast::{Action, Condition, FieldRef, PolicyRule, Value};
use crate::policy::evaluator::context::{
    ActionExecutionResult, ActionResult, EvaluationResult, PolicyExecutionResult,
};
use serde_json::json;

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
