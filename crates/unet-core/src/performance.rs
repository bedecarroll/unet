//! Performance optimization module for μNet Core
//!
//! This module provides performance profiling, connection pooling, caching,
//! async processing optimization, and benchmarking capabilities.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::{RwLock, Semaphore};
use tracing::{debug, info, warn};

/// Performance metrics for critical operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Operation name
    pub operation: String,
    /// Total execution count
    pub count: u64,
    /// Total execution time in milliseconds
    pub total_duration_ms: u64,
    /// Average execution time in milliseconds
    pub avg_duration_ms: f64,
    /// Minimum execution time in milliseconds
    pub min_duration_ms: u64,
    /// Maximum execution time in milliseconds
    pub max_duration_ms: u64,
    /// 95th percentile execution time in milliseconds
    pub p95_duration_ms: u64,
    /// 99th percentile execution time in milliseconds
    pub p99_duration_ms: u64,
    /// Operations per second
    pub ops_per_second: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            operation: String::new(),
            count: 0,
            total_duration_ms: 0,
            avg_duration_ms: 0.0,
            min_duration_ms: u64::MAX,
            max_duration_ms: 0,
            p95_duration_ms: 0,
            p99_duration_ms: 0,
            ops_per_second: 0.0,
        }
    }
}

/// Performance profiler for tracking critical path operations
#[derive(Debug)]
pub struct PerformanceProfiler {
    metrics: Arc<RwLock<HashMap<String, PerformanceMetrics>>>,
    operation_history: Arc<RwLock<HashMap<String, Vec<Duration>>>>,
    start_time: Instant,
}

impl Default for PerformanceProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceProfiler {
    /// Create a new performance profiler
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            operation_history: Arc::new(RwLock::new(HashMap::new())),
            start_time: Instant::now(),
        }
    }

    /// Start timing an operation
    pub fn start_operation(&self, operation: &str) -> OperationTimer {
        OperationTimer::new(operation.to_string(), self.clone())
    }

    /// Record an operation duration
    pub async fn record_operation(&self, operation: &str, duration: Duration) {
        let duration_ms = duration.as_millis() as u64;

        // Update metrics
        let mut metrics = self.metrics.write().await;
        let metric = metrics.entry(operation.to_string()).or_default();
        metric.operation = operation.to_string();
        metric.count += 1;
        metric.total_duration_ms += duration_ms;
        metric.avg_duration_ms = metric.total_duration_ms as f64 / metric.count as f64;
        metric.min_duration_ms = metric.min_duration_ms.min(duration_ms);
        metric.max_duration_ms = metric.max_duration_ms.max(duration_ms);

        // Calculate ops per second
        let elapsed_seconds = self.start_time.elapsed().as_secs_f64();
        if elapsed_seconds > 0.0 {
            metric.ops_per_second = metric.count as f64 / elapsed_seconds;
        }

        // Update operation history for percentile calculations
        let mut history = self.operation_history.write().await;
        let op_history = history.entry(operation.to_string()).or_default();
        op_history.push(duration);

        // Keep only last 1000 operations for percentile calculations
        if op_history.len() > 1000 {
            op_history.remove(0);
        }

        // Calculate percentiles
        let mut sorted_durations: Vec<u64> =
            op_history.iter().map(|d| d.as_millis() as u64).collect();
        sorted_durations.sort_unstable();

        if !sorted_durations.is_empty() {
            let p95_idx = (sorted_durations.len() as f64 * 0.95) as usize;
            let p99_idx = (sorted_durations.len() as f64 * 0.99) as usize;
            metric.p95_duration_ms = sorted_durations.get(p95_idx).copied().unwrap_or(0);
            metric.p99_duration_ms = sorted_durations.get(p99_idx).copied().unwrap_or(0);
        }

        debug!("Performance: {} took {}ms", operation, duration_ms);
    }

    /// Get performance metrics for an operation
    pub async fn get_metrics(&self, operation: &str) -> Option<PerformanceMetrics> {
        self.metrics.read().await.get(operation).cloned()
    }

    /// Get all performance metrics
    pub async fn get_all_metrics(&self) -> HashMap<String, PerformanceMetrics> {
        self.metrics.read().await.clone()
    }

    /// Reset metrics for an operation
    pub async fn reset_metrics(&self, operation: &str) {
        let mut metrics = self.metrics.write().await;
        metrics.remove(operation);
        let mut history = self.operation_history.write().await;
        history.remove(operation);
    }

    /// Reset all metrics
    pub async fn reset_all_metrics(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.clear();
        let mut history = self.operation_history.write().await;
        history.clear();
    }
}

impl Clone for PerformanceProfiler {
    fn clone(&self) -> Self {
        Self {
            metrics: Arc::clone(&self.metrics),
            operation_history: Arc::clone(&self.operation_history),
            start_time: self.start_time,
        }
    }
}

/// Timer for measuring operation duration
pub struct OperationTimer {
    operation: String,
    start_time: Instant,
    profiler: PerformanceProfiler,
}

impl OperationTimer {
    fn new(operation: String, profiler: PerformanceProfiler) -> Self {
        Self {
            operation,
            start_time: Instant::now(),
            profiler,
        }
    }

    /// Finish timing and record the operation
    pub async fn finish(self) {
        let duration = self.start_time.elapsed();
        self.profiler
            .record_operation(&self.operation, duration)
            .await;
    }
}

/// Connection pool for managing database connections
pub struct ConnectionPool<T> {
    connections: Arc<Mutex<Vec<T>>>,
    semaphore: Arc<Semaphore>,
    max_connections: usize,
    create_connection: Box<dyn Fn() -> Result<T> + Send + Sync>,
}

impl<T> std::fmt::Debug for ConnectionPool<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectionPool")
            .field("max_connections", &self.max_connections)
            .field("available_permits", &self.semaphore.available_permits())
            .finish()
    }
}

impl<T> ConnectionPool<T>
where
    T: Send + 'static,
{
    /// Create a new connection pool
    pub fn new<F>(max_connections: usize, create_connection: F) -> Self
    where
        F: Fn() -> Result<T> + Send + Sync + 'static,
    {
        Self {
            connections: Arc::new(Mutex::new(Vec::new())),
            semaphore: Arc::new(Semaphore::new(max_connections)),
            max_connections,
            create_connection: Box::new(create_connection),
        }
    }

    /// Get a connection from the pool
    pub async fn get_connection(&self) -> Result<PooledConnection<T>> {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to acquire connection permit: {}", e))?;

        let connection = {
            let mut connections = self
                .connections
                .lock()
                .map_err(|e| anyhow::anyhow!("Failed to lock connections: {}", e))?;

            if let Some(conn) = connections.pop() {
                conn
            } else {
                (self.create_connection)()?
            }
        };

        Ok(PooledConnection {
            connection: Some(connection),
            pool: Arc::clone(&self.connections),
        })
    }

    /// Get pool statistics
    pub fn get_stats(&self) -> ConnectionPoolStats {
        let available = self.connections.lock().map(|c| c.len()).unwrap_or(0);

        ConnectionPoolStats {
            max_connections: self.max_connections,
            available_connections: available,
            active_connections: self.max_connections
                - self.semaphore.available_permits()
                - available,
        }
    }
}

/// Statistics for connection pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolStats {
    pub max_connections: usize,
    pub available_connections: usize,
    pub active_connections: usize,
}

/// A pooled connection that returns to the pool when dropped
pub struct PooledConnection<T> {
    connection: Option<T>,
    pool: Arc<Mutex<Vec<T>>>,
}

impl<T> PooledConnection<T> {
    /// Get a reference to the connection
    pub fn connection(&self) -> &T {
        self.connection.as_ref().unwrap()
    }

    /// Get a mutable reference to the connection
    pub fn connection_mut(&mut self) -> &mut T {
        self.connection.as_mut().unwrap()
    }
}

impl<T> Drop for PooledConnection<T> {
    fn drop(&mut self) {
        if let Some(connection) = self.connection.take() {
            if let Ok(mut pool) = self.pool.lock() {
                pool.push(connection);
            }
        }
    }
}

/// Cache implementation for storing frequently accessed data
#[derive(Debug)]
pub struct Cache<K, V> {
    data: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    max_size: usize,
    default_ttl: Duration,
}

#[derive(Debug, Clone)]
struct CacheEntry<V> {
    value: V,
    expires_at: Instant,
    access_count: u64,
    last_accessed: Instant,
}

impl<K, V> Cache<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync,
    V: Clone + Send + Sync,
{
    /// Create a new cache
    pub fn new(max_size: usize, default_ttl: Duration) -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            max_size,
            default_ttl,
        }
    }

    /// Insert a value into the cache
    pub async fn insert(&self, key: K, value: V) {
        self.insert_with_ttl(key, value, self.default_ttl).await;
    }

    /// Insert a value with a specific TTL
    pub async fn insert_with_ttl(&self, key: K, value: V, ttl: Duration) {
        let mut data = self.data.write().await;

        // Check if we need to evict entries
        if data.len() >= self.max_size {
            self.evict_lru(&mut data).await;
        }

        let entry = CacheEntry {
            value,
            expires_at: Instant::now() + ttl,
            access_count: 1,
            last_accessed: Instant::now(),
        };

        data.insert(key, entry);
    }

    /// Get a value from the cache
    pub async fn get(&self, key: &K) -> Option<V> {
        let mut data = self.data.write().await;

        if let Some(entry) = data.get_mut(key) {
            // Check if expired
            if Instant::now() > entry.expires_at {
                data.remove(key);
                return None;
            }

            // Update access statistics
            entry.access_count += 1;
            entry.last_accessed = Instant::now();

            Some(entry.value.clone())
        } else {
            None
        }
    }

    /// Remove a value from the cache
    pub async fn remove(&self, key: &K) -> Option<V> {
        let mut data = self.data.write().await;
        data.remove(key).map(|entry| entry.value)
    }

    /// Clear all entries from the cache
    pub async fn clear(&self) {
        let mut data = self.data.write().await;
        data.clear();
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        let data = self.data.read().await;
        let now = Instant::now();

        let total_entries = data.len();
        let expired_entries = data.values().filter(|entry| now > entry.expires_at).count();

        CacheStats {
            total_entries,
            expired_entries,
            active_entries: total_entries - expired_entries,
            max_size: self.max_size,
            hit_ratio: 0.0, // Would need separate tracking
        }
    }

    /// Evict least recently used entry
    async fn evict_lru(&self, data: &mut HashMap<K, CacheEntry<V>>) {
        if let Some((key_to_remove, _)) = data
            .iter()
            .min_by_key(|(_, entry)| entry.last_accessed)
            .map(|(k, v)| (k.clone(), v.clone()))
        {
            data.remove(&key_to_remove);
            debug!("Evicted LRU cache entry");
        }
    }

    /// Clean up expired entries
    pub async fn cleanup_expired(&self) {
        let mut data = self.data.write().await;
        let now = Instant::now();

        let expired_keys: Vec<K> = data
            .iter()
            .filter(|(_, entry)| now > entry.expires_at)
            .map(|(k, _)| k.clone())
            .collect();

        for key in expired_keys {
            data.remove(&key);
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_entries: usize,
    pub expired_entries: usize,
    pub active_entries: usize,
    pub max_size: usize,
    pub hit_ratio: f64,
}

/// Async processing optimizer with rate limiting and batching
#[derive(Debug)]
pub struct AsyncProcessingOptimizer {
    max_concurrent_tasks: usize,
    batch_size: usize,
    batch_timeout: Duration,
    semaphore: Arc<Semaphore>,
}

impl AsyncProcessingOptimizer {
    /// Create a new async processing optimizer
    pub fn new(max_concurrent_tasks: usize, batch_size: usize, batch_timeout: Duration) -> Self {
        Self {
            max_concurrent_tasks,
            batch_size,
            batch_timeout,
            semaphore: Arc::new(Semaphore::new(max_concurrent_tasks)),
        }
    }

    /// Process tasks with concurrency limiting
    pub async fn process_tasks<T, F, Fut>(
        &self,
        tasks: Vec<T>,
        processor: F,
    ) -> Result<Vec<Result<(), anyhow::Error>>>
    where
        F: Fn(T) -> Fut + Clone + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send,
        T: Send + 'static,
    {
        let mut results = Vec::new();
        let tasks_count = tasks.len();

        info!(
            "Processing {} tasks with max {} concurrent",
            tasks_count, self.max_concurrent_tasks
        );

        for task in tasks {
            let processor = processor.clone();
            let semaphore = Arc::clone(&self.semaphore);

            let handle = tokio::spawn(async move {
                let permit = semaphore
                    .acquire()
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to acquire task permit: {}", e))?;
                let result = processor(task).await;
                drop(permit); // Release permit when task completes
                result
            });

            results.push(handle);
        }

        // Collect all results
        let mut final_results = Vec::new();
        for handle in results {
            let result = handle
                .await
                .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?;
            final_results.push(result);
        }

        Ok(final_results)
    }

    /// Process tasks in batches
    pub async fn process_batches<T, F, Fut>(
        &self,
        tasks: Vec<T>,
        processor: F,
    ) -> Result<Vec<Result<(), anyhow::Error>>>
    where
        F: Fn(Vec<T>) -> Fut + Clone + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send,
        T: Send + Clone + 'static,
    {
        let batches: Vec<Vec<T>> = tasks
            .chunks(self.batch_size)
            .map(|chunk| chunk.to_vec())
            .collect();

        info!(
            "Processing {} tasks in {} batches of size {}",
            batches.iter().map(|b| b.len()).sum::<usize>(),
            batches.len(),
            self.batch_size
        );

        let batch_processor = move |batch: Vec<T>| {
            let processor = processor.clone();
            async move { processor(batch).await }
        };

        self.process_tasks(batches, batch_processor).await
    }
}

/// Performance benchmark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    /// Number of iterations to run
    pub iterations: usize,
    /// Number of concurrent operations
    pub concurrency: usize,
    /// Duration to run the benchmark
    pub duration: Duration,
    /// Warmup iterations before measuring
    pub warmup_iterations: usize,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            iterations: 1000,
            concurrency: 10,
            duration: Duration::from_secs(60),
            warmup_iterations: 100,
        }
    }
}

/// Benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Benchmark name
    pub name: String,
    /// Configuration used
    pub config: BenchmarkConfig,
    /// Performance metrics
    pub metrics: PerformanceMetrics,
    /// Total operations completed
    pub total_operations: u64,
    /// Operations per second
    pub ops_per_second: f64,
    /// Total benchmark duration
    pub total_duration: Duration,
    /// Error count
    pub error_count: u64,
    /// Success rate percentage
    pub success_rate: f64,
}

/// Performance benchmarking framework
#[derive(Debug)]
pub struct PerformanceBenchmark {
    profiler: PerformanceProfiler,
}

impl Default for PerformanceBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceBenchmark {
    /// Create a new performance benchmark
    pub fn new() -> Self {
        Self {
            profiler: PerformanceProfiler::new(),
        }
    }

    /// Run a benchmark
    pub async fn run_benchmark<F, Fut>(
        &self,
        name: &str,
        config: BenchmarkConfig,
        operation: F,
    ) -> Result<BenchmarkResult>
    where
        F: Fn() -> Fut + Clone + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send,
    {
        info!("Starting benchmark: {}", name);

        // Warmup phase
        info!("Running {} warmup iterations", config.warmup_iterations);
        for _ in 0..config.warmup_iterations {
            let _ = operation().await;
        }

        // Reset metrics after warmup
        self.profiler.reset_metrics(name).await;

        let start_time = Instant::now();
        let mut total_operations = 0u64;
        let mut error_count = 0u64;

        // Run benchmark with concurrency
        let semaphore = Arc::new(Semaphore::new(config.concurrency));
        let mut tasks = Vec::new();

        for _ in 0..config.iterations {
            let operation = operation.clone();
            let profiler = self.profiler.clone();
            let benchmark_name = name.to_string();
            let semaphore_clone = Arc::clone(&semaphore);

            let task = tokio::spawn(async move {
                let permit = semaphore_clone
                    .acquire()
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to acquire benchmark permit: {}", e))?;
                let timer = profiler.start_operation(&benchmark_name);
                let result = operation().await;
                timer.finish().await;
                drop(permit);
                result
            });

            tasks.push(task);
        }

        // Collect results
        for task in tasks {
            match task.await {
                Ok(Ok(())) => total_operations += 1,
                Ok(Err(_)) => {
                    total_operations += 1;
                    error_count += 1;
                }
                Err(_) => error_count += 1,
            }
        }

        let total_duration = start_time.elapsed();
        let ops_per_second = total_operations as f64 / total_duration.as_secs_f64();
        let success_rate =
            ((total_operations - error_count) as f64 / total_operations as f64) * 100.0;

        let metrics = self.profiler.get_metrics(name).await.unwrap_or_default();

        let result = BenchmarkResult {
            name: name.to_string(),
            config,
            metrics,
            total_operations,
            ops_per_second,
            total_duration,
            error_count,
            success_rate,
        };

        info!(
            "Benchmark {} completed: {:.2} ops/sec, {:.1}% success rate",
            name, ops_per_second, success_rate
        );

        Ok(result)
    }

    /// Get profiler reference
    pub fn profiler(&self) -> &PerformanceProfiler {
        &self.profiler
    }
}

/// Performance optimization manager
#[derive(Debug)]
pub struct PerformanceManager {
    profiler: PerformanceProfiler,
    benchmark: PerformanceBenchmark,
    cache: Cache<String, serde_json::Value>,
    optimizer: AsyncProcessingOptimizer,
}

impl Default for PerformanceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceManager {
    /// Create a new performance manager
    pub fn new() -> Self {
        Self {
            profiler: PerformanceProfiler::new(),
            benchmark: PerformanceBenchmark::new(),
            cache: Cache::new(1000, Duration::from_secs(300)), // 5 minute default TTL
            optimizer: AsyncProcessingOptimizer::new(100, 50, Duration::from_millis(100)),
        }
    }

    /// Get performance profiler
    pub fn profiler(&self) -> &PerformanceProfiler {
        &self.profiler
    }

    /// Get benchmark framework
    pub fn benchmark(&self) -> &PerformanceBenchmark {
        &self.benchmark
    }

    /// Get cache
    pub fn cache(&self) -> &Cache<String, serde_json::Value> {
        &self.cache
    }

    /// Get async optimizer
    pub fn optimizer(&self) -> &AsyncProcessingOptimizer {
        &self.optimizer
    }

    /// Get comprehensive performance report
    pub async fn get_performance_report(&self) -> PerformanceReport {
        let metrics = self.profiler.get_all_metrics().await;
        let cache_stats = self.cache.get_stats().await;

        PerformanceReport {
            metrics,
            cache_stats,
            uptime: self.profiler.start_time.elapsed(),
        }
    }
}

/// Comprehensive performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub metrics: HashMap<String, PerformanceMetrics>,
    pub cache_stats: CacheStats,
    pub uptime: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_performance_profiler() {
        let profiler = PerformanceProfiler::new();

        let timer = profiler.start_operation("test_operation");
        sleep(Duration::from_millis(10)).await;
        timer.finish().await;

        let metrics = profiler.get_metrics("test_operation").await.unwrap();
        assert_eq!(metrics.count, 1);
        assert!(metrics.total_duration_ms >= 10);
    }

    #[tokio::test]
    async fn test_cache() {
        let cache = Cache::new(10, Duration::from_secs(1));

        cache
            .insert("key1".to_string(), serde_json::json!("value1"))
            .await;

        let value = cache.get(&"key1".to_string()).await;
        assert_eq!(value, Some(serde_json::json!("value1")));

        // Test expiration
        sleep(Duration::from_millis(1100)).await;
        let expired_value = cache.get(&"key1".to_string()).await;
        assert_eq!(expired_value, None);
    }

    #[tokio::test]
    async fn test_async_optimizer() {
        let optimizer = AsyncProcessingOptimizer::new(5, 3, Duration::from_millis(100));

        let tasks = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let results = optimizer
            .process_tasks(tasks, |_| async { Ok(()) })
            .await
            .unwrap();

        assert_eq!(results.len(), 10);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[tokio::test]
    async fn test_benchmark() {
        let benchmark = PerformanceBenchmark::new();
        let config = BenchmarkConfig {
            iterations: 10,
            concurrency: 2,
            duration: Duration::from_secs(1),
            warmup_iterations: 2,
        };

        let result = benchmark
            .run_benchmark("test_bench", config, || async { Ok(()) })
            .await
            .unwrap();

        assert_eq!(result.total_operations, 10);
        assert_eq!(result.error_count, 0);
        assert_eq!(result.success_rate, 100.0);
    }
}
