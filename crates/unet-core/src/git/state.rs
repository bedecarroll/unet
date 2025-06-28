//! Git repository state tracking

use crate::git::types::{GitError, GitResult, RepositoryInfo};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Git repository state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitState {
    /// Repository information
    pub repository: RepositoryInfo,
    /// Current HEAD commit hash
    pub head_commit: String,
    /// Current branch name
    pub current_branch: String,
    /// Whether the working directory is clean
    pub is_clean: bool,
    /// Number of untracked files
    pub untracked_files: usize,
    /// Number of modified files
    pub modified_files: usize,
    /// Number of staged files
    pub staged_files: usize,
    /// Last sync with remote timestamp
    pub last_sync: Option<DateTime<Utc>>,
    /// Last state check timestamp
    pub last_check: DateTime<Utc>,
    /// Error message if state check failed
    pub error: Option<String>,
}

impl GitState {
    /// Create a new Git state
    pub fn new(repository: RepositoryInfo) -> Self {
        Self {
            repository,
            head_commit: String::new(),
            current_branch: String::new(),
            is_clean: true,
            untracked_files: 0,
            modified_files: 0,
            staged_files: 0,
            last_sync: None,
            last_check: Utc::now(),
            error: None,
        }
    }

    /// Update the state with current repository status
    pub fn update_from_repository(&mut self, repo: &crate::git::repository::GitRepository) {
        match repo.status() {
            Ok(status) => {
                self.head_commit = status.latest_commit.hash;
                self.current_branch = status.current_branch;
                self.is_clean = status.is_clean;

                // Count different types of changes
                self.untracked_files = status
                    .changed_files
                    .iter()
                    .filter(|f| f.status == crate::git::types::FileStatus::Untracked)
                    .count();

                self.modified_files = status
                    .changed_files
                    .iter()
                    .filter(|f| f.status == crate::git::types::FileStatus::Modified)
                    .count();

                self.staged_files = status
                    .changed_files
                    .iter()
                    .filter(|f| f.status == crate::git::types::FileStatus::Added)
                    .count();

                self.last_check = Utc::now();
                self.error = None;
            }
            Err(e) => {
                warn!("Failed to update Git state: {}", e);
                self.error = Some(e.to_string());
                self.last_check = Utc::now();
            }
        }
    }

    /// Mark that a sync operation was performed
    pub fn mark_synced(&mut self) {
        self.last_sync = Some(Utc::now());
    }

    /// Check if the repository needs syncing
    pub fn needs_sync(&self, sync_interval_minutes: u64) -> bool {
        match self.last_sync {
            Some(last_sync) => {
                let elapsed = Utc::now().signed_duration_since(last_sync);
                elapsed.num_minutes() >= sync_interval_minutes as i64
            }
            None => true, // Never synced
        }
    }

    /// Check if the state information is stale
    pub fn is_stale(&self, max_age_minutes: u64) -> bool {
        let elapsed = Utc::now().signed_duration_since(self.last_check);
        elapsed.num_minutes() >= max_age_minutes as i64
    }

    /// Get a summary of changes
    pub fn change_summary(&self) -> String {
        if self.is_clean {
            "Clean working directory".to_string()
        } else {
            let mut parts = Vec::new();

            if self.modified_files > 0 {
                parts.push(format!("{} modified", self.modified_files));
            }
            if self.staged_files > 0 {
                parts.push(format!("{} staged", self.staged_files));
            }
            if self.untracked_files > 0 {
                parts.push(format!("{} untracked", self.untracked_files));
            }

            if parts.is_empty() {
                "Unknown changes".to_string()
            } else {
                parts.join(", ")
            }
        }
    }
}

/// State change events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateChangeEvent {
    /// Repository was cloned or opened
    RepositoryOpened {
        repository_url: String,
        local_path: PathBuf,
        timestamp: DateTime<Utc>,
    },
    /// Branch was switched
    BranchSwitched {
        repository_url: String,
        from_branch: String,
        to_branch: String,
        timestamp: DateTime<Utc>,
    },
    /// Changes were detected in working directory
    WorkingDirectoryChanged {
        repository_url: String,
        change_summary: String,
        timestamp: DateTime<Utc>,
    },
    /// Sync operation was performed
    SyncPerformed {
        repository_url: String,
        success: bool,
        error_message: Option<String>,
        timestamp: DateTime<Utc>,
    },
    /// Repository state check failed
    StateCheckFailed {
        repository_url: String,
        error_message: String,
        timestamp: DateTime<Utc>,
    },
}

/// Git state tracker for managing multiple repositories
#[derive(Debug)]
pub struct GitStateTracker {
    /// Repository states indexed by URL
    states: Arc<RwLock<HashMap<String, GitState>>>,
    /// State change event history
    events: Arc<RwLock<Vec<StateChangeEvent>>>,
    /// Maximum number of events to keep
    max_events: usize,
}

impl GitStateTracker {
    /// Create a new Git state tracker
    pub fn new() -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
            events: Arc::new(RwLock::new(Vec::new())),
            max_events: 1000,
        }
    }

    /// Create a new Git state tracker with custom event limit
    pub fn with_max_events(max_events: usize) -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
            events: Arc::new(RwLock::new(Vec::new())),
            max_events,
        }
    }

    /// Register a repository for state tracking
    pub async fn register_repository(&self, repository: RepositoryInfo) -> GitResult<()> {
        let mut states = self.states.write().await;
        let state = GitState::new(repository.clone());
        states.insert(repository.url.clone(), state);

        self.emit_event(StateChangeEvent::RepositoryOpened {
            repository_url: repository.url,
            local_path: repository.local_path,
            timestamp: Utc::now(),
        })
        .await;

        info!("Registered repository for state tracking");
        Ok(())
    }

    /// Update the state of a repository
    pub async fn update_repository_state(
        &self,
        repository_url: &str,
        repo: &crate::git::repository::GitRepository,
    ) -> GitResult<()> {
        let mut states = self.states.write().await;

        if let Some(state) = states.get_mut(repository_url) {
            let old_branch = state.current_branch.clone();
            let old_clean = state.is_clean;

            state.update_from_repository(repo);

            // Emit events for significant changes
            if old_branch != state.current_branch && !old_branch.is_empty() {
                self.emit_event(StateChangeEvent::BranchSwitched {
                    repository_url: repository_url.to_string(),
                    from_branch: old_branch,
                    to_branch: state.current_branch.clone(),
                    timestamp: Utc::now(),
                })
                .await;
            }

            if old_clean && !state.is_clean {
                self.emit_event(StateChangeEvent::WorkingDirectoryChanged {
                    repository_url: repository_url.to_string(),
                    change_summary: state.change_summary(),
                    timestamp: Utc::now(),
                })
                .await;
            }

            debug!("Updated repository state for {}", repository_url);
            Ok(())
        } else {
            Err(GitError::RepositoryOperation {
                message: format!("Repository not registered: {}", repository_url),
            })
        }
    }

    /// Mark that a sync operation was performed
    pub async fn mark_sync_performed(
        &self,
        repository_url: &str,
        success: bool,
        error_message: Option<String>,
    ) -> GitResult<()> {
        let mut states = self.states.write().await;

        if let Some(state) = states.get_mut(repository_url) {
            if success {
                state.mark_synced();
            }

            self.emit_event(StateChangeEvent::SyncPerformed {
                repository_url: repository_url.to_string(),
                success,
                error_message,
                timestamp: Utc::now(),
            })
            .await;

            Ok(())
        } else {
            Err(GitError::RepositoryOperation {
                message: format!("Repository not registered: {}", repository_url),
            })
        }
    }

    /// Get the current state of a repository
    pub async fn get_repository_state(&self, repository_url: &str) -> Option<GitState> {
        let states = self.states.read().await;
        states.get(repository_url).cloned()
    }

    /// Get all repository states
    pub async fn get_all_states(&self) -> HashMap<String, GitState> {
        let states = self.states.read().await;
        states.clone()
    }

    /// Get repositories that need syncing
    pub async fn get_repositories_needing_sync(&self, sync_interval_minutes: u64) -> Vec<String> {
        let states = self.states.read().await;
        states
            .iter()
            .filter(|(_, state)| state.needs_sync(sync_interval_minutes))
            .map(|(url, _)| url.clone())
            .collect()
    }

    /// Get repositories with stale state information
    pub async fn get_repositories_with_stale_state(&self, max_age_minutes: u64) -> Vec<String> {
        let states = self.states.read().await;
        states
            .iter()
            .filter(|(_, state)| state.is_stale(max_age_minutes))
            .map(|(url, _)| url.clone())
            .collect()
    }

    /// Get recent state change events
    pub async fn get_recent_events(&self, limit: Option<usize>) -> Vec<StateChangeEvent> {
        let events = self.events.read().await;
        let limit = limit.unwrap_or(self.max_events);

        if events.len() <= limit {
            events.clone()
        } else {
            events[events.len() - limit..].to_vec()
        }
    }

    /// Clear state for a repository
    pub async fn unregister_repository(&self, repository_url: &str) -> GitResult<()> {
        let mut states = self.states.write().await;

        if states.remove(repository_url).is_some() {
            info!("Unregistered repository: {}", repository_url);
            Ok(())
        } else {
            Err(GitError::RepositoryOperation {
                message: format!("Repository not registered: {}", repository_url),
            })
        }
    }

    /// Clear all state information
    pub async fn clear_all_state(&self) {
        let mut states = self.states.write().await;
        let mut events = self.events.write().await;

        states.clear();
        events.clear();

        info!("Cleared all Git state information");
    }

    // Private helper methods

    async fn emit_event(&self, event: StateChangeEvent) {
        let mut events = self.events.write().await;
        events.push(event);

        // Trim events if we exceed the maximum
        if events.len() > self.max_events {
            let events_len = events.len();
            events.drain(0..events_len - self.max_events);
        }
    }
}

impl Default for GitStateTracker {
    fn default() -> Self {
        Self::new()
    }
}
