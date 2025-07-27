//! Basic orchestrator functionality tests

use super::super::*;
use uuid::Uuid;

use super::orchestrator_test_helpers::{
    create_test_context, create_test_rule, setup_test_datastore,
};

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
    let rules = vec![rule];

    let _ = orchestrator.schedule_evaluation(node_id, context, rules);

    assert_eq!(orchestrator.pending_batch_count(), 1);
    assert!(orchestrator.has_pending_batch(&node_id));

    let batch = orchestrator.get_pending_batch(&node_id);
    assert!(batch.is_some());
    let batch_ref = batch.unwrap();
    assert_eq!(batch_ref.node_id, node_id);
    assert_eq!(batch_ref.rules.len(), 1);
}

#[tokio::test]
async fn test_execute_pending_batches_processes_all_batches() {
    let mut orchestrator = PolicyOrchestrator::default();
    let datastore = setup_test_datastore().await;

    // Schedule multiple evaluations
    let node_id_1 = Uuid::new_v4();
    let node_id_2 = Uuid::new_v4();
    let context = create_test_context();
    let rule = OrchestrationRule::new(create_test_rule());

    let _ = orchestrator.schedule_evaluation(node_id_1, context.clone(), vec![rule.clone()]);
    let _ = orchestrator.schedule_evaluation(node_id_2, context, vec![rule]);

    assert_eq!(orchestrator.pending_batch_count(), 2);

    let results = orchestrator
        .execute_pending_batches(&datastore)
        .await
        .unwrap();

    assert_eq!(results.len(), 2);
    assert_eq!(orchestrator.pending_batch_count(), 0);
}

#[tokio::test]
async fn test_execute_batch_with_single_rule() {
    let mut orchestrator = PolicyOrchestrator::default();
    let datastore = setup_test_datastore().await;
    let node_id = Uuid::new_v4();
    let context = create_test_context();
    let rule = OrchestrationRule::new(create_test_rule());

    let batch_id = orchestrator.schedule_evaluation(node_id, context, vec![rule]);

    let results = orchestrator
        .execute_pending_batches(&datastore)
        .await
        .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].batch_id, batch_id);
    assert_eq!(results[0].node_id, node_id);
}

#[tokio::test]
async fn test_evaluate_node_policies_single_workflow() {
    let mut orchestrator = PolicyOrchestrator::default();
    let datastore = setup_test_datastore().await;
    let node_id = Uuid::new_v4();
    let context = create_test_context();
    let rule = OrchestrationRule::new(create_test_rule());
    let rules = vec![rule];

    let result = orchestrator
        .evaluate_node_policies(node_id, context, rules, &datastore)
        .await;

    assert!(result.is_ok());
    let evaluation_result = result.unwrap();
    assert_eq!(evaluation_result.node_id, node_id);
    assert!(evaluation_result.total_rules > 0);
}
