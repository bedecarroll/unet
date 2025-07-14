//! Request and response types for policy API endpoints

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use unet_core::policy::{PolicyExecutionResult, PolicyRule};
use uuid::Uuid;

/// Request to evaluate policies against a node
#[derive(Debug, Deserialize)]
pub struct PolicyEvaluationRequest {
    /// Optional node IDs to evaluate (if empty, evaluates all nodes)
    pub node_ids: Option<Vec<Uuid>>,
    /// Optional policy rules to use (if empty, loads from configured source)
    pub policies: Option<Vec<PolicyRule>>,
    /// Whether to store results in the database
    pub store_results: Option<bool>,
}

/// Response for policy evaluation
#[derive(Debug, Serialize)]
pub struct PolicyEvaluationResponse {
    /// Results by node ID
    pub results: HashMap<Uuid, Vec<PolicyExecutionResult>>,
    /// Number of nodes evaluated
    pub nodes_evaluated: usize,
    /// Number of policies evaluated per node
    pub policies_evaluated: usize,
    /// Total evaluation time in milliseconds
    pub evaluation_time_ms: u64,
    /// Summary of results
    pub summary: PolicyEvaluationSummary,
}

/// Summary of policy evaluation results
#[derive(Debug, Serialize)]
pub struct PolicyEvaluationSummary {
    /// Total number of policy rules executed
    pub total_rules: usize,
    /// Number of satisfied rules
    pub satisfied_rules: usize,
    /// Number of unsatisfied rules
    pub unsatisfied_rules: usize,
    /// Number of rules that failed with errors
    pub error_rules: usize,
    /// Number of compliance failures
    pub compliance_failures: usize,
}

/// Query parameters for policy results
#[derive(Debug, Deserialize)]
pub struct PolicyResultsQuery {
    /// Filter by node ID
    pub node_id: Option<Uuid>,
    /// Limit number of results
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
}

/// Response for policy results
#[derive(Debug, Serialize)]
pub struct PolicyResultsResponse {
    /// Policy execution results
    pub results: Vec<PolicyExecutionResult>,
    /// Total number of results available
    pub total_count: usize,
    /// Number of results returned
    pub returned_count: usize,
}
