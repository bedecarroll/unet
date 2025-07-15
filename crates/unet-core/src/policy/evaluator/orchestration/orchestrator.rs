//! Policy orchestration engine
//!
//! Contains the main `PolicyOrchestrator` implementation for managing
//! complex policy evaluation workflows including batching and scheduling.

use super::super::context::{EvaluationContext, PolicyExecutionContext};
use super::super::results::AggregatedResult;
use super::config::OrchestrationConfig;
use super::core::{EvaluationBatch, OrchestrationRule};
use crate::datastore::DataStore;
use crate::policy::PolicyError;
use crate::policy::evaluator::PolicyEvaluator;
use std::collections::{HashMap, hash_map::DefaultHasher};
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};
use tokio::time::interval;
use uuid::Uuid;

/// Policy evaluation orchestrator for managing complex evaluation workflows
#[derive(Clone)]
pub struct PolicyOrchestrator {
    /// Orchestration configuration
    config: OrchestrationConfig,
    /// Cache for evaluation results
    cache: HashMap<String, super::core::CacheEntry>,
    /// Pending evaluation batches
    pending_batches: HashMap<Uuid, EvaluationBatch>,
}

impl Default for PolicyOrchestrator {
    fn default() -> Self {
        Self::new(OrchestrationConfig::default())
    }
}

impl PolicyOrchestrator {
    /// Create a new policy orchestrator with the given configuration
    #[must_use]
    pub fn new(config: OrchestrationConfig) -> Self {
        Self {
            config,
            cache: HashMap::new(),
            pending_batches: HashMap::new(),
        }
    }

    /// Add a batch of policy rules for evaluation
    #[must_use]
    pub fn schedule_evaluation(
        &mut self,
        node_id: Uuid,
        context: EvaluationContext,
        rules: Vec<OrchestrationRule>,
    ) -> String {
        let batch_id = format!(
            "batch_{}_{}",
            node_id.to_string().chars().take(8).collect::<String>(),
            Instant::now().elapsed().as_millis()
        );

        let batch = EvaluationBatch::new(
            node_id,
            context,
            Self::sort_rules_by_priority(rules),
            batch_id.clone(),
        );

        self.pending_batches.insert(node_id, batch);
        batch_id
    }

    /// Execute all pending batches
    ///
    /// # Errors
    /// Returns an error if batch execution fails or datastore operations fail
    pub async fn execute_pending_batches(
        &mut self,
        datastore: &dyn DataStore,
    ) -> Result<Vec<AggregatedResult>, PolicyError> {
        let mut results = Vec::new();
        let batches: Vec<_> = self.pending_batches.drain().collect();

        for (_node_id, batch) in batches {
            // Check cache first if enabled
            if self.config.enable_caching {
                let cache_key = Self::create_cache_key(&batch);
                if let Some(cached_result) = self.get_cached_result(&cache_key) {
                    results.push(cached_result);
                    continue;
                }
            }

            let result = self.execute_batch(&batch, datastore).await?;

            // Cache the result if enabled
            if self.config.enable_caching {
                let cache_key = Self::create_cache_key(&batch);
                self.cache_result(cache_key, &result);
            }

            results.push(result);
        }

        Ok(results)
    }

    /// Execute a single evaluation batch
    ///
    /// # Errors
    /// Returns an error if rule evaluation fails or datastore operations fail
    pub async fn execute_batch(
        &self,
        batch: &EvaluationBatch,
        datastore: &dyn DataStore,
    ) -> Result<AggregatedResult, PolicyError> {
        let start_time = Instant::now();
        let mut results = Vec::new();

        // Execute rules in priority order
        for orchestration_rule in &batch.rules {
            let exec_ctx = PolicyExecutionContext::new(&batch.context, datastore, &batch.node_id);
            let result = PolicyEvaluator::execute_rule(&orchestration_rule.rule, &exec_ctx).await?;
            results.push(result);
        }

        let execution_duration = start_time.elapsed();

        Ok(AggregatedResult::from_results(
            batch.node_id,
            batch.batch_id.clone(),
            results,
            execution_duration,
        ))
    }

    /// Execute policies for a single node with orchestration
    ///
    /// # Errors
    /// Returns an error if batch scheduling or execution fails
    pub async fn evaluate_node_policies(
        &mut self,
        node_id: Uuid,
        context: EvaluationContext,
        rules: Vec<OrchestrationRule>,
        datastore: &dyn DataStore,
    ) -> Result<AggregatedResult, PolicyError> {
        let batch_id = self.schedule_evaluation(node_id, context, rules);
        let results = self.execute_pending_batches(datastore).await?;

        results
            .into_iter()
            .find(|r| r.batch_id == batch_id)
            .ok_or_else(|| PolicyError::EvaluationError("Failed to find batch result".to_string()))
    }

    /// Start a background scheduler for automatic policy evaluation
    ///
    /// # Errors
    /// Returns an error if batch execution fails during scheduled runs
    pub async fn start_scheduler(
        &mut self,
        interval_duration: Duration,
        datastore: &dyn DataStore,
    ) -> Result<(), PolicyError> {
        let mut interval_timer = interval(interval_duration);

        loop {
            interval_timer.tick().await;

            // Clean expired cache entries
            self.clean_expired_cache();

            // Check for batches that have timed out and execute them if any exist
            let has_timed_out_batches = self
                .pending_batches
                .iter()
                .any(|(_, batch)| batch.is_timed_out(self.config.batch_timeout));

            // Execute timed out batches
            if has_timed_out_batches {
                self.execute_pending_batches(datastore).await?;
            }
        }
    }

    /// Sort rules by priority (highest first) and then by order
    fn sort_rules_by_priority(mut rules: Vec<OrchestrationRule>) -> Vec<OrchestrationRule> {
        rules.sort_by(|a, b| {
            // Sort by priority descending, then by order ascending
            match b.priority.cmp(&a.priority) {
                std::cmp::Ordering::Equal => a.order.cmp(&b.order),
                other => other,
            }
        });
        rules
    }

    /// Create a cache key for a batch
    fn create_cache_key(batch: &EvaluationBatch) -> String {
        let mut hasher = DefaultHasher::new();
        batch.node_id.hash(&mut hasher);

        // Hash rule content for cache invalidation
        for rule in &batch.rules {
            format!("{:?}", rule.rule).hash(&mut hasher);
            rule.priority.hash(&mut hasher);
            rule.order.hash(&mut hasher);
        }

        format!("cache_{:x}", hasher.finish())
    }

    /// Get cached result if available and not expired
    fn get_cached_result(&self, cache_key: &str) -> Option<AggregatedResult> {
        self.cache.get(cache_key).and_then(|entry| {
            if entry.is_expired() {
                None
            } else {
                Some(entry.result.clone())
            }
        })
    }

    /// Cache an evaluation result
    fn cache_result(&mut self, cache_key: String, result: &AggregatedResult) {
        let entry = super::core::CacheEntry::new(result.clone(), self.config.cache_ttl);
        self.cache.insert(cache_key, entry);
    }

    /// Clean expired cache entries
    fn clean_expired_cache(&mut self) {
        self.cache.retain(|_, entry| !entry.is_expired());
    }

    /// Get current cache statistics
    #[must_use]
    pub fn cache_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        stats.insert("total_entries".to_string(), self.cache.len());
        stats.insert("pending_batches".to_string(), self.pending_batches.len());

        let expired_count = self
            .cache
            .values()
            .filter(|entry| entry.is_expired())
            .count();
        stats.insert("expired_entries".to_string(), expired_count);

        stats
    }

    /// Clear all cached results
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Clear all pending batches
    pub fn clear_pending_batches(&mut self) {
        self.pending_batches.clear();
    }

    /// Get the number of pending batches
    #[must_use]
    pub fn pending_batch_count(&self) -> usize {
        self.pending_batches.len()
    }

    /// Get the number of cached results
    #[must_use]
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    /// Check if a node has a pending batch
    #[must_use]
    pub fn has_pending_batch(&self, node_id: &Uuid) -> bool {
        self.pending_batches.contains_key(node_id)
    }

    /// Get a pending batch for a node
    #[must_use]
    pub fn get_pending_batch(&self, node_id: &Uuid) -> Option<&EvaluationBatch> {
        self.pending_batches.get(node_id)
    }

    /// Remove a pending batch for a node
    pub fn remove_pending_batch(&mut self, node_id: &Uuid) -> Option<EvaluationBatch> {
        self.pending_batches.remove(node_id)
    }

    /// Update orchestration configuration
    pub const fn update_config(&mut self, config: OrchestrationConfig) {
        self.config = config;
    }

    /// Get current orchestration configuration
    #[must_use]
    pub const fn config(&self) -> &OrchestrationConfig {
        &self.config
    }
}
