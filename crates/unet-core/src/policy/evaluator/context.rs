//! Policy evaluation context and related types
//!
//! Contains the core context structures used during policy evaluation,
//! including evaluation context, execution context, and various result types.

use crate::datastore::DataStore;
use crate::policy::ast::{Action, FieldRef, PolicyRule};
use serde_json::Value as JsonValue;
use std::time::Instant;
use uuid::Uuid;

/// Context for policy evaluation containing node data
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    /// Node data from the datastore
    pub node_data: JsonValue,
    /// Optional derived data from SNMP polling or other sources
    pub derived_data: Option<JsonValue>,
}

impl EvaluationContext {
    /// Create a new evaluation context with node data
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

    /// Get a field value from the context using dot notation
    #[must_use]
    pub fn get_field(&self, field_path: &str) -> Option<&JsonValue> {
        field_path.strip_prefix("derived.").map_or_else(
            || Self::get_nested_value(&self.node_data, field_path),
            |path| {
                self.derived_data
                    .as_ref()
                    .and_then(|derived| Self::get_nested_value(derived, path))
            },
        )
    }

    /// Helper function to get nested JSON values using dot notation
    fn get_nested_value<'a>(value: &'a JsonValue, path: &str) -> Option<&'a JsonValue> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = value;

        for part in parts {
            if let JsonValue::Object(obj) = current {
                current = obj.get(part)?;
            } else {
                return None;
            }
        }

        Some(current)
    }
}

/// Complete context for policy execution including evaluation context and execution dependencies
pub struct PolicyExecutionContext<'a> {
    /// Node evaluation context
    pub context: &'a EvaluationContext,
    /// Datastore for persisting changes
    pub datastore: &'a dyn DataStore,
    /// ID of the node being evaluated
    pub node_id: &'a Uuid,
}

impl<'a> PolicyExecutionContext<'a> {
    /// Create a new policy execution context
    #[must_use]
    pub const fn new(
        context: &'a EvaluationContext,
        datastore: &'a dyn DataStore,
        node_id: &'a Uuid,
    ) -> Self {
        Self {
            context,
            datastore,
            node_id,
        }
    }
}

impl std::fmt::Debug for PolicyExecutionContext<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PolicyExecutionContext")
            .field("context", &self.context)
            .field("datastore", &"<DataStore>")
            .field("node_id", &self.node_id)
            .finish()
    }
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

impl PolicyTransaction {
    /// Create a new policy transaction
    #[must_use]
    pub fn new(transaction_id: String, node_id: Uuid) -> Self {
        Self {
            transaction_id,
            node_id,
            rollback_stack: Vec::new(),
            original_node_state: None,
            started_at: Instant::now(),
        }
    }

    /// Add rollback data to the transaction
    pub fn add_rollback(&mut self, rollback_data: RollbackData) {
        self.rollback_stack.push(rollback_data);
    }

    /// Set the original node state for this transaction
    pub fn set_original_state(&mut self, state: JsonValue) {
        self.original_node_state = Some(state);
    }
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

impl PolicyExecutionResult {
    /// Create a new policy execution result
    #[must_use]
    pub const fn new(
        rule: PolicyRule,
        evaluation_result: EvaluationResult,
        action_result: Option<ActionExecutionResult>,
    ) -> Self {
        Self {
            rule,
            evaluation_result,
            action_result,
        }
    }

    /// Check if the policy execution was successful
    #[must_use]
    pub fn is_successful(&self) -> bool {
        match &self.evaluation_result {
            EvaluationResult::Satisfied { .. } => self
                .action_result
                .as_ref()
                .is_some_and(|ar| matches!(ar.result, ActionResult::Success { .. })),
            EvaluationResult::NotSatisfied => true, // Not satisfied but no error
            EvaluationResult::Error { .. } => false,
        }
    }

    /// Check if the policy rule was satisfied (condition met)
    #[must_use]
    pub const fn is_satisfied(&self) -> bool {
        matches!(self.evaluation_result, EvaluationResult::Satisfied { .. })
    }

    /// Check if the policy execution resulted in an error
    #[must_use]
    pub fn is_error(&self) -> bool {
        match &self.evaluation_result {
            EvaluationResult::Error { .. } => true,
            EvaluationResult::Satisfied { .. } => self
                .action_result
                .as_ref()
                .is_some_and(|ar| matches!(ar.result, ActionResult::Error { .. })),
            EvaluationResult::NotSatisfied => false,
        }
    }

    /// Check if the policy execution resulted in a compliance failure
    #[must_use]
    pub fn is_compliance_failure(&self) -> bool {
        self.action_result
            .as_ref()
            .is_some_and(|ar| matches!(ar.result, ActionResult::ComplianceFailure { .. }))
    }

    /// Get error message if any part of execution failed
    #[must_use]
    pub fn get_error_message(&self) -> Option<&str> {
        match &self.evaluation_result {
            EvaluationResult::Error { message } => Some(message),
            EvaluationResult::Satisfied { .. } => {
                self.action_result
                    .as_ref()
                    .and_then(|action_result| match &action_result.result {
                        ActionResult::Error { message } => Some(message.as_str()),
                        _ => None,
                    })
            }
            EvaluationResult::NotSatisfied => None,
        }
    }

    /// Create a new policy execution result with an error
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn new_error(rule: PolicyRule, error_message: String) -> Self {
        Self {
            rule,
            evaluation_result: EvaluationResult::Error {
                message: error_message,
            },
            action_result: None,
        }
    }

    /// Create a new policy execution result with an error using just a rule ID
    #[must_use]
    pub fn new_error_with_id(rule_id: Option<String>, error_message: String) -> Self {
        // Create a minimal PolicyRule for error reporting
        let rule = PolicyRule {
            id: rule_id,
            condition: crate::policy::ast::Condition::True,
            action: crate::policy::ast::Action::Assert {
                field: crate::policy::ast::FieldRef {
                    path: vec!["error".to_string()],
                },
                expected: crate::policy::ast::Value::String("error".to_string()),
            },
        };
        Self::new_error(rule, error_message)
    }

    /// Get the rule ID (name) if available
    #[must_use]
    pub fn rule_id(&self) -> Option<&str> {
        self.rule.id.as_deref()
    }
}
