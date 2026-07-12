//! Comprehensive testing for the policy engine
//!
//! This module contains test suites that cover:
//! - Performance tests for large policy sets
//! - Error handling and edge case testing
//! - Complete grammar construct coverage
//! - Policy evaluation comprehensive scenarios

use super::*;
use crate::policy::{
    EvaluationContext, EvaluationResult, PolicyError, PolicyEvaluator, PolicyParser,
};
use serde_json::json;

// Re-export test modules
pub mod error_handling;
pub mod grammar;

// Common test utilities and helpers can be added here in the future
