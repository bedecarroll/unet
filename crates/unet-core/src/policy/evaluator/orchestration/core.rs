//! Core orchestration data structures
//!
//! Contains the primary data structures used for policy orchestration
//! including rules, batches, and cache entries.

use super::super::context::EvaluationContext;
use super::super::results::{AggregatedResult, PolicyPriority};
use crate::policy::ast::PolicyRule;
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Policy rule with orchestration metadata for execution ordering
#[derive(Debug, Clone)]
pub struct OrchestrationRule {
    /// The policy rule to execute
    pub rule: PolicyRule,
    /// Priority level for execution ordering
    pub priority: PolicyPriority,
    /// Numeric order within priority level
    pub order: u32,
    /// Tags for categorization and filtering
    pub tags: Vec<String>,
}

impl OrchestrationRule {
    /// Create a new orchestration rule with default priority and order
    #[must_use]
    pub const fn new(rule: PolicyRule) -> Self {
        Self {
            rule,
            priority: PolicyPriority::Medium,
            order: 0,
            tags: Vec::new(),
        }
    }

    /// Create a new orchestration rule with specified priority
    #[must_use]
    pub const fn with_priority(rule: PolicyRule, priority: PolicyPriority) -> Self {
        Self {
            rule,
            priority,
            order: 0,
            tags: Vec::new(),
        }
    }

    /// Create a new orchestration rule with priority and order
    #[must_use]
    pub const fn with_priority_and_order(
        rule: PolicyRule,
        priority: PolicyPriority,
        order: u32,
    ) -> Self {
        Self {
            rule,
            priority,
            order,
            tags: Vec::new(),
        }
    }

    /// Add tags to the orchestration rule
    #[must_use]
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Add a single tag to the orchestration rule
    #[must_use]
    pub fn with_tag(mut self, tag: String) -> Self {
        self.tags.push(tag);
        self
    }

    /// Check if this rule has a specific tag
    #[must_use]
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }
}

/// Batch of policy evaluations for a node
#[derive(Debug, Clone)]
pub struct EvaluationBatch {
    /// Unique identifier of the node being evaluated
    pub node_id: Uuid,
    /// Evaluation context with node data and state
    pub context: EvaluationContext,
    /// List of policy rules to evaluate in order
    pub rules: Vec<OrchestrationRule>,
    /// Unique identifier for this evaluation batch
    pub batch_id: String,
    /// Timestamp when the batch was created
    pub created_at: Instant,
}

impl EvaluationBatch {
    /// Create a new evaluation batch
    #[must_use]
    pub fn new(
        node_id: Uuid,
        context: EvaluationContext,
        rules: Vec<OrchestrationRule>,
        batch_id: String,
    ) -> Self {
        Self {
            node_id,
            context,
            rules,
            batch_id,
            created_at: Instant::now(),
        }
    }

    /// Get the age of this batch
    #[must_use]
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// Check if this batch has timed out
    #[must_use]
    pub fn is_timed_out(&self, timeout: Duration) -> bool {
        self.age() > timeout
    }

    /// Filter rules by tag
    #[must_use]
    pub fn rules_with_tag(&self, tag: &str) -> Vec<&OrchestrationRule> {
        self.rules.iter().filter(|rule| rule.has_tag(tag)).collect()
    }

    /// Get rules by priority level
    #[must_use]
    pub fn rules_by_priority(&self, priority: PolicyPriority) -> Vec<&OrchestrationRule> {
        self.rules
            .iter()
            .filter(|rule| rule.priority == priority)
            .collect()
    }
}

/// Cache entry for policy evaluation results
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// The cached result
    pub result: AggregatedResult,
    /// When this cache entry expires
    pub expires_at: Instant,
}

impl CacheEntry {
    /// Create a new cache entry with TTL
    #[must_use]
    pub fn new(result: AggregatedResult, ttl: Duration) -> Self {
        Self {
            result,
            expires_at: Instant::now() + ttl,
        }
    }

    /// Check if this cache entry has expired
    #[must_use]
    pub fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }
}
