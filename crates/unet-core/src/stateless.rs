use crate::error::{Error as CoreError, Result};
use crate::shared_state::SharedStateManager;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for stateless operation mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatelessConfig {
    /// Enable stateless operation mode
    pub enabled: bool,

    /// Shared state backend configuration
    pub shared_state: StatelessSharedState,

    /// Background task coordination configuration
    pub background_tasks: BackgroundTaskConfig,

    /// Distributed locking configuration
    pub distributed_locking: DistributedLockingConfig,

    /// Session management configuration
    pub session_management: SessionManagementConfig,

    /// Rate limiting configuration for stateless mode
    pub rate_limiting: StatelessRateLimitingConfig,
}

/// Shared state configuration for stateless operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatelessSharedState {
    /// Backend type: "memory" or "redis"
    pub backend: String,

    /// Redis connection URL (if using Redis backend)
    pub redis_url: Option<String>,

    /// Connection pool settings for Redis
    pub redis_pool: Option<RedisPoolConfig>,

    /// Key prefix for namespacing
    pub key_prefix: String,

    /// Default TTL for cached items (seconds)
    pub default_ttl: u64,

    /// Enable persistence across restarts
    pub persistence: bool,
}

/// Redis connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisPoolConfig {
    /// Maximum number of connections in the pool
    pub max_connections: u32,

    /// Minimum number of connections to maintain
    pub min_connections: u32,

    /// Connection timeout in seconds
    pub connection_timeout: u64,

    /// Command timeout in seconds
    pub command_timeout: u64,

    /// Maximum lifetime of a connection in seconds
    pub max_lifetime: u64,
}

/// Background task coordination configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundTaskConfig {
    /// Enable distributed background task coordination
    pub enabled: bool,

    /// Task coordination backend: "database" or "redis"
    pub backend: String,

    /// Leader election configuration
    pub leader_election: LeaderElectionConfig,

    /// Task assignment strategy: "round_robin", "least_loaded", "random"
    pub assignment_strategy: String,

    /// Heartbeat interval for task coordination (seconds)
    pub heartbeat_interval: u64,

    /// Task timeout before reassignment (seconds)
    pub task_timeout: u64,
}

/// Leader election configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderElectionConfig {
    /// Enable leader election for background tasks
    pub enabled: bool,

    /// Leader lease duration (seconds)
    pub lease_duration: u64,

    /// Leader renewal interval (seconds)
    pub renewal_interval: u64,

    /// Election timeout (seconds)
    pub election_timeout: u64,
}

/// Distributed locking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedLockingConfig {
    /// Enable distributed locking
    pub enabled: bool,

    /// Lock backend: "redis", "database", "etcd"
    pub backend: String,

    /// Default lock timeout (seconds)
    pub default_timeout: u64,

    /// Lock renewal interval (seconds)
    pub renewal_interval: u64,

    /// Maximum lock duration (seconds)
    pub max_duration: u64,

    /// Lock retry configuration
    pub retry: LockRetryConfig,
}

/// Lock retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockRetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,

    /// Initial retry delay (milliseconds)
    pub initial_delay: u64,

    /// Maximum retry delay (milliseconds)
    pub max_delay: u64,

    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
}

/// Session management configuration for stateless mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionManagementConfig {
    /// Session storage backend: "redis", "database", "jwt"
    pub backend: String,

    /// Session timeout (seconds)
    pub timeout: u64,

    /// Enable session cleanup
    pub cleanup_enabled: bool,

    /// Session cleanup interval (seconds)
    pub cleanup_interval: u64,

    /// JWT configuration for stateless sessions
    pub jwt: JwtSessionConfig,
}

/// JWT session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtSessionConfig {
    /// JWT issuer
    pub issuer: String,

    /// JWT audience
    pub audience: String,

    /// Token expiration time (seconds)
    pub expiration: u64,

    /// Enable token refresh
    pub refresh_enabled: bool,

    /// Refresh token expiration (seconds)
    pub refresh_expiration: u64,
}

/// Rate limiting configuration for stateless mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatelessRateLimitingConfig {
    /// Rate limiting backend: "redis", "database"
    pub backend: String,

    /// Default rate limit window (seconds)
    pub default_window: u64,

    /// Sliding window configuration
    pub sliding_window: SlidingWindowConfig,

    /// Burst handling configuration
    pub burst: BurstConfig,
}

/// Sliding window rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlidingWindowConfig {
    /// Enable sliding window rate limiting
    pub enabled: bool,

    /// Window size (seconds)
    pub window_size: u64,

    /// Number of sub-windows for precision
    pub sub_windows: u32,
}

/// Burst handling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurstConfig {
    /// Enable burst handling
    pub enabled: bool,

    /// Burst capacity multiplier
    pub capacity_multiplier: f64,

    /// Burst refill rate (per second)
    pub refill_rate: f64,
}

impl Default for StatelessConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            shared_state: StatelessSharedState::default(),
            background_tasks: BackgroundTaskConfig::default(),
            distributed_locking: DistributedLockingConfig::default(),
            session_management: SessionManagementConfig::default(),
            rate_limiting: StatelessRateLimitingConfig::default(),
        }
    }
}

impl Default for StatelessSharedState {
    fn default() -> Self {
        Self {
            backend: "memory".to_string(),
            redis_url: None,
            redis_pool: Some(RedisPoolConfig::default()),
            key_prefix: "unet".to_string(),
            default_ttl: 3600, // 1 hour
            persistence: false,
        }
    }
}

impl Default for RedisPoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_connections: 1,
            connection_timeout: 30,
            command_timeout: 10,
            max_lifetime: 3600, // 1 hour
        }
    }
}

impl Default for BackgroundTaskConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            backend: "database".to_string(),
            leader_election: LeaderElectionConfig::default(),
            assignment_strategy: "round_robin".to_string(),
            heartbeat_interval: 30,
            task_timeout: 300, // 5 minutes
        }
    }
}

impl Default for LeaderElectionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            lease_duration: 60,
            renewal_interval: 30,
            election_timeout: 120,
        }
    }
}

impl Default for DistributedLockingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            backend: "redis".to_string(),
            default_timeout: 30,
            renewal_interval: 10,
            max_duration: 300, // 5 minutes
            retry: LockRetryConfig::default(),
        }
    }
}

impl Default for LockRetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: 100, // 100ms
            max_delay: 1000,    // 1 second
            backoff_multiplier: 2.0,
        }
    }
}

impl Default for SessionManagementConfig {
    fn default() -> Self {
        Self {
            backend: "jwt".to_string(),
            timeout: 3600, // 1 hour
            cleanup_enabled: true,
            cleanup_interval: 300, // 5 minutes
            jwt: JwtSessionConfig::default(),
        }
    }
}

impl Default for JwtSessionConfig {
    fn default() -> Self {
        Self {
            issuer: "unet".to_string(),
            audience: "unet-api".to_string(),
            expiration: 3600, // 1 hour
            refresh_enabled: true,
            refresh_expiration: 86400, // 24 hours
        }
    }
}

impl Default for StatelessRateLimitingConfig {
    fn default() -> Self {
        Self {
            backend: "redis".to_string(),
            default_window: 60,
            sliding_window: SlidingWindowConfig::default(),
            burst: BurstConfig::default(),
        }
    }
}

impl Default for SlidingWindowConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            window_size: 60,
            sub_windows: 6,
        }
    }
}

impl Default for BurstConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            capacity_multiplier: 2.0,
            refill_rate: 1.0,
        }
    }
}

/// Stateless operation manager
pub struct StatelessManager {
    config: StatelessConfig,
    shared_state: SharedStateManager,
}

impl StatelessManager {
    /// Create a new stateless manager with configuration
    pub async fn new(config: StatelessConfig) -> Result<Self> {
        let shared_state = match config.shared_state.backend.as_str() {
            "memory" => SharedStateManager::in_memory(),
            "redis" => {
                let redis_url = config
                    .shared_state
                    .redis_url
                    .as_ref()
                    .ok_or_else(|| CoreError::config("Redis URL required for Redis backend"))?;
                SharedStateManager::redis(redis_url)?
            }
            backend => {
                return Err(CoreError::config(format!(
                    "Unsupported shared state backend: {}",
                    backend
                )));
            }
        };

        Ok(Self {
            config,
            shared_state,
        })
    }

    /// Get the shared state manager
    pub fn shared_state(&self) -> &SharedStateManager {
        &self.shared_state
    }

    /// Check if stateless mode is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Create a distributed lock key for the given resource
    pub fn create_lock_key(&self, key: &str) -> String {
        format!("lock:{}", key)
    }

    /// Acquire a distributed lock
    pub async fn acquire_lock(&self, key: &str) -> Result<bool> {
        if !self.config.distributed_locking.enabled {
            return Err(CoreError::config("Distributed locking is not enabled"));
        }

        let timeout = Duration::from_secs(self.config.distributed_locking.default_timeout);
        self.shared_state.acquire_lock(key, timeout).await
    }

    /// Release a distributed lock
    pub async fn release_lock(&self, key: &str) -> Result<bool> {
        if !self.config.distributed_locking.enabled {
            return Err(CoreError::config("Distributed locking is not enabled"));
        }

        self.shared_state.release_lock(key).await
    }

    /// Migrate from stateful to stateless operation
    pub async fn migrate_to_stateless(&self) -> Result<()> {
        // This would implement migration logic for:
        // 1. Moving in-memory caches to shared state
        // 2. Converting background tasks to distributed coordination
        // 3. Migrating rate limiting state
        // 4. Converting session storage

        // For now, this is a placeholder
        if self.config.enabled {
            tracing::info!("Migrating to stateless operation mode");
            // TODO: Implement actual migration logic
        }

        Ok(())
    }

    /// Health check for stateless components
    pub async fn health_check(&self) -> Result<StatelessHealthStatus> {
        let mut status = StatelessHealthStatus {
            enabled: self.config.enabled,
            shared_state_status: "unknown".to_string(),
            background_tasks_status: "unknown".to_string(),
            distributed_locking_status: "unknown".to_string(),
            session_management_status: "unknown".to_string(),
        };

        if self.config.enabled {
            // Check shared state health
            if let Ok(_) = self
                .shared_state
                .store_string("health_check", "ok", Some(Duration::from_secs(1)))
                .await
            {
                status.shared_state_status = "healthy".to_string();
            } else {
                status.shared_state_status = "unhealthy".to_string();
            }

            // Check other components
            status.background_tasks_status = if self.config.background_tasks.enabled {
                "configured".to_string()
            } else {
                "disabled".to_string()
            };

            status.distributed_locking_status = if self.config.distributed_locking.enabled {
                "configured".to_string()
            } else {
                "disabled".to_string()
            };

            status.session_management_status = "configured".to_string();
        } else {
            status.shared_state_status = "disabled".to_string();
            status.background_tasks_status = "disabled".to_string();
            status.distributed_locking_status = "disabled".to_string();
            status.session_management_status = "disabled".to_string();
        }

        Ok(status)
    }
}

/// Health status for stateless components
#[derive(Debug, Serialize, Deserialize)]
pub struct StatelessHealthStatus {
    pub enabled: bool,
    pub shared_state_status: String,
    pub background_tasks_status: String,
    pub distributed_locking_status: String,
    pub session_management_status: String,
}

/// Distributed lock guard for RAII-style lock management
pub struct DistributedLockGuard {
    manager: StatelessManager,
    key: String,
    acquired: bool,
}

impl DistributedLockGuard {
    /// Create a new lock guard and attempt to acquire the lock
    pub async fn new(manager: StatelessManager, key: String) -> Result<Option<Self>> {
        let acquired = manager.acquire_lock(&key).await?;
        if acquired {
            Ok(Some(Self {
                manager,
                key,
                acquired: true,
            }))
        } else {
            Ok(None)
        }
    }

    /// Check if the lock is currently held
    pub fn is_acquired(&self) -> bool {
        self.acquired
    }

    /// Manually release the lock
    pub async fn release(mut self) -> Result<bool> {
        if self.acquired {
            let result = self.manager.release_lock(&self.key).await;
            if result.is_ok() {
                self.acquired = false;
            }
            result
        } else {
            Ok(false)
        }
    }
}

impl Drop for DistributedLockGuard {
    fn drop(&mut self) {
        if self.acquired {
            // Best effort release on drop
            tracing::warn!(
                "Distributed lock guard dropped while acquired: {}",
                self.key
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stateless_config_default() {
        let config = StatelessConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.shared_state.backend, "memory");
        assert_eq!(config.shared_state.key_prefix, "unet");
    }

    #[tokio::test]
    async fn test_stateless_manager_creation() {
        let config = StatelessConfig::default();
        let manager = StatelessManager::new(config).await.unwrap();
        assert!(!manager.is_enabled());
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = StatelessConfig::default();
        let manager = StatelessManager::new(config).await.unwrap();
        let health = manager.health_check().await.unwrap();
        assert!(!health.enabled);
        assert_eq!(health.shared_state_status, "disabled");
    }

    #[tokio::test]
    async fn test_stateless_mode_enabled() {
        let mut config = StatelessConfig::default();
        config.enabled = true;

        let manager = StatelessManager::new(config).await.unwrap();
        assert!(manager.is_enabled());

        let health = manager.health_check().await.unwrap();
        assert!(health.enabled);
        assert_eq!(health.shared_state_status, "healthy");
    }
}
