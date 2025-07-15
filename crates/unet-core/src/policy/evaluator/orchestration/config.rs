//! Orchestration configuration types
//!
//! Contains configuration structures for policy orchestration settings
//! including concurrency, caching, and timeout parameters.

use std::time::Duration;

/// Configuration for policy orchestration engine
#[derive(Debug, Clone)]
pub struct OrchestrationConfig {
    /// Maximum number of concurrent evaluations
    pub max_concurrent: usize,
    /// Cache TTL for evaluation results
    pub cache_ttl: Duration,
    /// Batch timeout before forced evaluation
    pub batch_timeout: Duration,
    /// Enable result caching
    pub enable_caching: bool,
}

impl Default for OrchestrationConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 10,
            cache_ttl: Duration::from_secs(300),
            batch_timeout: Duration::from_secs(30),
            enable_caching: true,
        }
    }
}
