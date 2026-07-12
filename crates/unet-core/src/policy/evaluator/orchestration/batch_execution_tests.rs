//! Batch execution and timeout tests for orchestrator

use super::super::*;
use std::time::Duration;
use uuid::Uuid;

use super::orchestrator_test_helpers::{
    create_test_context, create_test_rule, setup_test_datastore,
};

#[tokio::test]
async fn test_batch_timeout_behavior() {
    let config = OrchestrationConfig {
        batch_timeout: Duration::from_millis(50), // Very short timeout
        ..Default::default()
    };
    let mut orchestrator = PolicyOrchestrator::new(config);
    let datastore = setup_test_datastore().await;
    let node_id = Uuid::new_v4();
    let context = create_test_context();
    let rule = OrchestrationRule::new(create_test_rule());

    // Schedule evaluation
    let _ = orchestrator.schedule_evaluation(node_id, context, vec![rule]);
    assert_eq!(orchestrator.pending_batch_count(), 1);

    // Wait for timeout
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Execute batches - timed out batches should still be processed
    let results = orchestrator
        .execute_pending_batches(&datastore)
        .await
        .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(orchestrator.pending_batch_count(), 0);
}

#[test]
fn test_evaluation_batch_age() {
    let node_id = Uuid::new_v4();
    let context = create_test_context();
    let rule = OrchestrationRule::new(create_test_rule());
    let batch = EvaluationBatch::new(node_id, context, vec![rule], "test_batch".to_string());

    // Age should be very small for newly created batch
    let age = batch.age();
    assert!(age.as_millis() < 100);
}

#[test]
fn test_evaluation_batch_is_timed_out() {
    let node_id = Uuid::new_v4();
    let context = create_test_context();
    let rule = OrchestrationRule::new(create_test_rule());
    let batch = EvaluationBatch::new(node_id, context, vec![rule], "test_batch".to_string());

    // Should not be timed out with large timeout
    assert!(!batch.is_timed_out(Duration::from_secs(60)));

    // Should be timed out with very small timeout
    assert!(batch.is_timed_out(Duration::from_nanos(1)));
}

#[test]
fn test_evaluation_batch_rules_with_tag() {
    let node_id = Uuid::new_v4();
    let context = create_test_context();

    let rule1 = OrchestrationRule::new(create_test_rule()).with_tag("important".to_string());
    let rule2 = OrchestrationRule::new(create_test_rule()).with_tag("optional".to_string());
    let rule3 = OrchestrationRule::new(create_test_rule()).with_tag("important".to_string());

    let batch = EvaluationBatch::new(
        node_id,
        context,
        vec![rule1, rule2, rule3],
        "test_batch".to_string(),
    );

    let important_rules = batch.rules_with_tag("important");
    assert_eq!(important_rules.len(), 2);

    let optional_rules = batch.rules_with_tag("optional");
    assert_eq!(optional_rules.len(), 1);

    let nonexistent_rules = batch.rules_with_tag("nonexistent");
    assert_eq!(nonexistent_rules.len(), 0);
}
