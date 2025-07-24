//! Tests for policy evaluation background task

use crate::background::policy_task::PolicyEvaluationTask;
use crate::background::scheduler::EvaluationStats;
use migration::{Migrator, MigratorTrait};
use std::sync::Arc;
use std::time::Duration;
use unet_core::{
    datastore::{DataStore, sqlite::SqliteStore},
    models::*,
    policy::{Action, ActionExecutionResult, ActionResult, EvaluationResult, PolicyRule},
    policy_integration::PolicyService,
};

async fn setup_test_datastore() -> SqliteStore {
    let store = SqliteStore::new("sqlite::memory:").await.unwrap();
    Migrator::up(store.connection(), None).await.unwrap();
    store
}

fn create_test_node() -> Node {
    let mut node = Node::new(
        "test-node".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );
    node.model = "ASR1000".to_string();
    node
}

#[tokio::test]
async fn test_policy_evaluation_task_run_empty_nodes() {
    let datastore = setup_test_datastore().await;
    let policy_service = PolicyService::with_local_dir("/tmp");

    let task = PolicyEvaluationTask::new(Arc::new(datastore), policy_service, 1);

    let result = task.executor.get_nodes_for_evaluation().await;
    assert!(result.is_ok());
    let nodes = result.unwrap();
    assert!(nodes.is_empty());
}

#[tokio::test]
async fn test_policy_evaluation_task_with_nodes() {
    let datastore = setup_test_datastore().await;
    let node = create_test_node();
    let _stored_node = datastore.create_node(&node).await.unwrap();

    let policy_service = PolicyService::with_local_dir("/tmp");

    let task = PolicyEvaluationTask::new(Arc::new(datastore), policy_service, 1);

    let result = task.executor.get_nodes_for_evaluation().await;
    assert!(result.is_ok());
    let nodes = result.unwrap();
    assert_eq!(nodes.len(), 1);
}

#[tokio::test]
async fn test_load_policies_for_evaluation() {
    let datastore = setup_test_datastore().await;
    let policy_service = PolicyService::with_local_dir("/tmp");

    let mut task = PolicyEvaluationTask::new(Arc::new(datastore), policy_service, 1);

    let result = task.executor.load_policies_for_evaluation();
    assert!(result.is_ok());
    let _policies = result.unwrap();
}

#[tokio::test]
async fn test_evaluate_nodes_empty() {
    let datastore = setup_test_datastore().await;
    let policy_service = PolicyService::with_local_dir("/tmp");

    let task = PolicyEvaluationTask::new(Arc::new(datastore), policy_service, 1);

    let processor = super::node_processor::NodeProcessor::new(
        &task.executor.datastore,
        &task.executor.policy_service,
    );
    let stats = processor.evaluate_nodes(&[]).await;
    assert_eq!(stats.total_results(), 0);
    assert_eq!(stats.successful_evaluations(), 0);
    assert_eq!(stats.failed_evaluations(), 0);
}

#[tokio::test]
async fn test_evaluate_nodes_with_data() {
    let datastore = setup_test_datastore().await;
    let policy_service = PolicyService::with_local_dir("/tmp");
    let nodes = vec![create_test_node()];

    let task = PolicyEvaluationTask::new(Arc::new(datastore), policy_service, 1);

    let processor = super::node_processor::NodeProcessor::new(
        &task.executor.datastore,
        &task.executor.policy_service,
    );
    let stats = processor.evaluate_nodes(&nodes).await;
    assert_eq!(stats.successful_evaluations(), 1);
    assert_eq!(stats.failed_evaluations(), 0);
}

#[tokio::test]
async fn test_run_policy_evaluation_cycle() {
    let datastore = setup_test_datastore().await;
    let policy_service = PolicyService::with_local_dir("/tmp");

    let mut task = PolicyEvaluationTask::new(Arc::new(datastore), policy_service, 1);

    task.executor.run_policy_evaluation_cycle().await;
}

#[tokio::test]
async fn test_evaluate_all_policies_no_nodes() {
    let datastore = setup_test_datastore().await;
    let policy_service = PolicyService::with_local_dir("/tmp");

    let mut task = PolicyEvaluationTask::new(Arc::new(datastore), policy_service, 1);

    let result = task.executor.evaluate_all_policies().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_evaluate_all_policies_with_node() {
    let datastore = setup_test_datastore().await;
    let node = create_test_node();
    let _stored_node = datastore.create_node(&node).await.unwrap();

    let policy_service = PolicyService::with_local_dir("/tmp");

    let mut task = PolicyEvaluationTask::new(Arc::new(datastore), policy_service, 1);

    let result = task.executor.evaluate_all_policies().await;
    assert!(result.is_ok());
}

#[test]
fn test_log_evaluation_results() {
    let nodes = vec![create_test_node()];
    let mut stats = EvaluationStats::new();
    stats.record_success(3);
    stats.record_failure();

    super::result_handler::ResultHandler::log_evaluation_results(
        &nodes,
        &stats,
        Duration::from_millis(100),
    );
}

#[tokio::test]
async fn test_store_evaluation_results() {
    let datastore = setup_test_datastore().await;
    let policy_service = PolicyService::with_local_dir("/tmp");
    let node = create_test_node();

    let rule = PolicyRule {
        id: Some("test-rule".to_string()),
        condition: unet_core::policy::Condition::Comparison {
            field: unet_core::policy::FieldRef {
                path: vec!["vendor".to_string()],
            },
            operator: unet_core::policy::ComparisonOperator::Equal,
            value: unet_core::policy::Value::String("cisco".to_string()),
        },
        action: Action::Assert {
            field: unet_core::policy::FieldRef {
                path: vec!["version".to_string()],
            },
            expected: unet_core::policy::Value::String("15.1".to_string()),
        },
    };

    let results = vec![unet_core::policy::PolicyExecutionResult::new(
        rule,
        EvaluationResult::Satisfied {
            action: Action::Assert {
                field: unet_core::policy::FieldRef {
                    path: vec!["version".to_string()],
                },
                expected: unet_core::policy::Value::String("15.1".to_string()),
            },
        },
        Some(ActionExecutionResult {
            result: ActionResult::Success {
                message: "Test passed".to_string(),
            },
            rollback_data: None,
        }),
    )];

    let task = PolicyEvaluationTask::new(Arc::new(datastore), policy_service.clone(), 1);

    let processor = super::node_processor::NodeProcessor::new(
        &task.executor.datastore,
        &task.executor.policy_service,
    );
    processor
        .store_evaluation_results(&policy_service, &node, &results)
        .await;
}

#[tokio::test]
async fn test_evaluate_all_policies_with_datastore_error() {
    use unet_core::datastore::{DataStoreError, MockDataStore};

    let mut mock_datastore = MockDataStore::new();
    mock_datastore
        .expect_get_nodes_for_policy_evaluation()
        .returning(|| {
            Box::pin(async move {
                Err(DataStoreError::ConnectionError {
                    message: "Database connection failed".to_string(),
                })
            })
        });

    let policy_service = PolicyService::with_local_dir("/tmp");
    let mut task = PolicyEvaluationTask::new(Arc::new(mock_datastore), policy_service, 1);

    let result = task.executor.evaluate_all_policies().await;
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Database connection failed"));
}

#[tokio::test]
async fn test_get_nodes_for_evaluation_error() {
    use unet_core::datastore::{DataStoreError, MockDataStore};

    let mut mock_datastore = MockDataStore::new();
    mock_datastore
        .expect_get_nodes_for_policy_evaluation()
        .returning(|| {
            Box::pin(async move {
                Err(DataStoreError::InternalError {
                    message: "Internal error".to_string(),
                })
            })
        });

    let policy_service = PolicyService::with_local_dir("/tmp");
    let task = PolicyEvaluationTask::new(Arc::new(mock_datastore), policy_service, 1);

    let result = task.executor.get_nodes_for_evaluation().await;
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Internal error"));
}

#[tokio::test]
async fn test_load_policies_for_evaluation_error() {
    let datastore = setup_test_datastore().await;
    // Use a non-existent directory to trigger an error
    let policy_service = PolicyService::with_local_dir("/non/existent/directory");

    let mut task = PolicyEvaluationTask::new(Arc::new(datastore), policy_service, 1);

    let result = task.executor.load_policies_for_evaluation();
    assert!(result.is_err());
}
