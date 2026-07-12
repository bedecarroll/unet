//! Policy Integration for μNet Core
//!
//! This module provides integration between the policy engine and the data layer,
//! enabling policy evaluation against live network data and storage of policy results.

pub use engine::{DefaultPolicyEvaluationEngine, PolicyEvaluationEngine};
pub use service::PolicyService;

mod engine;
mod service;

#[cfg(test)]
mod service_tests;
