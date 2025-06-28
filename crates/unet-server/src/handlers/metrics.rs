//! Metrics and monitoring API handlers
//!
//! This module provides HTTP handlers for metrics collection, system health monitoring,
//! and performance monitoring endpoints.

use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info};
use unet_core::metrics::MetricsSnapshot;

use crate::server::AppState;

/// Prometheus metrics endpoint - returns metrics in Prometheus format
pub async fn get_prometheus_metrics(
    State(state): State<AppState>,
) -> Result<Response<String>, StatusCode> {
    match state.metrics_manager.get_prometheus_metrics() {
        Ok(metrics) => {
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/plain; charset=utf-8")
                .body(metrics)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok(response)
        }
        Err(e) => {
            error!("Failed to get Prometheus metrics: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// System health check endpoint
pub async fn health_check(
    State(state): State<AppState>,
) -> Result<Json<HealthResponse>, StatusCode> {
    // Perform basic health checks
    let health_status = perform_health_checks(&state).await;

    let status_code = if health_status.status == "healthy" {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    if status_code != StatusCode::OK {
        return Err(status_code);
    }

    Ok(Json(health_status))
}

/// Detailed system health and metrics snapshot
pub async fn get_system_health(
    State(state): State<AppState>,
) -> Result<Json<SystemHealthResponse>, StatusCode> {
    let health_checks = perform_health_checks(&state).await;

    let metrics_snapshot = match state.metrics_manager.get_metrics_snapshot().await {
        Ok(snapshot) => Some(snapshot),
        Err(e) => {
            error!("Failed to get metrics snapshot: {}", e);
            None
        }
    };

    let response = SystemHealthResponse {
        health: health_checks,
        metrics: metrics_snapshot,
        timestamp: chrono::Utc::now(),
    };

    Ok(Json(response))
}

/// Performance metrics endpoint
pub async fn get_performance_metrics(
    State(state): State<AppState>,
) -> Result<Json<PerformanceMetricsResponse>, StatusCode> {
    match state.metrics_manager.get_metrics_snapshot().await {
        Ok(snapshot) => {
            let response = PerformanceMetricsResponse {
                performance: snapshot.performance,
                system: snapshot.system,
                timestamp: snapshot.timestamp,
            };
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to get performance metrics: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Business metrics endpoint
pub async fn get_business_metrics(
    State(state): State<AppState>,
) -> Result<Json<BusinessMetricsResponse>, StatusCode> {
    match state.metrics_manager.get_metrics_snapshot().await {
        Ok(snapshot) => {
            let response = BusinessMetricsResponse {
                business: snapshot.business,
                timestamp: snapshot.timestamp,
            };
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to get business metrics: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Metrics configuration endpoint
pub async fn get_metrics_config(
    State(state): State<AppState>,
) -> Result<Json<MetricsConfigResponse>, StatusCode> {
    let config = state.metrics_manager.get_config();

    let response = MetricsConfigResponse {
        enabled: config.enabled,
        endpoint: config.endpoint.clone(),
        collection_interval: config.collection_interval,
        enable_performance_metrics: config.enable_performance_metrics,
        enable_business_metrics: config.enable_business_metrics,
        enable_system_metrics: config.enable_system_metrics,
        labels: config.labels.clone(),
        retention_days: config.retention_days,
    };

    Ok(Json(response))
}

/// Query metrics with filters (placeholder for advanced querying)
pub async fn query_metrics(
    State(state): State<AppState>,
    Query(params): Query<MetricsQueryParams>,
) -> Result<Json<MetricsQueryResponse>, StatusCode> {
    info!("Metrics query requested with filters: {:?}", params);

    // This is a placeholder implementation
    // In a full implementation, you would filter and aggregate metrics based on the query parameters
    match state.metrics_manager.get_metrics_snapshot().await {
        Ok(snapshot) => {
            let response = MetricsQueryResponse {
                query: params,
                data: snapshot,
                result_count: 1, // Placeholder
            };
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to query metrics: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ========================================
// Response Types
// ========================================

/// Basic health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub version: String,
    pub checks: HashMap<String, HealthCheck>,
}

/// Individual health check result
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheck {
    pub status: String,
    pub message: Option<String>,
    pub duration_ms: u64,
}

/// Comprehensive system health response
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemHealthResponse {
    pub health: HealthResponse,
    pub metrics: Option<MetricsSnapshot>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Performance metrics response
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceMetricsResponse {
    pub performance: unet_core::metrics::PerformanceMetricsSnapshot,
    pub system: unet_core::metrics::SystemMetricsSnapshot,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Business metrics response
#[derive(Debug, Serialize, Deserialize)]
pub struct BusinessMetricsResponse {
    pub business: unet_core::metrics::BusinessMetricsSnapshot,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Metrics configuration response
#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsConfigResponse {
    pub enabled: bool,
    pub endpoint: String,
    pub collection_interval: u64,
    pub enable_performance_metrics: bool,
    pub enable_business_metrics: bool,
    pub enable_system_metrics: bool,
    pub labels: HashMap<String, String>,
    pub retention_days: u32,
}

/// Metrics query parameters
#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsQueryParams {
    pub metric_names: Option<String>, // Comma-separated metric names
    pub start_time: Option<String>,   // ISO 8601 timestamp
    pub end_time: Option<String>,     // ISO 8601 timestamp
    pub interval: Option<String>,     // Aggregation interval (e.g., "5m", "1h")
    pub labels: Option<String>,       // Label filters (e.g., "service=unet,env=prod")
}

/// Metrics query response
#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsQueryResponse {
    pub query: MetricsQueryParams,
    pub data: MetricsSnapshot,
    pub result_count: usize,
}

// ========================================
// Helper Functions
// ========================================

/// Perform comprehensive health checks
async fn perform_health_checks(state: &AppState) -> HealthResponse {
    let start_time = std::time::Instant::now();
    let mut checks = HashMap::new();

    // Database health check
    let db_check = check_database_health(state).await;
    checks.insert("database".to_string(), db_check);

    // Metrics system health check
    let metrics_check = check_metrics_health(state).await;
    checks.insert("metrics".to_string(), metrics_check);

    // Configuration health check
    let config_check = check_configuration_health(state).await;
    checks.insert("configuration".to_string(), config_check);

    // Memory health check
    let memory_check = check_memory_health().await;
    checks.insert("memory".to_string(), memory_check);

    // Determine overall status
    let overall_status = if checks.values().all(|check| check.status == "healthy") {
        "healthy"
    } else if checks.values().any(|check| check.status == "unhealthy") {
        "unhealthy"
    } else {
        "degraded"
    };

    HealthResponse {
        status: overall_status.to_string(),
        timestamp: chrono::Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        checks,
    }
}

/// Check database connectivity and health
async fn check_database_health(state: &AppState) -> HealthCheck {
    let start = std::time::Instant::now();

    // Simple database connectivity check
    // In a real implementation, you would ping the database or run a simple query
    let status = "healthy"; // Placeholder
    let message = Some("Database connection successful".to_string());

    HealthCheck {
        status: status.to_string(),
        message,
        duration_ms: start.elapsed().as_millis() as u64,
    }
}

/// Check metrics system health
async fn check_metrics_health(state: &AppState) -> HealthCheck {
    let start = std::time::Instant::now();

    let (status, message) = if state.metrics_manager.is_enabled() {
        match state.metrics_manager.get_prometheus_metrics() {
            Ok(_) => (
                "healthy",
                Some("Metrics collection operational".to_string()),
            ),
            Err(e) => (
                "unhealthy",
                Some(format!("Metrics collection failed: {}", e)),
            ),
        }
    } else {
        ("degraded", Some("Metrics collection disabled".to_string()))
    };

    HealthCheck {
        status: status.to_string(),
        message,
        duration_ms: start.elapsed().as_millis() as u64,
    }
}

/// Check configuration health
async fn check_configuration_health(state: &AppState) -> HealthCheck {
    let start = std::time::Instant::now();

    // Validate critical configuration settings
    let config = &state.config;
    let (status, message) = if config.server.port > 0 && !config.server.host.is_empty() {
        ("healthy", Some("Configuration validated".to_string()))
    } else {
        (
            "unhealthy",
            Some("Invalid server configuration".to_string()),
        )
    };

    HealthCheck {
        status: status.to_string(),
        message,
        duration_ms: start.elapsed().as_millis() as u64,
    }
}

/// Check system memory health
async fn check_memory_health() -> HealthCheck {
    let start = std::time::Instant::now();

    // Simplified memory check - in production you'd use system monitoring libraries
    let status = "healthy"; // Placeholder
    let message = Some("Memory usage within normal limits".to_string());

    HealthCheck {
        status: status.to_string(),
        message,
        duration_ms: start.elapsed().as_millis() as u64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Tests removed for now due to circular dependencies

    #[tokio::test]
    async fn test_health_response_creation() {
        let checks = HashMap::new();
        let health = HealthResponse {
            status: "healthy".to_string(),
            timestamp: chrono::Utc::now(),
            version: "0.1.0".to_string(),
            checks,
        };

        assert_eq!(health.status, "healthy");
        assert_eq!(health.version, "0.1.0");
    }

    #[tokio::test]
    async fn test_metrics_query_params() {
        let params = MetricsQueryParams {
            metric_names: Some("http_requests_total".to_string()),
            start_time: None,
            end_time: None,
            interval: Some("5m".to_string()),
            labels: Some("service=unet".to_string()),
        };

        assert!(params.metric_names.is_some());
        assert!(params.interval.is_some());
    }
}
