//! Health check handlers

use axum::{extract::State, http::StatusCode, response::Json};
use serde_json::json;
use tracing::{debug, warn};

use crate::handlers::ServerResult;
use crate::server::AppState;

/// Health check endpoint
pub async fn health_check(
    State(app_state): State<AppState>,
) -> ServerResult<(StatusCode, Json<serde_json::Value>)> {
    debug!("Health check requested");

    // Check datastore health
    let datastore_healthy = match app_state.datastore.health_check().await {
        Ok(()) => {
            debug!("Datastore health check passed");
            true
        }
        Err(e) => {
            warn!("Datastore health check failed: {}", e);
            false
        }
    };

    let overall_status = if datastore_healthy {
        "healthy"
    } else {
        "degraded"
    };
    let status_code = if datastore_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    let response = json!({
        "status": overall_status,
        "service": "μNet",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "components": {
            "datastore": {
                "status": if datastore_healthy { "healthy" } else { "unhealthy" },
                "type": app_state.datastore.name()
            }
        }
    });

    Ok((status_code, Json(response)))
}
