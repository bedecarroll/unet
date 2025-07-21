//! Node fetching logic for policy evaluation

use crate::error::ServerError;
use tracing::{error, warn};
use unet_core::prelude::{DataStore, Node};
use uuid::Uuid;

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

#[cfg(test)]
mod tests {
    use super::*;
    use migration::{Migrator, MigratorTrait};
    use unet_core::{datastore::sqlite::SqliteStore, models::*};

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
