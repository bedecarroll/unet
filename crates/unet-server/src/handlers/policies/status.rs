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
    let evaluation_interval_seconds = state.policy_service.evaluation_interval_seconds();
    let last_evaluation = state
        .policy_service
        .last_evaluation()
        .map(|time| chrono::DateTime::<chrono::Utc>::from(time).to_rfc3339());

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
        "last_evaluation": last_evaluation,
        "evaluation_frequency": format!("every {evaluation_interval_seconds} seconds"),
        "evaluation_frequency_seconds": evaluation_interval_seconds
    })))
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
    async fn test_get_policy_status() {
        test_support::sqlite::with_savepoint("pol_status", |store| async move {
            let temp_dir = tempfile::tempdir().unwrap();
            let policies_directory = temp_dir.path().to_string_lossy().into_owned();
            let _node = create_test_node(&store).await;
            let app_state = AppState {
                datastore: Arc::new(store),
                policy_service: PolicyService::with_local_dir(&policies_directory),
            };

            let result = get_policy_status(State(app_state)).await;
            assert!(result.is_ok());

            let response = result.unwrap().0;
            assert_eq!(response["policy_engine_enabled"], true);
            assert_eq!(response["nodes_available"], 1);
            assert_eq!(response["evaluation_frequency"], "every 300 seconds");
            assert_eq!(response["evaluation_frequency_seconds"], 300);
            assert!(response["last_evaluation"].is_null());
        })
        .await;
    }

    #[tokio::test]
    async fn test_get_policy_status_reports_last_evaluation() {
        test_support::sqlite::with_savepoint("pol_status_last_eval", |store| async move {
            let temp_dir = tempfile::tempdir().unwrap();
            let policies_directory = temp_dir.path().to_string_lossy().into_owned();
            let app_state = AppState {
                datastore: Arc::new(store),
                policy_service: PolicyService::with_local_dir(&policies_directory),
            };
            app_state.policy_service.record_evaluation_run();

            let response = get_policy_status(State(app_state)).await.unwrap().0;
            let last_evaluation = response["last_evaluation"].as_str().unwrap();
            assert!(chrono::DateTime::parse_from_rfc3339(last_evaluation).is_ok());
        })
        .await;
    }
}
