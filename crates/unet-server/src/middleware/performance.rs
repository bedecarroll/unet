//! Performance monitoring middleware

use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use tracing::{Instrument, debug, info_span, warn};
use unet_core::{
    metrics::MetricsManager,
    performance::{PerformanceManager, PerformanceProfiler},
};

/// Performance monitoring middleware that tracks HTTP request durations
pub async fn performance_monitoring_middleware(
    State(metrics_manager): State<MetricsManager>,
    request: Request,
    next: Next,
) -> Response {
    let start_time = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();

    // Increment HTTP request counter
    metrics_manager.increment_http_requests().await;

    // Process the request with timing
    let response = next.run(request).await;

    // Calculate duration and record metrics
    let duration = start_time.elapsed();
    metrics_manager.record_http_request_duration(duration).await;

    debug!(
        method = %method,
        uri = %uri,
        status = response.status().as_u16(),
        duration_ms = duration.as_millis(),
        "HTTP request completed"
    );

    response
}

/// Database query timing wrapper function
pub async fn time_database_operation<F, T>(
    metrics_manager: &MetricsManager,
    operation_name: &str,
    operation: F,
) -> T
where
    F: std::future::Future<Output = T>,
{
    let start = Instant::now();
    let result = operation.await;
    let duration = start.elapsed();

    metrics_manager
        .record_database_query_duration(duration)
        .await;
    debug!(
        operation = operation_name,
        duration_ms = duration.as_millis(),
        "Database operation completed"
    );

    result
}

/// Policy evaluation timing wrapper function
pub async fn time_policy_evaluation<F, T>(
    metrics_manager: &MetricsManager,
    policy_name: &str,
    evaluation: F,
) -> T
where
    F: std::future::Future<Output = T>,
{
    let start = Instant::now();
    let result = evaluation.await;
    let duration = start.elapsed();

    metrics_manager
        .record_policy_evaluation_duration(duration)
        .await;
    debug!(
        policy = policy_name,
        duration_ms = duration.as_millis(),
        "Policy evaluation completed"
    );

    result
}

/// Template rendering timing wrapper function
pub async fn time_template_rendering<F, T>(
    metrics_manager: &MetricsManager,
    template_name: &str,
    rendering: F,
) -> T
where
    F: std::future::Future<Output = T>,
{
    let start = Instant::now();
    let result = rendering.await;
    let duration = start.elapsed();

    metrics_manager
        .record_template_rendering_duration(duration)
        .await;
    debug!(
        template = template_name,
        duration_ms = duration.as_millis(),
        "Template rendering completed"
    );

    result
}

/// Enhanced performance profiling middleware with detailed metrics
pub async fn enhanced_performance_middleware(
    profiler: PerformanceProfiler,
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path();

    // Create operation name from method and path
    let operation = format!("{} {}", method, path);

    // Start timing the operation
    let timer = profiler.start_operation(&operation);

    // Process the request
    let response = next.run(request).await;
    let status = response.status();

    // Finish timing and record metrics
    timer.finish().await;

    // Add performance headers to response
    let mut response = response;
    if let Some(metrics) = profiler.get_metrics(&operation).await {
        response.headers_mut().insert(
            "X-Avg-Response-Time",
            format!("{:.2}", metrics.avg_duration_ms).parse().unwrap(),
        );
        response.headers_mut().insert(
            "X-Request-Count",
            metrics.count.to_string().parse().unwrap(),
        );
        response.headers_mut().insert(
            "X-Ops-Per-Second",
            format!("{:.2}", metrics.ops_per_second).parse().unwrap(),
        );
    }

    response
}

/// Adaptive rate limiting middleware based on performance metrics
pub async fn adaptive_rate_limiting_middleware(
    profiler: PerformanceProfiler,
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path();

    // Create operation name from method and path
    let operation = format!("{} {}", method, path);

    // Check if this operation is currently slow
    if let Some(metrics) = profiler.get_metrics(&operation).await {
        // If average response time is over 5 seconds, apply rate limiting
        if metrics.avg_duration_ms > 5000.0 {
            warn!(
                "Rate limiting {} due to slow performance (avg: {:.2}ms)",
                operation, metrics.avg_duration_ms
            );

            // Return 429 Too Many Requests for slow operations with high load
            if metrics.ops_per_second > 10.0 {
                return Response::builder()
                    .status(StatusCode::TOO_MANY_REQUESTS)
                    .header("Retry-After", "30")
                    .header("X-Rate-Limit-Reason", "performance")
                    .body(Body::from("Rate limited due to performance"))
                    .unwrap();
            }
        }

        // Check for high error rates (if we track errors in the future)
        if metrics.p99_duration_ms > 10000 {
            warn!(
                "High P99 latency detected for {}: {}ms",
                operation, metrics.p99_duration_ms
            );
        }
    }

    // Process the request normally
    next.run(request).await
}

/// Caching middleware for frequently accessed data
pub async fn caching_middleware(
    performance_manager: PerformanceManager,
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();

    // Only cache GET requests
    if method != axum::http::Method::GET {
        return next.run(request).await;
    }

    let cache_key = format!("{} {}", method, uri);
    let cache = performance_manager.cache();

    // Check if we have a cached response
    if let Some(cached_response) = cache.get(&cache_key).await {
        debug!("Cache hit for {}", cache_key);

        // Return cached response (this is simplified - in reality we'd need to deserialize properly)
        return Response::builder()
            .status(StatusCode::OK)
            .header("X-Cache", "HIT")
            .body(Body::from(cached_response.to_string()))
            .unwrap();
    }

    // Process the request
    let response = next.run(request).await;

    // Cache successful responses
    if response.status().is_success() {
        debug!("Caching response for {}", cache_key);

        // In a real implementation, we'd properly serialize the response
        let cache_value = serde_json::json!({
            "status": response.status().as_u16(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "uri": uri.to_string()
        });

        cache.insert(cache_key, cache_value).await;
    }

    // Add cache status header
    let mut response = response;
    response
        .headers_mut()
        .insert("X-Cache", "MISS".parse().unwrap());

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::time::Duration;
    use unet_core::config::MetricsConfig;

    #[tokio::test]
    async fn test_database_timing() {
        let config = MetricsConfig {
            enabled: true,
            endpoint: "/metrics".to_string(),
            bind_address: None,
            collection_interval: 15,
            enable_performance_metrics: true,
            enable_business_metrics: true,
            enable_system_metrics: true,
            labels: HashMap::new(),
            retention_days: 7,
        };

        let metrics_manager = MetricsManager::new(config).unwrap();

        let result = time_database_operation(&metrics_manager, "test_query", async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            "query_result"
        })
        .await;

        assert_eq!(result, "query_result");
    }

    #[tokio::test]
    async fn test_policy_timing() {
        let config = MetricsConfig {
            enabled: true,
            endpoint: "/metrics".to_string(),
            bind_address: None,
            collection_interval: 15,
            enable_performance_metrics: true,
            enable_business_metrics: true,
            enable_system_metrics: true,
            labels: HashMap::new(),
            retention_days: 7,
        };

        let metrics_manager = MetricsManager::new(config).unwrap();

        let result = time_policy_evaluation(&metrics_manager, "test_policy", async {
            tokio::time::sleep(Duration::from_millis(5)).await;
            true
        })
        .await;

        assert!(result);
    }

    #[tokio::test]
    async fn test_template_timing() {
        let config = MetricsConfig {
            enabled: true,
            endpoint: "/metrics".to_string(),
            bind_address: None,
            collection_interval: 15,
            enable_performance_metrics: true,
            enable_business_metrics: true,
            enable_system_metrics: true,
            labels: HashMap::new(),
            retention_days: 7,
        };

        let metrics_manager = MetricsManager::new(config).unwrap();

        let result = time_template_rendering(&metrics_manager, "test_template", async {
            tokio::time::sleep(Duration::from_millis(8)).await;
            "rendered_content"
        })
        .await;

        assert_eq!(result, "rendered_content");
    }
}
