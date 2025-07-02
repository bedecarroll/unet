use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tokio::time::{interval, timeout};
use uuid::Uuid;

use crate::error::{Error as CoreError, Result};
use crate::shared_state::SharedStateManager;

/// Distributed locking abstraction layer
#[async_trait]
pub trait DistributedLockProvider: Send + Sync {
    /// Acquire a lock with timeout
    async fn acquire_lock(
        &self,
        lock_key: &str,
        lock_value: &str,
        timeout: Duration,
    ) -> Result<bool>;

    /// Release a lock
    async fn release_lock(&self, lock_key: &str, lock_value: &str) -> Result<bool>;

    /// Extend lock duration
    async fn extend_lock(
        &self,
        lock_key: &str,
        lock_value: &str,
        extension: Duration,
    ) -> Result<bool>;

    /// Check if a lock exists
    async fn lock_exists(&self, lock_key: &str) -> Result<bool>;

    /// Get lock metadata
    async fn get_lock_info(&self, lock_key: &str) -> Result<Option<LockInfo>>;

    /// List all active locks
    async fn list_locks(&self) -> Result<Vec<LockInfo>>;

    /// Get lock statistics
    async fn get_lock_stats(&self) -> Result<LockStats>;
}

/// Lock information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockInfo {
    pub key: String,
    pub value: String,
    pub acquired_at: SystemTime,
    pub expires_at: SystemTime,
    pub owner_id: String,
    pub lock_type: LockType,
    pub renewal_count: u64,
}

/// Lock type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LockType {
    Exclusive,
    Shared,
    Leader,
    Critical,
}

/// Lock statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockStats {
    pub total_locks: u64,
    pub active_locks: u64,
    pub expired_locks: u64,
    pub acquisition_attempts: u64,
    pub successful_acquisitions: u64,
    pub failed_acquisitions: u64,
    pub renewals: u64,
    pub deadlock_detections: u64,
    pub average_hold_time: Duration,
}

/// Redis-based distributed lock provider
pub struct RedisLockProvider {
    shared_state: Arc<SharedStateManager>,
    instance_id: String,
    stats: Arc<RwLock<LockStats>>,
}

impl RedisLockProvider {
    pub fn new(shared_state: Arc<SharedStateManager>) -> Self {
        Self {
            shared_state,
            instance_id: Uuid::new_v4().to_string(),
            stats: Arc::new(RwLock::new(LockStats::default())),
        }
    }

    async fn update_stats<F>(&self, update_fn: F)
    where
        F: FnOnce(&mut LockStats),
    {
        let mut stats = self.stats.write().await;
        update_fn(&mut stats);
    }
}

#[async_trait]
impl DistributedLockProvider for RedisLockProvider {
    async fn acquire_lock(
        &self,
        lock_key: &str,
        lock_value: &str,
        timeout: Duration,
    ) -> Result<bool> {
        self.update_stats(|stats| stats.acquisition_attempts += 1)
            .await;

        let lock_info = LockInfo {
            key: lock_key.to_string(),
            value: lock_value.to_string(),
            acquired_at: SystemTime::now(),
            expires_at: SystemTime::now() + timeout,
            owner_id: self.instance_id.clone(),
            lock_type: LockType::Exclusive,
            renewal_count: 0,
        };

        let lock_data = serde_json::to_string(&lock_info)
            .map_err(|e| CoreError::other("serialization", e.to_string()))?;

        let acquired = self
            .shared_state
            .acquire_lock(lock_key, timeout)
            .await
            .unwrap_or(false);

        if acquired {
            // Store detailed lock information
            let _ = self
                .shared_state
                .store_string(
                    &format!("lock_info:{}", lock_key),
                    &lock_data,
                    Some(timeout),
                )
                .await;

            self.update_stats(|stats| {
                stats.successful_acquisitions += 1;
                stats.active_locks += 1;
                stats.total_locks += 1;
            })
            .await;
        } else {
            self.update_stats(|stats| stats.failed_acquisitions += 1)
                .await;
        }

        Ok(acquired)
    }

    async fn release_lock(&self, lock_key: &str, lock_value: &str) -> Result<bool> {
        // Verify lock ownership before releasing
        if let Some(lock_info) = self.get_lock_info(lock_key).await? {
            if lock_info.value != lock_value || lock_info.owner_id != self.instance_id {
                return Ok(false); // Not the owner
            }
        }

        let released = self
            .shared_state
            .release_lock(lock_key)
            .await
            .unwrap_or(false);

        if released {
            // Clean up lock info
            let _ = self
                .shared_state
                .retrieve_string(&format!("lock_info:{}", lock_key))
                .await;

            self.update_stats(|stats| {
                if stats.active_locks > 0 {
                    stats.active_locks -= 1;
                }
            })
            .await;
        }

        Ok(released)
    }

    async fn extend_lock(
        &self,
        lock_key: &str,
        lock_value: &str,
        extension: Duration,
    ) -> Result<bool> {
        // Verify lock ownership
        if let Some(mut lock_info) = self.get_lock_info(lock_key).await? {
            if lock_info.value != lock_value || lock_info.owner_id != self.instance_id {
                return Ok(false); // Not the owner
            }

            // Update lock info with new expiration
            lock_info.expires_at = SystemTime::now() + extension;
            lock_info.renewal_count += 1;

            let lock_data = serde_json::to_string(&lock_info)
                .map_err(|e| CoreError::other("serialization", e.to_string()))?;

            let _ = self
                .shared_state
                .store_string(
                    &format!("lock_info:{}", lock_key),
                    &lock_data,
                    Some(extension),
                )
                .await;

            self.update_stats(|stats| stats.renewals += 1).await;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn lock_exists(&self, lock_key: &str) -> Result<bool> {
        self.shared_state
            .retrieve_string(&format!("lock:{}", lock_key))
            .await
            .map(|result| result.is_some())
    }

    async fn get_lock_info(&self, lock_key: &str) -> Result<Option<LockInfo>> {
        if let Some(lock_data) = self
            .shared_state
            .retrieve_string(&format!("lock_info:{}", lock_key))
            .await?
        {
            match serde_json::from_str::<LockInfo>(&lock_data) {
                Ok(lock_info) => {
                    // Check if lock has expired
                    if SystemTime::now() > lock_info.expires_at {
                        self.update_stats(|stats| stats.expired_locks += 1).await;
                        Ok(None)
                    } else {
                        Ok(Some(lock_info))
                    }
                }
                Err(_) => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    async fn list_locks(&self) -> Result<Vec<LockInfo>> {
        // This is a simplified implementation
        // In a real Redis implementation, we would use SCAN to find all lock keys
        Ok(Vec::new())
    }

    async fn get_lock_stats(&self) -> Result<LockStats> {
        let stats = self.stats.read().await;
        Ok(stats.clone())
    }
}

/// PostgreSQL advisory locks provider
pub struct PostgresLockProvider {
    // Database connection would go here
    stats: Arc<RwLock<LockStats>>,
}

impl PostgresLockProvider {
    pub fn new() -> Self {
        Self {
            stats: Arc::new(RwLock::new(LockStats::default())),
        }
    }

    async fn update_stats<F>(&self, update_fn: F)
    where
        F: FnOnce(&mut LockStats),
    {
        let mut stats = self.stats.write().await;
        update_fn(&mut stats);
    }
}

#[async_trait]
impl DistributedLockProvider for PostgresLockProvider {
    async fn acquire_lock(
        &self,
        _lock_key: &str,
        _lock_value: &str,
        _timeout: Duration,
    ) -> Result<bool> {
        // TODO: Implement PostgreSQL advisory locks
        // This would use pg_advisory_lock functions
        self.update_stats(|stats| stats.acquisition_attempts += 1)
            .await;
        Err(CoreError::not_implemented(
            "PostgreSQL advisory locks",
            "PostgreSQL integration not yet implemented",
        ))
    }

    async fn release_lock(&self, _lock_key: &str, _lock_value: &str) -> Result<bool> {
        // TODO: Implement PostgreSQL advisory lock release
        Err(CoreError::not_implemented(
            "PostgreSQL advisory locks",
            "PostgreSQL integration not yet implemented",
        ))
    }

    async fn extend_lock(
        &self,
        _lock_key: &str,
        _lock_value: &str,
        _extension: Duration,
    ) -> Result<bool> {
        // TODO: Implement PostgreSQL advisory lock extension
        Err(CoreError::not_implemented(
            "PostgreSQL advisory locks",
            "PostgreSQL integration not yet implemented",
        ))
    }

    async fn lock_exists(&self, _lock_key: &str) -> Result<bool> {
        // TODO: Implement PostgreSQL advisory lock check
        Err(CoreError::not_implemented(
            "PostgreSQL advisory locks",
            "PostgreSQL integration not yet implemented",
        ))
    }

    async fn get_lock_info(&self, _lock_key: &str) -> Result<Option<LockInfo>> {
        // TODO: Implement PostgreSQL advisory lock info
        Err(CoreError::not_implemented(
            "PostgreSQL advisory locks",
            "PostgreSQL integration not yet implemented",
        ))
    }

    async fn list_locks(&self) -> Result<Vec<LockInfo>> {
        // TODO: Implement PostgreSQL advisory lock listing
        Err(CoreError::not_implemented(
            "PostgreSQL advisory locks",
            "PostgreSQL integration not yet implemented",
        ))
    }

    async fn get_lock_stats(&self) -> Result<LockStats> {
        let stats = self.stats.read().await;
        Ok(stats.clone())
    }
}

/// Distributed lock manager with retry and timeout mechanisms
pub struct DistributedLockManager {
    provider: Arc<dyn DistributedLockProvider>,
    config: DistributedLockConfig,
    deadlock_detector: Arc<DeadlockDetector>,
}

impl DistributedLockManager {
    pub fn new(provider: Arc<dyn DistributedLockProvider>, config: DistributedLockConfig) -> Self {
        Self {
            provider,
            config: config.clone(),
            deadlock_detector: Arc::new(DeadlockDetector::new(config)),
        }
    }

    /// Acquire a lock with retry logic
    pub async fn acquire_lock_with_retry(&self, lock_key: &str) -> Result<DistributedLock> {
        let lock_value = Uuid::new_v4().to_string();
        let mut attempt = 0;
        let mut delay = Duration::from_millis(self.config.retry.initial_delay);

        while attempt < self.config.retry.max_attempts {
            attempt += 1;

            // Check for potential deadlock
            self.deadlock_detector
                .check_potential_deadlock(lock_key)
                .await?;

            match timeout(
                Duration::from_secs(self.config.default_timeout),
                self.provider.acquire_lock(
                    lock_key,
                    &lock_value,
                    Duration::from_secs(self.config.default_timeout),
                ),
            )
            .await
            {
                Ok(Ok(true)) => {
                    // Record successful acquisition
                    self.deadlock_detector
                        .record_lock_acquisition(lock_key)
                        .await;

                    return Ok(DistributedLock::new(
                        self.provider.clone(),
                        lock_key.to_string(),
                        lock_value,
                        Duration::from_secs(self.config.default_timeout),
                        self.config.renewal_interval,
                    ));
                }
                Ok(Ok(false)) => {
                    // Lock acquisition failed, retry with backoff
                    if attempt < self.config.retry.max_attempts {
                        tokio::time::sleep(delay).await;
                        delay = std::cmp::min(
                            Duration::from_millis(
                                (delay.as_millis() as f64 * self.config.retry.backoff_multiplier)
                                    as u64,
                            ),
                            Duration::from_millis(self.config.retry.max_delay),
                        );
                    }
                }
                Ok(Err(e)) => return Err(e),
                Err(_) => {
                    return Err(CoreError::other(
                        "lock_timeout",
                        "Lock acquisition timed out".to_string(),
                    ));
                }
            }
        }

        Err(CoreError::other(
            "lock_acquisition_failed",
            format!(
                "Failed to acquire lock '{}' after {} attempts",
                lock_key, self.config.retry.max_attempts
            ),
        ))
    }

    /// Create a distributed mutex for critical sections
    pub async fn create_mutex(&self, mutex_key: &str) -> Result<DistributedMutex> {
        Ok(DistributedMutex::new(
            self.provider.clone(),
            mutex_key.to_string(),
            self.config.clone(),
        ))
    }

    /// Get lock statistics
    pub async fn get_stats(&self) -> Result<LockStats> {
        self.provider.get_lock_stats().await
    }

    /// List all active locks
    pub async fn list_active_locks(&self) -> Result<Vec<LockInfo>> {
        self.provider.list_locks().await
    }

    /// Monitor locks for potential issues
    pub async fn monitor_locks(&self) -> Result<LockMonitorReport> {
        let locks = self.list_active_locks().await?;
        let stats = self.get_stats().await?;
        let deadlock_info = self.deadlock_detector.get_deadlock_report().await;

        Ok(LockMonitorReport {
            timestamp: SystemTime::now(),
            total_locks: locks.len(),
            expired_locks: locks
                .iter()
                .filter(|lock| SystemTime::now() > lock.expires_at)
                .count(),
            long_running_locks: locks
                .iter()
                .filter(|lock| {
                    SystemTime::now()
                        .duration_since(lock.acquired_at)
                        .unwrap_or(Duration::ZERO)
                        > Duration::from_secs(300)
                })
                .count(),
            stats,
            deadlock_info,
            recommendations: self.generate_recommendations(&locks).await,
        })
    }

    async fn generate_recommendations(&self, locks: &[LockInfo]) -> Vec<String> {
        let mut recommendations = Vec::new();

        let long_running_count = locks
            .iter()
            .filter(|lock| {
                SystemTime::now()
                    .duration_since(lock.acquired_at)
                    .unwrap_or(Duration::ZERO)
                    > Duration::from_secs(300)
            })
            .count();

        if long_running_count > 0 {
            recommendations.push(format!(
                "Found {} long-running locks (>5 minutes). Consider reviewing lock usage patterns.",
                long_running_count
            ));
        }

        let expired_count = locks
            .iter()
            .filter(|lock| SystemTime::now() > lock.expires_at)
            .count();

        if expired_count > 0 {
            recommendations.push(format!(
                "Found {} expired locks. Consider implementing automatic cleanup.",
                expired_count
            ));
        }

        if locks.len() > 100 {
            recommendations.push(
                "High number of active locks detected. Consider optimizing lock granularity."
                    .to_string(),
            );
        }

        recommendations
    }
}

/// RAII-style distributed lock with automatic renewal
pub struct DistributedLock {
    provider: Arc<dyn DistributedLockProvider>,
    key: String,
    value: String,
    _renewal_handle: tokio::task::JoinHandle<()>,
    acquired: Arc<tokio::sync::Mutex<bool>>,
}

impl DistributedLock {
    fn new(
        provider: Arc<dyn DistributedLockProvider>,
        key: String,
        value: String,
        duration: Duration,
        renewal_interval: u64,
    ) -> Self {
        let acquired = Arc::new(tokio::sync::Mutex::new(true));
        let renewal_acquired = acquired.clone();
        let renewal_provider = provider.clone();
        let renewal_key = key.clone();
        let renewal_value = value.clone();

        // Start renewal task
        let renewal_handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(renewal_interval));

            loop {
                interval.tick().await;

                let is_acquired = {
                    let acquired_lock = renewal_acquired.lock().await;
                    *acquired_lock
                };

                if !is_acquired {
                    break;
                }

                match renewal_provider
                    .extend_lock(&renewal_key, &renewal_value, duration)
                    .await
                {
                    Ok(true) => {
                        tracing::debug!("Successfully renewed lock: {}", renewal_key);
                    }
                    Ok(false) => {
                        tracing::warn!("Failed to renew lock (not owner): {}", renewal_key);
                        let mut acquired_lock = renewal_acquired.lock().await;
                        *acquired_lock = false;
                        break;
                    }
                    Err(e) => {
                        tracing::error!("Error renewing lock {}: {}", renewal_key, e);
                        let mut acquired_lock = renewal_acquired.lock().await;
                        *acquired_lock = false;
                        break;
                    }
                }
            }
        });

        Self {
            provider,
            key,
            value,
            _renewal_handle: renewal_handle,
            acquired,
        }
    }

    /// Check if the lock is still held
    pub async fn is_held(&self) -> bool {
        let acquired_lock = self.acquired.lock().await;
        *acquired_lock
    }

    /// Manually release the lock
    pub async fn release(self) -> Result<()> {
        {
            let mut acquired_lock = self.acquired.lock().await;
            *acquired_lock = false;
        }

        self.provider.release_lock(&self.key, &self.value).await?;
        Ok(())
    }
}

impl Drop for DistributedLock {
    fn drop(&mut self) {
        // Best effort cleanup - renewal task will stop automatically
        tracing::debug!("Dropping distributed lock: {}", self.key);
    }
}

/// Distributed mutex for critical sections
pub struct DistributedMutex {
    provider: Arc<dyn DistributedLockProvider>,
    key: String,
    config: DistributedLockConfig,
}

impl DistributedMutex {
    fn new(
        provider: Arc<dyn DistributedLockProvider>,
        key: String,
        config: DistributedLockConfig,
    ) -> Self {
        Self {
            provider,
            key,
            config,
        }
    }

    /// Execute a closure while holding the mutex
    pub async fn with_lock<F, T, Fut>(&self, f: F) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let lock_value = Uuid::new_v4().to_string();

        let acquired = self
            .provider
            .acquire_lock(
                &self.key,
                &lock_value,
                Duration::from_secs(self.config.default_timeout),
            )
            .await?;

        if !acquired {
            return Err(CoreError::other(
                "mutex_acquisition_failed",
                format!("Failed to acquire mutex: {}", self.key),
            ));
        }

        let result = f().await;

        // Always try to release the lock
        let _ = self.provider.release_lock(&self.key, &lock_value).await;

        result
    }
}

/// Leader election implementation
pub struct LeaderElection {
    provider: Arc<dyn DistributedLockProvider>,
    election_key: String,
    instance_id: String,
    lease_duration: Duration,
    current_lock: Arc<tokio::sync::Mutex<Option<DistributedLock>>>,
}

impl LeaderElection {
    pub fn new(
        provider: Arc<dyn DistributedLockProvider>,
        election_key: String,
        lease_duration: Duration,
    ) -> Self {
        Self {
            provider,
            election_key,
            instance_id: Uuid::new_v4().to_string(),
            lease_duration,
            current_lock: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }

    /// Attempt to become leader
    pub async fn try_become_leader(&self) -> Result<bool> {
        let acquired = self
            .provider
            .acquire_lock(&self.election_key, &self.instance_id, self.lease_duration)
            .await?;

        if acquired {
            let lock = DistributedLock::new(
                self.provider.clone(),
                self.election_key.clone(),
                self.instance_id.clone(),
                self.lease_duration,
                10, // Renew every 10 seconds
            );

            let mut current_lock = self.current_lock.lock().await;
            *current_lock = Some(lock);
        }

        Ok(acquired)
    }

    /// Check if this instance is the current leader
    pub async fn is_leader(&self) -> bool {
        let current_lock = self.current_lock.lock().await;
        if let Some(lock) = &*current_lock {
            lock.is_held().await
        } else {
            false
        }
    }

    /// Resign from leadership
    pub async fn resign(&self) -> Result<()> {
        let mut current_lock = self.current_lock.lock().await;
        if let Some(lock) = current_lock.take() {
            lock.release().await?;
        }
        Ok(())
    }
}

/// Deadlock detection system
pub struct DeadlockDetector {
    lock_dependencies: Arc<RwLock<HashMap<String, Vec<String>>>>,
    lock_holders: Arc<RwLock<HashMap<String, String>>>,
    deadlock_count: Arc<RwLock<u64>>,
}

impl DeadlockDetector {
    pub fn new(_config: DistributedLockConfig) -> Self {
        Self {
            lock_dependencies: Arc::new(RwLock::new(HashMap::new())),
            lock_holders: Arc::new(RwLock::new(HashMap::new())),
            deadlock_count: Arc::new(RwLock::new(0)),
        }
    }

    /// Check for potential deadlock before acquiring a lock
    pub async fn check_potential_deadlock(&self, lock_key: &str) -> Result<()> {
        let dependencies = self.lock_dependencies.read().await;
        let holders = self.lock_holders.read().await;

        // Simple cycle detection in lock dependency graph
        if self.has_cycle(&dependencies, &holders, lock_key).await {
            let mut deadlock_count = self.deadlock_count.write().await;
            *deadlock_count += 1;

            return Err(CoreError::other(
                "potential_deadlock",
                format!("Potential deadlock detected involving lock: {}", lock_key),
            ));
        }

        Ok(())
    }

    /// Record lock acquisition for deadlock tracking
    pub async fn record_lock_acquisition(&self, lock_key: &str) {
        let mut holders = self.lock_holders.write().await;
        holders.insert(lock_key.to_string(), "current_thread".to_string());
    }

    async fn has_cycle(
        &self,
        _dependencies: &HashMap<String, Vec<String>>,
        _holders: &HashMap<String, String>,
        _lock_key: &str,
    ) -> bool {
        // Simplified deadlock detection
        // In a real implementation, this would perform proper cycle detection
        false
    }

    /// Get deadlock detection report
    pub async fn get_deadlock_report(&self) -> DeadlockInfo {
        let deadlock_count = self.deadlock_count.read().await;
        DeadlockInfo {
            deadlock_detections: *deadlock_count,
            last_detection: None,   // Would track actual timestamp
            active_dependencies: 0, // Would count current dependencies
        }
    }
}

/// Deadlock detection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadlockInfo {
    pub deadlock_detections: u64,
    pub last_detection: Option<SystemTime>,
    pub active_dependencies: u64,
}

/// Lock monitoring report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockMonitorReport {
    pub timestamp: SystemTime,
    pub total_locks: usize,
    pub expired_locks: usize,
    pub long_running_locks: usize,
    pub stats: LockStats,
    pub deadlock_info: DeadlockInfo,
    pub recommendations: Vec<String>,
}

/// Distributed locking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedLockConfig {
    pub enabled: bool,
    pub backend: String,
    pub default_timeout: u64,
    pub renewal_interval: u64,
    pub max_duration: u64,
    pub retry: LockRetryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockRetryConfig {
    pub max_attempts: u32,
    pub initial_delay: u64,
    pub max_delay: u64,
    pub backoff_multiplier: f64,
}

impl Default for DistributedLockConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            backend: "redis".to_string(),
            default_timeout: 30,
            renewal_interval: 10,
            max_duration: 300,
            retry: LockRetryConfig::default(),
        }
    }
}

impl Default for LockRetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: 100,
            max_delay: 1000,
            backoff_multiplier: 2.0,
        }
    }
}

impl Default for LockStats {
    fn default() -> Self {
        Self {
            total_locks: 0,
            active_locks: 0,
            expired_locks: 0,
            acquisition_attempts: 0,
            successful_acquisitions: 0,
            failed_acquisitions: 0,
            renewals: 0,
            deadlock_detections: 0,
            average_hold_time: Duration::from_secs(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared_state::SharedStateManager;

    #[tokio::test]
    async fn test_redis_lock_provider() {
        let shared_state = Arc::new(SharedStateManager::in_memory());
        let provider = RedisLockProvider::new(shared_state);

        // Test basic lock acquisition and release
        let acquired = provider
            .acquire_lock("test_lock", "test_value", Duration::from_secs(10))
            .await
            .unwrap();
        assert!(acquired);

        let exists = provider.lock_exists("test_lock").await.unwrap();
        assert!(exists);

        let released = provider
            .release_lock("test_lock", "test_value")
            .await
            .unwrap();
        assert!(released);
    }

    #[tokio::test]
    async fn test_distributed_lock_manager() {
        let shared_state = Arc::new(SharedStateManager::in_memory());
        let provider = Arc::new(RedisLockProvider::new(shared_state));
        let config = DistributedLockConfig::default();
        let manager = DistributedLockManager::new(provider, config);

        // Test lock acquisition with retry
        let lock = manager.acquire_lock_with_retry("test_lock").await.unwrap();
        assert!(lock.is_held().await);

        // Test mutex
        let mutex = manager.create_mutex("test_mutex").await.unwrap();
        let result = mutex.with_lock(|| async { Ok(42) }).await.unwrap();
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_leader_election() {
        let shared_state = Arc::new(SharedStateManager::in_memory());
        let provider = Arc::new(RedisLockProvider::new(shared_state));

        let election = LeaderElection::new(
            provider,
            "test_election".to_string(),
            Duration::from_secs(10),
        );

        let became_leader = election.try_become_leader().await.unwrap();
        assert!(became_leader);
        assert!(election.is_leader().await);

        election.resign().await.unwrap();
        assert!(!election.is_leader().await);
    }

    #[tokio::test]
    async fn test_lock_stats() {
        let shared_state = Arc::new(SharedStateManager::in_memory());
        let provider = RedisLockProvider::new(shared_state);

        // Acquire and release some locks to generate stats
        let _ = provider
            .acquire_lock("lock1", "value1", Duration::from_secs(10))
            .await;
        let _ = provider
            .acquire_lock("lock2", "value2", Duration::from_secs(10))
            .await;

        let stats = provider.get_lock_stats().await.unwrap();
        assert!(stats.acquisition_attempts >= 2);
    }
}
