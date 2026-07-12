//! Policy evaluation engine
//!
//! Contains the core `PolicyEvaluator` implementation for evaluating
//! policy rules against network nodes.

#[cfg(test)]
mod tests;

use super::actions::ActionExecutor;
use super::context::{
    EvaluationContext, EvaluationResult, PolicyExecutionContext, PolicyExecutionResult,
    PolicyTransaction,
};
use crate::policy::PolicyError;
use crate::policy::ast::PolicyRule;
use std::time::Instant;

/// Policy evaluation engine
pub struct PolicyEvaluator;

impl PolicyEvaluator {
    /// Evaluate a single policy rule against the given context
    ///
    /// # Errors
    /// Returns an error if condition evaluation fails
    pub fn evaluate_rule(
        rule: &PolicyRule,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult, PolicyError> {
        if super::conditions::evaluate_condition(&rule.condition, context)? {
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
    /// Returns an error if condition evaluation or action execution fails
    pub async fn execute_rule(
        rule: &PolicyRule,
        exec_ctx: &PolicyExecutionContext<'_>,
    ) -> Result<PolicyExecutionResult, PolicyError> {
        let evaluation_result = Self::evaluate_rule(rule, exec_ctx.context)?;

        let action_result = match &evaluation_result {
            EvaluationResult::Satisfied { action } => {
                Some(ActionExecutor::execute_action_with_rollback(action, exec_ctx).await?)
            }
            _ => None,
        };

        Ok(PolicyExecutionResult::new(
            rule.clone(),
            evaluation_result,
            action_result,
        ))
    }

    /// Execute multiple policy rules against the given context
    ///
    /// # Errors
    /// Returns an error if any rule evaluation fails
    pub fn evaluate_rules(
        rules: &[PolicyRule],
        context: &EvaluationContext,
    ) -> Result<Vec<EvaluationResult>, PolicyError> {
        rules
            .iter()
            .map(|rule| Self::evaluate_rule(rule, context))
            .collect()
    }

    /// Execute multiple rules with transaction support for rollback
    ///
    /// # Errors
    /// Returns an error if rule evaluation fails or transaction creation fails
    pub async fn execute_rules_with_transaction(
        rules: &[PolicyRule],
        exec_ctx: &PolicyExecutionContext<'_>,
    ) -> Result<(Vec<PolicyExecutionResult>, PolicyTransaction), PolicyError> {
        // Create a new transaction
        let transaction_id = format!(
            "tx_{}_{}",
            exec_ctx.node_id,
            Instant::now().elapsed().as_millis()
        );
        let mut transaction = PolicyTransaction::new(transaction_id, *exec_ctx.node_id);

        // Capture original node state
        let original_node = exec_ctx
            .datastore
            .get_node(exec_ctx.node_id)
            .await
            .map_err(|e| PolicyError::DataStoreError {
                message: e.to_string(),
            })?
            .ok_or_else(|| PolicyError::NodeNotFound {
                node_id: exec_ctx.node_id.to_string(),
            })?;

        transaction.set_original_state(serde_json::to_value(&original_node).map_err(|e| {
            PolicyError::ValidationError {
                message: e.to_string(),
            }
        })?);

        let mut results = Vec::new();

        for rule in rules {
            let result = Self::execute_rule(rule, exec_ctx).await?;

            // If the action was executed successfully, add rollback data to transaction
            if let Some(action_result) = &result.action_result {
                if let Some(rollback_data) = &action_result.rollback_data {
                    transaction.add_rollback(rollback_data.clone());
                }
            }

            results.push(result);
        }

        Ok((results, transaction))
    }
}
