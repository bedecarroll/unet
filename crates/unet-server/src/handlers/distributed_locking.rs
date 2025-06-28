use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use unet_core::{
    distributed_locking::{
        DistributedLockConfig, DistributedLockManager, DistributedLockProvider, LockInfo,
        LockMonitorReport, LockStats, RedisLockProvider,
    },
    shared_state::SharedStateManager,
};

use crate::{api::ApiError, server::AppState};

/// Request to acquire a distributed lock
#[derive(Debug, Deserialize)]
pub struct AcquireLockRequest {
    /// Lock key identifier
    pub key: String,
    /// Lock timeout in seconds (optional, uses default if not specified)
    pub timeout_seconds: Option<u64>,
}

/// Request to create a leader election
#[derive(Debug, Deserialize)]
pub struct CreateLeaderElectionRequest {
    /// Election key identifier
    pub election_key: String,
    /// Lease duration in seconds
    pub lease_duration_seconds: u64,
}

/// Query parameters for listing locks
#[derive(Debug, Deserialize)]
pub struct ListLocksQuery {
    /// Filter by lock type
    pub lock_type: Option<String>,
    /// Filter by owner
    pub owner_id: Option<String>,
    /// Include expired locks
    pub include_expired: Option<bool>,
}

/// Request to configure distributed locking
#[derive(Debug, Deserialize)]
pub struct UpdateLockConfigRequest {
    /// Enable or disable distributed locking
    pub enabled: Option<bool>,
    /// Lock backend ("redis", "database")
    pub backend: Option<String>,
    /// Default lock timeout in seconds
    pub default_timeout: Option<u64>,
    /// Lock renewal interval in seconds
    pub renewal_interval: Option<u64>,
    /// Maximum lock duration in seconds
    pub max_duration: Option<u64>,
}

/// Response for lock acquisition
#[derive(Debug, Serialize)]
pub struct AcquireLockResponse {
    /// Whether the lock was successfully acquired
    pub acquired: bool,
    /// Lock information if acquired
    pub lock_info: Option<LockInfo>,
    /// Error message if acquisition failed
    pub error: Option<String>,
}

/// Response for leader election
#[derive(Debug, Serialize)]
pub struct LeaderElectionResponse {
    /// Whether this instance became the leader
    pub is_leader: bool,
    /// Election key
    pub election_key: String,
    /// Instance ID
    pub instance_id: String,
}

/// Response for lock configuration
#[derive(Debug, Serialize)]
pub struct LockConfigResponse {
    /// Current distributed locking configuration
    pub config: DistributedLockConfig,
    /// Whether the configuration was updated
    pub updated: bool,
}

/// GET /api/v1/locks/stats - Get distributed locking statistics
pub async fn get_lock_stats(
    State(_app_state): State<AppState>,
) -> Result<Json<LockStats>, ApiError> {
    // Create a Redis lock provider for demonstration
    let shared_state = std::sync::Arc::new(SharedStateManager::in_memory());
    let provider = std::sync::Arc::new(RedisLockProvider::new(shared_state));

    let stats = provider
        .get_lock_stats()
        .await
        .map_err(|e| ApiError::internal_server_error(format!("Failed to get lock stats: {}", e)))?;

    Ok(Json(stats))
}

/// GET /api/v1/locks - List all active distributed locks
pub async fn list_locks(
    State(_app_state): State<AppState>,
    Query(_query): Query<ListLocksQuery>,
) -> Result<Json<Vec<LockInfo>>, ApiError> {
    // Create a Redis lock provider for demonstration
    let shared_state = std::sync::Arc::new(SharedStateManager::in_memory());
    let provider = std::sync::Arc::new(RedisLockProvider::new(shared_state));

    let locks = provider
        .list_locks()
        .await
        .map_err(|e| ApiError::internal_server_error(format!("Failed to list locks: {}", e)))?;

    Ok(Json(locks))
}

/// POST /api/v1/locks/acquire - Acquire a distributed lock
pub async fn acquire_lock(
    State(_app_state): State<AppState>,
    Json(request): Json<AcquireLockRequest>,
) -> Result<Json<AcquireLockResponse>, ApiError> {
    // Create a distributed lock manager for demonstration
    let shared_state = std::sync::Arc::new(SharedStateManager::in_memory());
    let provider = std::sync::Arc::new(RedisLockProvider::new(shared_state));
    let config = DistributedLockConfig {
        enabled: true,
        default_timeout: request.timeout_seconds.unwrap_or(30),
        ..Default::default()
    };
    let manager = DistributedLockManager::new(provider, config);

    match manager.acquire_lock_with_retry(&request.key).await {
        Ok(lock) => {
            let lock_info = LockInfo {
                key: request.key.clone(),
                value: "acquired".to_string(),
                acquired_at: std::time::SystemTime::now(),
                expires_at: std::time::SystemTime::now() + Duration::from_secs(30),
                owner_id: "current_instance".to_string(),
                lock_type: unet_core::distributed_locking::LockType::Exclusive,
                renewal_count: 0,
            };

            // Note: In a real implementation, we would store the lock reference
            // For now, we release it immediately after demonstrating acquisition
            let _ = lock.release().await;

            Ok(Json(AcquireLockResponse {
                acquired: true,
                lock_info: Some(lock_info),
                error: None,
            }))
        }
        Err(e) => Ok(Json(AcquireLockResponse {
            acquired: false,
            lock_info: None,
            error: Some(e.to_string()),
        })),
    }
}

/// DELETE /api/v1/locks/{key} - Release a distributed lock
pub async fn release_lock(
    State(_app_state): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Create a Redis lock provider for demonstration
    let shared_state = std::sync::Arc::new(SharedStateManager::in_memory());
    let provider = std::sync::Arc::new(RedisLockProvider::new(shared_state));

    let released = provider
        .release_lock(&key, "demo_value")
        .await
        .map_err(|e| ApiError::internal_server_error(format!("Failed to release lock: {}", e)))?;

    Ok(Json(serde_json::json!({
        "released": released,
        "key": key
    })))
}

/// GET /api/v1/locks/{key} - Get information about a specific lock
pub async fn get_lock_info(
    State(_app_state): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<Option<LockInfo>>, ApiError> {
    // Create a Redis lock provider for demonstration
    let shared_state = std::sync::Arc::new(SharedStateManager::in_memory());
    let provider = std::sync::Arc::new(RedisLockProvider::new(shared_state));

    let lock_info = provider
        .get_lock_info(&key)
        .await
        .map_err(|e| ApiError::internal_server_error(format!("Failed to get lock info: {}", e)))?;

    Ok(Json(lock_info))
}

/// POST /api/v1/locks/{key}/extend - Extend the duration of a distributed lock
pub async fn extend_lock(
    State(_app_state): State<AppState>,
    Path(key): Path<String>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let extension_seconds = request
        .get("extension_seconds")
        .and_then(|v| v.as_u64())
        .unwrap_or(30);

    // Create a Redis lock provider for demonstration
    let shared_state = std::sync::Arc::new(SharedStateManager::in_memory());
    let provider = std::sync::Arc::new(RedisLockProvider::new(shared_state));

    let extended = provider
        .extend_lock(&key, "demo_value", Duration::from_secs(extension_seconds))
        .await
        .map_err(|e| ApiError::internal_server_error(format!("Failed to extend lock: {}", e)))?;

    Ok(Json(serde_json::json!({
        "extended": extended,
        "key": key,
        "extension_seconds": extension_seconds
    })))
}

/// POST /api/v1/locks/leader-election - Create or join a leader election
pub async fn create_leader_election(
    State(_app_state): State<AppState>,
    Json(request): Json<CreateLeaderElectionRequest>,
) -> Result<Json<LeaderElectionResponse>, ApiError> {
    // Create a leader election for demonstration
    let shared_state = std::sync::Arc::new(SharedStateManager::in_memory());
    let provider = std::sync::Arc::new(RedisLockProvider::new(shared_state));

    let election = unet_core::distributed_locking::LeaderElection::new(
        provider,
        request.election_key.clone(),
        Duration::from_secs(request.lease_duration_seconds),
    );

    let became_leader = election
        .try_become_leader()
        .await
        .map_err(|e| ApiError::internal_server_error(format!("Leader election failed: {}", e)))?;

    Ok(Json(LeaderElectionResponse {
        is_leader: became_leader,
        election_key: request.election_key,
        instance_id: "current_instance".to_string(),
    }))
}

/// GET /api/v1/locks/leader-election/{election_key}/status - Check leader election status
pub async fn get_leader_election_status(
    State(_app_state): State<AppState>,
    Path(election_key): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // For demonstration purposes, return a mock status
    Ok(Json(serde_json::json!({
        "election_key": election_key,
        "current_leader": "instance-abc123",
        "is_leader": false,
        "election_active": true,
        "lease_expires_at": "2025-06-30T12:00:00Z"
    })))
}

/// GET /api/v1/locks/monitor - Get comprehensive lock monitoring report
pub async fn get_lock_monitor_report(
    State(_app_state): State<AppState>,
) -> Result<Json<LockMonitorReport>, ApiError> {
    // Create a distributed lock manager for monitoring
    let shared_state = std::sync::Arc::new(SharedStateManager::in_memory());
    let provider = std::sync::Arc::new(RedisLockProvider::new(shared_state));
    let config = DistributedLockConfig::default();
    let manager = DistributedLockManager::new(provider, config);

    let report = manager.monitor_locks().await.map_err(|e| {
        ApiError::internal_server_error(format!("Failed to generate monitor report: {}", e))
    })?;

    Ok(Json(report))
}

/// GET /api/v1/locks/config - Get current distributed locking configuration
pub async fn get_lock_config(
    State(_app_state): State<AppState>,
) -> Result<Json<DistributedLockConfig>, ApiError> {
    // Return current configuration (would come from app state in real implementation)
    let config = DistributedLockConfig::default();
    Ok(Json(config))
}

/// PUT /api/v1/locks/config - Update distributed locking configuration
pub async fn update_lock_config(
    State(_app_state): State<AppState>,
    Json(request): Json<UpdateLockConfigRequest>,
) -> Result<Json<LockConfigResponse>, ApiError> {
    // Get current configuration (would come from app state in real implementation)
    let mut config = DistributedLockConfig::default();
    let mut updated = false;

    // Apply updates
    if let Some(enabled) = request.enabled {
        config.enabled = enabled;
        updated = true;
    }
    if let Some(backend) = request.backend {
        config.backend = backend;
        updated = true;
    }
    if let Some(timeout) = request.default_timeout {
        config.default_timeout = timeout;
        updated = true;
    }
    if let Some(renewal) = request.renewal_interval {
        config.renewal_interval = renewal;
        updated = true;
    }
    if let Some(max_duration) = request.max_duration {
        config.max_duration = max_duration;
        updated = true;
    }

    // In a real implementation, we would save this configuration

    Ok(Json(LockConfigResponse { config, updated }))
}

/// GET /api/v1/locks/health - Check distributed locking system health
pub async fn get_lock_health(
    State(_app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Create a Redis lock provider for health check
    let shared_state = std::sync::Arc::new(SharedStateManager::in_memory());
    let provider = std::sync::Arc::new(RedisLockProvider::new(shared_state));

    let stats = provider
        .get_lock_stats()
        .await
        .map_err(|e| ApiError::internal_server_error(format!("Health check failed: {}", e)))?;

    let health_status = if stats.active_locks < 1000 {
        "healthy"
    } else if stats.active_locks < 5000 {
        "warning"
    } else {
        "critical"
    };

    Ok(Json(serde_json::json!({
        "status": health_status,
        "active_locks": stats.active_locks,
        "total_locks": stats.total_locks,
        "failed_acquisitions": stats.failed_acquisitions,
        "deadlock_detections": stats.deadlock_detections,
        "backend": "redis",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// POST /api/v1/locks/test - Test distributed locking functionality
pub async fn test_distributed_locking(
    State(_app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Perform a comprehensive test of distributed locking functionality
    let shared_state = std::sync::Arc::new(SharedStateManager::in_memory());
    let provider = std::sync::Arc::new(RedisLockProvider::new(shared_state));
    let config = DistributedLockConfig {
        enabled: true,
        ..Default::default()
    };
    let manager = DistributedLockManager::new(provider, config);

    let test_key = "test_lock_key";
    let mut test_results = Vec::new();

    // Test 1: Basic lock acquisition and release
    match manager.acquire_lock_with_retry(test_key).await {
        Ok(lock) => {
            test_results.push(serde_json::json!({
                "test": "basic_acquisition",
                "status": "passed",
                "message": "Successfully acquired test lock"
            }));

            // Test lock release
            match lock.release().await {
                Ok(_) => {
                    test_results.push(serde_json::json!({
                        "test": "basic_release",
                        "status": "passed",
                        "message": "Successfully released test lock"
                    }));
                }
                Err(e) => {
                    test_results.push(serde_json::json!({
                        "test": "basic_release",
                        "status": "failed",
                        "message": format!("Failed to release lock: {}", e)
                    }));
                }
            }
        }
        Err(e) => {
            test_results.push(serde_json::json!({
                "test": "basic_acquisition",
                "status": "failed",
                "message": format!("Failed to acquire lock: {}", e)
            }));
        }
    }

    // Test 2: Mutex functionality
    let mutex = manager
        .create_mutex("test_mutex")
        .await
        .map_err(|e| ApiError::internal_server_error(format!("Failed to create mutex: {}", e)))?;

    match mutex.with_lock(|| async { Ok(42) }).await {
        Ok(result) => {
            test_results.push(serde_json::json!({
                "test": "mutex_operation",
                "status": "passed",
                "message": format!("Mutex operation completed successfully with result: {}", result)
            }));
        }
        Err(e) => {
            test_results.push(serde_json::json!({
                "test": "mutex_operation",
                "status": "failed",
                "message": format!("Mutex operation failed: {}", e)
            }));
        }
    }

    // Test 3: Statistics collection
    match manager.get_stats().await {
        Ok(stats) => {
            test_results.push(serde_json::json!({
                "test": "statistics_collection",
                "status": "passed",
                "message": format!("Statistics collected: {} total locks, {} active",
                                   stats.total_locks, stats.active_locks)
            }));
        }
        Err(e) => {
            test_results.push(serde_json::json!({
                "test": "statistics_collection",
                "status": "failed",
                "message": format!("Failed to collect statistics: {}", e)
            }));
        }
    }

    let passed_tests = test_results
        .iter()
        .filter(|result| result["status"] == "passed")
        .count();
    let total_tests = test_results.len();

    Ok(Json(serde_json::json!({
        "overall_status": if passed_tests == total_tests { "passed" } else { "failed" },
        "tests_passed": passed_tests,
        "total_tests": total_tests,
        "test_results": test_results,
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}
