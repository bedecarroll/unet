//! Health check handlers

use axum::{extract::State, http::StatusCode, response::Json};
use serde_json::json;
use tracing::{debug, warn};
use unet_core::datastore::DataStore;

use crate::handlers::ServerResult;
use crate::server::AppState;

/// Health check endpoint
pub async fn health_check(
    State(app_state): State<AppState>,
) -> ServerResult<(StatusCode, Json<serde_json::Value>)> {
    debug!("Health check requested");

    let datastore_healthy = check_datastore_health(&*app_state.datastore).await;
    let (status_code, overall_status) = determine_service_status(datastore_healthy);
    let response = build_health_response(overall_status, datastore_healthy, &*app_state.datastore);

    Ok((status_code, Json(response)))
}

async fn check_datastore_health(datastore: &dyn DataStore) -> bool {
    match datastore.health_check().await {
        Ok(()) => {
            debug!("Datastore health check passed");
            true
        }
        Err(e) => {
            warn!("Datastore health check failed: {}", e);
            false
        }
    }
}

const fn determine_service_status(datastore_healthy: bool) -> (StatusCode, &'static str) {
    if datastore_healthy {
        (StatusCode::OK, "healthy")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "degraded")
    }
}

fn build_health_response(
    overall_status: &str,
    datastore_healthy: bool,
    datastore: &dyn DataStore,
) -> serde_json::Value {
    json!({
        "status": overall_status,
        "service": "μNet",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "components": {
            "datastore": {
                "status": if datastore_healthy { "healthy" } else { "unhealthy" },
                "type": datastore.name()
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::AppState;
    use axum::{extract::State, http::StatusCode};
    use std::sync::Arc;
    use unet_core::datastore::csv::CsvStore;
    use unet_core::policy_integration::PolicyService;

    async fn create_healthy_app_state() -> AppState {
        let git_config = unet_core::config::GitConfig {
            repository_url: None,
            local_directory: None,
            branch: "main".to_string(),
            auth_token: None,
            sync_interval: 300,
            policies_repo: None,
            templates_repo: None,
        };

        // Use a temp directory for testing
        let temp_dir = std::env::temp_dir().join("unet_test_data");
        let _ = std::fs::create_dir_all(&temp_dir);

        AppState {
            datastore: Arc::new(CsvStore::new(&temp_dir).await.unwrap()),
            policy_service: PolicyService::new(git_config),
        }
    }

    #[tokio::test]
    async fn test_health_check_healthy() {
        let app_state = create_healthy_app_state().await;
        let state = State(app_state);

        let result = health_check(state).await;
        assert!(result.is_ok());

        let (status_code, response) = result.unwrap();
        assert_eq!(status_code, StatusCode::OK);

        let body = response.0;
        assert_eq!(body["status"], "healthy");
        assert_eq!(body["service"], "μNet");
        assert!(body["version"].is_string());
        assert!(body["timestamp"].is_string());
        assert_eq!(body["components"]["datastore"]["status"], "healthy");
        assert_eq!(body["components"]["datastore"]["type"], "CSV");
    }

    #[test]
    fn test_determine_service_status_healthy() {
        let (status_code, status_str) = determine_service_status(true);
        assert_eq!(status_code, StatusCode::OK);
        assert_eq!(status_str, "healthy");
    }

    #[test]
    fn test_determine_service_status_unhealthy() {
        let (status_code, status_str) = determine_service_status(false);
        assert_eq!(status_code, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(status_str, "degraded");
    }

    #[tokio::test]
    async fn test_build_health_response_healthy() {
        let temp_dir = std::env::temp_dir().join("unet_test_data_2");
        let _ = std::fs::create_dir_all(&temp_dir);
        let datastore = CsvStore::new(&temp_dir).await.unwrap();
        let response = build_health_response("healthy", true, &datastore);

        assert_eq!(response["status"], "healthy");
        assert_eq!(response["service"], "μNet");
        assert!(response["version"].is_string());
        assert!(response["timestamp"].is_string());
        assert_eq!(response["components"]["datastore"]["status"], "healthy");
        assert_eq!(response["components"]["datastore"]["type"], "CSV");
    }

    #[tokio::test]
    async fn test_build_health_response_unhealthy() {
        let temp_dir = std::env::temp_dir().join("unet_test_data_3");
        let _ = std::fs::create_dir_all(&temp_dir);
        let datastore = CsvStore::new(&temp_dir).await.unwrap();
        let response = build_health_response("degraded", false, &datastore);

        assert_eq!(response["status"], "degraded");
        assert_eq!(response["service"], "μNet");
        assert!(response["version"].is_string());
        assert!(response["timestamp"].is_string());
        assert_eq!(response["components"]["datastore"]["status"], "unhealthy");
        assert_eq!(response["components"]["datastore"]["type"], "CSV");
    }

    #[tokio::test]
    async fn test_health_response_structure() {
        let temp_dir = std::env::temp_dir().join("unet_test_data_4");
        let _ = std::fs::create_dir_all(&temp_dir);
        let datastore = CsvStore::new(&temp_dir).await.unwrap();
        let response = build_health_response("healthy", true, &datastore);

        // Check that all required fields are present
        assert!(response["status"].is_string());
        assert!(response["service"].is_string());
        assert!(response["version"].is_string());
        assert!(response["timestamp"].is_string());
        assert!(response["components"].is_object());
        assert!(response["components"]["datastore"].is_object());
        assert!(response["components"]["datastore"]["status"].is_string());
        assert!(response["components"]["datastore"]["type"].is_string());
    }

    #[tokio::test]
    async fn test_health_response_version() {
        let temp_dir = std::env::temp_dir().join("unet_test_data_5");
        let _ = std::fs::create_dir_all(&temp_dir);
        let datastore = CsvStore::new(&temp_dir).await.unwrap();
        let response = build_health_response("healthy", true, &datastore);

        // Version should be a non-empty string
        let version = response["version"].as_str().unwrap();
        assert!(!version.is_empty());
    }

    #[tokio::test]
    async fn test_health_response_timestamp_format() {
        let temp_dir = std::env::temp_dir().join("unet_test_data_6");
        let _ = std::fs::create_dir_all(&temp_dir);
        let datastore = CsvStore::new(&temp_dir).await.unwrap();
        let response = build_health_response("healthy", true, &datastore);

        // Timestamp should be a valid RFC3339 string
        let timestamp = response["timestamp"].as_str().unwrap();
        assert!(chrono::DateTime::parse_from_rfc3339(timestamp).is_ok());
    }

    #[tokio::test]
    async fn test_health_response_service_name() {
        let temp_dir = std::env::temp_dir().join("unet_test_data_7");
        let _ = std::fs::create_dir_all(&temp_dir);
        let datastore = CsvStore::new(&temp_dir).await.unwrap();
        let response = build_health_response("healthy", true, &datastore);

        // Service name should be μNet
        assert_eq!(response["service"], "μNet");
    }

    #[tokio::test]
    async fn test_health_response_components_structure() {
        let temp_dir = std::env::temp_dir().join("unet_test_data_8");
        let _ = std::fs::create_dir_all(&temp_dir);
        let datastore = CsvStore::new(&temp_dir).await.unwrap();
        let response = build_health_response("healthy", true, &datastore);

        let components = &response["components"];
        assert!(components.is_object());

        let datastore_component = &components["datastore"];
        assert!(datastore_component.is_object());
        assert!(datastore_component["status"].is_string());
        assert!(datastore_component["type"].is_string());
    }

    #[tokio::test]
    async fn test_service_status_consistency() {
        // Test that both healthy and unhealthy states are consistent
        let temp_dir = std::env::temp_dir().join("unet_test_data_9");
        let _ = std::fs::create_dir_all(&temp_dir);
        let datastore = CsvStore::new(&temp_dir).await.unwrap();

        let healthy_response = build_health_response("healthy", true, &datastore);
        let unhealthy_response = build_health_response("degraded", false, &datastore);

        // Both should have the same structure
        assert_eq!(healthy_response["service"], unhealthy_response["service"]);
        assert_eq!(healthy_response["version"], unhealthy_response["version"]);
        assert!(
            healthy_response["components"]["datastore"]["type"]
                == unhealthy_response["components"]["datastore"]["type"]
        );

        // But different statuses
        assert_ne!(healthy_response["status"], unhealthy_response["status"]);
        assert_ne!(
            healthy_response["components"]["datastore"]["status"],
            unhealthy_response["components"]["datastore"]["status"]
        );
    }
}
