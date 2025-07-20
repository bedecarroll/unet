//! Policy evaluation logic and utilities

use crate::error::ServerError;
use std::collections::HashMap;
use tracing::{error, warn};
use unet_core::policy::PolicyExecutionResult;
use unet_core::prelude::{DataStore, Node, PolicyService};
use uuid::Uuid;

use super::types::PolicyEvaluationSummary;

/// Get nodes for policy evaluation based on request
pub async fn get_nodes_for_evaluation(
    datastore: &dyn DataStore,
    node_ids: Option<&Vec<Uuid>>,
) -> Result<Vec<Node>, ServerError> {
    match node_ids {
        Some(node_ids) => get_specific_nodes(datastore, node_ids).await,
        None => get_all_nodes_for_evaluation(datastore).await,
    }
}

async fn get_specific_nodes(
    datastore: &dyn DataStore,
    node_ids: &[Uuid],
) -> Result<Vec<Node>, ServerError> {
    let mut nodes = Vec::new();
    for node_id in node_ids {
        if let Some(node) = fetch_node_by_id(datastore, node_id).await? {
            nodes.push(node);
        } else {
            warn!("Node {} not found for policy evaluation", node_id);
        }
    }
    Ok(nodes)
}

async fn fetch_node_by_id(
    datastore: &dyn DataStore,
    node_id: &Uuid,
) -> Result<Option<Node>, ServerError> {
    datastore.get_node(node_id).await.map_err(|e| {
        error!("Failed to get node {}: {}", node_id, e);
        ServerError::Internal(format!("Failed to get node {node_id}: {e}"))
    })
}

async fn get_all_nodes_for_evaluation(datastore: &dyn DataStore) -> Result<Vec<Node>, ServerError> {
    datastore
        .get_nodes_for_policy_evaluation()
        .await
        .map_err(|e| {
            error!("Failed to get nodes for policy evaluation: {}", e);
            ServerError::Internal(format!("Failed to get nodes for policy evaluation: {e}"))
        })
}

/// Process evaluation results for a single node
pub async fn process_node_evaluation(
    policy_service: &mut PolicyService,
    datastore: &dyn DataStore,
    node: &Node,
    store_results: bool,
    summary: &mut PolicyEvaluationSummary,
    all_results: &mut HashMap<Uuid, Vec<PolicyExecutionResult>>,
) {
    match policy_service.evaluate_node(datastore, node).await {
        Ok(results) => {
            // Update summary statistics
            for result in &results {
                summary.total_rules += 1;

                if result.is_error() {
                    summary.error_rules += 1;
                } else if result.is_satisfied() {
                    summary.satisfied_rules += 1;
                } else {
                    summary.unsatisfied_rules += 1;
                }

                if result.is_compliance_failure() {
                    summary.compliance_failures += 1;
                }
            }

            // Store results if requested
            if store_results {
                if let Err(e) = policy_service
                    .store_results(datastore, &node.id, &results)
                    .await
                {
                    warn!("Failed to store policy results for node {}: {}", node.id, e);
                }
            }

            all_results.insert(node.id, results);
        }
        Err(e) => {
            error!("Failed to evaluate policies for node {}: {}", node.id, e);
            // Create error result for this node
            let error_result = PolicyExecutionResult::new_error_with_id(
                Some("evaluation".to_string()),
                format!("Failed to evaluate policies: {e}"),
            );
            all_results.insert(node.id, vec![error_result]);
            summary.error_rules += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::policies::types::PolicyEvaluationSummary;
    use migration::{Migrator, MigratorTrait};
    use std::collections::HashMap;
    use unet_core::{datastore::sqlite::SqliteStore, models::*, policy_integration::PolicyService};

    async fn setup_test_datastore() -> SqliteStore {
        let store = SqliteStore::new("sqlite::memory:").await.unwrap();
        Migrator::up(store.connection(), None).await.unwrap();
        store
    }

    async fn create_test_node(datastore: &SqliteStore) -> Node {
        let mut node = Node::new(
            "test-node".to_string(),
            "example.com".to_string(),
            Vendor::Cisco,
            DeviceRole::Router,
        );
        node.model = "ASR1000".to_string();
        node.lifecycle = Lifecycle::Live;
        datastore.create_node(&node).await.unwrap()
    }

    #[tokio::test]
    async fn test_get_nodes_for_evaluation_all_nodes() {
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;

        let result = get_nodes_for_evaluation(&datastore, None).await;
        assert!(result.is_ok());
        let nodes = result.unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].id, node.id);
    }

    #[tokio::test]
    async fn test_get_nodes_for_evaluation_specific_nodes() {
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;
        let node_ids = vec![node.id];

        let result = get_nodes_for_evaluation(&datastore, Some(&node_ids)).await;
        assert!(result.is_ok());
        let nodes = result.unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].id, node.id);
    }

    #[tokio::test]
    async fn test_get_specific_nodes_with_nonexistent() {
        let datastore = setup_test_datastore().await;
        let existing_node = create_test_node(&datastore).await;
        let nonexistent_id = Uuid::new_v4();
        let node_ids = vec![existing_node.id, nonexistent_id];

        let result = get_specific_nodes(&datastore, &node_ids).await;
        assert!(result.is_ok());
        let nodes = result.unwrap();
        // Should only return the existing node, warning about the missing one
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].id, existing_node.id);
    }

    #[tokio::test]
    async fn test_fetch_node_by_id_success() {
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;

        let result = fetch_node_by_id(&datastore, &node.id).await;
        assert!(result.is_ok());
        let fetched_node = result.unwrap();
        assert!(fetched_node.is_some());
        assert_eq!(fetched_node.unwrap().id, node.id);
    }

    #[tokio::test]
    async fn test_fetch_node_by_id_not_found() {
        let datastore = setup_test_datastore().await;
        let nonexistent_id = Uuid::new_v4();

        let result = fetch_node_by_id(&datastore, &nonexistent_id).await;
        assert!(result.is_ok());
        let fetched_node = result.unwrap();
        assert!(fetched_node.is_none());
    }

    #[tokio::test]
    async fn test_get_all_nodes_for_evaluation() {
        let datastore = setup_test_datastore().await;
        let _node = create_test_node(&datastore).await;

        let result = get_all_nodes_for_evaluation(&datastore).await;
        assert!(result.is_ok());
        let nodes = result.unwrap();
        assert_eq!(nodes.len(), 1);
    }

    #[tokio::test]
    async fn test_process_node_evaluation_success() {
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;
        let mut policy_service = PolicyService::with_local_dir("/tmp");
        let mut summary = PolicyEvaluationSummary {
            total_rules: 0,
            satisfied_rules: 0,
            unsatisfied_rules: 0,
            error_rules: 0,
            compliance_failures: 0,
        };
        let mut all_results = HashMap::new();

        process_node_evaluation(
            &mut policy_service,
            &datastore,
            &node,
            false,
            &mut summary,
            &mut all_results,
        )
        .await;

        // Should have results for the node
        assert!(all_results.contains_key(&node.id));
        // With no policies loaded, there should be zero rules evaluated
        assert_eq!(summary.total_rules, 0);
    }

    #[tokio::test]
    async fn test_process_node_evaluation_with_store_results() {
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;
        let mut policy_service = PolicyService::with_local_dir("/tmp");
        let mut summary = PolicyEvaluationSummary {
            total_rules: 0,
            satisfied_rules: 0,
            unsatisfied_rules: 0,
            error_rules: 0,
            compliance_failures: 0,
        };
        let mut all_results = HashMap::new();

        process_node_evaluation(
            &mut policy_service,
            &datastore,
            &node,
            true, // Store results
            &mut summary,
            &mut all_results,
        )
        .await;

        // Should have results for the node
        assert!(all_results.contains_key(&node.id));
    }

    #[tokio::test]
    async fn test_get_nodes_for_evaluation_empty_list() {
        let datastore = setup_test_datastore().await;
        let empty_list = vec![];

        let result = get_nodes_for_evaluation(&datastore, Some(&empty_list)).await;
        assert!(result.is_ok());
        let nodes = result.unwrap();
        assert!(nodes.is_empty());
    }

    #[tokio::test]
    async fn test_get_specific_nodes_empty_result() {
        let datastore = setup_test_datastore().await;
        let nonexistent_ids = vec![Uuid::new_v4(), Uuid::new_v4()];

        let result = get_specific_nodes(&datastore, &nonexistent_ids).await;
        assert!(result.is_ok());
        let nodes = result.unwrap();
        assert!(nodes.is_empty());
    }
}
