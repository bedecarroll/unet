use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

use crate::error::Error as CoreError;

/// Shared state abstraction layer for horizontal scaling
/// Provides distributed state management for multiple service instances
#[async_trait]
pub trait SharedStateProvider: Send + Sync {
    /// Store a value with optional expiration
    async fn set(&self, key: &str, value: &[u8], expiry: Option<Duration>)
    -> Result<(), CoreError>;

    /// Retrieve a value by key
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, CoreError>;

    /// Delete a key
    async fn delete(&self, key: &str) -> Result<bool, CoreError>;

    /// Check if a key exists
    async fn exists(&self, key: &str) -> Result<bool, CoreError>;

    /// Increment a counter atomically
    async fn increment(&self, key: &str, delta: i64) -> Result<i64, CoreError>;

    /// Set if not exists (atomic)
    async fn set_nx(
        &self,
        key: &str,
        value: &[u8],
        expiry: Option<Duration>,
    ) -> Result<bool, CoreError>;

    /// Get multiple keys at once
    async fn get_multi(&self, keys: &[String]) -> Result<HashMap<String, Vec<u8>>, CoreError>;

    /// Set multiple keys at once
    async fn set_multi(
        &self,
        values: HashMap<String, Vec<u8>>,
        expiry: Option<Duration>,
    ) -> Result<(), CoreError>;

    /// Expire a key after duration
    async fn expire(&self, key: &str, duration: Duration) -> Result<bool, CoreError>;

    /// Get time to live for a key
    async fn ttl(&self, key: &str) -> Result<Option<Duration>, CoreError>;
}

/// In-memory shared state provider for single-instance deployments
#[derive(Debug)]
pub struct InMemoryStateProvider {
    store: Arc<RwLock<HashMap<String, StateValue>>>,
}

#[derive(Debug, Clone)]
struct StateValue {
    data: Vec<u8>,
    expires_at: Option<SystemTime>,
}

impl StateValue {
    fn new(data: Vec<u8>, expiry: Option<Duration>) -> Self {
        let expires_at = expiry.map(|d| SystemTime::now() + d);
        Self { data, expires_at }
    }

    fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            SystemTime::now() > expires_at
        } else {
            false
        }
    }
}

impl InMemoryStateProvider {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn cleanup_expired(&self) {
        let mut store = self.store.write().await;
        store.retain(|_, value| !value.is_expired());
    }
}

impl Default for InMemoryStateProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SharedStateProvider for InMemoryStateProvider {
    async fn set(
        &self,
        key: &str,
        value: &[u8],
        expiry: Option<Duration>,
    ) -> Result<(), CoreError> {
        let mut store = self.store.write().await;
        store.insert(key.to_string(), StateValue::new(value.to_vec(), expiry));
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, CoreError> {
        self.cleanup_expired().await;
        let store = self.store.read().await;
        if let Some(value) = store.get(key) {
            if value.is_expired() {
                Ok(None)
            } else {
                Ok(Some(value.data.clone()))
            }
        } else {
            Ok(None)
        }
    }

    async fn delete(&self, key: &str) -> Result<bool, CoreError> {
        let mut store = self.store.write().await;
        Ok(store.remove(key).is_some())
    }

    async fn exists(&self, key: &str) -> Result<bool, CoreError> {
        self.cleanup_expired().await;
        let store = self.store.read().await;
        if let Some(value) = store.get(key) {
            Ok(!value.is_expired())
        } else {
            Ok(false)
        }
    }

    async fn increment(&self, key: &str, delta: i64) -> Result<i64, CoreError> {
        let mut store = self.store.write().await;

        let current_value = if let Some(value) = store.get(key) {
            if value.is_expired() {
                0
            } else {
                match String::from_utf8(value.data.clone()) {
                    Ok(s) => s.parse::<i64>().unwrap_or(0),
                    Err(_) => 0,
                }
            }
        } else {
            0
        };

        let new_value = current_value + delta;
        store.insert(
            key.to_string(),
            StateValue::new(new_value.to_string().into_bytes(), None),
        );
        Ok(new_value)
    }

    async fn set_nx(
        &self,
        key: &str,
        value: &[u8],
        expiry: Option<Duration>,
    ) -> Result<bool, CoreError> {
        self.cleanup_expired().await;
        let mut store = self.store.write().await;

        if let Some(existing) = store.get(key) {
            if !existing.is_expired() {
                return Ok(false);
            }
        }

        store.insert(key.to_string(), StateValue::new(value.to_vec(), expiry));
        Ok(true)
    }

    async fn get_multi(&self, keys: &[String]) -> Result<HashMap<String, Vec<u8>>, CoreError> {
        self.cleanup_expired().await;
        let store = self.store.read().await;
        let mut result = HashMap::new();

        for key in keys {
            if let Some(value) = store.get(key) {
                if !value.is_expired() {
                    result.insert(key.clone(), value.data.clone());
                }
            }
        }

        Ok(result)
    }

    async fn set_multi(
        &self,
        values: HashMap<String, Vec<u8>>,
        expiry: Option<Duration>,
    ) -> Result<(), CoreError> {
        let mut store = self.store.write().await;
        for (key, value) in values {
            store.insert(key, StateValue::new(value, expiry));
        }
        Ok(())
    }

    async fn expire(&self, key: &str, duration: Duration) -> Result<bool, CoreError> {
        let mut store = self.store.write().await;
        if let Some(value) = store.get_mut(key) {
            value.expires_at = Some(SystemTime::now() + duration);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn ttl(&self, key: &str) -> Result<Option<Duration>, CoreError> {
        let store = self.store.read().await;
        if let Some(value) = store.get(key) {
            if let Some(expires_at) = value.expires_at {
                if let Ok(duration) = expires_at.duration_since(SystemTime::now()) {
                    Ok(Some(duration))
                } else {
                    Ok(Some(Duration::from_secs(0)))
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}

/// Redis-based shared state provider for distributed deployments
#[derive(Debug)]
pub struct RedisStateProvider {
    // TODO: Implement Redis client integration
    // client: redis::Client,
    _placeholder: bool,
}

impl RedisStateProvider {
    pub fn new(_connection_url: &str) -> Result<Self, CoreError> {
        // TODO: Implement Redis connection
        Ok(Self { _placeholder: true })
    }
}

#[async_trait]
impl SharedStateProvider for RedisStateProvider {
    async fn set(
        &self,
        _key: &str,
        _value: &[u8],
        _expiry: Option<Duration>,
    ) -> Result<(), CoreError> {
        // TODO: Implement Redis SET with optional EXPIRE
        Err(CoreError::not_implemented(
            "Redis state provider",
            "Redis client integration not yet implemented",
        ))
    }

    async fn get(&self, _key: &str) -> Result<Option<Vec<u8>>, CoreError> {
        // TODO: Implement Redis GET
        Err(CoreError::not_implemented(
            "Redis state provider",
            "Redis client integration not yet implemented",
        ))
    }

    async fn delete(&self, _key: &str) -> Result<bool, CoreError> {
        // TODO: Implement Redis DEL
        Err(CoreError::not_implemented(
            "Redis state provider",
            "Redis client integration not yet implemented",
        ))
    }

    async fn exists(&self, _key: &str) -> Result<bool, CoreError> {
        // TODO: Implement Redis EXISTS
        Err(CoreError::not_implemented(
            "Redis state provider",
            "Redis client integration not yet implemented",
        ))
    }

    async fn increment(&self, _key: &str, _delta: i64) -> Result<i64, CoreError> {
        // TODO: Implement Redis INCRBY
        Err(CoreError::not_implemented(
            "Redis state provider",
            "Redis client integration not yet implemented",
        ))
    }

    async fn set_nx(
        &self,
        _key: &str,
        _value: &[u8],
        _expiry: Option<Duration>,
    ) -> Result<bool, CoreError> {
        // TODO: Implement Redis SET NX
        Err(CoreError::not_implemented(
            "Redis state provider",
            "Redis client integration not yet implemented",
        ))
    }

    async fn get_multi(&self, _keys: &[String]) -> Result<HashMap<String, Vec<u8>>, CoreError> {
        // TODO: Implement Redis MGET
        Err(CoreError::not_implemented(
            "Redis state provider",
            "Redis client integration not yet implemented",
        ))
    }

    async fn set_multi(
        &self,
        _values: HashMap<String, Vec<u8>>,
        _expiry: Option<Duration>,
    ) -> Result<(), CoreError> {
        // TODO: Implement Redis MSET
        Err(CoreError::not_implemented(
            "Redis state provider",
            "Redis client integration not yet implemented",
        ))
    }

    async fn expire(&self, _key: &str, _duration: Duration) -> Result<bool, CoreError> {
        // TODO: Implement Redis EXPIRE
        Err(CoreError::not_implemented(
            "Redis state provider",
            "Redis client integration not yet implemented",
        ))
    }

    async fn ttl(&self, _key: &str) -> Result<Option<Duration>, CoreError> {
        // TODO: Implement Redis TTL
        Err(CoreError::not_implemented(
            "Redis state provider",
            "Redis client integration not yet implemented",
        ))
    }
}

/// Shared state manager with configurable backend
pub struct SharedStateManager {
    provider: Arc<dyn SharedStateProvider>,
}

impl SharedStateManager {
    pub fn new(provider: Arc<dyn SharedStateProvider>) -> Self {
        Self { provider }
    }

    pub fn in_memory() -> Self {
        Self::new(Arc::new(InMemoryStateProvider::new()))
    }

    pub fn redis(connection_url: &str) -> Result<Self, CoreError> {
        Ok(Self::new(Arc::new(RedisStateProvider::new(
            connection_url,
        )?)))
    }

    /// High-level methods for common use cases

    /// Store serializable object
    pub async fn store_json<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        expiry: Option<Duration>,
    ) -> Result<(), CoreError> {
        let json_data = serde_json::to_vec(value)
            .map_err(|e| CoreError::other("serialization", e.to_string()))?;
        self.provider.set(key, &json_data, expiry).await
    }

    /// Retrieve and deserialize object
    pub async fn retrieve_json<T: for<'de> Deserialize<'de>>(
        &self,
        key: &str,
    ) -> Result<Option<T>, CoreError> {
        if let Some(data) = self.provider.get(key).await? {
            let value = serde_json::from_slice(&data)
                .map_err(|e| CoreError::other("serialization", e.to_string()))?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// Store string value
    pub async fn store_string(
        &self,
        key: &str,
        value: &str,
        expiry: Option<Duration>,
    ) -> Result<(), CoreError> {
        self.provider.set(key, value.as_bytes(), expiry).await
    }

    /// Retrieve string value
    pub async fn retrieve_string(&self, key: &str) -> Result<Option<String>, CoreError> {
        if let Some(data) = self.provider.get(key).await? {
            String::from_utf8(data)
                .map(Some)
                .map_err(|e| CoreError::other("string_conversion", e.to_string()))
        } else {
            Ok(None)
        }
    }

    /// Distributed lock implementation
    pub async fn acquire_lock(
        &self,
        lock_key: &str,
        duration: Duration,
    ) -> Result<bool, CoreError> {
        let lock_value = format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
        self.provider
            .set_nx(
                &format!("lock:{}", lock_key),
                lock_value.as_bytes(),
                Some(duration),
            )
            .await
    }

    /// Release distributed lock
    pub async fn release_lock(&self, lock_key: &str) -> Result<bool, CoreError> {
        self.provider.delete(&format!("lock:{}", lock_key)).await
    }

    /// Rate limiting helper
    pub async fn check_rate_limit(
        &self,
        identifier: &str,
        limit: u64,
        window: Duration,
    ) -> Result<bool, CoreError> {
        let key = format!("rate_limit:{}", identifier);
        let current = self.provider.increment(&key, 1).await?;

        if current == 1 {
            // First request in window, set expiration
            self.provider.expire(&key, window).await?;
        }

        Ok(current <= limit as i64)
    }

    /// Cache pattern helper
    pub async fn cache_get_or_set<T, F, Fut>(
        &self,
        key: &str,
        compute_fn: F,
        expiry: Duration,
    ) -> Result<T, CoreError>
    where
        T: Serialize + for<'de> Deserialize<'de>,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, CoreError>>,
    {
        if let Some(cached) = self.retrieve_json::<T>(key).await? {
            return Ok(cached);
        }

        let computed = compute_fn().await?;
        self.store_json(key, &computed, Some(expiry)).await?;
        Ok(computed)
    }
}

/// Configuration for shared state backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedStateConfig {
    /// Backend type: "memory" or "redis"
    pub backend: String,

    /// Redis connection URL (if using Redis backend)
    pub redis_url: Option<String>,

    /// Default expiration for cached items
    pub default_expiry_seconds: u64,

    /// Key prefix for namespacing
    pub key_prefix: String,
}

impl Default for SharedStateConfig {
    fn default() -> Self {
        Self {
            backend: "memory".to_string(),
            redis_url: None,
            default_expiry_seconds: 3600, // 1 hour
            key_prefix: "unet".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_in_memory_provider_basic_operations() {
        let provider = InMemoryStateProvider::new();

        // Test set and get
        provider.set("test_key", b"test_value", None).await.unwrap();
        let value = provider.get("test_key").await.unwrap();
        assert_eq!(value, Some(b"test_value".to_vec()));

        // Test exists
        assert!(provider.exists("test_key").await.unwrap());
        assert!(!provider.exists("nonexistent").await.unwrap());

        // Test delete
        assert!(provider.delete("test_key").await.unwrap());
        assert!(!provider.exists("test_key").await.unwrap());
    }

    #[tokio::test]
    async fn test_expiration() {
        let provider = InMemoryStateProvider::new();

        // Set with short expiration
        provider
            .set("expire_key", b"value", Some(Duration::from_millis(100)))
            .await
            .unwrap();
        assert!(provider.exists("expire_key").await.unwrap());

        // Wait for expiration
        sleep(Duration::from_millis(150)).await;
        assert!(!provider.exists("expire_key").await.unwrap());
    }

    #[tokio::test]
    async fn test_increment() {
        let provider = InMemoryStateProvider::new();

        // Test increment on new key
        let val1 = provider.increment("counter", 5).await.unwrap();
        assert_eq!(val1, 5);

        // Test increment on existing key
        let val2 = provider.increment("counter", 3).await.unwrap();
        assert_eq!(val2, 8);
    }

    #[tokio::test]
    async fn test_set_nx() {
        let provider = InMemoryStateProvider::new();

        // Test set if not exists - should succeed
        assert!(provider.set_nx("nx_key", b"value1", None).await.unwrap());

        // Test set if not exists - should fail
        assert!(!provider.set_nx("nx_key", b"value2", None).await.unwrap());

        // Verify original value
        let value = provider.get("nx_key").await.unwrap();
        assert_eq!(value, Some(b"value1".to_vec()));
    }

    #[tokio::test]
    async fn test_shared_state_manager() {
        let manager = SharedStateManager::in_memory();

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct TestData {
            name: String,
            value: i32,
        }

        let test_data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        // Test JSON storage and retrieval
        manager
            .store_json("test_object", &test_data, None)
            .await
            .unwrap();
        let retrieved: TestData = manager.retrieve_json("test_object").await.unwrap().unwrap();
        assert_eq!(retrieved, test_data);

        // Test string storage and retrieval
        manager
            .store_string("test_string", "hello world", None)
            .await
            .unwrap();
        let retrieved_string = manager
            .retrieve_string("test_string")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(retrieved_string, "hello world");
    }

    #[tokio::test]
    async fn test_distributed_lock() {
        let manager = SharedStateManager::in_memory();

        // Acquire lock
        assert!(
            manager
                .acquire_lock("test_lock", Duration::from_secs(1))
                .await
                .unwrap()
        );

        // Try to acquire same lock - should fail
        assert!(
            !manager
                .acquire_lock("test_lock", Duration::from_secs(1))
                .await
                .unwrap()
        );

        // Release lock
        assert!(manager.release_lock("test_lock").await.unwrap());

        // Should be able to acquire again
        assert!(
            manager
                .acquire_lock("test_lock", Duration::from_secs(1))
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let manager = SharedStateManager::in_memory();

        // Test rate limiting
        assert!(
            manager
                .check_rate_limit("user1", 2, Duration::from_secs(60))
                .await
                .unwrap()
        );
        assert!(
            manager
                .check_rate_limit("user1", 2, Duration::from_secs(60))
                .await
                .unwrap()
        );
        assert!(
            !manager
                .check_rate_limit("user1", 2, Duration::from_secs(60))
                .await
                .unwrap()
        );
    }
}
