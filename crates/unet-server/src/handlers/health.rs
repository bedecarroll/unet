//! Health check handlers with load balancer compatibility

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use tracing::{debug, warn};
use unet_core::load_balancer::{HealthCheckResult, HealthStatus, LoadBalancerHealthManager};

use crate::handlers::ServerResult;
use crate::server::AppState;

/// Health check query parameters
#[derive(Debug, Deserialize)]
pub struct HealthCheckQuery {
    /// Include detailed component information
    pub detailed: Option<bool>,
    /// Include dependencies in health check
    pub include_deps: Option<bool>,
}

/// Basic health check endpoint
pub async fn health_check(
    Query(params): Query<HealthCheckQuery>,
    State(app_state): State<AppState>,
) -> ServerResult<(StatusCode, Json<serde_json::Value>)> {
    debug!("Health check requested with params: {:?}", params);

    // Create load balancer health manager for this request
    let health_manager = LoadBalancerHealthManager::new(app_state.config.load_balancer.clone());

    // Check datastore health
    let datastore_healthy = match app_state.datastore.health_check().await {
        Ok(()) => {
            debug!("Datastore health check passed");
            health_manager
                .update_component_health(
                    "datastore",
                    HealthStatus::Healthy,
                    Some("Database connection active".to_string()),
                )
                .await;
            true
        }
        Err(e) => {
            warn!("Datastore health check failed: {}", e);
            health_manager
                .update_component_health(
                    "datastore",
                    HealthStatus::Unhealthy,
                    Some(format!("Database error: {}", e)),
                )
                .await;
            false
        }
    };

    // Check metrics system
    let metrics_healthy = app_state.metrics_manager.is_operational().await;
    health_manager
        .update_component_health(
            "metrics",
            if metrics_healthy {
                HealthStatus::Healthy
            } else {
                HealthStatus::Degraded
            },
            Some(
                if metrics_healthy {
                    "Metrics system operational"
                } else {
                    "Metrics system degraded"
                }
                .to_string(),
            ),
        )
        .await;

    let detailed = params.detailed.unwrap_or(false);
    let health_result = health_manager.get_health_result(detailed).await;

    let status_code = match health_result.status {
        HealthStatus::Healthy => StatusCode::OK,
        HealthStatus::Degraded => StatusCode::OK, // Still OK for degraded
        HealthStatus::Starting => StatusCode::SERVICE_UNAVAILABLE,
        HealthStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
        HealthStatus::ShuttingDown => StatusCode::SERVICE_UNAVAILABLE,
    };

    Ok((status_code, Json(serde_json::to_value(health_result)?)))
}

/// Detailed health check endpoint
pub async fn detailed_health_check(
    State(app_state): State<AppState>,
) -> ServerResult<Json<HealthCheckResult>> {
    debug!("Detailed health check requested");

    let health_manager = LoadBalancerHealthManager::new(app_state.config.load_balancer.clone());

    // Perform comprehensive health checks
    let datastore_healthy = match app_state.datastore.health_check().await {
        Ok(()) => {
            health_manager
                .update_component_health(
                    "datastore",
                    HealthStatus::Healthy,
                    Some("Database connection active".to_string()),
                )
                .await;
            true
        }
        Err(e) => {
            health_manager
                .update_component_health(
                    "datastore",
                    HealthStatus::Unhealthy,
                    Some(format!("Database error: {}", e)),
                )
                .await;
            false
        }
    };

    let metrics_healthy = app_state.metrics_manager.is_operational().await;
    health_manager
        .update_component_health(
            "metrics",
            if metrics_healthy {
                HealthStatus::Healthy
            } else {
                HealthStatus::Degraded
            },
            Some(
                if metrics_healthy {
                    "Metrics system operational"
                } else {
                    "Metrics system degraded"
                }
                .to_string(),
            ),
        )
        .await;

    // Check auth service if enabled
    if app_state.config.auth.enabled {
        health_manager
            .update_component_health(
                "auth",
                HealthStatus::Healthy,
                Some("Authentication service enabled".to_string()),
            )
            .await;
    }

    let health_result = health_manager.get_health_result(true).await;
    Ok(Json(health_result))
}

/// Kubernetes readiness probe endpoint
pub async fn readiness_check(
    State(app_state): State<AppState>,
) -> ServerResult<(StatusCode, Json<serde_json::Value>)> {
    debug!("Readiness check requested");

    let health_manager = LoadBalancerHealthManager::new(app_state.config.load_balancer.clone());
    let is_ready = health_manager.is_ready().await;

    if is_ready {
        // Perform quick checks for readiness
        let datastore_ok = app_state.datastore.health_check().await.is_ok();

        if datastore_ok {
            Ok((StatusCode::OK, Json(json!({"status": "ready"}))))
        } else {
            Ok((
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({"status": "not_ready", "reason": "datastore_unavailable"})),
            ))
        }
    } else {
        Ok((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"status": "not_ready", "reason": "service_not_ready"})),
        ))
    }
}

/// Kubernetes liveness probe endpoint
pub async fn liveness_check(
    State(app_state): State<AppState>,
) -> ServerResult<(StatusCode, Json<serde_json::Value>)> {
    debug!("Liveness check requested");

    let health_manager = LoadBalancerHealthManager::new(app_state.config.load_balancer.clone());
    let is_alive = health_manager.is_alive().await;

    if is_alive {
        Ok((StatusCode::OK, Json(json!({"status": "alive"}))))
    } else {
        Ok((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"status": "dead"})),
        ))
    }
}

/// Load balancer status endpoint with specific headers
pub async fn load_balancer_status(
    State(app_state): State<AppState>,
) -> ServerResult<(StatusCode, Json<serde_json::Value>)> {
    debug!("Load balancer status check requested");

    let health_manager = LoadBalancerHealthManager::new(app_state.config.load_balancer.clone());
    let health_result = health_manager.get_health_result(false).await;

    let response = json!({
        "status": match health_result.status {
            HealthStatus::Healthy => "UP",
            HealthStatus::Degraded => "WARN",
            _ => "DOWN"
        },
        "service": "μNet",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": health_result.timestamp,
        "load_balancer": health_result.metadata.load_balancer,
        "uptime_seconds": health_result.metadata.runtime.uptime_seconds,
        "active_connections": health_result.metadata.runtime.active_connections
    });

    let status_code = match health_result.status {
        HealthStatus::Healthy | HealthStatus::Degraded => StatusCode::OK,
        _ => StatusCode::SERVICE_UNAVAILABLE,
    };

    Ok((status_code, Json(response)))
}
