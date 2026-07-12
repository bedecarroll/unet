//! Tests for serde serialization of policy evaluation types

use super::super::context::*;
use crate::policy::ast::{Action, Condition, FieldRef, PolicyRule, Value};
use serde_json::json;

#[test]
fn test_serde_serialization() {
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

    let result = PolicyExecutionResult::new(
        rule,
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
                field: "version".to_string(),
                expected: json!("15.1"),
                actual: json!("14.2"),
            },
            rollback_data: Some(RollbackData::SetRollback {
                field: FieldRef {
                    path: vec!["version".to_string()],
                },
                previous_value: Some(json!("14.2")),
            }),
        }),
    );

    // Test serialization
    let json_str = serde_json::to_string(&result).unwrap();
    assert!(!json_str.is_empty());

    // Test deserialization
    let deserialized: PolicyExecutionResult = serde_json::from_str(&json_str).unwrap();

    assert_eq!(deserialized.rule.id, Some("test_rule".to_string()));
    assert!(matches!(
        deserialized.evaluation_result,
        EvaluationResult::Satisfied { .. }
    ));
    assert!(deserialized.action_result.is_some());
    assert!(deserialized.is_compliance_failure());
}
