//! Tests for policy evaluation result summary generation

use super::test_helpers::*;
use crate::policy::ast::{Action, FieldRef, Value};
use crate::policy::evaluator::context::{ActionExecutionResult, ActionResult, EvaluationResult};
use crate::policy::evaluator::results::AggregatedResult;
use std::time::Duration;
use uuid::Uuid;

#[test]
fn test_generate_summary_all_cases() {
    let node_id = Uuid::new_v4();
    let batch_id = "test".to_string();

    // Test empty results
    let empty_result =
        AggregatedResult::from_results(node_id, batch_id.clone(), vec![], Duration::from_millis(0));
    assert_eq!(empty_result.summary, "No policies evaluated");

    // Test various combinations
    let results_with_all_types = vec![
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
        create_test_execution_result(create_test_rule(None), EvaluationResult::NotSatisfied, None),
        create_test_execution_result(
            create_test_rule(None),
            EvaluationResult::Error {
                message: "Error".to_string(),
            },
            None,
        ),
    ];

    let mixed_result = AggregatedResult::from_results(
        node_id,
        batch_id,
        results_with_all_types,
        Duration::from_millis(10),
    );

    assert!(mixed_result.summary.contains("3 total policies"));
    assert!(mixed_result.summary.contains("1 satisfied"));
    assert!(mixed_result.summary.contains("1 errors"));
    assert!(mixed_result.summary.contains("1 not applicable"));
}
