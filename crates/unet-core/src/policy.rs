//! Policy Engine for μNet
//!
//! This module provides a Domain Specific Language (DSL) for defining network
//! configuration policies. Policies use a WHEN/THEN structure to evaluate
//! conditions against network nodes and execute appropriate actions.
//!
//! # Example
//!
//! ```rust
//! use unet_core::policy::{PolicyParser, PolicyEvaluator, EvaluationContext};
//! use serde_json::json;
//!
//! // Parse a policy rule
//! let rule = PolicyParser::parse_rule(
//!     r#"WHEN node.vendor == "cisco" THEN ASSERT node.version IS "15.1""#
//! ).unwrap();
//!
//! // Create evaluation context with node data
//! let context = EvaluationContext::new(json!({
//!     "node": {
//!         "vendor": "cisco",
//!         "version": "15.1"
//!     }
//! }));
//!
//! // Evaluate the rule
//! let result = PolicyEvaluator::evaluate_rule(&rule, &context).unwrap();
//! ```

mod ast;
mod evaluator;
mod grammar;
mod loader;
mod parser;

#[cfg(test)]
mod tests;

pub use ast::{Action, ComparisonOperator, Condition, FieldRef, PolicyRule, Value};
pub use evaluator::{
    ActionExecutionResult, ActionResult, AggregatedResult, EvaluationBatch, EvaluationContext,
    EvaluationResult, OrchestrationConfig, OrchestrationRule, PolicyEvaluator,
    PolicyExecutionResult, PolicyOrchestrator, PolicyPriority, PolicyTransaction, RollbackData,
    RollbackResult,
};
pub use loader::{
    CacheStats, LoadResult, PolicyFile, PolicyLoader, ValidationError, ValidationResult,
};
pub use parser::{ParseError, PolicyParser};

/// Policy engine errors
#[derive(Debug, thiserror::Error)]
pub enum PolicyError {
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),

    #[error("Evaluation error: {message}")]
    Evaluation { message: String },

    #[error("Field not found: {field}")]
    FieldNotFound { field: String },

    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },

    #[error("Invalid regex: {pattern}")]
    InvalidRegex { pattern: String },

    #[error("Validation error: {message}")]
    ValidationError { message: String },

    #[error("DataStore error: {message}")]
    DataStoreError { message: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Policy engine result type
pub type PolicyResult<T> = Result<T, PolicyError>;
