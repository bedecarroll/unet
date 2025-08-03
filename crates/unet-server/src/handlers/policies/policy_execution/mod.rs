//! Policy execution and evaluation logic
//!
//! This module provides functionality for evaluating policies on nodes,
//! including single node processing, bulk evaluation, and policy loading.

pub use self::bulk_evaluator::evaluate_nodes_against_policies;
pub use self::policy_loader::load_policies_for_request;

mod bulk_evaluator;
mod node_evaluator;
mod policy_loader;

#[cfg(test)]
mod tests;
