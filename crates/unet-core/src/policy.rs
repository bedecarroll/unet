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
    /// Parse error from policy parsing
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),

    /// Error during policy evaluation
    #[error("Evaluation error: {message}")]
    Evaluation {
        /// Error message describing the evaluation failure
        message: String,
    },

    #[error("Field not found: {field}")]
    /// Field reference could not be resolved
    FieldNotFound {
        /// Name of the field that was not found
        field: String,
    },

    #[error("Type mismatch: expected {expected}, got {actual}")]
    /// Value type does not match expected type
    TypeMismatch {
        /// Expected type name
        expected: String,
        /// Actual type name
        actual: String,
    },

    #[error("Invalid regex: {pattern}")]
    /// Regular expression pattern is invalid
    InvalidRegex {
        /// The invalid regex pattern
        pattern: String,
    },

    #[error("Validation error: {message}")]
    /// Policy validation failed
    ValidationError {
        /// Validation error message
        message: String,
    },

    #[error("DataStore error: {message}")]
    /// `DataStore` operation failed
    DataStoreError {
        /// `DataStore` error message
        message: String,
    },

    #[error("IO error: {0}")]
    /// Input/output error
    Io(#[from] std::io::Error),
}

/// Policy engine result type
pub type PolicyResult<T> = Result<T, PolicyError>;
