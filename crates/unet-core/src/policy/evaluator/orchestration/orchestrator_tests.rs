//! Tests for policy orchestration components
//!
//! Contains comprehensive tests for the policy orchestrator including
//! batch scheduling, execution, caching, and priority handling.

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::datastore::sqlite::SqliteStore;
    use crate::policy::ast::*;
    use crate::policy::evaluator::context::EvaluationContext;
    use crate::policy::evaluator::results::PolicyPriority;
    use migration::Migrator;
    use sea_orm_migration::MigratorTrait;
    use serde_json::json;
    use std::time::Duration;
    use uuid::Uuid;

    /// Set up a test datastore for testing
    async fn setup_test_datastore() -> SqliteStore {
        let store = SqliteStore::new("sqlite::memory:").await.unwrap();
        Migrator::up(store.connection(), None).await.unwrap();
        store
    }

    /// Create a simple test policy rule for testing
    fn create_test_rule() -> PolicyRule {
        PolicyRule {
            id: Some("test_rule_1".to_string()),
            condition: Condition::Comparison {
                field: FieldRef {
                    path: vec!["node".to_string(), "vendor".to_string()],
                },
                operator: ComparisonOperator::Equal,
                value: Value::String("cisco".to_string()),
            },
            action: Action::Assert {
                field: FieldRef {
                    path: vec!["node".to_string(), "vendor".to_string()],
                },
                expected: Value::String("cisco".to_string()),
            },
        }
    }

    /// Create test evaluation context
    fn create_test_context() -> EvaluationContext {
        EvaluationContext::new(json!({
            "node": {
                "vendor": "cisco",
                "model": "2960"
            }
        }))
    }

    #[tokio::test]
    async fn test_orchestrator_new_creates_default_instance() {
        let config = OrchestrationConfig::default();
        let orchestrator = PolicyOrchestrator::new(config.clone());

        assert_eq!(orchestrator.config().max_concurrent, config.max_concurrent);
        assert_eq!(orchestrator.cache_size(), 0);
        assert_eq!(orchestrator.pending_batch_count(), 0);
    }

    #[tokio::test]
    async fn test_orchestrator_default_creates_valid_instance() {
        let orchestrator = PolicyOrchestrator::default();

        assert_eq!(orchestrator.config().max_concurrent, 10);
        assert!(orchestrator.config().enable_caching);
        assert_eq!(orchestrator.cache_size(), 0);
        assert_eq!(orchestrator.pending_batch_count(), 0);
    }

    #[tokio::test]
    async fn test_schedule_evaluation_creates_batch_with_correct_id() {
        let mut orchestrator = PolicyOrchestrator::default();
        let node_id = Uuid::new_v4();
        let context = create_test_context();
        let rule = OrchestrationRule::new(create_test_rule());
        let rules = vec![rule];

        let batch_id = orchestrator.schedule_evaluation(node_id, context, rules);

        assert!(batch_id.starts_with("batch_"));
        assert_eq!(orchestrator.pending_batch_count(), 1);
        assert!(orchestrator.has_pending_batch(&node_id));
    }

    #[tokio::test]
    async fn test_schedule_evaluation_stores_batch_correctly() {
        let mut orchestrator = PolicyOrchestrator::default();
        let node_id = Uuid::new_v4();
        let context = create_test_context();
        let rule = OrchestrationRule::new(create_test_rule());
        let rules = vec![rule.clone()];

        let _ = orchestrator.schedule_evaluation(node_id, context, rules);

        let batch = orchestrator.get_pending_batch(&node_id).unwrap();
        assert_eq!(batch.node_id, node_id);
        assert_eq!(batch.rules.len(), 1);
        assert_eq!(batch.rules[0].rule.id, rule.rule.id);
    }

    #[tokio::test]
    async fn test_execute_pending_batches_processes_all_batches() {
        let mut orchestrator = PolicyOrchestrator::default();
        let datastore = setup_test_datastore().await;

        let node_id1 = Uuid::new_v4();
        let node_id2 = Uuid::new_v4();
        let context = create_test_context();
        let rule = OrchestrationRule::new(create_test_rule());

        let _ = orchestrator.schedule_evaluation(node_id1, context.clone(), vec![rule.clone()]);
        let _ = orchestrator.schedule_evaluation(node_id2, context, vec![rule]);

        let results = orchestrator
            .execute_pending_batches(&datastore)
            .await
            .unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(orchestrator.pending_batch_count(), 0);
    }

    #[tokio::test]
    async fn test_execute_batch_with_single_rule() {
        let orchestrator = PolicyOrchestrator::default();
        let datastore = setup_test_datastore().await;
        let node_id = Uuid::new_v4();
        let context = create_test_context();
        let rule = OrchestrationRule::new(create_test_rule());
        let batch = EvaluationBatch::new(node_id, context, vec![rule], "test_batch".to_string());

        let result = orchestrator.execute_batch(&batch, &datastore).await;

        assert!(result.is_ok());
        let aggregated = result.unwrap();
        assert_eq!(aggregated.node_id, node_id);
        assert_eq!(aggregated.batch_id, "test_batch");
    }

    #[tokio::test]
    async fn test_evaluate_node_policies_single_workflow() {
        let mut orchestrator = PolicyOrchestrator::default();
        let datastore = setup_test_datastore().await;
        let node_id = Uuid::new_v4();
        let context = create_test_context();
        let rule = OrchestrationRule::new(create_test_rule());

        let result = orchestrator
            .evaluate_node_policies(node_id, context, vec![rule], &datastore)
            .await;

        assert!(result.is_ok());
        let aggregated = result.unwrap();
        assert_eq!(aggregated.node_id, node_id);
        assert_eq!(orchestrator.pending_batch_count(), 0);
    }

    #[tokio::test]
    async fn test_orchestrator_sorts_rules_by_priority_during_scheduling() {
        let mut orchestrator = PolicyOrchestrator::default();
        let node_id = Uuid::new_v4();
        let context = create_test_context();

        let high_rule = OrchestrationRule::with_priority(create_test_rule(), PolicyPriority::High);
        let low_rule = OrchestrationRule::with_priority(create_test_rule(), PolicyPriority::Low);
        let medium_rule =
            OrchestrationRule::with_priority(create_test_rule(), PolicyPriority::Medium);

        let rules = vec![low_rule, high_rule, medium_rule];
        let _ = orchestrator.schedule_evaluation(node_id, context, rules);

        let batch = orchestrator.get_pending_batch(&node_id).unwrap();
        assert_eq!(batch.rules[0].priority, PolicyPriority::High);
        assert_eq!(batch.rules[1].priority, PolicyPriority::Medium);
        assert_eq!(batch.rules[2].priority, PolicyPriority::Low);
    }

    #[test]
    fn test_cache_stats_returns_correct_counts() {
        let orchestrator = PolicyOrchestrator::default();
        let stats = orchestrator.cache_stats();

        assert_eq!(stats.get("total_entries"), Some(&0));
        assert_eq!(stats.get("pending_batches"), Some(&0));
        assert_eq!(stats.get("expired_entries"), Some(&0));
    }

    #[test]
    fn test_clear_cache_empties_cache() {
        let mut orchestrator = PolicyOrchestrator::default();
        orchestrator.clear_cache();

        assert_eq!(orchestrator.cache_size(), 0);
    }

    #[test]
    fn test_clear_pending_batches_empties_batches() {
        let mut orchestrator = PolicyOrchestrator::default();
        let node_id = Uuid::new_v4();
        let context = create_test_context();
        let rule = OrchestrationRule::new(create_test_rule());

        let _ = orchestrator.schedule_evaluation(node_id, context, vec![rule]);
        assert_eq!(orchestrator.pending_batch_count(), 1);

        orchestrator.clear_pending_batches();
        assert_eq!(orchestrator.pending_batch_count(), 0);
    }

    #[test]
    fn test_remove_pending_batch_removes_specific_batch() {
        let mut orchestrator = PolicyOrchestrator::default();
        let node_id = Uuid::new_v4();
        let context = create_test_context();
        let rule = OrchestrationRule::new(create_test_rule());

        let _ = orchestrator.schedule_evaluation(node_id, context, vec![rule]);
        assert!(orchestrator.has_pending_batch(&node_id));

        let removed = orchestrator.remove_pending_batch(&node_id);
        assert!(removed.is_some());
        assert!(!orchestrator.has_pending_batch(&node_id));
    }

    #[test]
    fn test_update_config_changes_configuration() {
        let mut orchestrator = PolicyOrchestrator::default();
        let new_config = OrchestrationConfig {
            max_concurrent: 20,
            cache_ttl: Duration::from_secs(600),
            batch_timeout: Duration::from_secs(60),
            enable_caching: false,
        };

        orchestrator.update_config(new_config);

        assert_eq!(orchestrator.config().max_concurrent, 20);
        assert_eq!(orchestrator.config().cache_ttl, Duration::from_secs(600));
        assert!(!orchestrator.config().enable_caching);
    }

    #[tokio::test]
    async fn test_caching_disabled_does_not_cache_results() {
        let config = OrchestrationConfig {
            enable_caching: false,
            ..Default::default()
        };
        let mut orchestrator = PolicyOrchestrator::new(config);
        let datastore = setup_test_datastore().await;
        let node_id = Uuid::new_v4();
        let context = create_test_context();
        let rule = OrchestrationRule::new(create_test_rule());

        let _ = orchestrator.schedule_evaluation(node_id, context, vec![rule]);
        let _results = orchestrator
            .execute_pending_batches(&datastore)
            .await
            .unwrap();

        assert_eq!(orchestrator.cache_size(), 0);
    }

    #[tokio::test]
    async fn test_caching_enabled_caches_results() {
        let config = OrchestrationConfig {
            enable_caching: true,
            ..Default::default()
        };
        let mut orchestrator = PolicyOrchestrator::new(config);
        let datastore = setup_test_datastore().await;
        let node_id = Uuid::new_v4();
        let context = create_test_context();
        let rule = OrchestrationRule::new(create_test_rule());

        let _ = orchestrator.schedule_evaluation(node_id, context, vec![rule]);
        let _results = orchestrator
            .execute_pending_batches(&datastore)
            .await
            .unwrap();

        assert_eq!(orchestrator.cache_size(), 1);
    }
}
