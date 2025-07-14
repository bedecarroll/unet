//! Policy caching logic and types

use crate::policy::PolicyRule;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

/// Cached policy with metadata
#[derive(Debug, Clone)]
pub struct CachedPolicy {
    /// Parsed policy rules
    pub rules: Vec<PolicyRule>,
    /// File modification time when cached
    pub mtime: SystemTime,
    /// Cache timestamp
    pub cached_at: SystemTime,
}

/// Statistics about the policy cache state
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Total number of entries in the cache
    pub total_entries: usize,
    /// Number of expired cache entries
    pub expired_entries: usize,
    /// Number of valid cache entries
    pub valid_entries: usize,
}

impl CachedPolicy {
    /// Create a new cached policy entry
    #[must_use]
    pub fn new(rules: Vec<PolicyRule>, mtime: SystemTime) -> Self {
        Self {
            rules,
            mtime,
            cached_at: SystemTime::now(),
        }
    }

    /// Check if the cached policy is still valid based on TTL and file modification time
    #[must_use]
    pub fn is_valid(&self, ttl: std::time::Duration, current_mtime: SystemTime) -> bool {
        // Check if cache has expired
        let cache_expired = SystemTime::now()
            .duration_since(self.cached_at)
            .map_or(true, |age| age > ttl);

        // Check if file has been modified
        let file_modified = current_mtime != self.mtime;

        !cache_expired && !file_modified
    }
}

/// Cache management operations
pub trait CacheManager {
    /// Get cache statistics
    fn get_cache_stats(&self, ttl: std::time::Duration) -> CacheStats;

    /// Clear expired cache entries
    fn clear_expired_cache(&mut self, ttl: std::time::Duration) -> usize;

    /// Clear all cache entries
    fn clear_cache(&mut self);
}

impl CacheManager for HashMap<PathBuf, CachedPolicy> {
    fn get_cache_stats(&self, ttl: std::time::Duration) -> CacheStats {
        let now = SystemTime::now();
        let mut expired_count = 0;
        let mut valid_count = 0;

        for cached_policy in self.values() {
            let cache_expired = now
                .duration_since(cached_policy.cached_at)
                .map_or(true, |age| age > ttl);

            if cache_expired {
                expired_count += 1;
            } else {
                valid_count += 1;
            }
        }

        CacheStats {
            total_entries: self.len(),
            expired_entries: expired_count,
            valid_entries: valid_count,
        }
    }

    fn clear_expired_cache(&mut self, ttl: std::time::Duration) -> usize {
        let now = SystemTime::now();
        let initial_count = self.len();

        self.retain(|_, cached_policy| {
            let cache_expired = now
                .duration_since(cached_policy.cached_at)
                .map_or(true, |age| age > ttl);
            !cache_expired
        });

        initial_count - self.len()
    }

    fn clear_cache(&mut self) {
        self.clear();
    }
}
