//! Policy orchestration and batch processing
//!
//! Contains the orchestration engine for managing complex policy evaluation
//! workflows including batching, caching, and scheduling.

pub mod config;
pub mod core;
pub mod orchestrator;

#[cfg(test)]
mod orchestrator_tests;

// Re-export commonly used types
pub use config::OrchestrationConfig;
pub use core::{EvaluationBatch, OrchestrationRule};
pub use orchestrator::PolicyOrchestrator;

// Re-export tests if available
#[cfg(test)]
mod tests {
    use super::core::CacheEntry;
    use super::*;
    use crate::policy::ast::{Action, Condition, FieldRef, PolicyRule, Value};
    use crate::policy::evaluator::results::PolicyPriority;
    use std::time::Duration;

    #[test]
    fn test_orchestration_rule_creation() {
        let rule = PolicyRule {
            id: Some("test-rule".to_string()),
            condition: Condition::True,
            action: Action::Assert {
                field: FieldRef {
                    path: vec!["test".to_string()],
                },
                expected: Value::String("test".to_string()),
            },
        };

        let orchestration_rule = OrchestrationRule::new(rule.clone());
        assert_eq!(orchestration_rule.rule, rule);
        assert_eq!(orchestration_rule.priority, PolicyPriority::Medium);
        assert_eq!(orchestration_rule.order, 0);
        assert!(orchestration_rule.tags.is_empty());

        let orchestration_rule_with_priority =
            OrchestrationRule::with_priority(rule.clone(), PolicyPriority::High);
        assert_eq!(
            orchestration_rule_with_priority.priority,
            PolicyPriority::High
        );

        let orchestration_rule_with_tag =
            OrchestrationRule::new(rule).with_tag("test-tag".to_string());
        assert!(orchestration_rule_with_tag.has_tag("test-tag"));
        assert!(!orchestration_rule_with_tag.has_tag("other-tag"));
    }

    #[test]
    fn test_cache_entry() {
        use crate::policy::evaluator::results::AggregatedResult;
        use std::time::Duration;
        use uuid::Uuid;

        let result = AggregatedResult::from_results(
            Uuid::new_v4(),
            "test-batch".to_string(),
            vec![],
            Duration::from_millis(100),
        );

        let cache_entry = CacheEntry::new(result, Duration::from_secs(60));
        assert!(!cache_entry.is_expired());
    }

    #[test]
    fn test_orchestration_config_default() {
        let config = OrchestrationConfig::default();
        assert_eq!(config.max_concurrent, 10);
        assert_eq!(config.cache_ttl, Duration::from_secs(300));
        assert_eq!(config.batch_timeout, Duration::from_secs(30));
        assert!(config.enable_caching);
    }

    #[test]
    fn test_policy_orchestrator_creation() {
        let config = OrchestrationConfig::default();
        let orchestrator = PolicyOrchestrator::new(config);
        assert_eq!(orchestrator.cache_size(), 0);
        assert_eq!(orchestrator.pending_batch_count(), 0);
    }
}
