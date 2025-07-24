//! Policy action execution with rollback support
//!
//! Contains the implementation for executing policy actions (SET, ASSERT, APPLY)
//! with proper rollback support for transactional policy evaluation.

use super::context::{
    ActionExecutionResult, ActionResult, EvaluationContext, PolicyExecutionContext, RollbackData,
};
use crate::policy::PolicyError;
use crate::policy::ast::{Action, FieldRef, Value};
use serde_json::Value as JsonValue;

/// Action executor for policy actions
pub struct ActionExecutor;

impl ActionExecutor {
    /// Execute a specific action with rollback information
    ///
    /// # Errors
    /// Returns an error if action execution fails
    pub async fn execute_action_with_rollback(
        action: &Action,
        exec_ctx: &PolicyExecutionContext<'_>,
    ) -> Result<ActionExecutionResult, PolicyError> {
        match action {
            Action::Assert { field, expected } => {
                let result = Self::execute_assert_action(field, expected, exec_ctx.context)?;
                Ok(ActionExecutionResult {
                    result,
                    rollback_data: Some(RollbackData::AssertRollback),
                })
            }
            Action::Set { field, value } => {
                Self::execute_set_action_with_rollback(field, value, exec_ctx).await
            }
            Action::ApplyTemplate { template_path } => {
                Self::execute_apply_template_action_with_rollback(template_path, exec_ctx).await
            }
        }
    }

    /// Execute ASSERT action - checks compliance by verifying field has expected value
    ///
    /// # Errors
    /// Returns an error if field resolution fails
    pub fn execute_assert_action(
        field: &FieldRef,
        expected: &Value,
        context: &EvaluationContext,
    ) -> Result<ActionResult, PolicyError> {
        let actual_value = Self::resolve_field(field, context)?;
        let expected_value = Self::resolve_value(expected, context)?;

        if Self::json_values_equal(&actual_value, &expected_value) {
            Ok(ActionResult::Success {
                message: format!("Compliance check passed: {field} == {expected}"),
            })
        } else {
            Ok(ActionResult::ComplianceFailure {
                field: field.to_string(),
                expected: expected_value,
                actual: actual_value,
            })
        }
    }

    /// Execute SET action with rollback support - updates `custom_data` field with new value
    ///
    /// # Errors
    /// Returns an error if datastore operations fail or field validation fails
    pub async fn execute_set_action_with_rollback(
        field: &FieldRef,
        value: &Value,
        exec_ctx: &PolicyExecutionContext<'_>,
    ) -> Result<ActionExecutionResult, PolicyError> {
        // For SET actions, we only support setting values in custom_data
        if field.path.is_empty() || field.path[0] != "custom_data" {
            return Ok(ActionExecutionResult {
                result: ActionResult::Error {
                    message: format!("SET action only supports custom_data fields, got: {field}"),
                },
                rollback_data: None,
            });
        }

        // Get the current node
        let mut node = exec_ctx
            .datastore
            .get_node(exec_ctx.node_id)
            .await
            .map_err(|e| PolicyError::DataStoreError {
                message: e.to_string(),
            })?
            .ok_or_else(|| PolicyError::NodeNotFound {
                node_id: exec_ctx.node_id.to_string(),
            })?;

        // Get the current value for rollback
        let previous_value = Self::get_nested_field(&node.custom_data, &field.path[1..]);

        // Parse the new value
        let new_value = Self::resolve_value(value, exec_ctx.context)?;

        // Update the custom_data field
        let mut custom_data = node.custom_data.clone();
        Self::set_nested_field(&mut custom_data, &field.path[1..], new_value)?;
        node.custom_data = custom_data;

        // Save the updated node
        exec_ctx
            .datastore
            .update_node(&node)
            .await
            .map_err(|e| PolicyError::DataStoreError {
                message: e.to_string(),
            })?;

        Ok(ActionExecutionResult {
            result: ActionResult::Success {
                message: format!("Successfully set {field} to {value}"),
            },
            rollback_data: Some(RollbackData::SetRollback {
                field: field.clone(),
                previous_value,
            }),
        })
    }

    /// Execute APPLY action with rollback support - assigns template to node for configuration generation
    ///
    /// # Errors
    /// Returns an error if the node cannot be found or datastore operations fail
    pub async fn execute_apply_template_action_with_rollback(
        template_path: &str,
        exec_ctx: &PolicyExecutionContext<'_>,
    ) -> Result<ActionExecutionResult, PolicyError> {
        // Get the current node
        let mut node = exec_ctx
            .datastore
            .get_node(exec_ctx.node_id)
            .await
            .map_err(|e| PolicyError::DataStoreError {
                message: e.to_string(),
            })?
            .ok_or_else(|| PolicyError::NodeNotFound {
                node_id: exec_ctx.node_id.to_string(),
            })?;

        // Add template assignment to custom_data
        let mut custom_data = node.custom_data.clone();
        if !custom_data.is_object() {
            custom_data = JsonValue::Object(serde_json::Map::new());
        }

        let mut template_was_already_assigned = false;

        if let JsonValue::Object(map) = &mut custom_data {
            // Add or update the assigned templates array
            let templates = map
                .entry("assigned_templates".to_string())
                .or_insert_with(|| JsonValue::Array(vec![]));

            if let JsonValue::Array(templates_array) = templates {
                let template_value = JsonValue::String(template_path.to_string());
                if templates_array.contains(&template_value) {
                    template_was_already_assigned = true;
                } else {
                    templates_array.push(template_value);
                }
            }
        }

        if !template_was_already_assigned {
            node.custom_data = custom_data;

            // Save the updated node
            exec_ctx.datastore.update_node(&node).await.map_err(|e| {
                PolicyError::DataStoreError {
                    message: e.to_string(),
                }
            })?;
        }

        Ok(ActionExecutionResult {
            result: ActionResult::Success {
                message: if template_was_already_assigned {
                    format!("Template '{template_path}' was already assigned to node")
                } else {
                    format!("Successfully applied template '{template_path}' to node")
                },
            },
            rollback_data: if template_was_already_assigned {
                None // No rollback needed if template was already assigned
            } else {
                Some(RollbackData::ApplyRollback {
                    template_path: template_path.to_string(),
                })
            },
        })
    }

    /// Helper function to set a nested field in JSON data
    fn set_nested_field(
        data: &mut JsonValue,
        path: &[String],
        value: JsonValue,
    ) -> Result<(), PolicyError> {
        if path.is_empty() {
            return Err(PolicyError::ValidationError {
                message: "Cannot set empty path".to_string(),
            });
        }

        // Ensure data is an object
        if !data.is_object() {
            *data = JsonValue::Object(serde_json::Map::new());
        }

        if let JsonValue::Object(map) = data {
            if path.len() == 1 {
                // Final field - set the value
                map.insert(path[0].clone(), value);
            } else {
                // Intermediate field - navigate deeper
                let next_data = map
                    .entry(path[0].clone())
                    .or_insert_with(|| JsonValue::Object(serde_json::Map::new()));
                Self::set_nested_field(next_data, &path[1..], value)?;
            }
        }

        Ok(())
    }

    /// Helper function to get a nested field value from JSON data for rollback
    fn get_nested_field(data: &JsonValue, path: &[String]) -> Option<JsonValue> {
        if path.is_empty() {
            return Some(data.clone());
        }

        let mut current = data;
        for part in path {
            current = current.get(part)?;
        }
        Some(current.clone())
    }

    /// Resolve a field reference to its actual value in the context
    fn resolve_field(
        field: &FieldRef,
        context: &EvaluationContext,
    ) -> Result<JsonValue, PolicyError> {
        context
            .get_field(&field.path.join("."))
            .cloned()
            .ok_or_else(|| PolicyError::ValidationError {
                message: format!("Field not found: {}", field.path.join(".")),
            })
    }

    /// Resolve a value (which may be a literal or field reference) to its actual JSON value
    fn resolve_value(value: &Value, context: &EvaluationContext) -> Result<JsonValue, PolicyError> {
        match value {
            Value::String(s) => Ok(JsonValue::String(s.clone())),
            Value::Number(n) => Ok(JsonValue::Number(
                serde_json::Number::from_f64(*n).unwrap_or_else(|| serde_json::Number::from(0)),
            )),
            Value::Boolean(b) => Ok(JsonValue::Bool(*b)),
            Value::Null => Ok(JsonValue::Null),
            Value::Array(arr) => {
                let resolved_arr: Result<Vec<JsonValue>, PolicyError> = arr
                    .iter()
                    .map(|v| Self::resolve_value(v, context))
                    .collect();
                Ok(JsonValue::Array(resolved_arr?))
            }
            Value::Object(obj) => {
                let mut resolved_obj = serde_json::Map::new();
                for (key, val) in obj {
                    resolved_obj.insert(key.clone(), Self::resolve_value(val, context)?);
                }
                Ok(JsonValue::Object(resolved_obj))
            }
            Value::FieldRef(field_ref) => Self::resolve_field(field_ref, context),
            Value::Regex(pattern) => Ok(JsonValue::String(pattern.clone())),
        }
    }

    /// Compare two JSON values for equality with special handling for numbers
    fn json_values_equal(a: &JsonValue, b: &JsonValue) -> bool {
        match (a, b) {
            (JsonValue::Number(n1), JsonValue::Number(n2)) => {
                // Handle floating point comparison with some tolerance
                if let (Some(f1), Some(f2)) = (n1.as_f64(), n2.as_f64()) {
                    (f1 - f2).abs() < 1e-10
                } else {
                    n1 == n2
                }
            }
            _ => a == b,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_json_values_equal() {
        // Test exact equality
        assert!(ActionExecutor::json_values_equal(
            &json!("test"),
            &json!("test")
        ));
        assert!(ActionExecutor::json_values_equal(&json!(42), &json!(42)));
        assert!(ActionExecutor::json_values_equal(
            &json!(true),
            &json!(true)
        ));

        // Test inequality
        assert!(!ActionExecutor::json_values_equal(
            &json!("test"),
            &json!("other")
        ));
        assert!(!ActionExecutor::json_values_equal(&json!(42), &json!(43)));

        // Test floating point comparison
        let num1 = serde_json::Number::from_f64(1.0).unwrap();
        let num2 = serde_json::Number::from_f64(1.000_000_000_000_1).unwrap();
        assert!(ActionExecutor::json_values_equal(
            &JsonValue::Number(num1),
            &JsonValue::Number(num2)
        ));
    }

    #[test]
    fn test_set_nested_field() {
        let mut data = json!({});
        let path = vec!["config".to_string(), "vlan".to_string()];
        let value = json!(100);

        ActionExecutor::set_nested_field(&mut data, &path, value).unwrap();

        assert_eq!(data["config"]["vlan"], json!(100));
    }

    #[test]
    fn test_get_nested_field() {
        let data = json!({
            "config": {
                "vlan": 100,
                "name": "test"
            }
        });

        let path = vec!["config".to_string(), "vlan".to_string()];
        let result = ActionExecutor::get_nested_field(&data, &path);

        assert_eq!(result, Some(json!(100)));
    }
}

#[cfg(test)]
mod actions_tests;
