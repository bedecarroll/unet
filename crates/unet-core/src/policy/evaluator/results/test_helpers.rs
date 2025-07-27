//! Shared test helpers for policy evaluation results tests

use crate::policy::ast::{Action, Condition, FieldRef, PolicyRule, Value};
use crate::policy::evaluator::context::{
    ActionExecutionResult, EvaluationResult, PolicyExecutionResult,
};

/// Helper function to create test policy rule
pub fn create_test_rule(id: Option<String>) -> PolicyRule {
    PolicyRule {
        id,
        condition: Condition::True,
        action: Action::Assert {
            field: FieldRef {
                path: vec!["test".to_string()],
            },
            expected: Value::String("value".to_string()),
        },
    }
}

/// Helper function to create test execution result
pub fn create_test_execution_result(
    rule: PolicyRule,
    evaluation_result: EvaluationResult,
    action_result: Option<ActionExecutionResult>,
) -> PolicyExecutionResult {
    PolicyExecutionResult::new(rule, evaluation_result, action_result)
}
