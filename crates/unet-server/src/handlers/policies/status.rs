//! Policy status handlers and utilities

use axum::{Json, extract::State};
use tracing::{info, warn};

use crate::{error::ServerResult, server::AppState};

/// Get policy engine status
///
/// # Errors
/// Returns an error if datastore operations fail.
pub async fn get_policy_status(
    State(state): State<AppState>,
) -> ServerResult<Json<serde_json::Value>> {
    info!("Getting policy engine status");

    // Get some basic statistics
    let nodes_count = match state.datastore.get_nodes_for_policy_evaluation().await {
        Ok(nodes) => nodes.len(),
        Err(e) => {
            warn!("Failed to get nodes count: {}", e);
            0
        }
    };

    let mut policy_service = state.policy_service.clone();
    let policies_count = match policy_service.load_policies() {
        Ok(policies) => policies.len(),
        Err(e) => {
            warn!("Failed to load policies for status: {}", e);
            0
        }
    };

    Ok(Json(serde_json::json!({
        "policy_engine_enabled": true,
        "nodes_available": nodes_count,
        "policies_available": policies_count,
        "last_evaluation": null, // TODO: Track last evaluation time
        "evaluation_frequency": "on-demand" // TODO: Configure scheduled evaluations
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::AppState;
    use std::sync::Arc;
    use unet_core::{
        datastore::DataStore,
        models::*,
        policy_integration::PolicyService,
    };

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
    async fn test_get_policy_status() {
        test_support::sqlite::with_savepoint("pol_status", |store| async move {
            let _node = create_test_node(&store).await;
            let app_state = AppState {
                datastore: Arc::new(store),
                policy_service: PolicyService::with_local_dir("/tmp"),
            };

            let result = get_policy_status(State(app_state)).await;
            assert!(result.is_ok());

            let response = result.unwrap().0;
            assert_eq!(response["policy_engine_enabled"], true);
            assert_eq!(response["nodes_available"], 1);
        }).await;
    }
}
