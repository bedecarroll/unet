//! Result types and aggregation for policy evaluation
//!
//! Contains result types for policy evaluation and aggregation functions
//! for summarizing evaluation results across multiple rules.

use super::context::{ActionResult, EvaluationResult, PolicyExecutionResult};
use serde_json::Value as JsonValue;
use std::time::Duration;
use uuid::Uuid;

/// Priority level for policy rules
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
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

impl Default for PolicyPriority {
    fn default() -> Self {
        Self::Medium
    }
}

impl std::fmt::Display for PolicyPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

/// Aggregated results for a node's policy evaluation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

impl AggregatedResult {
    /// Create a new aggregated result from a list of policy execution results
    #[must_use]
    pub fn from_results(
        node_id: Uuid,
        batch_id: String,
        results: Vec<PolicyExecutionResult>,
        execution_duration: Duration,
    ) -> Self {
        let total_rules = results.len();
        let mut satisfied_rules = 0;
        let mut failed_rules = 0;
        let mut error_rules = 0;
        let mut compliance_failures = 0;

        for result in &results {
            match &result.evaluation_result {
                EvaluationResult::Satisfied { .. } => {
                    satisfied_rules += 1;
                    // Check if action execution had compliance failures
                    if let Some(action_result) = &result.action_result {
                        match &action_result.result {
                            ActionResult::ComplianceFailure { .. } => {
                                compliance_failures += 1;
                                failed_rules += 1;
                            }
                            ActionResult::Error { .. } => {
                                error_rules += 1;
                            }
                            ActionResult::Success { .. } => {
                                // Success, no additional counting needed
                            }
                        }
                    }
                }
                EvaluationResult::NotSatisfied => {
                    // Rule condition not met, but this is not an error
                }
                EvaluationResult::Error { .. } => {
                    error_rules += 1;
                }
            }
        }

        let summary = Self::generate_summary(
            total_rules,
            satisfied_rules,
            failed_rules,
            error_rules,
            compliance_failures,
        );

        Self {
            node_id,
            batch_id,
            total_rules,
            satisfied_rules,
            failed_rules,
            error_rules,
            compliance_failures,
            execution_duration,
            results,
            summary,
        }
    }

    /// Generate a human-readable summary of the evaluation results
    fn generate_summary(
        total: usize,
        satisfied: usize,
        failed: usize,
        errors: usize,
        compliance_failures: usize,
    ) -> String {
        if total == 0 {
            return "No policies evaluated".to_string();
        }

        let mut parts = Vec::new();
        parts.push(format!("{total} total policies"));

        if satisfied > 0 {
            parts.push(format!("{satisfied} satisfied"));
        }

        if failed > 0 {
            parts.push(format!("{failed} failed"));
        }

        if errors > 0 {
            parts.push(format!("{errors} errors"));
        }

        if compliance_failures > 0 {
            parts.push(format!("{compliance_failures} compliance violations"));
        }

        let not_satisfied = total - satisfied - errors;
        if not_satisfied > 0 {
            parts.push(format!("{not_satisfied} not applicable"));
        }

        parts.join(", ")
    }

    /// Check if all evaluations were successful (no errors or failures)
    #[must_use]
    pub const fn is_fully_successful(&self) -> bool {
        self.error_rules == 0 && self.failed_rules == 0
    }

    /// Check if there were any compliance failures
    #[must_use]
    pub const fn has_compliance_failures(&self) -> bool {
        self.compliance_failures > 0
    }

    /// Get the success rate as a percentage
    #[must_use]
    pub fn success_rate(&self) -> f64 {
        if self.total_rules == 0 {
            return 100.0;
        }

        let successful = self.total_rules - self.failed_rules - self.error_rules;
        // Casting is acceptable here for percentage calculation
        #[allow(clippy::cast_precision_loss)]
        {
            (successful as f64 / self.total_rules as f64) * 100.0
        }
    }

    /// Get all error messages from failed executions
    #[must_use]
    pub fn get_error_messages(&self) -> Vec<String> {
        let mut errors = Vec::new();

        for result in &self.results {
            if let Some(error_msg) = result.get_error_message() {
                errors.push(error_msg.to_string());
            }
        }

        errors
    }

    /// Get all compliance failure details
    #[must_use]
    pub fn get_compliance_failures(&self) -> Vec<ComplianceFailureDetail> {
        let mut failures = Vec::new();

        for result in &self.results {
            if let Some(action_result) = &result.action_result {
                if let ActionResult::ComplianceFailure {
                    field,
                    expected,
                    actual,
                } = &action_result.result
                {
                    failures.push(ComplianceFailureDetail {
                        rule_name: result
                            .rule
                            .id
                            .clone()
                            .unwrap_or_else(|| "unnamed".to_string()),
                        field: field.clone(),
                        expected: expected.clone(),
                        actual: actual.clone(),
                    });
                }
            }
        }

        failures
    }

    /// Filter results by evaluation result type
    #[must_use]
    pub fn filter_by_result_type(&self, satisfied_only: bool) -> Vec<&PolicyExecutionResult> {
        self.results
            .iter()
            .filter(|result| {
                if satisfied_only {
                    matches!(result.evaluation_result, EvaluationResult::Satisfied { .. })
                } else {
                    !matches!(result.evaluation_result, EvaluationResult::Satisfied { .. })
                }
            })
            .collect()
    }
}

/// Details of a compliance failure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComplianceFailureDetail {
    /// Name of the rule that failed
    pub rule_name: String,
    /// Field that failed compliance
    pub field: String,
    /// Expected value
    pub expected: JsonValue,
    /// Actual value found
    pub actual: JsonValue,
}

/// Batch evaluation statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BatchStatistics {
    /// Total number of nodes evaluated
    pub total_nodes: usize,
    /// Total number of rules evaluated across all nodes
    pub total_rules_evaluated: usize,
    /// Total execution time for the batch
    pub total_execution_time: Duration,
    /// Average execution time per node
    pub avg_execution_time_per_node: Duration,
    /// Number of nodes with compliance failures
    pub nodes_with_failures: usize,
    /// Number of nodes with errors
    pub nodes_with_errors: usize,
    /// Overall success rate across all evaluations
    pub overall_success_rate: f64,
}

impl Default for BatchStatistics {
    fn default() -> Self {
        Self {
            total_nodes: 0,
            total_rules_evaluated: 0,
            total_execution_time: Duration::ZERO,
            avg_execution_time_per_node: Duration::ZERO,
            nodes_with_failures: 0,
            nodes_with_errors: 0,
            overall_success_rate: 100.0,
        }
    }
}

#[cfg(test)]
mod results_comprehensive_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregated_result_creation() {
        let node_id = Uuid::new_v4();
        let batch_id = "test-batch".to_string();
        let execution_duration = Duration::from_millis(100);

        // Create some test results
        let results = vec![];

        let aggregated =
            AggregatedResult::from_results(node_id, batch_id.clone(), results, execution_duration);

        assert_eq!(aggregated.node_id, node_id);
        assert_eq!(aggregated.batch_id, batch_id);
        assert_eq!(aggregated.total_rules, 0);
        assert_eq!(aggregated.execution_duration, execution_duration);
        assert!(aggregated.is_fully_successful());
        assert!(!aggregated.has_compliance_failures());
        assert!((aggregated.success_rate() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_policy_priority_ordering() {
        assert!(PolicyPriority::Critical > PolicyPriority::High);
        assert!(PolicyPriority::High > PolicyPriority::Medium);
        assert!(PolicyPriority::Medium > PolicyPriority::Low);
    }

    #[test]
    fn test_policy_priority_display() {
        assert_eq!(PolicyPriority::Low.to_string(), "low");
        assert_eq!(PolicyPriority::Medium.to_string(), "medium");
        assert_eq!(PolicyPriority::High.to_string(), "high");
        assert_eq!(PolicyPriority::Critical.to_string(), "critical");
    }

    #[test]
    fn test_batch_statistics_default() {
        let stats = BatchStatistics::default();
        assert_eq!(stats.total_nodes, 0);
        assert_eq!(stats.total_rules_evaluated, 0);
        assert!((stats.overall_success_rate - 100.0).abs() < f64::EPSILON);
    }
}
