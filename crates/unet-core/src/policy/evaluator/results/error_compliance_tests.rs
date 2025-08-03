//! Tests for error handling and compliance failure detection in policy evaluation results

use super::test_helpers::*;
use crate::policy::ast::{Action, FieldRef, Value};
use crate::policy::evaluator::context::{
    ActionExecutionResult, ActionResult, EvaluationResult, RollbackData,
};
use crate::policy::evaluator::results::AggregatedResult;
use serde_json::json;
use std::time::Duration;
use uuid::Uuid;

#[test]
fn test_get_error_messages() {
    let node_id = Uuid::new_v4();
    let batch_id = "test".to_string();

    let results = vec![
        create_test_execution_result(
            create_test_rule(None),
            EvaluationResult::Error {
                message: "Eval error 1".to_string(),
            },
            None,
        ),
        create_test_execution_result(
            create_test_rule(None),
            EvaluationResult::Satisfied {
                action: Action::Set {
                    field: FieldRef {
                        path: vec!["test".to_string()],
                    },
                    value: Value::String("value".to_string()),
                },
            },
            Some(ActionExecutionResult {
                result: ActionResult::Error {
                    message: "Action error 1".to_string(),
                },
                rollback_data: Some(RollbackData::SetRollback {
                    field: FieldRef {
                        path: vec!["test".to_string()],
                    },
                    previous_value: Some(json!("old_value")),
                }),
            }),
        ),
        create_test_execution_result(
            create_test_rule(None),
            EvaluationResult::Satisfied {
                action: Action::Assert {
                    field: FieldRef {
                        path: vec!["test".to_string()],
                    },
                    expected: Value::String("value".to_string()),
                },
            },
            Some(ActionExecutionResult {
                result: ActionResult::Success {
                    message: "OK".to_string(),
                },
                rollback_data: Some(RollbackData::AssertRollback),
            }),
        ),
        create_test_execution_result(create_test_rule(None), EvaluationResult::NotSatisfied, None),
    ];

    let aggregated =
        AggregatedResult::from_results(node_id, batch_id, results, Duration::from_millis(10));
    let error_messages = aggregated.get_error_messages();

    assert_eq!(error_messages.len(), 2);
    assert!(
        error_messages
            .iter()
            .any(|msg| msg.contains("Eval error 1"))
    );
    assert!(
        error_messages
            .iter()
            .any(|msg| msg.contains("Action error 1"))
    );
}

#[test]
fn test_get_compliance_failures() {
    let node_id = Uuid::new_v4();
    let batch_id = "test".to_string();

    let results = vec![
        create_test_execution_result(
            create_test_rule(Some("compliance_rule_1".to_string())),
            EvaluationResult::Satisfied {
                action: Action::Assert {
                    field: FieldRef {
                        path: vec!["version".to_string()],
                    },
                    expected: Value::String("2.0".to_string()),
                },
            },
            Some(ActionExecutionResult {
                result: ActionResult::ComplianceFailure {
                    field: "version".to_string(),
                    expected: json!("2.0"),
                    actual: json!("1.0"),
                },
                rollback_data: Some(RollbackData::AssertRollback),
            }),
        ),
        create_test_execution_result(
            create_test_rule(None), // Rule with no ID
            EvaluationResult::Satisfied {
                action: Action::Assert {
                    field: FieldRef {
                        path: vec!["status".to_string()],
                    },
                    expected: Value::String("active".to_string()),
                },
            },
            Some(ActionExecutionResult {
                result: ActionResult::ComplianceFailure {
                    field: "status".to_string(),
                    expected: json!("active"),
                    actual: json!("inactive"),
                },
                rollback_data: Some(RollbackData::AssertRollback),
            }),
        ),
        create_test_execution_result(
            create_test_rule(None),
            EvaluationResult::Satisfied {
                action: Action::Assert {
                    field: FieldRef {
                        path: vec!["test".to_string()],
                    },
                    expected: Value::String("value".to_string()),
                },
            },
            Some(ActionExecutionResult {
                result: ActionResult::Success {
                    message: "OK".to_string(),
                },
                rollback_data: Some(RollbackData::AssertRollback),
            }),
        ),
    ];

    let aggregated =
        AggregatedResult::from_results(node_id, batch_id, results, Duration::from_millis(10));
    let failures = aggregated.get_compliance_failures();

    assert_eq!(failures.len(), 2);

    let first_failure = &failures[0];
    assert_eq!(first_failure.rule_name, "compliance_rule_1");
    assert_eq!(first_failure.field, "version");
    assert_eq!(first_failure.expected, json!("2.0"));
    assert_eq!(first_failure.actual, json!("1.0"));

    let second_failure = &failures[1];
    assert_eq!(second_failure.rule_name, "unnamed");
    assert_eq!(second_failure.field, "status");
    assert_eq!(second_failure.expected, json!("active"));
    assert_eq!(second_failure.actual, json!("inactive"));
}
