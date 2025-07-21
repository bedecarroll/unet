//! Policy results handlers and utilities

use axum::{
    Json,
    extract::{Query, State},
};
use tracing::{error, info};

use crate::{
    error::{ServerError, ServerResult},
    server::AppState,
};

use super::types::{PolicyResultsQuery, PolicyResultsResponse};

/// Get policy evaluation results
pub async fn get_policy_results(
    State(state): State<AppState>,
    Query(query): Query<PolicyResultsQuery>,
) -> ServerResult<Json<PolicyResultsResponse>> {
    info!("Getting policy results with filter: {:?}", query);

    // For now, return results for a specific node if requested
    if let Some(node_id) = query.node_id {
        match state.datastore.get_policy_results(&node_id).await {
            Ok(results) => {
                let total_count = results.len();
                let offset = query.offset.unwrap_or(0);
                let limit = query.limit.unwrap_or(100);

                let paginated_results: Vec<_> =
                    results.into_iter().skip(offset).take(limit).collect();

                let returned_count = paginated_results.len();

                Ok(Json(PolicyResultsResponse {
                    results: paginated_results,
                    total_count,
                    returned_count,
                }))
            }
            Err(e) => {
                error!("Failed to get policy results for node {}: {}", node_id, e);
                Err(ServerError::Internal(format!(
                    "Failed to get policy results: {e}"
                )))
            }
        }
    } else {
        // Return empty results for now - implementing cross-node queries requires more complex logic
        Ok(Json(PolicyResultsResponse {
            results: Vec::new(),
            total_count: 0,
            returned_count: 0,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::AppState;
    use migration::{Migrator, MigratorTrait};
    use std::sync::Arc;
    use unet_core::{
        datastore::{DataStore, sqlite::SqliteStore},
        models::*,
        policy_integration::PolicyService,
    };

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
    async fn test_get_policy_results_with_node_id() {
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;
        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
        };

        let query = PolicyResultsQuery {
            node_id: Some(node.id),
            offset: None,
            limit: None,
        };

        let result = get_policy_results(State(app_state), axum::extract::Query(query)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert_eq!(response.total_count, 0);
        assert_eq!(response.returned_count, 0);
    }

    #[tokio::test]
    async fn test_get_policy_results_without_node_id() {
        let datastore = setup_test_datastore().await;
        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
        };

        let query = PolicyResultsQuery {
            node_id: None,
            offset: None,
            limit: None,
        };

        let result = get_policy_results(State(app_state), axum::extract::Query(query)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert_eq!(response.total_count, 0);
        assert_eq!(response.returned_count, 0);
    }

    #[tokio::test]
    async fn test_get_policy_results_with_pagination() {
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;
        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
        };

        let query = PolicyResultsQuery {
            node_id: Some(node.id),
            offset: Some(0),
            limit: Some(10),
        };

        let result = get_policy_results(State(app_state), axum::extract::Query(query)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert_eq!(response.total_count, 0);
        assert_eq!(response.returned_count, 0);
    }
}
