//! Policy status handlers and utilities

use axum::{Json, extract::State};
use tracing::{info, warn};

use crate::{error::ServerResult, server::AppState};

/// Get policy engine status
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
    async fn test_get_policy_status() {
        let datastore = setup_test_datastore().await;
        let _node = create_test_node(&datastore).await;
        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
        };

        let result = get_policy_status(State(app_state)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert_eq!(response["policy_engine_enabled"], true);
        assert_eq!(response["nodes_available"], 1);
    }
}
