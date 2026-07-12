//! Caching functionality tests for orchestrator

use super::super::*;
use std::time::Duration;
use uuid::Uuid;

use super::orchestrator_test_helpers::{
    create_test_context, create_test_rule, setup_test_datastore,
};

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

#[tokio::test]
async fn test_execute_pending_batches_with_cache_hit() {
    let config = OrchestrationConfig {
        enable_caching: true,
        cache_ttl: Duration::from_secs(300),
        ..Default::default()
    };
    let mut orchestrator = PolicyOrchestrator::new(config);
    let datastore = setup_test_datastore().await;
    let node_id = Uuid::new_v4();
    let context = create_test_context();
    let rule = OrchestrationRule::new(create_test_rule());

    // First execution - should cache result
    let _ = orchestrator.schedule_evaluation(node_id, context.clone(), vec![rule.clone()]);
    let first_results = orchestrator
        .execute_pending_batches(&datastore)
        .await
        .unwrap();

    assert_eq!(orchestrator.cache_size(), 1);
    assert_eq!(first_results.len(), 1);

    // Second execution with same parameters - should hit cache
    let _ = orchestrator.schedule_evaluation(node_id, context, vec![rule]);
    let second_results = orchestrator
        .execute_pending_batches(&datastore)
        .await
        .unwrap();

    assert_eq!(orchestrator.cache_size(), 1);
    assert_eq!(second_results.len(), 1);

    // Results should be equivalent
    assert_eq!(first_results[0].node_id, second_results[0].node_id);
}

#[tokio::test]
async fn test_cache_expiry_through_execution() {
    let config = OrchestrationConfig {
        enable_caching: true,
        cache_ttl: Duration::from_millis(50), // Very short TTL
        ..Default::default()
    };
    let mut orchestrator = PolicyOrchestrator::new(config);
    let datastore = setup_test_datastore().await;
    let node_id = Uuid::new_v4();
    let context = create_test_context();
    let rule = OrchestrationRule::new(create_test_rule());

    // First execution - should cache result
    let _ = orchestrator.schedule_evaluation(node_id, context.clone(), vec![rule.clone()]);
    let _first_results = orchestrator
        .execute_pending_batches(&datastore)
        .await
        .unwrap();

    assert_eq!(orchestrator.cache_size(), 1);

    // Wait for cache to expire
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Second execution - cache should be expired and cleaned up
    let _ = orchestrator.schedule_evaluation(node_id, context, vec![rule]);
    let _second_results = orchestrator
        .execute_pending_batches(&datastore)
        .await
        .unwrap();

    // Cache should be cleaned up and have new entry
    assert_eq!(orchestrator.cache_size(), 1);
}

#[tokio::test]
async fn test_cache_cleanup_basic() {
    let config = OrchestrationConfig {
        enable_caching: true,
        cache_ttl: Duration::from_millis(100),
        ..Default::default()
    };
    let mut orchestrator = PolicyOrchestrator::new(config);
    let datastore = setup_test_datastore().await;

    // Create multiple evaluations
    for i in 0..3 {
        let node_id = Uuid::new_v4();
        let context = create_test_context();
        let mut rule = OrchestrationRule::new(create_test_rule());
        rule.rule.id = Some(format!("rule_{i}"));

        let _ = orchestrator.schedule_evaluation(node_id, context, vec![rule]);
        let _results = orchestrator
            .execute_pending_batches(&datastore)
            .await
            .unwrap();

        // Small delay between executions
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    // Should have cached multiple results
    assert!(orchestrator.cache_size() > 0);
}

// Tests for basic orchestration features

#[test]
fn test_orchestration_rule_with_tags() {
    let rule = create_test_rule();
    let tags = vec!["test".to_string(), "validation".to_string()];
    let orchestration_rule = OrchestrationRule::new(rule.clone()).with_tags(tags.clone());

    assert_eq!(orchestration_rule.rule.id, rule.id);
    assert_eq!(orchestration_rule.tags, tags);
}
