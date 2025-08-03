//! Tests for aggregated policy evaluation results

use super::test_helpers::*;
use crate::policy::ast::{Action, FieldRef, Value};
use crate::policy::evaluator::context::{ActionExecutionResult, ActionResult, EvaluationResult};
use crate::policy::evaluator::results::AggregatedResult;
use serde_json::json;
use std::time::Duration;
use uuid::Uuid;

#[test]
fn test_aggregated_result_with_mixed_results() {
    let node_id = Uuid::new_v4();
    let batch_id = "test-batch".to_string();
    let execution_duration = Duration::from_millis(500);

    // Create test results with different outcomes
    let results = vec![
        // Satisfied rule with successful action
        create_test_execution_result(
            create_test_rule(Some("rule1".to_string())),
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
                rollback_data: Some(
                    crate::policy::evaluator::context::RollbackData::AssertRollback,
                ),
            }),
        ),
        // Satisfied rule with compliance failure
        create_test_execution_result(
            create_test_rule(Some("rule2".to_string())),
            EvaluationResult::Satisfied {
                action: Action::Assert {
                    field: FieldRef {
                        path: vec!["test".to_string()],
                    },
                    expected: Value::String("value".to_string()),
                },
            },
            Some(ActionExecutionResult {
                result: ActionResult::ComplianceFailure {
                    field: "test".to_string(),
                    expected: json!("value"),
                    actual: json!("wrong"),
                },
                rollback_data: Some(
                    crate::policy::evaluator::context::RollbackData::AssertRollback,
                ),
            }),
        ),
        // Rule with evaluation error
        create_test_execution_result(
            create_test_rule(Some("rule3".to_string())),
            EvaluationResult::Error {
                message: "Evaluation failed".to_string(),
            },
            None,
        ),
        // Rule not satisfied
        create_test_execution_result(
            create_test_rule(Some("rule4".to_string())),
            EvaluationResult::NotSatisfied,
            None,
        ),
        // Satisfied rule with action error
        create_test_execution_result(
            create_test_rule(Some("rule5".to_string())),
            EvaluationResult::Satisfied {
                action: Action::Set {
                    field: FieldRef {
                        path: vec!["test".to_string()],
                    },
                    value: Value::String("new_value".to_string()),
                },
            },
            Some(ActionExecutionResult {
                result: ActionResult::Error {
                    message: "Action failed".to_string(),
                },
                rollback_data: Some(
                    crate::policy::evaluator::context::RollbackData::SetRollback {
                        field: FieldRef {
                            path: vec!["test".to_string()],
                        },
                        previous_value: Some(json!("old_value")),
                    },
                ),
            }),
        ),
    ];

    let aggregated =
        AggregatedResult::from_results(node_id, batch_id.clone(), results, execution_duration);

    assert_eq!(aggregated.node_id, node_id);
    assert_eq!(aggregated.batch_id, batch_id);
    assert_eq!(aggregated.total_rules, 5);
    assert_eq!(aggregated.satisfied_rules, 3); // 3 rules were satisfied (though some had action failures)
    assert_eq!(aggregated.failed_rules, 1); // 1 compliance failure
    assert_eq!(aggregated.error_rules, 2); // 1 evaluation error + 1 action error
    assert_eq!(aggregated.compliance_failures, 1);
    assert_eq!(aggregated.execution_duration, execution_duration);
    assert!(!aggregated.is_fully_successful());
    assert!(aggregated.has_compliance_failures());

    // Success rate should be 2/5 = 40% (2 successful out of 5 total)
    assert!((aggregated.success_rate() - 40.0).abs() < f64::EPSILON);
}

#[test]
fn test_aggregated_result_edge_cases() {
    let node_id = Uuid::new_v4();
    let batch_id = "edge_test".to_string();

    // Test with only satisfied rules
    let satisfied_only = vec![create_test_execution_result(
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
            result: ActionResult::Success {
                message: "OK".to_string(),
            },
            rollback_data: Some(
                crate::policy::evaluator::context::RollbackData::SetRollback {
                    field: FieldRef {
                        path: vec!["test".to_string()],
                    },
                    previous_value: Some(json!("old_value")),
                },
            ),
        }),
    )];

    let satisfied_result = AggregatedResult::from_results(
        node_id,
        batch_id.clone(),
        satisfied_only,
        Duration::from_millis(10),
    );

    assert!(satisfied_result.is_fully_successful());
    assert!(!satisfied_result.has_compliance_failures());
    assert!((satisfied_result.success_rate() - 100.0).abs() < f64::EPSILON);

    // Test with only errors
    let error_only = vec![create_test_execution_result(
        create_test_rule(None),
        EvaluationResult::Error {
            message: "Critical error".to_string(),
        },
        None,
    )];

    let error_result =
        AggregatedResult::from_results(node_id, batch_id, error_only, Duration::from_millis(10));

    assert!(!error_result.is_fully_successful());
    assert!(!error_result.has_compliance_failures());
    assert!((error_result.success_rate() - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_success_rate_calculation() {
    let node_id = Uuid::new_v4();
    let batch_id = "rate_test".to_string();

    // Test edge case: empty results should have 100% success rate
    let empty = AggregatedResult::from_results(node_id, batch_id.clone(), vec![], Duration::ZERO);
    assert!((empty.success_rate() - 100.0).abs() < f64::EPSILON);

    // Test with mixed results: 2 successful, 1 failed, 1 error out of 4 total = 50%
    let mixed = vec![
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
                rollback_data: Some(
                    crate::policy::evaluator::context::RollbackData::AssertRollback,
                ),
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
                    message: "Also OK".to_string(),
                },
                rollback_data: Some(
                    crate::policy::evaluator::context::RollbackData::AssertRollback,
                ),
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
                result: ActionResult::ComplianceFailure {
                    field: "test".to_string(),
                    expected: json!("value"),
                    actual: json!("wrong"),
                },
                rollback_data: Some(
                    crate::policy::evaluator::context::RollbackData::AssertRollback,
                ),
            }),
        ),
        create_test_execution_result(
            create_test_rule(None),
            EvaluationResult::Error {
                message: "Something went wrong".to_string(),
            },
            None,
        ),
    ];

    let mixed_result =
        AggregatedResult::from_results(node_id, batch_id, mixed, Duration::from_millis(100));

    assert!((mixed_result.success_rate() - 50.0).abs() < f64::EPSILON);
    assert_eq!(mixed_result.total_rules, 4);
    assert!(!mixed_result.is_fully_successful());
}
