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
