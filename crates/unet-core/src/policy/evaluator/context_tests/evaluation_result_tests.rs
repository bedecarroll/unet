//! Tests for `EvaluationResult` and `ActionResult` variants

use super::super::context::*;
use crate::policy::ast::{Action, FieldRef, Value};
use serde_json::json;

#[test]
fn test_evaluation_result_variants() {
    let action = Action::Assert {
        field: FieldRef {
            path: vec!["status".to_string()],
        },
        expected: Value::String("active".to_string()),
    };

    let satisfied = EvaluationResult::Satisfied { action };

    let not_satisfied = EvaluationResult::NotSatisfied;

    let error = EvaluationResult::Error {
        message: "Evaluation failed".to_string(),
    };

    // Test pattern matching
    match satisfied {
        EvaluationResult::Satisfied { action: _ } => {}
        _ => panic!("Expected Satisfied variant"),
    }

    match not_satisfied {
        EvaluationResult::NotSatisfied => {}
        _ => panic!("Expected NotSatisfied variant"),
    }

    match &error {
        EvaluationResult::Error { message } => {
            assert_eq!(message, "Evaluation failed");
        }
        _ => panic!("Expected Error variant"),
    }
}

#[test]
fn test_action_result_variants() {
    let success = ActionResult::Success {
        message: "Action completed successfully".to_string(),
    };

    let compliance_failure = ActionResult::ComplianceFailure {
        field: "version".to_string(),
        expected: json!("15.1"),
        actual: json!("14.2"),
    };

    let error = ActionResult::Error {
        message: "Action execution failed".to_string(),
    };

    // Test pattern matching and values
    match &success {
        ActionResult::Success { message } => {
            assert_eq!(message, "Action completed successfully");
        }
        _ => panic!("Expected Success variant"),
    }

    match &compliance_failure {
        ActionResult::ComplianceFailure {
            field,
            expected,
            actual,
        } => {
            assert_eq!(field, "version");
            assert_eq!(expected, &json!("15.1"));
            assert_eq!(actual, &json!("14.2"));
        }
        _ => panic!("Expected ComplianceFailure variant"),
    }

    match &error {
        ActionResult::Error { message } => {
            assert_eq!(message, "Action execution failed");
        }
        _ => panic!("Expected Error variant"),
    }
}

#[test]
fn test_rollback_data_variants() {
    let set_rollback = RollbackData::SetRollback {
        field: FieldRef {
            path: vec!["version".to_string()],
        },
        previous_value: Some(json!("14.2")),
    };

    let apply_rollback = RollbackData::ApplyRollback {
        template_path: "/templates/cisco_config.j2".to_string(),
    };

    let assert_rollback = RollbackData::AssertRollback;

    // Test serialization/deserialization
    let set_json = serde_json::to_string(&set_rollback).unwrap();
    let set_deserialized: RollbackData = serde_json::from_str(&set_json).unwrap();

    match set_deserialized {
        RollbackData::SetRollback {
            field,
            previous_value,
        } => {
            assert_eq!(field.path, vec!["version".to_string()]);
            assert_eq!(previous_value, Some(json!("14.2")));
        }
        _ => panic!("Expected SetRollback variant"),
    }

    // Test other variants
    match &apply_rollback {
        RollbackData::ApplyRollback { template_path } => {
            assert_eq!(template_path, "/templates/cisco_config.j2");
        }
        _ => panic!("Expected ApplyRollback variant"),
    }

    match &assert_rollback {
        RollbackData::AssertRollback => {}
        _ => panic!("Expected AssertRollback variant"),
    }
}

#[test]
fn test_action_execution_result() {
    let success_result = ActionExecutionResult {
        result: ActionResult::Success {
            message: "Configuration applied".to_string(),
        },
        rollback_data: Some(RollbackData::SetRollback {
            field: FieldRef {
                path: vec!["config".to_string(), "vlan".to_string()],
            },
            previous_value: Some(json!(10)),
        }),
    };

    // Test serialization/deserialization
    let json_str = serde_json::to_string(&success_result).unwrap();
    let deserialized: ActionExecutionResult = serde_json::from_str(&json_str).unwrap();

    match &deserialized.result {
        ActionResult::Success { message } => {
            assert_eq!(message, "Configuration applied");
        }
        _ => panic!("Expected Success result"),
    }

    assert!(deserialized.rollback_data.is_some());
}
