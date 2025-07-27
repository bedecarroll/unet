//! Tests for filtering policy evaluation results

use super::test_helpers::*;
use crate::policy::ast::{Action, FieldRef, Value};
use crate::policy::evaluator::context::EvaluationResult;
use crate::policy::evaluator::results::AggregatedResult;
use std::time::Duration;
use uuid::Uuid;

#[test]
fn test_filter_by_result_type() {
    let node_id = Uuid::new_v4();
    let batch_id = "test".to_string();

    let results = vec![
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
            None,
        ),
        create_test_execution_result(create_test_rule(None), EvaluationResult::NotSatisfied, None),
        create_test_execution_result(
            create_test_rule(None),
            EvaluationResult::Error {
                message: "Error".to_string(),
            },
            None,
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
            None,
        ),
    ];

    let aggregated =
        AggregatedResult::from_results(node_id, batch_id, results, Duration::from_millis(10));

    // Filter for satisfied only
    let satisfied = aggregated.filter_by_result_type(true);
    assert_eq!(satisfied.len(), 2);

    // Filter for non-satisfied
    let non_satisfied = aggregated.filter_by_result_type(false);
    assert_eq!(non_satisfied.len(), 2);
}
