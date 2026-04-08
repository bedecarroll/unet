//! Policy results handlers and utilities

use axum::{
    Json,
    extract::{Query, State},
};
use tracing::info;

use crate::{
    error::{ServerError, ServerResult},
    server::AppState,
};

use super::types::{PolicyResultsQuery, PolicyResultsResponse};

/// Get policy evaluation results
///
/// # Errors
/// Returns an error if datastore operations fail.
pub async fn get_policy_results(
    State(state): State<AppState>,
    Query(query): Query<PolicyResultsQuery>,
) -> ServerResult<Json<PolicyResultsResponse>> {
    info!("Getting policy results with filter: {:?}", query);

    let node_id = query.node_id.ok_or_else(|| {
        ServerError::BadRequest(
            "node_id query parameter is required; cross-node policy result queries are not supported"
                .to_string(),
        )
    })?;

    let results = state.datastore.get_policy_results(&node_id).await?;
    let total_count = results.len();
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(100);
    let paginated_results: Vec<_> = results.into_iter().skip(offset).take(limit).collect();
    let returned_count = paginated_results.len();

    Ok(Json(PolicyResultsResponse {
        results: paginated_results,
        total_count,
        returned_count,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::AppState;
    use std::sync::Arc;
    use unet_core::{datastore::DataStore, models::*, policy_integration::PolicyService};

    async fn create_test_node(datastore: &dyn DataStore) -> Node {
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
        test_support::sqlite::with_savepoint("pol_results_node", |store| async move {
            let node = create_test_node(&store).await;
            let app_state = AppState {
                datastore: Arc::new(store),
                policy_service: PolicyService::with_local_dir("/tmp"),
            };

            let query = PolicyResultsQuery {
                node_id: Some(node.id),
                offset: None,
                limit: None,
            };

            let result = get_policy_results(State(app_state), axum::extract::Query(query)).await;
            assert!(result.is_err());
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("get_policy_results")
            );
        })
        .await;
    }

    #[tokio::test]
    async fn test_get_policy_results_without_node_id() {
        test_support::sqlite::with_savepoint("pol_results_none", |store| async move {
            let app_state = AppState {
                datastore: Arc::new(store),
                policy_service: PolicyService::with_local_dir("/tmp"),
            };

            let query = PolicyResultsQuery {
                node_id: None,
                offset: None,
                limit: None,
            };

            let result = get_policy_results(State(app_state), axum::extract::Query(query)).await;
            assert!(result.is_err());
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("node_id query parameter is required")
            );
        })
        .await;
    }

    #[tokio::test]
    async fn test_get_policy_results_with_pagination() {
        test_support::sqlite::with_savepoint("pol_results_page", |store| async move {
            let node = create_test_node(&store).await;
            let app_state = AppState {
                datastore: Arc::new(store),
                policy_service: PolicyService::with_local_dir("/tmp"),
            };

            let query = PolicyResultsQuery {
                node_id: Some(node.id),
                offset: Some(0),
                limit: Some(10),
            };

            let result = get_policy_results(State(app_state), axum::extract::Query(query)).await;
            assert!(result.is_err());
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("get_policy_results")
            );
        })
        .await;
    }
}
