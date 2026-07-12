//! Condition evaluation logic
//!
//! Contains functions for evaluating policy conditions against
//! evaluation contexts, including field resolution and existence checks.

use super::context::EvaluationContext;
use crate::policy::PolicyError;
use crate::policy::ast::{ComparisonOperator, Condition, FieldRef, Value};
use serde_json::Value as JsonValue;

/// Evaluate a condition against the given context
pub fn evaluate_condition(
    condition: &Condition,
    context: &EvaluationContext,
) -> Result<bool, PolicyError> {
    match condition {
        Condition::And(left, right) => {
            let left_result = evaluate_condition(left, context)?;
            let right_result = evaluate_condition(right, context)?;
            Ok(left_result && right_result)
        }
        Condition::Or(left, right) => {
            let left_result = evaluate_condition(left, context)?;
            let right_result = evaluate_condition(right, context)?;
            Ok(left_result || right_result)
        }
        Condition::Not(condition) => {
            let result = evaluate_condition(condition, context)?;
            Ok(!result)
        }
        Condition::Comparison {
            field,
            operator,
            value,
        } => evaluate_comparison(field, operator, value, context),
        Condition::Existence { field, is_null } => Ok(evaluate_existence(field, *is_null, context)),
        Condition::True => Ok(true),
        Condition::False => Ok(false),
    }
}

/// Evaluate a comparison condition
pub fn evaluate_comparison(
    field: &FieldRef,
    operator: &ComparisonOperator,
    value: &Value,
    context: &EvaluationContext,
) -> Result<bool, PolicyError> {
    let field_value = resolve_field(field, context)?;
    let comparison_value = resolve_value(value, context)?;

    match operator {
        ComparisonOperator::Equal => {
            super::comparisons::compare_json_values(&field_value, &comparison_value, |a, b| {
                (a - b).abs() < 1e-10
            })
            .or_else(|_| Ok(field_value == comparison_value))
        }
        ComparisonOperator::NotEqual => {
            super::comparisons::compare_json_values(&field_value, &comparison_value, |a, b| {
                (a - b).abs() >= 1e-10
            })
            .or_else(|_| Ok(field_value != comparison_value))
        }
        ComparisonOperator::LessThan => {
            super::comparisons::compare_json_values(&field_value, &comparison_value, |a, b| a < b)
        }
        ComparisonOperator::LessThanOrEqual => {
            super::comparisons::compare_json_values(&field_value, &comparison_value, |a, b| a <= b)
        }
        ComparisonOperator::GreaterThan => {
            super::comparisons::compare_json_values(&field_value, &comparison_value, |a, b| a > b)
        }
        ComparisonOperator::GreaterThanOrEqual => {
            super::comparisons::compare_json_values(&field_value, &comparison_value, |a, b| a >= b)
        }
        ComparisonOperator::Contains => {
            super::comparisons::evaluate_contains_json(&field_value, &comparison_value)
        }
        ComparisonOperator::Matches => {
            super::comparisons::evaluate_regex_match_json(&field_value, &comparison_value)
        }
    }
}

/// Evaluate existence condition
pub fn evaluate_existence(field: &FieldRef, is_null: bool, context: &EvaluationContext) -> bool {
    context
        .get_field(&field.path.join("."))
        .map_or(is_null, |value| is_null == value.is_null())
}

/// Resolve a field reference to its actual value in the context
pub fn resolve_field(
    field: &FieldRef,
    context: &EvaluationContext,
) -> Result<JsonValue, PolicyError> {
    context
        .get_field(&field.path.join("."))
        .cloned()
        .ok_or_else(|| PolicyError::FieldNotFound {
            field: field.path.join("."),
        })
}

/// Resolve a value (which may be a literal or field reference) to its actual JSON value
pub fn resolve_value(value: &Value, context: &EvaluationContext) -> Result<JsonValue, PolicyError> {
    match value {
        Value::String(s) => Ok(JsonValue::String(s.clone())),
        Value::Number(n) => Ok(JsonValue::Number(
            serde_json::Number::from_f64(*n).unwrap_or_else(|| serde_json::Number::from(0)),
        )),
        Value::Boolean(b) => Ok(JsonValue::Bool(*b)),
        Value::Null => Ok(JsonValue::Null),
        Value::Array(arr) => {
            let resolved_arr: Result<Vec<JsonValue>, PolicyError> =
                arr.iter().map(|v| resolve_value(v, context)).collect();
            Ok(JsonValue::Array(resolved_arr?))
        }
        Value::Object(obj) => {
            let mut resolved_obj = serde_json::Map::new();
            for (key, val) in obj {
                resolved_obj.insert(key.clone(), resolve_value(val, context)?);
            }
            Ok(JsonValue::Object(resolved_obj))
        }
        Value::FieldRef(field_ref) => resolve_field(field_ref, context),
        Value::Regex(pattern) => Ok(JsonValue::String(pattern.clone())),
    }
}
