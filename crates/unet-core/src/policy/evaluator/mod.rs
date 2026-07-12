//! Policy evaluation engine
//!
//! This module provides the evaluation engine that takes parsed policy rules
//! and evaluates them against network nodes to determine compliance and actions.

pub mod actions;
pub mod comparisons;
pub mod conditions;
pub mod context;
pub mod engine;
pub mod orchestration;
pub mod results;
pub mod rollback;

// Re-export commonly used types
pub use context::{
    ActionExecutionResult, ActionResult, EvaluationContext, EvaluationResult,
    PolicyExecutionContext, PolicyExecutionResult, PolicyTransaction, RollbackData,
};
pub use engine::PolicyEvaluator;
pub use orchestration::{
    EvaluationBatch, OrchestrationConfig, OrchestrationRule, PolicyOrchestrator,
};
pub use results::{AggregatedResult, PolicyPriority};
pub use rollback::RollbackResult;

#[cfg(test)]
mod context_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::ast::{ComparisonOperator, Condition, FieldRef, Value};
    use serde_json::json;

    #[test]
    fn test_evaluate_simple_condition() {
        let context = EvaluationContext::new(json!({
            "name": "test-node",
            "config": {
                "vlan": 100
            }
        }));

        // Test True condition
        let result = conditions::evaluate_condition(&Condition::True, &context).unwrap();
        assert!(result);

        // Test False condition
        let result = conditions::evaluate_condition(&Condition::False, &context).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_evaluate_comparison() {
        let context = EvaluationContext::new(json!({
            "config": {
                "vlan": 100,
                "name": "test"
            }
        }));

        let field = FieldRef {
            path: vec!["config".to_string(), "vlan".to_string()],
        };
        let value = Value::Number(100.0);

        let result =
            conditions::evaluate_comparison(&field, &ComparisonOperator::Equal, &value, &context)
                .unwrap();
        assert!(result);

        let result = conditions::evaluate_comparison(
            &field,
            &ComparisonOperator::GreaterThan,
            &Value::Number(50.0),
            &context,
        )
        .unwrap();
        assert!(result);
    }

    #[test]
    fn test_evaluate_existence() {
        let context = EvaluationContext::new(json!({
            "config": {
                "vlan": 100,
                "name": null
            }
        }));

        let existing_field = FieldRef {
            path: vec!["config".to_string(), "vlan".to_string()],
        };
        assert!(!conditions::evaluate_existence(
            &existing_field,
            true,
            &context
        ));
        assert!(conditions::evaluate_existence(
            &existing_field,
            false,
            &context
        ));

        let null_field = FieldRef {
            path: vec!["config".to_string(), "name".to_string()],
        };
        assert!(conditions::evaluate_existence(&null_field, true, &context));
        assert!(!conditions::evaluate_existence(
            &null_field,
            false,
            &context
        ));

        let missing_field = FieldRef {
            path: vec!["missing".to_string()],
        };
        assert!(conditions::evaluate_existence(
            &missing_field,
            true,
            &context
        ));
        assert!(!conditions::evaluate_existence(
            &missing_field,
            false,
            &context
        ));
    }

    #[test]
    fn test_resolve_field() {
        let context = EvaluationContext::new(json!({
            "config": {
                "vlan": 100
            }
        }));

        let field = FieldRef {
            path: vec!["config".to_string(), "vlan".to_string()],
        };
        let result = conditions::resolve_field(&field, &context).unwrap();
        assert_eq!(result, json!(100));

        let missing_field = FieldRef {
            path: vec!["missing".to_string()],
        };
        assert!(conditions::resolve_field(&missing_field, &context).is_err());
    }
}
