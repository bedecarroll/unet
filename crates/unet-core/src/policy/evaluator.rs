//! Policy evaluation engine
//!
//! This module provides the evaluation engine that takes parsed policy rules
//! and evaluates them against network nodes to determine compliance and actions.

use crate::datastore::DataStore;
use crate::policy::PolicyError;
use crate::policy::ast::{Action, ComparisonOperator, Condition, FieldRef, PolicyRule, Value};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::interval;
use uuid::Uuid;

/// Context for policy evaluation containing node data
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    /// Node data from the datastore
    pub node_data: JsonValue,
    /// Optional derived data from SNMP polling or other sources
    pub derived_data: Option<JsonValue>,
}

/// Result of policy evaluation
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum EvaluationResult {
    /// Policy condition was met and action should be executed
    Satisfied {
        /// Action to execute
        action: Action,
    },
    /// Policy condition was not met
    NotSatisfied,
    /// Policy evaluation failed due to an error
    Error {
        /// Error message describing the failure
        message: String,
    },
}

/// Result of action execution
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ActionResult {
    /// Action executed successfully
    Success {
        /// Success message
        message: String,
    },
    /// Action failed compliance check
    ComplianceFailure {
        /// Field that failed compliance
        field: String,
        /// Expected value
        expected: JsonValue,
        /// Actual value found
        actual: JsonValue,
    },
    /// Action failed due to an error
    Error {
        /// Error message describing the failure
        message: String,
    },
}

/// Rollback information for reversing an action
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum RollbackData {
    /// SET action rollback - contains the previous value of the field
    SetRollback {
        /// Field that was modified
        field: FieldRef,
        /// Previous value before modification
        previous_value: Option<JsonValue>,
    },
    /// APPLY action rollback - contains template that was applied
    ApplyRollback {
        /// Template path that was applied
        template_path: String,
    },
    /// ASSERT action rollback - no rollback needed (read-only)
    AssertRollback,
}

/// Result of action execution with rollback information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ActionExecutionResult {
    /// The result of the action
    pub result: ActionResult,
    /// Rollback data to undo this action
    pub rollback_data: Option<RollbackData>,
}

/// Transaction context for policy execution with rollback support
#[derive(Debug, Clone)]
pub struct PolicyTransaction {
    /// Unique transaction ID
    pub transaction_id: String,
    /// Node being modified
    pub node_id: Uuid,
    /// List of rollback operations in reverse order (last action first)
    pub rollback_stack: Vec<RollbackData>,
    /// Original node state before any modifications
    pub original_node_state: Option<JsonValue>,
    /// Timestamp of transaction start
    pub started_at: Instant,
}

/// Result of transaction rollback
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RollbackResult {
    /// Number of actions successfully rolled back
    pub actions_rolled_back: usize,
    /// Number of rollback operations that failed
    pub rollback_failures: usize,
    /// List of error messages from failed rollbacks
    pub error_messages: Vec<String>,
    /// Whether the rollback was fully successful
    pub success: bool,
}

/// Complete result of policy rule execution (evaluation + action)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PolicyExecutionResult {
    /// The rule that was executed
    pub rule: PolicyRule,
    /// Result of condition evaluation
    pub evaluation_result: EvaluationResult,
    /// Result of action execution (if condition was satisfied)
    pub action_result: Option<ActionExecutionResult>,
}

/// Priority level for policy rules
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PolicyPriority {
    /// Low priority execution
    Low = 0,
    /// Medium priority execution
    Medium = 1,
    /// High priority execution
    High = 2,
    /// Critical priority execution (highest)
    Critical = 3,
}

/// Policy rule with orchestration metadata for execution ordering
#[derive(Debug, Clone)]
pub struct OrchestrationRule {
    /// The policy rule to execute
    pub rule: PolicyRule,
    /// Priority level for execution ordering
    pub priority: PolicyPriority,
    /// Numeric order within priority level
    pub order: u32,
    /// Tags for categorization and filtering
    pub tags: Vec<String>,
}

/// Batch of policy evaluations for a node
#[derive(Debug, Clone)]
pub struct EvaluationBatch {
    /// Unique identifier of the node being evaluated
    pub node_id: Uuid,
    /// Evaluation context with node data and state
    pub context: EvaluationContext,
    /// List of policy rules to evaluate in order
    pub rules: Vec<OrchestrationRule>,
    /// Unique identifier for this evaluation batch
    pub batch_id: String,
    /// Timestamp when the batch was created
    pub created_at: Instant,
}

/// Aggregated results for a node's policy evaluation
#[derive(Debug, Clone)]
pub struct AggregatedResult {
    /// Unique identifier of the evaluated node
    pub node_id: Uuid,
    /// Unique identifier of the evaluation batch
    pub batch_id: String,
    /// Total number of policy rules evaluated
    pub total_rules: usize,
    /// Number of rules that passed/were satisfied
    pub satisfied_rules: usize,
    /// Number of rules that failed compliance checks
    pub failed_rules: usize,
    /// Number of rules that encountered execution errors
    pub error_rules: usize,
    /// Number of compliance violations found
    pub compliance_failures: usize,
    /// Total time taken to execute all rules
    pub execution_duration: Duration,
    /// Detailed results for each policy rule execution
    pub results: Vec<PolicyExecutionResult>,
    /// Human-readable summary of the evaluation
    pub summary: String,
}

/// Cache entry for policy evaluation results
#[derive(Debug, Clone)]
struct CacheEntry {
    result: AggregatedResult,
    expires_at: Instant,
}

/// Configuration for policy orchestration
#[derive(Debug, Clone)]
pub struct OrchestrationConfig {
    /// Maximum number of concurrent evaluations
    pub max_concurrent: usize,
    /// Cache TTL for evaluation results
    pub cache_ttl: Duration,
    /// Batch timeout before forced evaluation
    pub batch_timeout: Duration,
    /// Enable result caching
    pub enable_caching: bool,
}

/// Policy evaluation orchestrator for managing complex evaluation workflows
#[derive(Clone)]
pub struct PolicyOrchestrator {
    config: OrchestrationConfig,
    cache: HashMap<String, CacheEntry>,
    pending_batches: HashMap<Uuid, EvaluationBatch>,
}

impl Default for PolicyOrchestrator {
    fn default() -> Self {
        Self::new(OrchestrationConfig {
            max_concurrent: 10,
            cache_ttl: Duration::from_secs(300), // 5 minutes
            batch_timeout: Duration::from_secs(30),
            enable_caching: true,
        })
    }
}

/// Policy evaluation engine
pub struct PolicyEvaluator;

impl PolicyEvaluator {
    /// Evaluate a single policy rule against the given context
    ///
    /// # Errors
    /// Returns an error if condition evaluation fails.
    pub fn evaluate_rule(
        rule: &PolicyRule,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult, PolicyError> {
        if Self::evaluate_condition(&rule.condition, context)? {
            Ok(EvaluationResult::Satisfied {
                action: rule.action.clone(),
            })
        } else {
            Ok(EvaluationResult::NotSatisfied)
        }
    }

    /// Execute a single policy rule (evaluate condition and execute action if satisfied)
    ///
    /// # Errors
    /// Returns an error if condition evaluation or action execution fails.
    pub async fn execute_rule(
        rule: &PolicyRule,
        context: &EvaluationContext,
        datastore: &dyn DataStore,
        node_id: &Uuid,
    ) -> Result<PolicyExecutionResult, PolicyError> {
        let evaluation_result = Self::evaluate_rule(rule, context)?;

        let action_result = match &evaluation_result {
            EvaluationResult::Satisfied { action } => {
                Some(Self::execute_action_with_rollback(action, context, datastore, node_id).await?)
            }
            _ => None,
        };

        Ok(PolicyExecutionResult {
            rule: rule.clone(),
            evaluation_result,
            action_result,
        })
    }

    /// Execute multiple policy rules against the given context
    ///
    /// # Errors
    /// Returns an error if rule execution fails for any rule.
    pub async fn execute_rules(
        rules: &[PolicyRule],
        context: &EvaluationContext,
        datastore: &dyn DataStore,
        node_id: &Uuid,
    ) -> Result<Vec<PolicyExecutionResult>, PolicyError> {
        let mut results = Vec::new();

        for rule in rules {
            let result = Self::execute_rule(rule, context, datastore, node_id).await?;
            results.push(result);
        }

        Ok(results)
    }

    /// Execute a specific action with rollback information
    async fn execute_action_with_rollback(
        action: &Action,
        context: &EvaluationContext,
        datastore: &dyn DataStore,
        node_id: &Uuid,
    ) -> Result<ActionExecutionResult, PolicyError> {
        match action {
            Action::Assert { field, expected } => {
                let result = Self::execute_assert_action(field, expected, context)?;
                Ok(ActionExecutionResult {
                    result,
                    rollback_data: Some(RollbackData::AssertRollback),
                })
            }
            Action::Set { field, value } => {
                Self::execute_set_action_with_rollback(field, value, context, datastore, node_id)
                    .await
            }
            Action::ApplyTemplate { template_path } => {
                Self::execute_apply_template_action_with_rollback(
                    template_path,
                    context,
                    datastore,
                    node_id,
                )
                .await
            }
        }
    }

    /// Execute ASSERT action - checks compliance by verifying field has expected value
    fn execute_assert_action(
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
    /// Returns an error if datastore operations fail or field validation fails.
    pub async fn execute_set_action_with_rollback(
        field: &FieldRef,
        value: &Value,
        context: &EvaluationContext,
        datastore: &dyn DataStore,
        node_id: &Uuid,
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
        let mut node = datastore.get_node_required(node_id).await.map_err(|e| {
            PolicyError::DataStoreError {
                message: e.to_string(),
            }
        })?;

        // Get the current value for rollback
        let previous_value = Self::get_nested_field(&node.custom_data, &field.path[1..]);

        // Parse the new value
        let new_value = Self::resolve_value(value, context)?;

        // Update the custom_data field
        let mut custom_data = node.custom_data.clone();
        Self::set_nested_field(&mut custom_data, &field.path[1..], new_value)?;
        node.custom_data = custom_data;

        // Save the updated node
        datastore
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
    ///
    /// Returns `PolicyError` if:
    /// - The node cannot be found in the datastore
    /// - The datastore operation fails
    /// - Template assignment fails
    pub async fn execute_apply_template_action_with_rollback(
        template_path: &str,
        _context: &EvaluationContext,
        datastore: &dyn DataStore,
        node_id: &Uuid,
    ) -> Result<ActionExecutionResult, PolicyError> {
        // Get the current node
        let mut node = datastore.get_node_required(node_id).await.map_err(|e| {
            PolicyError::DataStoreError {
                message: e.to_string(),
            }
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
            datastore
                .update_node(&node)
                .await
                .map_err(|e| PolicyError::DataStoreError {
                    message: e.to_string(),
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

    /// Execute rollback operation for a single action
    ///
    /// # Errors
    ///
    /// Returns `PolicyError` if:
    /// - The node cannot be found in the datastore
    /// - The datastore operation fails
    /// - Rollback data is invalid or corrupted
    pub async fn execute_rollback(
        rollback_data: &RollbackData,
        datastore: &dyn DataStore,
        node_id: &Uuid,
    ) -> Result<(), PolicyError> {
        match rollback_data {
            RollbackData::SetRollback {
                field,
                previous_value,
            } => {
                Self::rollback_set_action(field, previous_value.as_ref(), datastore, node_id).await
            }
            RollbackData::ApplyRollback { template_path } => {
                Self::rollback_apply_action(template_path, datastore, node_id).await
            }
            RollbackData::AssertRollback => {
                // ASSERT actions are read-only, no rollback needed
                Ok(())
            }
        }
    }

    /// Rollback a SET action by restoring the previous value
    async fn rollback_set_action(
        field: &FieldRef,
        previous_value: Option<&JsonValue>,
        datastore: &dyn DataStore,
        node_id: &Uuid,
    ) -> Result<(), PolicyError> {
        // Get the current node
        let mut node = datastore.get_node_required(node_id).await.map_err(|e| {
            PolicyError::DataStoreError {
                message: e.to_string(),
            }
        })?;

        // Restore the previous value
        let mut custom_data = node.custom_data.clone();
        if !custom_data.is_object() {
            custom_data = JsonValue::Object(serde_json::Map::new());
        }

        match previous_value {
            Some(value) => {
                // Restore the previous value
                Self::set_nested_field(&mut custom_data, &field.path[1..], value.clone())?;
            }
            None => {
                // Field didn't exist before, remove it
                Self::remove_nested_field(&mut custom_data, &field.path[1..])?;
            }
        }

        node.custom_data = custom_data;

        // Save the updated node
        datastore
            .update_node(&node)
            .await
            .map_err(|e| PolicyError::DataStoreError {
                message: e.to_string(),
            })?;

        Ok(())
    }

    /// Rollback an APPLY action by removing the template assignment
    async fn rollback_apply_action(
        template_path: &str,
        datastore: &dyn DataStore,
        node_id: &Uuid,
    ) -> Result<(), PolicyError> {
        // Get the current node
        let mut node = datastore.get_node_required(node_id).await.map_err(|e| {
            PolicyError::DataStoreError {
                message: e.to_string(),
            }
        })?;

        // Remove template assignment from custom_data
        let mut custom_data = node.custom_data.clone();
        if let JsonValue::Object(map) = &mut custom_data {
            if let Some(JsonValue::Array(templates_array)) = map.get_mut("assigned_templates") {
                let template_value = JsonValue::String(template_path.to_string());
                templates_array.retain(|template| template != &template_value);

                // If the array is now empty, remove it entirely
                if templates_array.is_empty() {
                    map.remove("assigned_templates");
                }
            }
        }

        node.custom_data = custom_data;

        // Save the updated node
        datastore
            .update_node(&node)
            .await
            .map_err(|e| PolicyError::DataStoreError {
                message: e.to_string(),
            })?;

        Ok(())
    }

    /// Helper function to remove a nested field from JSON data
    fn remove_nested_field(data: &mut JsonValue, path: &[String]) -> Result<(), PolicyError> {
        if path.is_empty() {
            return Err(PolicyError::ValidationError {
                message: "Cannot remove empty path".to_string(),
            });
        }

        // Ensure data is an object
        if !data.is_object() {
            return Ok(()); // Nothing to remove if not an object
        }

        if let JsonValue::Object(map) = data {
            if path.len() == 1 {
                // Final field - remove it
                map.remove(&path[0]);
            } else {
                // Intermediate field - navigate deeper
                if let Some(next_data) = map.get_mut(&path[0]) {
                    Self::remove_nested_field(next_data, &path[1..])?;

                    // Clean up empty parent objects
                    if let JsonValue::Object(child_map) = next_data {
                        if child_map.is_empty() {
                            map.remove(&path[0]);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Execute multiple rules with transaction support for rollback
    ///
    /// # Errors
    ///
    /// Returns `PolicyError` if:
    /// - Any rule evaluation fails
    /// - Datastore operations fail
    /// - Transaction creation or management fails
    pub async fn execute_rules_with_transaction(
        rules: &[PolicyRule],
        context: &EvaluationContext,
        datastore: &dyn DataStore,
        node_id: &Uuid,
    ) -> Result<(Vec<PolicyExecutionResult>, PolicyTransaction), PolicyError> {
        // Create a new transaction
        let transaction_id = format!("tx_{}_{}", node_id, Instant::now().elapsed().as_millis());
        let mut transaction = PolicyTransaction {
            transaction_id,
            node_id: *node_id,
            rollback_stack: Vec::new(),
            original_node_state: None,
            started_at: Instant::now(),
        };

        // Capture original node state
        let original_node = datastore.get_node_required(node_id).await.map_err(|e| {
            PolicyError::DataStoreError {
                message: e.to_string(),
            }
        })?;
        transaction.original_node_state =
            Some(serde_json::to_value(&original_node).map_err(|e| {
                PolicyError::ValidationError {
                    message: e.to_string(),
                }
            })?);

        let mut results = Vec::new();

        for rule in rules {
            let result = Self::execute_rule(rule, context, datastore, node_id).await?;

            // If the action was executed successfully, add rollback data to transaction
            if let Some(action_result) = &result.action_result {
                if let Some(rollback_data) = &action_result.rollback_data {
                    transaction.rollback_stack.push(rollback_data.clone());
                }
            }

            results.push(result);
        }

        Ok((results, transaction))
    }

    /// Rollback a complete transaction
    ///
    /// # Errors
    ///
    /// Returns `PolicyError` if critical rollback operations fail
    /// Note: Individual rollback failures are collected but don't stop the process
    pub async fn rollback_transaction(
        transaction: &PolicyTransaction,
        datastore: &dyn DataStore,
    ) -> Result<RollbackResult, PolicyError> {
        let mut actions_rolled_back = 0;
        let mut rollback_failures = 0;
        let mut error_messages = Vec::new();

        // Execute rollbacks in reverse order (LIFO)
        for rollback_data in transaction.rollback_stack.iter().rev() {
            match Self::execute_rollback(rollback_data, datastore, &transaction.node_id).await {
                Ok(()) => actions_rolled_back += 1,
                Err(e) => {
                    rollback_failures += 1;
                    error_messages.push(format!("Rollback failed: {e}"));
                }
            }
        }

        Ok(RollbackResult {
            actions_rolled_back,
            rollback_failures,
            error_messages,
            success: rollback_failures == 0,
        })
    }

    /// Restore node to its original state before the transaction
    ///
    /// # Errors
    ///
    /// Returns `PolicyError` if:
    /// - Original state cannot be deserialized
    /// - Datastore update operation fails
    pub async fn restore_original_state(
        transaction: &PolicyTransaction,
        datastore: &dyn DataStore,
    ) -> Result<(), PolicyError> {
        if let Some(original_state) = &transaction.original_node_state {
            // Deserialize the original node state
            let original_node: crate::models::Node = serde_json::from_value(original_state.clone())
                .map_err(|e| PolicyError::ValidationError {
                    message: e.to_string(),
                })?;

            // Restore the node to its original state
            datastore.update_node(&original_node).await.map_err(|e| {
                PolicyError::DataStoreError {
                    message: e.to_string(),
                }
            })?;
        }

        Ok(())
    }

    /// Evaluate multiple policy rules against the given context
    ///
    /// # Errors
    ///
    /// Returns `PolicyError` if any rule evaluation fails due to:
    /// - Invalid field references
    /// - Type mismatches in comparisons
    /// - Invalid regex patterns
    pub fn evaluate_rules(
        rules: &[PolicyRule],
        context: &EvaluationContext,
    ) -> Result<Vec<EvaluationResult>, PolicyError> {
        rules
            .iter()
            .map(|rule| Self::evaluate_rule(rule, context))
            .collect()
    }

    fn evaluate_condition(
        condition: &Condition,
        context: &EvaluationContext,
    ) -> Result<bool, PolicyError> {
        match condition {
            Condition::And(left, right) => {
                let left_result = Self::evaluate_condition(left, context)?;
                let right_result = Self::evaluate_condition(right, context)?;
                Ok(left_result && right_result)
            }
            Condition::Or(left, right) => {
                let left_result = Self::evaluate_condition(left, context)?;
                let right_result = Self::evaluate_condition(right, context)?;
                Ok(left_result || right_result)
            }
            Condition::Not(condition) => {
                let result = Self::evaluate_condition(condition, context)?;
                Ok(!result)
            }
            Condition::Comparison {
                field,
                operator,
                value,
            } => Self::evaluate_comparison(field, operator, value, context),
            Condition::Existence { field, is_null } => {
                Ok(Self::evaluate_existence(field, *is_null, context))
            }
        }
    }

    fn evaluate_comparison(
        field: &FieldRef,
        operator: &ComparisonOperator,
        expected: &Value,
        context: &EvaluationContext,
    ) -> Result<bool, PolicyError> {
        let field_value = Self::resolve_field(field, context)?;
        let expected_value = Self::resolve_value(expected, context)?;

        match operator {
            ComparisonOperator::Equal => Ok(Self::json_values_equal(&field_value, &expected_value)),
            ComparisonOperator::NotEqual => {
                Ok(!Self::json_values_equal(&field_value, &expected_value))
            }
            ComparisonOperator::LessThan => {
                Self::compare_json_values(&field_value, &expected_value, |a, b| a < b)
            }
            ComparisonOperator::LessThanOrEqual => {
                Self::compare_json_values(&field_value, &expected_value, |a, b| a <= b)
            }
            ComparisonOperator::GreaterThan => {
                Self::compare_json_values(&field_value, &expected_value, |a, b| a > b)
            }
            ComparisonOperator::GreaterThanOrEqual => {
                Self::compare_json_values(&field_value, &expected_value, |a, b| a >= b)
            }
            ComparisonOperator::Contains => {
                Self::evaluate_contains_json(&field_value, &expected_value)
            }
            ComparisonOperator::Matches => {
                Self::evaluate_regex_match_json(&field_value, &expected_value)
            }
        }
    }

    fn evaluate_existence(field: &FieldRef, is_null: bool, context: &EvaluationContext) -> bool {
        Self::resolve_field(field, context)
            .map_or(is_null, |value| is_null == matches!(value, JsonValue::Null))
    }

    fn resolve_field(
        field: &FieldRef,
        context: &EvaluationContext,
    ) -> Result<JsonValue, PolicyError> {
        let mut current = &context.node_data;

        for part in &field.path {
            current = current
                .get(part)
                .ok_or_else(|| PolicyError::FieldNotFound {
                    field: field.to_string(),
                })?;
        }

        Ok(current.clone())
    }

    fn resolve_value(value: &Value, context: &EvaluationContext) -> Result<JsonValue, PolicyError> {
        match value {
            Value::String(s) => Ok(JsonValue::String(s.clone())),
            Value::Number(n) => Ok(JsonValue::Number(serde_json::Number::from_f64(*n).unwrap())),
            Value::Boolean(b) => Ok(JsonValue::Bool(*b)),
            Value::Null => Ok(JsonValue::Null),
            Value::Regex(r) => Ok(JsonValue::String(r.clone())), // Regex as string for comparison
            Value::FieldRef(field) => Self::resolve_field(field, context),
        }
    }

    fn json_values_equal(actual: &JsonValue, expected: &JsonValue) -> bool {
        match (actual, expected) {
            (JsonValue::String(a), JsonValue::String(e)) => a == e,
            (JsonValue::Number(a), JsonValue::Number(e)) => a.as_f64() == e.as_f64(),
            (JsonValue::Bool(a), JsonValue::Bool(e)) => a == e,
            (JsonValue::Null, JsonValue::Null) => true,
            _ => false,
        }
    }

    fn compare_json_values<F>(
        actual: &JsonValue,
        expected: &JsonValue,
        compare: F,
    ) -> Result<bool, PolicyError>
    where
        F: Fn(f64, f64) -> bool,
    {
        match (actual, expected) {
            (JsonValue::Number(a), JsonValue::Number(e)) => {
                let a_val = a.as_f64().ok_or_else(|| PolicyError::TypeMismatch {
                    expected: "number".to_string(),
                    actual: "invalid number".to_string(),
                })?;
                let e_val = e.as_f64().ok_or_else(|| PolicyError::TypeMismatch {
                    expected: "number".to_string(),
                    actual: "invalid number".to_string(),
                })?;
                Ok(compare(a_val, e_val))
            }
            _ => Err(PolicyError::TypeMismatch {
                expected: "number".to_string(),
                actual: format!("{actual:?}"),
            }),
        }
    }

    fn evaluate_contains_json(
        actual: &JsonValue,
        expected: &JsonValue,
    ) -> Result<bool, PolicyError> {
        match (actual, expected) {
            (JsonValue::String(haystack), JsonValue::String(needle)) => {
                Ok(haystack.contains(needle))
            }
            _ => Err(PolicyError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{actual:?}"),
            }),
        }
    }

    fn evaluate_regex_match_json(
        actual: &JsonValue,
        expected: &JsonValue,
    ) -> Result<bool, PolicyError> {
        match (actual, expected) {
            (JsonValue::String(text), JsonValue::String(pattern)) => {
                let regex = regex::Regex::new(pattern).map_err(|_| PolicyError::InvalidRegex {
                    pattern: pattern.clone(),
                })?;
                Ok(regex.is_match(text))
            }
            _ => Err(PolicyError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{actual:?}"),
            }),
        }
    }
}

impl PolicyOrchestrator {
    /// Create a new policy orchestrator with the given configuration
    #[must_use]
    pub fn new(config: OrchestrationConfig) -> Self {
        Self {
            config,
            cache: HashMap::new(),
            pending_batches: HashMap::new(),
        }
    }

    /// Add a batch of policy rules for evaluation
    pub fn schedule_evaluation(
        &mut self,
        node_id: Uuid,
        context: EvaluationContext,
        rules: Vec<OrchestrationRule>,
    ) -> String {
        let batch_id = format!(
            "batch_{}_{}",
            node_id.to_string().chars().take(8).collect::<String>(),
            Instant::now().elapsed().as_millis()
        );

        let batch = EvaluationBatch {
            node_id,
            context,
            rules: Self::sort_rules_by_priority(rules),
            batch_id: batch_id.clone(),
            created_at: Instant::now(),
        };

        self.pending_batches.insert(node_id, batch);
        batch_id
    }

    /// Execute all pending batches
    ///
    /// # Errors
    ///
    /// Returns `PolicyError` if:
    /// - Any batch execution fails
    /// - Datastore operations fail during evaluation
    pub async fn execute_pending_batches(
        &mut self,
        datastore: &dyn DataStore,
    ) -> Result<Vec<AggregatedResult>, PolicyError> {
        let mut results = Vec::new();
        let batches: Vec<_> = self.pending_batches.drain().collect();

        for (_node_id, batch) in batches {
            // Check cache first if enabled
            if self.config.enable_caching {
                let cache_key = Self::create_cache_key(&batch);
                if let Some(cached_result) = self.get_cached_result(&cache_key) {
                    results.push(cached_result);
                    continue;
                }
            }

            let result = self.execute_batch(&batch, datastore).await?;

            // Cache the result if enabled
            if self.config.enable_caching {
                let cache_key = Self::create_cache_key(&batch);
                self.cache_result(cache_key, &result);
            }

            results.push(result);
        }

        Ok(results)
    }

    /// Execute a single evaluation batch
    ///
    /// # Errors
    ///
    /// Returns `PolicyError` if:
    /// - Rule evaluation fails
    /// - Context creation fails
    /// - Datastore operations fail
    pub async fn execute_batch(
        &self,
        batch: &EvaluationBatch,
        datastore: &dyn DataStore,
    ) -> Result<AggregatedResult, PolicyError> {
        let start_time = Instant::now();
        let mut results = Vec::new();
        let mut satisfied_count = 0;
        let mut failed_count = 0;
        let mut error_count = 0;
        let mut compliance_failures = 0;

        // Execute rules in priority order
        for orchestration_rule in &batch.rules {
            let result = PolicyEvaluator::execute_rule(
                &orchestration_rule.rule,
                &batch.context,
                datastore,
                &batch.node_id,
            )
            .await?;

            // Count result types
            match &result.evaluation_result {
                EvaluationResult::Satisfied { .. } => {
                    satisfied_count += 1;
                    if let Some(action_exec_result) = &result.action_result {
                        if let ActionResult::ComplianceFailure { .. } = &action_exec_result.result {
                            compliance_failures += 1;
                        }
                    }
                }
                EvaluationResult::NotSatisfied => failed_count += 1,
                EvaluationResult::Error { .. } => error_count += 1,
            }

            results.push(result);
        }

        let execution_duration = start_time.elapsed();
        let total_rules = batch.rules.len();

        let summary = Self::create_summary(
            total_rules,
            satisfied_count,
            failed_count,
            error_count,
            compliance_failures,
        );

        Ok(AggregatedResult {
            node_id: batch.node_id,
            batch_id: batch.batch_id.clone(),
            total_rules,
            satisfied_rules: satisfied_count,
            failed_rules: failed_count,
            error_rules: error_count,
            compliance_failures,
            execution_duration,
            results,
            summary,
        })
    }

    /// Execute policies for a single node with orchestration
    ///
    /// # Errors
    ///
    /// Returns `PolicyError` if:
    /// - Batch scheduling fails
    /// - Policy evaluation fails
    /// - Datastore operations fail
    pub async fn evaluate_node_policies(
        &mut self,
        node_id: Uuid,
        context: EvaluationContext,
        rules: Vec<OrchestrationRule>,
        datastore: &dyn DataStore,
    ) -> Result<AggregatedResult, PolicyError> {
        let batch_id = self.schedule_evaluation(node_id, context, rules);
        let results = self.execute_pending_batches(datastore).await?;

        results
            .into_iter()
            .find(|r| r.batch_id == batch_id)
            .ok_or_else(|| PolicyError::Evaluation {
                message: "Failed to find batch result".to_string(),
            })
    }

    /// Start a background scheduler for automatic policy evaluation
    ///
    /// # Errors
    ///
    /// Returns `PolicyError` if:
    /// - Batch execution fails during scheduled runs
    /// - Timer or interval management fails
    pub async fn start_scheduler(
        &mut self,
        interval_duration: Duration,
        datastore: &dyn DataStore,
    ) -> Result<(), PolicyError> {
        let mut interval_timer = interval(interval_duration);

        loop {
            interval_timer.tick().await;

            // Clean expired cache entries
            self.clean_expired_cache();

            // Check for batches that have timed out and execute them if any exist
            let has_timed_out_batches = self
                .pending_batches
                .iter()
                .any(|(_, batch)| batch.created_at.elapsed() > self.config.batch_timeout);

            // Execute timed out batches
            if has_timed_out_batches {
                self.execute_pending_batches(datastore).await?;
            }
        }
    }

    /// Sort rules by priority (highest first) and then by order
    fn sort_rules_by_priority(mut rules: Vec<OrchestrationRule>) -> Vec<OrchestrationRule> {
        rules.sort_by(|a, b| {
            // Sort by priority descending, then by order ascending
            match b.priority.cmp(&a.priority) {
                std::cmp::Ordering::Equal => a.order.cmp(&b.order),
                other => other,
            }
        });
        rules
    }

    /// Create a cache key for a batch
    fn create_cache_key(batch: &EvaluationBatch) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        batch.node_id.hash(&mut hasher);

        // Hash rule content for cache invalidation
        for rule in &batch.rules {
            format!("{:?}", rule.rule).hash(&mut hasher);
            rule.priority.hash(&mut hasher);
            rule.order.hash(&mut hasher);
        }

        format!("cache_{:x}", hasher.finish())
    }

    /// Get cached result if available and not expired
    fn get_cached_result(&self, cache_key: &str) -> Option<AggregatedResult> {
        self.cache.get(cache_key).and_then(|entry| {
            if entry.expires_at > Instant::now() {
                Some(entry.result.clone())
            } else {
                None
            }
        })
    }

    /// Cache an evaluation result
    fn cache_result(&mut self, cache_key: String, result: &AggregatedResult) {
        let expires_at = Instant::now() + self.config.cache_ttl;
        let entry = CacheEntry {
            result: result.clone(),
            expires_at,
        };
        self.cache.insert(cache_key, entry);
    }

    /// Clean expired cache entries
    fn clean_expired_cache(&mut self) {
        let now = Instant::now();
        self.cache.retain(|_, entry| entry.expires_at > now);
    }

    /// Create a summary string for the aggregated result
    fn create_summary(
        total: usize,
        satisfied: usize,
        _failed: usize,
        error: usize,
        compliance_failures: usize,
    ) -> String {
        let success_rate = if total > 0 {
            // Convert to f64 safely - usize fits in f64 for reasonable values
            let satisfied_f64 = f64::from(u32::try_from(satisfied).unwrap_or(u32::MAX));
            let total_f64 = f64::from(u32::try_from(total).unwrap_or(u32::MAX));
            (satisfied_f64 / total_f64) * 100.0
        } else {
            0.0
        };

        format!(
            "Policy evaluation: {satisfied}/{total} rules satisfied ({success_rate:.1}% success). Failures: {compliance_failures} compliance, {error} errors"
        )
    }

    /// Get current cache statistics
    #[must_use]
    pub fn cache_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        stats.insert("total_entries".to_string(), self.cache.len());
        stats.insert("pending_batches".to_string(), self.pending_batches.len());

        let expired_count = self
            .cache
            .values()
            .filter(|entry| entry.expires_at <= Instant::now())
            .count();
        stats.insert("expired_entries".to_string(), expired_count);

        stats
    }
}

impl Default for OrchestrationConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 10,
            cache_ttl: Duration::from_secs(300),
            batch_timeout: Duration::from_secs(30),
            enable_caching: true,
        }
    }
}

impl OrchestrationRule {
    /// Create a new orchestration rule with default priority and order
    #[must_use]
    pub const fn new(rule: PolicyRule) -> Self {
        Self {
            rule,
            priority: PolicyPriority::Medium,
            order: 0,
            tags: Vec::new(),
        }
    }

    /// Create a new orchestration rule with specified priority
    #[must_use]
    pub const fn with_priority(rule: PolicyRule, priority: PolicyPriority) -> Self {
        Self {
            rule,
            priority,
            order: 0,
            tags: Vec::new(),
        }
    }

    /// Create a new orchestration rule with priority and order
    #[must_use]
    pub const fn with_priority_and_order(
        rule: PolicyRule,
        priority: PolicyPriority,
        order: u32,
    ) -> Self {
        Self {
            rule,
            priority,
            order,
            tags: Vec::new(),
        }
    }

    /// Add tags to the orchestration rule
    #[must_use]
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

impl EvaluationContext {
    /// Create a new evaluation context from node data
    #[must_use]
    pub const fn new(node_data: JsonValue) -> Self {
        Self {
            node_data,
            derived_data: None,
        }
    }

    /// Create a new evaluation context with both node and derived data
    #[must_use]
    pub const fn with_derived_data(node_data: JsonValue, derived_data: JsonValue) -> Self {
        Self {
            node_data,
            derived_data: Some(derived_data),
        }
    }
}

impl PolicyExecutionResult {
    /// Creates a new error result for a policy rule
    #[must_use]
    pub fn new_error(rule_id: &str, message: String) -> Self {
        Self {
            rule: PolicyRule {
                id: Some(rule_id.to_string()),
                condition: Condition::Comparison {
                    field: FieldRef {
                        path: vec!["error".to_string()],
                    },
                    operator: ComparisonOperator::Equal,
                    value: Value::Boolean(false),
                },
                action: Action::Assert {
                    field: FieldRef {
                        path: vec!["error".to_string()],
                    },
                    expected: Value::Boolean(false),
                },
            },
            evaluation_result: EvaluationResult::Error {
                message: message.clone(),
            },
            action_result: Some(ActionExecutionResult {
                result: ActionResult::Error { message },
                rollback_data: None,
            }),
        }
    }

    /// Gets the rule ID if available
    #[must_use]
    pub const fn rule_id(&self) -> Option<&String> {
        self.rule.id.as_ref()
    }

    /// Checks if this result represents an error
    #[must_use]
    pub const fn is_error(&self) -> bool {
        matches!(self.evaluation_result, EvaluationResult::Error { .. })
    }

    /// Checks if this result represents a satisfied policy
    #[must_use]
    pub const fn is_satisfied(&self) -> bool {
        matches!(self.evaluation_result, EvaluationResult::Satisfied { .. })
    }

    /// Checks if this result represents a compliance failure
    #[must_use]
    pub fn is_compliance_failure(&self) -> bool {
        self.action_result
            .as_ref()
            .is_some_and(|action_exec_result| {
                matches!(
                    action_exec_result.result,
                    ActionResult::ComplianceFailure { .. }
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::ast::*;
    use serde_json::json;

    #[test]
    fn test_simple_condition_evaluation() {
        let context = EvaluationContext::new(json!({
            "node": {
                "vendor": "cisco",
                "version": "15.1"
            }
        }));

        let condition = Condition::Comparison {
            field: FieldRef {
                path: vec!["node".to_string(), "vendor".to_string()],
            },
            operator: ComparisonOperator::Equal,
            value: Value::String("cisco".to_string()),
        };

        let result = PolicyEvaluator::evaluate_condition(&condition, &context);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_boolean_and_condition() {
        let context = EvaluationContext::new(json!({
            "node": {
                "vendor": "cisco",
                "model": "2960"
            }
        }));

        let left = Condition::Comparison {
            field: FieldRef {
                path: vec!["node".to_string(), "vendor".to_string()],
            },
            operator: ComparisonOperator::Equal,
            value: Value::String("cisco".to_string()),
        };

        let right = Condition::Comparison {
            field: FieldRef {
                path: vec!["node".to_string(), "model".to_string()],
            },
            operator: ComparisonOperator::Contains,
            value: Value::String("29".to_string()),
        };

        let condition = Condition::And(Box::new(left), Box::new(right));
        let result = PolicyEvaluator::evaluate_condition(&condition, &context);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_rule_evaluation() {
        let context = EvaluationContext::new(json!({
            "node": {
                "vendor": "cisco"
            }
        }));

        let rule = PolicyRule {
            id: None,
            condition: Condition::Comparison {
                field: FieldRef {
                    path: vec!["node".to_string(), "vendor".to_string()],
                },
                operator: ComparisonOperator::Equal,
                value: Value::String("cisco".to_string()),
            },
            action: Action::Assert {
                field: FieldRef {
                    path: vec!["node".to_string(), "version".to_string()],
                },
                expected: Value::String("15.1".to_string()),
            },
        };

        let result = PolicyEvaluator::evaluate_rule(&rule, &context);
        assert!(result.is_ok());
        match result.unwrap() {
            EvaluationResult::Satisfied { .. } => (),
            _ => panic!("Expected satisfied result"),
        }
    }

    #[test]
    fn test_existence_check() {
        let context = EvaluationContext::new(json!({
            "node": {
                "vendor": "cisco",
                "location": null
            }
        }));

        let condition = Condition::Existence {
            field: FieldRef {
                path: vec!["node".to_string(), "location".to_string()],
            },
            is_null: true,
        };

        let result = PolicyEvaluator::evaluate_condition(&condition, &context);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_assert_action_success() {
        let context = EvaluationContext::new(json!({
            "node": {
                "vendor": "cisco",
                "version": "15.1"
            }
        }));

        let field = FieldRef {
            path: vec!["node".to_string(), "version".to_string()],
        };
        let expected = Value::String("15.1".to_string());

        let result = PolicyEvaluator::execute_assert_action(&field, &expected, &context);
        assert!(result.is_ok());
        match result.unwrap() {
            ActionResult::Success { .. } => (),
            _ => panic!("Expected success result"),
        }
    }

    #[tokio::test]
    async fn test_assert_action_failure() {
        let context = EvaluationContext::new(json!({
            "node": {
                "vendor": "cisco",
                "version": "14.0"
            }
        }));

        let field = FieldRef {
            path: vec!["node".to_string(), "version".to_string()],
        };
        let expected = Value::String("15.1".to_string());

        let result = PolicyEvaluator::execute_assert_action(&field, &expected, &context);
        assert!(result.is_ok());
        match result.unwrap() {
            ActionResult::ComplianceFailure {
                field: f,
                expected: e,
                actual: a,
            } => {
                assert_eq!(f, "node.version");
                assert_eq!(e, json!("15.1"));
                assert_eq!(a, json!("14.0"));
            }
            _ => panic!("Expected compliance failure"),
        }
    }

    #[test]
    fn test_set_nested_field() {
        let mut data = json!({});
        let path = vec![
            "custom_data".to_string(),
            "location".to_string(),
            "rack".to_string(),
        ];
        let value = json!("R42");

        let result = PolicyEvaluator::set_nested_field(&mut data, &path, value);
        assert!(result.is_ok());
        assert_eq!(data["custom_data"]["location"]["rack"], json!("R42"));
    }

    #[test]
    fn test_set_nested_field_existing() {
        let mut data = json!({
            "custom_data": {
                "location": {
                    "building": "DC1"
                }
            }
        });
        let path = vec![
            "custom_data".to_string(),
            "location".to_string(),
            "rack".to_string(),
        ];
        let value = json!("R42");

        let result = PolicyEvaluator::set_nested_field(&mut data, &path, value);
        assert!(result.is_ok());
        assert_eq!(data["custom_data"]["location"]["rack"], json!("R42"));
        assert_eq!(data["custom_data"]["location"]["building"], json!("DC1"));
    }

    #[test]
    fn test_orchestration_rule_creation() {
        let rule = PolicyRule {
            id: None,
            condition: Condition::Comparison {
                field: FieldRef {
                    path: vec!["node".to_string(), "vendor".to_string()],
                },
                operator: ComparisonOperator::Equal,
                value: Value::String("cisco".to_string()),
            },
            action: Action::Assert {
                field: FieldRef {
                    path: vec!["node".to_string(), "version".to_string()],
                },
                expected: Value::String("15.1".to_string()),
            },
        };

        let orch_rule = OrchestrationRule::new(rule.clone());
        assert_eq!(orch_rule.priority, PolicyPriority::Medium);
        assert_eq!(orch_rule.order, 0);
        assert!(orch_rule.tags.is_empty());

        let orch_rule_high = OrchestrationRule::with_priority(rule.clone(), PolicyPriority::High);
        assert_eq!(orch_rule_high.priority, PolicyPriority::High);

        let orch_rule_ordered =
            OrchestrationRule::with_priority_and_order(rule, PolicyPriority::Critical, 5);
        assert_eq!(orch_rule_ordered.priority, PolicyPriority::Critical);
        assert_eq!(orch_rule_ordered.order, 5);
    }

    #[test]
    fn test_policy_priority_ordering() {
        assert!(PolicyPriority::Critical > PolicyPriority::High);
        assert!(PolicyPriority::High > PolicyPriority::Medium);
        assert!(PolicyPriority::Medium > PolicyPriority::Low);
    }

    #[test]
    fn test_rule_sorting_by_priority() {
        let base_rule = PolicyRule {
            id: None,
            condition: Condition::Comparison {
                field: FieldRef {
                    path: vec!["node".to_string(), "vendor".to_string()],
                },
                operator: ComparisonOperator::Equal,
                value: Value::String("cisco".to_string()),
            },
            action: Action::Assert {
                field: FieldRef {
                    path: vec!["node".to_string(), "version".to_string()],
                },
                expected: Value::String("15.1".to_string()),
            },
        };

        let rules = vec![
            OrchestrationRule::with_priority_and_order(base_rule.clone(), PolicyPriority::Low, 3),
            OrchestrationRule::with_priority_and_order(base_rule.clone(), PolicyPriority::High, 1),
            OrchestrationRule::with_priority_and_order(
                base_rule.clone(),
                PolicyPriority::Critical,
                2,
            ),
            OrchestrationRule::with_priority_and_order(base_rule, PolicyPriority::Medium, 0),
        ];

        let sorted = PolicyOrchestrator::sort_rules_by_priority(rules);

        // Should be sorted by priority (Critical, High, Medium, Low)
        assert_eq!(sorted[0].priority, PolicyPriority::Critical);
        assert_eq!(sorted[1].priority, PolicyPriority::High);
        assert_eq!(sorted[2].priority, PolicyPriority::Medium);
        assert_eq!(sorted[3].priority, PolicyPriority::Low);
    }

    #[test]
    fn test_orchestration_config_default() {
        let config = OrchestrationConfig::default();
        assert_eq!(config.max_concurrent, 10);
        assert_eq!(config.cache_ttl, Duration::from_secs(300));
        assert_eq!(config.batch_timeout, Duration::from_secs(30));
        assert!(config.enable_caching);
    }

    #[tokio::test]
    async fn test_policy_orchestrator_cache_stats() {
        let orchestrator = PolicyOrchestrator::default();
        let stats = orchestrator.cache_stats();

        assert_eq!(stats.get("total_entries"), Some(&0));
        assert_eq!(stats.get("pending_batches"), Some(&0));
        assert_eq!(stats.get("expired_entries"), Some(&0));
    }

    #[test]
    fn test_cache_key_generation() {
        let node_id = Uuid::new_v4();
        let context = EvaluationContext::new(json!({"node": {"vendor": "cisco"}}));

        let rule = PolicyRule {
            id: None,
            condition: Condition::Comparison {
                field: FieldRef {
                    path: vec!["node".to_string(), "vendor".to_string()],
                },
                operator: ComparisonOperator::Equal,
                value: Value::String("cisco".to_string()),
            },
            action: Action::Assert {
                field: FieldRef {
                    path: vec!["node".to_string(), "version".to_string()],
                },
                expected: Value::String("15.1".to_string()),
            },
        };

        let batch = EvaluationBatch {
            node_id,
            context,
            rules: vec![OrchestrationRule::new(rule)],
            batch_id: "test_batch".to_string(),
            created_at: Instant::now(),
        };

        let cache_key = PolicyOrchestrator::create_cache_key(&batch);
        assert!(cache_key.starts_with("cache_"));
        assert!(cache_key.len() > 10); // Should be a reasonable length hash
    }

    #[test]
    fn test_summary_creation() {
        let summary = PolicyOrchestrator::create_summary(10, 8, 1, 1, 2);
        assert!(summary.contains("8/10 rules satisfied"));
        assert!(summary.contains("80.0% success"));
        assert!(summary.contains("2 compliance"));
        assert!(summary.contains("1 errors"));
    }

    #[tokio::test]
    async fn test_set_action_rollback() {
        use crate::datastore::csv::CsvStore;
        use crate::models::{DeviceRole, Node, Vendor};
        use serde_json::json;
        use tempfile::tempdir;

        // Create a test node with initial custom_data
        let mut node = Node::new(
            "test-node".to_string(),
            "test.example.com".to_string(),
            Vendor::Cisco,
            DeviceRole::Router,
        );
        node.model = "ISR4431".to_string();
        node.custom_data = json!({"location": {"rack": "R1"}});

        // Create CSV datastore in temp directory
        let temp_dir = tempdir().unwrap();
        let datastore = CsvStore::new(temp_dir.path()).await.unwrap();
        let node_id = node.id;
        datastore.create_node(&node).await.unwrap();

        // Test SET action with rollback
        let field = FieldRef {
            path: vec![
                "custom_data".to_string(),
                "location".to_string(),
                "rack".to_string(),
            ],
        };
        let new_value = Value::String("R2".to_string());
        let context = EvaluationContext::new(json!({"node": {"vendor": "cisco"}}));

        // Execute SET action with rollback
        let result = PolicyEvaluator::execute_set_action_with_rollback(
            &field, &new_value, &context, &datastore, &node_id,
        )
        .await
        .unwrap();

        // Verify action succeeded
        assert!(matches!(result.result, ActionResult::Success { .. }));
        assert!(result.rollback_data.is_some());

        // Verify field was updated
        let updated_node = datastore.get_node_required(&node_id).await.unwrap();
        assert_eq!(updated_node.custom_data["location"]["rack"], json!("R2"));

        // Execute rollback
        if let Some(rollback_data) = &result.rollback_data {
            PolicyEvaluator::execute_rollback(rollback_data, &datastore, &node_id)
                .await
                .unwrap();
        }

        // Verify field was restored
        let restored_node = datastore.get_node_required(&node_id).await.unwrap();
        assert_eq!(restored_node.custom_data["location"]["rack"], json!("R1"));
    }

    #[tokio::test]
    async fn test_apply_action_rollback() {
        use crate::datastore::csv::CsvStore;
        use crate::models::{DeviceRole, Node, Vendor};
        use serde_json::json;
        use tempfile::tempdir;

        // Create a test node
        let mut node = Node::new(
            "test-node".to_string(),
            "test.example.com".to_string(),
            Vendor::Cisco,
            DeviceRole::Switch,
        );
        node.model = "Catalyst2960".to_string();

        // Create CSV datastore in temp directory
        let temp_dir = tempdir().unwrap();
        let datastore = CsvStore::new(temp_dir.path()).await.unwrap();
        let node_id = node.id;
        datastore.create_node(&node).await.unwrap();

        let template_path = "templates/cisco-switch.j2";
        let context = EvaluationContext::new(json!({"node": {"vendor": "cisco"}}));

        // Execute APPLY action with rollback
        let result = PolicyEvaluator::execute_apply_template_action_with_rollback(
            template_path,
            &context,
            &datastore,
            &node_id,
        )
        .await
        .unwrap();

        // Verify action succeeded
        assert!(matches!(result.result, ActionResult::Success { .. }));
        assert!(result.rollback_data.is_some());

        // Verify template was assigned
        let updated_node = datastore.get_node_required(&node_id).await.unwrap();
        let assigned_templates = updated_node.custom_data["assigned_templates"]
            .as_array()
            .unwrap();
        assert!(assigned_templates.contains(&json!(template_path)));

        // Execute rollback
        if let Some(rollback_data) = &result.rollback_data {
            PolicyEvaluator::execute_rollback(rollback_data, &datastore, &node_id)
                .await
                .unwrap();
        }

        // Verify template assignment was removed
        let restored_node = datastore.get_node_required(&node_id).await.unwrap();
        assert!(
            !restored_node
                .custom_data
                .get("assigned_templates")
                .and_then(|t| t.as_array())
                .is_some_and(|arr| arr.contains(&json!(template_path)))
        );
    }

    // Helper function to create test node and datastore
    async fn setup_transaction_test() -> (
        crate::models::Node,
        crate::datastore::csv::CsvStore,
        uuid::Uuid,
    ) {
        use crate::datastore::csv::CsvStore;
        use crate::models::{DeviceRole, Node, Vendor};
        use serde_json::json;
        use tempfile::tempdir;

        let mut node = Node::new(
            "test-node".to_string(),
            "test.example.com".to_string(),
            Vendor::Cisco,
            DeviceRole::Router,
        );
        node.model = "ISR4431".to_string();
        node.custom_data = json!({"location": {"rack": "R1", "building": "DC1"}});

        let temp_dir = tempdir().unwrap();
        let datastore = CsvStore::new(temp_dir.path()).await.unwrap();
        let node_id = node.id;
        datastore.create_node(&node).await.unwrap();

        (node, datastore, node_id)
    }

    // Helper function to create test policy rules
    fn create_transaction_test_rules() -> Vec<PolicyRule> {
        vec![
            PolicyRule {
                id: Some("rule1".to_string()),
                condition: Condition::Comparison {
                    field: FieldRef {
                        path: vec!["node".to_string(), "vendor".to_string()],
                    },
                    operator: ComparisonOperator::Equal,
                    value: Value::String("cisco".to_string()),
                },
                action: Action::Set {
                    field: FieldRef {
                        path: vec![
                            "custom_data".to_string(),
                            "location".to_string(),
                            "rack".to_string(),
                        ],
                    },
                    value: Value::String("R2".to_string()),
                },
            },
            PolicyRule {
                id: Some("rule2".to_string()),
                condition: Condition::Comparison {
                    field: FieldRef {
                        path: vec!["node".to_string(), "vendor".to_string()],
                    },
                    operator: ComparisonOperator::Equal,
                    value: Value::String("cisco".to_string()),
                },
                action: Action::Set {
                    field: FieldRef {
                        path: vec![
                            "custom_data".to_string(),
                            "compliance".to_string(),
                            "status".to_string(),
                        ],
                    },
                    value: Value::String("passed".to_string()),
                },
            },
            PolicyRule {
                id: Some("rule3".to_string()),
                condition: Condition::Comparison {
                    field: FieldRef {
                        path: vec!["node".to_string(), "vendor".to_string()],
                    },
                    operator: ComparisonOperator::Equal,
                    value: Value::String("cisco".to_string()),
                },
                action: Action::ApplyTemplate {
                    template_path: "templates/cisco-router.j2".to_string(),
                },
            },
        ]
    }

    #[tokio::test]
    async fn test_transaction_execution() {
        use serde_json::json;

        let (_node, datastore, node_id) = setup_transaction_test().await;
        let rules = create_transaction_test_rules();

        let context = EvaluationContext::new(json!({
            "node": {
                "vendor": "cisco",
                "model": "ISR4431"
            }
        }));

        // Execute rules with transaction
        let (results, _transaction) =
            PolicyEvaluator::execute_rules_with_transaction(&rules, &context, &datastore, &node_id)
                .await
                .unwrap();

        // Verify all rules executed successfully
        assert_eq!(results.len(), 3);
        for result in &results {
            assert!(matches!(
                result.evaluation_result,
                EvaluationResult::Satisfied { .. }
            ));
        }

        // Verify modifications were applied
        let modified_node = datastore.get_node_required(&node_id).await.unwrap();
        assert_eq!(modified_node.custom_data["location"]["rack"], json!("R2"));
        assert_eq!(
            modified_node.custom_data["compliance"]["status"],
            json!("passed")
        );
        let assigned_templates = modified_node.custom_data["assigned_templates"]
            .as_array()
            .unwrap();
        assert!(assigned_templates.contains(&json!("templates/cisco-router.j2")));
    }

    #[tokio::test]
    async fn test_transaction_rollback() {
        use serde_json::json;

        let (_node, datastore, node_id) = setup_transaction_test().await;
        let rules = create_transaction_test_rules();

        let context = EvaluationContext::new(json!({
            "node": {
                "vendor": "cisco",
                "model": "ISR4431"
            }
        }));

        // Execute rules with transaction
        let (_results, transaction) =
            PolicyEvaluator::execute_rules_with_transaction(&rules, &context, &datastore, &node_id)
                .await
                .unwrap();

        // Execute transaction rollback
        let rollback_result = PolicyEvaluator::rollback_transaction(&transaction, &datastore)
            .await
            .unwrap();
        assert!(rollback_result.success);
        assert_eq!(rollback_result.actions_rolled_back, 3);
        assert_eq!(rollback_result.rollback_failures, 0);

        // Verify all changes were rolled back
        let restored_node = datastore.get_node_required(&node_id).await.unwrap();
        assert_eq!(restored_node.custom_data["location"]["rack"], json!("R1"));
        assert_eq!(
            restored_node.custom_data["location"]["building"],
            json!("DC1")
        );
        assert!(restored_node.custom_data.get("compliance").is_none());
        assert!(
            restored_node
                .custom_data
                .get("assigned_templates")
                .is_none()
        );
    }

    #[tokio::test]
    async fn test_rollback_with_missing_fields() {
        use crate::datastore::csv::CsvStore;
        use crate::models::{DeviceRole, Node, Vendor};
        use serde_json::json;
        use tempfile::tempdir;

        // Create a test node with empty custom_data
        let mut node = Node::new(
            "test-node".to_string(),
            "test.example.com".to_string(),
            Vendor::Juniper,
            DeviceRole::Router,
        );
        node.model = "MX240".to_string();

        // Create CSV datastore in temp directory
        let temp_dir = tempdir().unwrap();
        let datastore = CsvStore::new(temp_dir.path()).await.unwrap();
        let node_id = node.id;
        datastore.create_node(&node).await.unwrap();

        // Test SET action on non-existent field
        let field = FieldRef {
            path: vec!["custom_data".to_string(), "new_field".to_string()],
        };
        let new_value = Value::String("new_value".to_string());
        let context = EvaluationContext::new(json!({"node": {"vendor": "juniper"}}));

        // Execute SET action with rollback
        let result = PolicyEvaluator::execute_set_action_with_rollback(
            &field, &new_value, &context, &datastore, &node_id,
        )
        .await
        .unwrap();

        // Verify action succeeded and captured that field didn't exist
        assert!(matches!(result.result, ActionResult::Success { .. }));
        if let Some(RollbackData::SetRollback { previous_value, .. }) = &result.rollback_data {
            assert!(previous_value.is_none());
        }

        // Verify field was created
        let updated_node = datastore.get_node_required(&node_id).await.unwrap();
        assert_eq!(updated_node.custom_data["new_field"], json!("new_value"));

        // Execute rollback
        if let Some(rollback_data) = &result.rollback_data {
            PolicyEvaluator::execute_rollback(rollback_data, &datastore, &node_id)
                .await
                .unwrap();
        }

        // Verify field was removed (since it didn't exist before)
        let restored_node = datastore.get_node_required(&node_id).await.unwrap();
        assert!(restored_node.custom_data.get("new_field").is_none());
    }

    #[tokio::test]
    async fn test_original_state_restoration() {
        use crate::datastore::csv::CsvStore;
        use crate::models::{DeviceRole, Node, Vendor};
        use serde_json::json;
        use tempfile::tempdir;

        // Create a test node with complex initial state
        let mut node = Node::new(
            "test-node".to_string(),
            "test.example.com".to_string(),
            Vendor::Arista,
            DeviceRole::Switch,
        );
        node.model = "7050SX".to_string();
        node.custom_data = json!({
            "location": {"rack": "R1", "building": "DC1"},
            "compliance": {"status": "pending", "last_check": "2024-01-01"},
            "monitoring": {"enabled": true}
        });

        // Create CSV datastore in temp directory
        let temp_dir = tempdir().unwrap();
        let datastore = CsvStore::new(temp_dir.path()).await.unwrap();
        let node_id = node.id;
        let original_custom_data = node.custom_data.clone();
        datastore.create_node(&node).await.unwrap();

        // Create rules that make extensive modifications
        let rules = vec![
            PolicyRule {
                id: Some("rule1".to_string()),
                condition: Condition::Comparison {
                    field: FieldRef {
                        path: vec!["node".to_string(), "vendor".to_string()],
                    },
                    operator: ComparisonOperator::Equal,
                    value: Value::String("arista".to_string()),
                },
                action: Action::Set {
                    field: FieldRef {
                        path: vec![
                            "custom_data".to_string(),
                            "location".to_string(),
                            "rack".to_string(),
                        ],
                    },
                    value: Value::String("R3".to_string()),
                },
            },
            PolicyRule {
                id: Some("rule2".to_string()),
                condition: Condition::Comparison {
                    field: FieldRef {
                        path: vec!["node".to_string(), "vendor".to_string()],
                    },
                    operator: ComparisonOperator::Equal,
                    value: Value::String("arista".to_string()),
                },
                action: Action::Set {
                    field: FieldRef {
                        path: vec![
                            "custom_data".to_string(),
                            "compliance".to_string(),
                            "status".to_string(),
                        ],
                    },
                    value: Value::String("failed".to_string()),
                },
            },
        ];

        let context = EvaluationContext::new(json!({
            "node": {
                "vendor": "arista",
                "model": "7050SX"
            }
        }));

        // Execute rules with transaction
        let (_results, transaction) =
            PolicyEvaluator::execute_rules_with_transaction(&rules, &context, &datastore, &node_id)
                .await
                .unwrap();

        // Verify modifications were applied
        let modified_node = datastore.get_node_required(&node_id).await.unwrap();
        assert_eq!(modified_node.custom_data["location"]["rack"], json!("R3"));
        assert_eq!(
            modified_node.custom_data["compliance"]["status"],
            json!("failed")
        );

        // Restore original state
        PolicyEvaluator::restore_original_state(&transaction, &datastore)
            .await
            .unwrap();

        // Verify complete restoration to original state
        let restored_node = datastore.get_node_required(&node_id).await.unwrap();
        assert_eq!(restored_node.custom_data, original_custom_data);
        assert_eq!(restored_node.custom_data["location"]["rack"], json!("R1"));
        assert_eq!(
            restored_node.custom_data["compliance"]["status"],
            json!("pending")
        );
        assert_eq!(
            restored_node.custom_data["monitoring"]["enabled"],
            json!(true)
        );
    }

    #[test]
    fn test_rollback_data_serialization() {
        // Test that rollback data can be serialized/deserialized
        let set_rollback = RollbackData::SetRollback {
            field: FieldRef {
                path: vec!["custom_data".to_string(), "test".to_string()],
            },
            previous_value: Some(json!("old_value")),
        };

        let serialized = serde_json::to_string(&set_rollback).unwrap();
        let deserialized: RollbackData = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            RollbackData::SetRollback {
                field,
                previous_value,
            } => {
                assert_eq!(field.path, vec!["custom_data", "test"]);
                assert_eq!(previous_value, Some(json!("old_value")));
            }
            _ => panic!("Expected SetRollback"),
        }

        let apply_rollback = RollbackData::ApplyRollback {
            template_path: "templates/test.j2".to_string(),
        };

        let serialized = serde_json::to_string(&apply_rollback).unwrap();
        let deserialized: RollbackData = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            RollbackData::ApplyRollback { template_path } => {
                assert_eq!(template_path, "templates/test.j2");
            }
            _ => panic!("Expected ApplyRollback"),
        }
    }
}
