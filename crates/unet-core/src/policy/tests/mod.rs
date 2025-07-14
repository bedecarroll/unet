//! Comprehensive testing for the policy engine
//!
//! This module contains test suites that cover:
//! - Performance tests for large policy sets
//! - Error handling and edge case testing
//! - Complete grammar construct coverage
//! - Policy evaluation comprehensive scenarios

use super::*;
use crate::policy::{
    EvaluationContext, EvaluationResult, OrchestrationRule, PolicyError, PolicyEvaluator,
    PolicyParser, PolicyPriority,
};
use serde_json::json;
use std::time::{Duration, Instant};

// Re-export test modules
pub mod error_handling;
pub mod grammar;
pub mod performance;

// Common test utilities and helpers can be added here in the future
