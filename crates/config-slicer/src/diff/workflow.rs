//! Diff workflow orchestration
//!
//! This module provides end-to-end workflow orchestration for configuration diffs,
//! including caching, history tracking, and approval workflows.

use crate::diff::engine::DiffEngine;
use crate::diff::types::{DiffOptions, DiffResult};
use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Represents the status of a diff workflow
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorkflowStatus {
    /// Diff is being computed
    Computing,
    /// Diff computation completed successfully
    Completed,
    /// Diff computation failed
    Failed,
    /// Diff is pending approval
    PendingApproval,
    /// Diff has been approved
    Approved,
    /// Diff has been rejected
    Rejected,
    /// Diff has been archived
    Archived,
}

/// Represents a workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    /// Unique identifier for this workflow execution
    pub id: Uuid,
    /// Source identifier (e.g., file path, node ID)
    pub source: String,
    /// Target identifier (e.g., file path, node ID)
    pub target: String,
    /// Current status of the workflow
    pub status: WorkflowStatus,
    /// Timestamp when the workflow was created
    pub created_at: u64,
    /// Timestamp when the workflow was last updated
    pub updated_at: u64,
    /// Optional metadata about the workflow
    pub metadata: HashMap<String, String>,
    /// Diff options used for this workflow
    pub diff_options: DiffOptions,
    /// Optional approval information
    pub approval_info: Option<ApprovalInfo>,
    /// Error message if workflow failed
    pub error_message: Option<String>,
}

/// Approval information for a diff workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalInfo {
    /// User who requested approval
    pub requester: String,
    /// Users who can approve this diff
    pub approvers: Vec<String>,
    /// User who approved/rejected this diff
    pub reviewer: Option<String>,
    /// Timestamp when approval was requested
    pub requested_at: u64,
    /// Timestamp when approval was granted/denied
    pub reviewed_at: Option<u64>,
    /// Approval/rejection reason
    pub reason: Option<String>,
    /// Priority level for this approval
    pub priority: ApprovalPriority,
}

/// Priority levels for approval workflows
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ApprovalPriority {
    /// Low priority changes
    Low,
    /// Normal priority changes
    Normal,
    /// High priority changes
    High,
    /// Emergency changes requiring immediate attention
    Emergency,
}

/// Cached diff result with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedDiffResult {
    /// The actual diff result
    pub result: DiffResult,
    /// Timestamp when this result was cached
    pub cached_at: u64,
    /// Time-to-live for this cached result (in seconds)
    pub ttl: u64,
    /// Hash of the input configurations
    pub input_hash: String,
    /// Options used to generate this diff
    pub options: DiffOptions,
}

impl CachedDiffResult {
    /// Check if this cached result is still valid
    pub fn is_valid(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now < self.cached_at + self.ttl
    }

    /// Get the age of this cached result in seconds
    pub fn age(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now.saturating_sub(self.cached_at)
    }
}

/// History entry for tracking diff workflow executions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowHistoryEntry {
    /// Workflow execution ID
    pub execution_id: Uuid,
    /// Source and target identifiers
    pub source: String,
    pub target: String,
    /// Status of the workflow
    pub status: WorkflowStatus,
    /// Timestamp of this history entry
    pub timestamp: u64,
    /// User who triggered this action
    pub user: Option<String>,
    /// Action description
    pub action: String,
    /// Additional details about the action
    pub details: Option<String>,
}

/// Main workflow orchestrator
pub struct DiffWorkflowOrchestrator {
    /// Diff engine for computing diffs
    diff_engine: Arc<DiffEngine>,
    /// Cache for storing diff results
    result_cache: Arc<RwLock<HashMap<String, CachedDiffResult>>>,
    /// Active workflow executions
    active_workflows: Arc<RwLock<HashMap<Uuid, WorkflowExecution>>>,
    /// Workflow history
    history: Arc<RwLock<Vec<WorkflowHistoryEntry>>>,
    /// Cache configuration
    cache_config: CacheConfig,
}

/// Configuration for the diff result cache
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Default TTL for cached results (in seconds)
    pub default_ttl: u64,
    /// Maximum number of entries to keep in cache
    pub max_entries: usize,
    /// Whether to enable cache compression
    pub enable_compression: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            default_ttl: 3600, // 1 hour
            max_entries: 1000,
            enable_compression: false,
        }
    }
}

impl DiffWorkflowOrchestrator {
    /// Create a new workflow orchestrator
    pub fn new(diff_engine: DiffEngine) -> Self {
        Self::with_cache_config(diff_engine, CacheConfig::default())
    }

    /// Create a new workflow orchestrator with custom cache configuration
    pub fn with_cache_config(diff_engine: DiffEngine, cache_config: CacheConfig) -> Self {
        Self {
            diff_engine: Arc::new(diff_engine),
            result_cache: Arc::new(RwLock::new(HashMap::new())),
            active_workflows: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(Vec::new())),
            cache_config,
        }
    }

    /// Execute a complete diff workflow
    pub async fn execute_workflow(
        &self,
        source: &str,
        target: &str,
        old_config: &str,
        new_config: &str,
        options: DiffOptions,
        approval_required: bool,
    ) -> Result<Uuid> {
        let execution_id = Uuid::new_v4();
        info!("Starting diff workflow execution {}", execution_id);

        // Create workflow execution
        let mut workflow = WorkflowExecution {
            id: execution_id,
            source: source.to_string(),
            target: target.to_string(),
            status: WorkflowStatus::Computing,
            created_at: current_timestamp(),
            updated_at: current_timestamp(),
            metadata: HashMap::new(),
            diff_options: options.clone(),
            approval_info: None,
            error_message: None,
        };

        // Store in active workflows
        {
            let mut workflows = self.active_workflows.write().unwrap();
            workflows.insert(execution_id, workflow.clone());
        }

        // Record workflow start
        self.record_history_entry(
            execution_id,
            source,
            target,
            WorkflowStatus::Computing,
            None,
            "Workflow started".to_string(),
            None,
        );

        // Check cache first
        let cache_key = self.generate_cache_key(old_config, new_config, &options);
        if let Some(_cached_result) = self.get_cached_result(&cache_key) {
            info!("Using cached diff result for workflow {}", execution_id);
            workflow.status = if approval_required {
                WorkflowStatus::PendingApproval
            } else {
                WorkflowStatus::Completed
            };
            workflow.updated_at = current_timestamp();

            // Update workflow
            {
                let mut workflows = self.active_workflows.write().unwrap();
                workflows.insert(execution_id, workflow.clone());
            }

            self.record_history_entry(
                execution_id,
                source,
                target,
                workflow.status.clone(),
                None,
                "Used cached result".to_string(),
                None,
            );

            return Ok(execution_id);
        }

        // Compute diff
        let diff_result = match self.diff_engine.diff(old_config, new_config) {
            Ok(result) => {
                info!("Diff computation completed for workflow {}", execution_id);
                result
            }
            Err(e) => {
                error!(
                    "Diff computation failed for workflow {}: {}",
                    execution_id, e
                );
                workflow.status = WorkflowStatus::Failed;
                workflow.error_message = Some(e.to_string());
                workflow.updated_at = current_timestamp();

                // Update workflow
                {
                    let mut workflows = self.active_workflows.write().unwrap();
                    workflows.insert(execution_id, workflow);
                }

                self.record_history_entry(
                    execution_id,
                    source,
                    target,
                    WorkflowStatus::Failed,
                    None,
                    "Diff computation failed".to_string(),
                    Some(e.to_string()),
                );

                return Err(Error::DiffWorkflowError(format!(
                    "Diff computation failed: {}",
                    e
                )));
            }
        };

        // Cache the result
        self.cache_result(&cache_key, diff_result, &options);

        // Update workflow status
        workflow.status = if approval_required {
            WorkflowStatus::PendingApproval
        } else {
            WorkflowStatus::Completed
        };
        workflow.updated_at = current_timestamp();

        // Clone status before moving workflow
        let status_clone = workflow.status.clone();

        // Update workflow
        {
            let mut workflows = self.active_workflows.write().unwrap();
            workflows.insert(execution_id, workflow);
        }

        self.record_history_entry(
            execution_id,
            source,
            target,
            status_clone,
            None,
            "Diff computation completed".to_string(),
            None,
        );

        info!("Diff workflow execution {} completed", execution_id);
        Ok(execution_id)
    }

    /// Request approval for a diff workflow
    pub fn request_approval(
        &self,
        execution_id: Uuid,
        requester: &str,
        approvers: Vec<String>,
        priority: ApprovalPriority,
    ) -> Result<()> {
        let mut workflows = self.active_workflows.write().unwrap();
        let workflow = workflows
            .get_mut(&execution_id)
            .ok_or_else(|| Error::WorkflowNotFound(execution_id.to_string()))?;

        let approval_info = ApprovalInfo {
            requester: requester.to_string(),
            approvers,
            reviewer: None,
            requested_at: current_timestamp(),
            reviewed_at: None,
            reason: None,
            priority,
        };

        workflow.approval_info = Some(approval_info);
        workflow.status = WorkflowStatus::PendingApproval;
        workflow.updated_at = current_timestamp();

        self.record_history_entry(
            execution_id,
            &workflow.source,
            &workflow.target,
            WorkflowStatus::PendingApproval,
            Some(requester.to_string()),
            "Approval requested".to_string(),
            None,
        );

        info!("Approval requested for workflow {}", execution_id);
        Ok(())
    }

    /// Approve a diff workflow
    pub fn approve_workflow(
        &self,
        execution_id: Uuid,
        reviewer: &str,
        reason: Option<String>,
    ) -> Result<()> {
        let mut workflows = self.active_workflows.write().unwrap();
        let workflow = workflows
            .get_mut(&execution_id)
            .ok_or_else(|| Error::WorkflowNotFound(execution_id.to_string()))?;

        // Check if approval is required
        let approval_info = workflow
            .approval_info
            .as_mut()
            .ok_or_else(|| Error::ApprovalNotRequired(execution_id.to_string()))?;

        // Check if reviewer is authorized
        if !approval_info.approvers.contains(&reviewer.to_string()) {
            return Err(Error::UnauthorizedApprover(reviewer.to_string()));
        }

        approval_info.reviewer = Some(reviewer.to_string());
        approval_info.reviewed_at = Some(current_timestamp());
        approval_info.reason = reason.clone();

        workflow.status = WorkflowStatus::Approved;
        workflow.updated_at = current_timestamp();

        self.record_history_entry(
            execution_id,
            &workflow.source,
            &workflow.target,
            WorkflowStatus::Approved,
            Some(reviewer.to_string()),
            "Workflow approved".to_string(),
            reason,
        );

        info!("Workflow {} approved by {}", execution_id, reviewer);
        Ok(())
    }

    /// Reject a diff workflow
    pub fn reject_workflow(
        &self,
        execution_id: Uuid,
        reviewer: &str,
        reason: Option<String>,
    ) -> Result<()> {
        let mut workflows = self.active_workflows.write().unwrap();
        let workflow = workflows
            .get_mut(&execution_id)
            .ok_or_else(|| Error::WorkflowNotFound(execution_id.to_string()))?;

        // Check if approval is required
        let approval_info = workflow
            .approval_info
            .as_mut()
            .ok_or_else(|| Error::ApprovalNotRequired(execution_id.to_string()))?;

        // Check if reviewer is authorized
        if !approval_info.approvers.contains(&reviewer.to_string()) {
            return Err(Error::UnauthorizedApprover(reviewer.to_string()));
        }

        approval_info.reviewer = Some(reviewer.to_string());
        approval_info.reviewed_at = Some(current_timestamp());
        approval_info.reason = reason.clone();

        workflow.status = WorkflowStatus::Rejected;
        workflow.updated_at = current_timestamp();

        self.record_history_entry(
            execution_id,
            &workflow.source,
            &workflow.target,
            WorkflowStatus::Rejected,
            Some(reviewer.to_string()),
            "Workflow rejected".to_string(),
            reason,
        );

        warn!("Workflow {} rejected by {}", execution_id, reviewer);
        Ok(())
    }

    /// Get a workflow execution by ID
    pub fn get_workflow(&self, execution_id: Uuid) -> Option<WorkflowExecution> {
        let workflows = self.active_workflows.read().unwrap();
        workflows.get(&execution_id).cloned()
    }

    /// List all active workflows
    pub fn list_workflows(&self) -> Vec<WorkflowExecution> {
        let workflows = self.active_workflows.read().unwrap();
        workflows.values().cloned().collect()
    }

    /// Get workflow history
    pub fn get_history(&self, limit: Option<usize>) -> Vec<WorkflowHistoryEntry> {
        let history = self.history.read().unwrap();
        let mut entries = history.clone();
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        if let Some(limit) = limit {
            entries.truncate(limit);
        }

        entries
    }

    /// Get workflow history for a specific execution
    pub fn get_workflow_history(&self, execution_id: Uuid) -> Vec<WorkflowHistoryEntry> {
        let history = self.history.read().unwrap();
        history
            .iter()
            .filter(|entry| entry.execution_id == execution_id)
            .cloned()
            .collect()
    }

    /// Archive a completed workflow
    pub fn archive_workflow(&self, execution_id: Uuid) -> Result<()> {
        let mut workflows = self.active_workflows.write().unwrap();
        let workflow = workflows
            .get_mut(&execution_id)
            .ok_or_else(|| Error::WorkflowNotFound(execution_id.to_string()))?;

        workflow.status = WorkflowStatus::Archived;
        workflow.updated_at = current_timestamp();

        self.record_history_entry(
            execution_id,
            &workflow.source,
            &workflow.target,
            WorkflowStatus::Archived,
            None,
            "Workflow archived".to_string(),
            None,
        );

        info!("Workflow {} archived", execution_id);
        Ok(())
    }

    /// Clean up expired cache entries
    pub fn cleanup_cache(&self) {
        let mut cache = self.result_cache.write().unwrap();
        let before_count = cache.len();

        cache.retain(|_, cached_result| cached_result.is_valid());

        let after_count = cache.len();
        if before_count != after_count {
            debug!(
                "Cleaned up {} expired cache entries",
                before_count - after_count
            );
        }

        // Also enforce max_entries limit
        if cache.len() > self.cache_config.max_entries {
            let excess = cache.len() - self.cache_config.max_entries;
            let mut entries: Vec<_> = cache
                .iter()
                .map(|(k, v)| (k.clone(), v.cached_at))
                .collect();
            entries.sort_by(|a, b| a.1.cmp(&b.1));

            let keys_to_remove: Vec<_> = entries.into_iter().take(excess).map(|(k, _)| k).collect();

            for key in keys_to_remove {
                cache.remove(&key);
            }

            debug!("Evicted {} cache entries to enforce size limit", excess);
        }
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> CacheStats {
        let cache = self.result_cache.read().unwrap();
        let total_entries = cache.len();
        let valid_entries = cache.values().filter(|entry| entry.is_valid()).count();
        let expired_entries = total_entries - valid_entries;

        CacheStats {
            total_entries,
            valid_entries,
            expired_entries,
            hit_rate: 0.0, // Would need to track hits/misses to calculate this
        }
    }

    fn generate_cache_key(
        &self,
        old_config: &str,
        new_config: &str,
        options: &DiffOptions,
    ) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        old_config.hash(&mut hasher);
        new_config.hash(&mut hasher);

        // Create a hashable representation of options
        let options_str = format!("{:?}", options);
        options_str.hash(&mut hasher);

        format!("{:x}", hasher.finish())
    }

    fn get_cached_result(&self, cache_key: &str) -> Option<DiffResult> {
        let cache = self.result_cache.read().unwrap();
        cache
            .get(cache_key)
            .filter(|entry| entry.is_valid())
            .map(|entry| entry.result.clone())
    }

    fn cache_result(&self, cache_key: &str, result: DiffResult, options: &DiffOptions) {
        let mut cache = self.result_cache.write().unwrap();

        let cached_result = CachedDiffResult {
            result,
            cached_at: current_timestamp(),
            ttl: self.cache_config.default_ttl,
            input_hash: cache_key.to_string(),
            options: options.clone(),
        };

        cache.insert(cache_key.to_string(), cached_result);
    }

    fn record_history_entry(
        &self,
        execution_id: Uuid,
        source: &str,
        target: &str,
        status: WorkflowStatus,
        user: Option<String>,
        action: String,
        details: Option<String>,
    ) {
        let entry = WorkflowHistoryEntry {
            execution_id,
            source: source.to_string(),
            target: target.to_string(),
            status,
            timestamp: current_timestamp(),
            user,
            action,
            details,
        };

        let mut history = self.history.write().unwrap();
        history.push(entry);
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Total number of entries in cache
    pub total_entries: usize,
    /// Number of valid (non-expired) entries
    pub valid_entries: usize,
    /// Number of expired entries
    pub expired_entries: usize,
    /// Cache hit rate (0.0 to 1.0)
    pub hit_rate: f64,
}

/// Get current timestamp as seconds since UNIX epoch
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::engine::DiffEngine;

    #[tokio::test]
    async fn test_workflow_creation() {
        let engine = DiffEngine::new().unwrap();
        let orchestrator = DiffWorkflowOrchestrator::new(engine);

        let old_config = "interface GigabitEthernet0/1\n ip address 192.168.1.1 255.255.255.0";
        let new_config = "interface GigabitEthernet0/1\n ip address 192.168.1.2 255.255.255.0";

        let execution_id = orchestrator
            .execute_workflow(
                "test_source",
                "test_target",
                old_config,
                new_config,
                DiffOptions::default(),
                false,
            )
            .await
            .unwrap();

        let workflow = orchestrator.get_workflow(execution_id).unwrap();
        assert_eq!(workflow.status, WorkflowStatus::Completed);
        assert_eq!(workflow.source, "test_source");
        assert_eq!(workflow.target, "test_target");
    }

    #[tokio::test]
    async fn test_approval_workflow() {
        let engine = DiffEngine::new().unwrap();
        let orchestrator = DiffWorkflowOrchestrator::new(engine);

        let old_config = "interface GigabitEthernet0/1\n ip address 192.168.1.1 255.255.255.0";
        let new_config = "interface GigabitEthernet0/1\n ip address 192.168.1.2 255.255.255.0";

        let execution_id = orchestrator
            .execute_workflow(
                "test_source",
                "test_target",
                old_config,
                new_config,
                DiffOptions::default(),
                true,
            )
            .await
            .unwrap();

        // Request approval
        orchestrator
            .request_approval(
                execution_id,
                "user1",
                vec!["approver1".to_string(), "approver2".to_string()],
                ApprovalPriority::Normal,
            )
            .unwrap();

        let workflow = orchestrator.get_workflow(execution_id).unwrap();
        assert_eq!(workflow.status, WorkflowStatus::PendingApproval);
        assert!(workflow.approval_info.is_some());

        // Approve workflow
        orchestrator
            .approve_workflow(execution_id, "approver1", Some("Looks good".to_string()))
            .unwrap();

        let workflow = orchestrator.get_workflow(execution_id).unwrap();
        assert_eq!(workflow.status, WorkflowStatus::Approved);
    }

    #[test]
    fn test_cache_validity() {
        let cached_result = CachedDiffResult {
            result: DiffResult {
                text_diff: crate::diff::types::TextDiff {
                    changes: Vec::new(),
                    additions: 0,
                    deletions: 0,
                    modifications: 0,
                    context_lines: 3,
                },
                hierarchical_diff: crate::diff::types::HierarchicalDiff {
                    sections: Vec::new(),
                    structure_changes: Vec::new(),
                    path_changes: std::collections::HashMap::new(),
                },
                semantic_diff: crate::diff::types::SemanticDiff {
                    functional_changes: Vec::new(),
                    impact_analysis: Vec::new(),
                    change_groups: Vec::new(),
                },
                summary: crate::diff::types::DiffSummary {
                    total_changes: 0,
                    additions: 0,
                    deletions: 0,
                    modifications: 0,
                    sections_affected: 0,
                    highest_risk: crate::diff::types::RiskLevel::Low,
                    complexity: crate::diff::types::ChangeComplexity::Simple,
                },
                options: DiffOptions::default(),
            },
            cached_at: current_timestamp(),
            ttl: 3600,
            input_hash: "test_hash".to_string(),
            options: DiffOptions::default(),
        };

        assert!(cached_result.is_valid());
        assert_eq!(cached_result.age(), 0); // Just created
    }

    #[test]
    fn test_cache_key_generation() {
        let engine = DiffEngine::new().unwrap();
        let orchestrator = DiffWorkflowOrchestrator::new(engine);

        let old_config = "interface GigabitEthernet0/1";
        let new_config = "interface GigabitEthernet0/2";
        let options = DiffOptions::default();

        let key1 = orchestrator.generate_cache_key(old_config, new_config, &options);
        let key2 = orchestrator.generate_cache_key(old_config, new_config, &options);
        let key3 = orchestrator.generate_cache_key(new_config, old_config, &options);

        assert_eq!(key1, key2); // Same inputs should generate same key
        assert_ne!(key1, key3); // Different inputs should generate different keys
    }

    #[test]
    fn test_workflow_history() {
        let engine = DiffEngine::new().unwrap();
        let orchestrator = DiffWorkflowOrchestrator::new(engine);

        let execution_id = Uuid::new_v4();
        orchestrator.record_history_entry(
            execution_id,
            "source",
            "target",
            WorkflowStatus::Computing,
            Some("user1".to_string()),
            "Test action".to_string(),
            None,
        );

        let history = orchestrator.get_workflow_history(execution_id);
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].action, "Test action");
        assert_eq!(history[0].user, Some("user1".to_string()));
    }
}
