//! File change tracking for μNet policy and template files
//!
//! This module provides specialized change tracking for policy and template files,
//! with notification support and file integrity validation.

use crate::git::types::{FileChange, FileStatus, GitError, GitResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// File type classification for μNet files
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileType {
    /// Policy files (.pest, .policy, etc.)
    Policy,
    /// Template files (.j2, .jinja2, .template, etc.)
    Template,
    /// Configuration files (.toml, .yaml, .json, etc.)
    Config,
    /// Documentation files (.md, .txt, etc.)
    Documentation,
    /// Other files
    Other,
}

impl FileType {
    /// Determine file type from path extension
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            match extension.to_lowercase().as_str() {
                "pest" | "policy" => Self::Policy,
                "j2" | "jinja2" | "template" => Self::Template,
                "toml" | "yaml" | "yml" | "json" => Self::Config,
                "md" | "txt" | "rst" => Self::Documentation,
                _ => Self::Other,
            }
        } else {
            Self::Other
        }
    }

    /// Check if this file type should be tracked for changes
    pub fn should_track(&self) -> bool {
        matches!(self, Self::Policy | Self::Template | Self::Config)
    }
}

/// Detailed file change information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedFileChange {
    /// Basic file change information
    pub file_change: FileChange,
    /// File type classification
    pub file_type: FileType,
    /// File content hash (for integrity checking)
    pub content_hash: Option<String>,
    /// Previous content hash (for change detection)
    pub previous_hash: Option<String>,
    /// Timestamp when change was detected
    pub detected_at: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl TrackedFileChange {
    /// Create a new tracked file change
    pub fn new(file_change: FileChange) -> Self {
        let file_type = FileType::from_path(&file_change.path);

        Self {
            file_change,
            file_type,
            content_hash: None,
            previous_hash: None,
            detected_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Set content hash for integrity checking
    pub fn with_content_hash(mut self, hash: String) -> Self {
        self.content_hash = Some(hash);
        self
    }

    /// Set previous content hash for change detection
    pub fn with_previous_hash(mut self, hash: String) -> Self {
        self.previous_hash = Some(hash);
        self
    }

    /// Add metadata entry
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Check if this change represents a content modification
    pub fn is_content_modified(&self) -> bool {
        if let (Some(current), Some(previous)) = (&self.content_hash, &self.previous_hash) {
            current != previous
        } else {
            // If we don't have hashes, fall back to status check
            matches!(
                self.file_change.status,
                FileStatus::Modified | FileStatus::Added | FileStatus::Deleted
            )
        }
    }
}

/// Change notification event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeNotificationEvent {
    /// Policy file was modified
    PolicyFileChanged {
        repository_url: String,
        file_path: PathBuf,
        change_type: FileStatus,
        content_hash: Option<String>,
        timestamp: DateTime<Utc>,
    },
    /// Template file was modified
    TemplateFileChanged {
        repository_url: String,
        file_path: PathBuf,
        change_type: FileStatus,
        content_hash: Option<String>,
        timestamp: DateTime<Utc>,
    },
    /// Configuration file was modified
    ConfigFileChanged {
        repository_url: String,
        file_path: PathBuf,
        change_type: FileStatus,
        content_hash: Option<String>,
        timestamp: DateTime<Utc>,
    },
    /// File integrity validation failed
    IntegrityValidationFailed {
        repository_url: String,
        file_path: PathBuf,
        expected_hash: String,
        actual_hash: String,
        timestamp: DateTime<Utc>,
    },
    /// Batch of changes detected
    BatchChangesDetected {
        repository_url: String,
        change_count: usize,
        policy_changes: usize,
        template_changes: usize,
        config_changes: usize,
        timestamp: DateTime<Utc>,
    },
}

/// File change tracking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeTrackerConfig {
    /// Whether to enable content hashing for integrity checks
    pub enable_content_hashing: bool,
    /// Whether to track all file types or only important ones
    pub track_all_files: bool,
    /// Maximum number of change events to keep in memory
    pub max_events: usize,
    /// File patterns to ignore (glob patterns)
    pub ignore_patterns: Vec<String>,
    /// File patterns to force tracking (overrides ignore patterns)
    pub force_track_patterns: Vec<String>,
}

impl Default for ChangeTrackerConfig {
    fn default() -> Self {
        Self {
            enable_content_hashing: true,
            track_all_files: false,
            max_events: 1000,
            ignore_patterns: vec![
                "*.tmp".to_string(),
                "*.bak".to_string(),
                "*~".to_string(),
                ".git/**".to_string(),
                "target/**".to_string(),
            ],
            force_track_patterns: vec![
                "*.policy".to_string(),
                "*.pest".to_string(),
                "*.j2".to_string(),
                "*.jinja2".to_string(),
                "*.template".to_string(),
            ],
        }
    }
}

/// File change tracker for monitoring μNet-specific files
#[derive(Debug)]
pub struct FileChangeTracker {
    /// Tracker configuration
    config: ChangeTrackerConfig,
    /// Current file states indexed by repository URL and file path
    file_states: Arc<RwLock<HashMap<String, HashMap<PathBuf, TrackedFileChange>>>>,
    /// Change notification events
    events: Arc<RwLock<Vec<ChangeNotificationEvent>>>,
}

impl FileChangeTracker {
    /// Create a new file change tracker
    pub fn new() -> Self {
        Self::with_config(ChangeTrackerConfig::default())
    }

    /// Create a new file change tracker with custom configuration
    pub fn with_config(config: ChangeTrackerConfig) -> Self {
        Self {
            config,
            file_states: Arc::new(RwLock::new(HashMap::new())),
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Process file changes for a repository
    pub async fn process_changes(
        &self,
        repository_url: &str,
        changes: &[FileChange],
        repository_path: &Path,
    ) -> GitResult<Vec<TrackedFileChange>> {
        let mut tracked_changes = Vec::new();
        let mut policy_changes = 0;
        let mut template_changes = 0;
        let mut config_changes = 0;

        for change in changes {
            if self.should_track_file(&change.path) {
                let mut tracked_change = TrackedFileChange::new(change.clone());

                // Calculate content hash if enabled
                if self.config.enable_content_hashing {
                    if let Ok(hash) = self
                        .calculate_file_hash(repository_path, &change.path)
                        .await
                    {
                        tracked_change = tracked_change.with_content_hash(hash);
                    }
                }

                // Get previous state for comparison
                let file_states = self.file_states.read().await;
                if let Some(repo_states) = file_states.get(repository_url) {
                    if let Some(previous_state) = repo_states.get(&change.path) {
                        if let Some(previous_hash) = &previous_state.content_hash {
                            tracked_change =
                                tracked_change.with_previous_hash(previous_hash.clone());
                        }
                    }
                }
                drop(file_states);

                // Add file size metadata if available
                if let Ok(metadata) = std::fs::metadata(repository_path.join(&change.path)) {
                    tracked_change = tracked_change
                        .with_metadata("file_size".to_string(), metadata.len().to_string());
                }

                // Count changes by type
                match tracked_change.file_type {
                    FileType::Policy => policy_changes += 1,
                    FileType::Template => template_changes += 1,
                    FileType::Config => config_changes += 1,
                    _ => {}
                }

                // Emit individual change events
                self.emit_change_event(repository_url, &tracked_change)
                    .await;

                tracked_changes.push(tracked_change);
            }
        }

        // Update file states
        self.update_file_states(repository_url, &tracked_changes)
            .await;

        // Emit batch event if there were significant changes
        if policy_changes > 0 || template_changes > 0 || config_changes > 0 {
            self.emit_event(ChangeNotificationEvent::BatchChangesDetected {
                repository_url: repository_url.to_string(),
                change_count: tracked_changes.len(),
                policy_changes,
                template_changes,
                config_changes,
                timestamp: Utc::now(),
            })
            .await;
        }

        info!(
            "Processed {} file changes for repository {}: {} policy, {} template, {} config",
            tracked_changes.len(),
            repository_url,
            policy_changes,
            template_changes,
            config_changes
        );

        Ok(tracked_changes)
    }

    /// Get tracked changes for a repository
    pub async fn get_repository_changes(
        &self,
        repository_url: &str,
    ) -> HashMap<PathBuf, TrackedFileChange> {
        let file_states = self.file_states.read().await;
        file_states.get(repository_url).cloned().unwrap_or_default()
    }

    /// Get recent change events
    pub async fn get_recent_events(&self, limit: Option<usize>) -> Vec<ChangeNotificationEvent> {
        let events = self.events.read().await;
        let limit = limit.unwrap_or(self.config.max_events);

        if events.len() <= limit {
            events.clone()
        } else {
            events[events.len() - limit..].to_vec()
        }
    }

    /// Validate file integrity for tracked files
    pub async fn validate_file_integrity(
        &self,
        repository_url: &str,
        repository_path: &Path,
    ) -> GitResult<Vec<PathBuf>> {
        let mut integrity_failures = Vec::new();
        let file_states = self.file_states.read().await;

        if let Some(repo_states) = file_states.get(repository_url) {
            for (file_path, tracked_change) in repo_states {
                if let Some(expected_hash) = &tracked_change.content_hash {
                    match self.calculate_file_hash(repository_path, file_path).await {
                        Ok(actual_hash) => {
                            if &actual_hash != expected_hash {
                                integrity_failures.push(file_path.clone());

                                self.emit_event(
                                    ChangeNotificationEvent::IntegrityValidationFailed {
                                        repository_url: repository_url.to_string(),
                                        file_path: file_path.clone(),
                                        expected_hash: expected_hash.clone(),
                                        actual_hash,
                                        timestamp: Utc::now(),
                                    },
                                )
                                .await;
                            }
                        }
                        Err(e) => {
                            warn!(
                                "Failed to calculate hash for {}: {}",
                                file_path.display(),
                                e
                            );
                        }
                    }
                }
            }
        }

        if !integrity_failures.is_empty() {
            warn!(
                "File integrity validation failed for {} files in repository {}",
                integrity_failures.len(),
                repository_url
            );
        }

        Ok(integrity_failures)
    }

    /// Clear tracked changes for a repository
    pub async fn clear_repository_changes(&self, repository_url: &str) {
        let mut file_states = self.file_states.write().await;
        file_states.remove(repository_url);
        debug!("Cleared tracked changes for repository: {}", repository_url);
    }

    /// Clear all tracked changes
    pub async fn clear_all_changes(&self) {
        let mut file_states = self.file_states.write().await;
        let mut events = self.events.write().await;

        file_states.clear();
        events.clear();

        info!("Cleared all tracked file changes");
    }

    // Private helper methods

    fn should_track_file(&self, file_path: &Path) -> bool {
        let path_str = file_path.to_string_lossy();

        // Check force track patterns first
        for pattern in &self.config.force_track_patterns {
            if glob_match::glob_match(pattern, &path_str) {
                return true;
            }
        }

        // Check ignore patterns
        for pattern in &self.config.ignore_patterns {
            if glob_match::glob_match(pattern, &path_str) {
                return false;
            }
        }

        // If track_all_files is enabled, track everything not ignored
        if self.config.track_all_files {
            return true;
        }

        // Otherwise, only track important file types
        FileType::from_path(file_path).should_track()
    }

    async fn calculate_file_hash(
        &self,
        repository_path: &Path,
        file_path: &Path,
    ) -> GitResult<String> {
        let full_path = repository_path.join(file_path);

        match tokio::fs::read(&full_path).await {
            Ok(content) => {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(&content);
                Ok(format!("{:x}", hasher.finalize()))
            }
            Err(e) => Err(GitError::Io { source: e }),
        }
    }

    async fn update_file_states(
        &self,
        repository_url: &str,
        tracked_changes: &[TrackedFileChange],
    ) {
        let mut file_states = self.file_states.write().await;
        let repo_states = file_states
            .entry(repository_url.to_string())
            .or_insert_with(HashMap::new);

        for tracked_change in tracked_changes {
            repo_states.insert(
                tracked_change.file_change.path.clone(),
                tracked_change.clone(),
            );
        }
    }

    async fn emit_change_event(&self, repository_url: &str, tracked_change: &TrackedFileChange) {
        let event = match tracked_change.file_type {
            FileType::Policy => ChangeNotificationEvent::PolicyFileChanged {
                repository_url: repository_url.to_string(),
                file_path: tracked_change.file_change.path.clone(),
                change_type: tracked_change.file_change.status,
                content_hash: tracked_change.content_hash.clone(),
                timestamp: tracked_change.detected_at,
            },
            FileType::Template => ChangeNotificationEvent::TemplateFileChanged {
                repository_url: repository_url.to_string(),
                file_path: tracked_change.file_change.path.clone(),
                change_type: tracked_change.file_change.status,
                content_hash: tracked_change.content_hash.clone(),
                timestamp: tracked_change.detected_at,
            },
            FileType::Config => ChangeNotificationEvent::ConfigFileChanged {
                repository_url: repository_url.to_string(),
                file_path: tracked_change.file_change.path.clone(),
                change_type: tracked_change.file_change.status,
                content_hash: tracked_change.content_hash.clone(),
                timestamp: tracked_change.detected_at,
            },
            _ => return, // Don't emit events for other file types
        };

        self.emit_event(event).await;
    }

    async fn emit_event(&self, event: ChangeNotificationEvent) {
        let mut events = self.events.write().await;
        events.push(event);

        // Trim events if we exceed the maximum
        if events.len() > self.config.max_events {
            let events_len = events.len();
            events.drain(0..events_len - self.config.max_events);
        }
    }
}

impl Default for FileChangeTracker {
    fn default() -> Self {
        Self::new()
    }
}
