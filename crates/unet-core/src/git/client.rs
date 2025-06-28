//! High-level Git client for μNet

use crate::git::credentials::{GitCredentialProvider, MemoryCredentialProvider};
use crate::git::repository::GitRepository;
use crate::git::state::{GitState, GitStateTracker};
use crate::git::types::{GitError, GitResult};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Configuration for the Git client
#[derive(Debug, Clone)]
pub struct GitClientConfig {
    /// Base directory for cloning repositories
    pub base_directory: PathBuf,
    /// Default sync interval in minutes
    pub default_sync_interval: u64,
    /// Maximum age for state information in minutes
    pub max_state_age: u64,
    /// Whether to automatically fetch on repository open
    pub auto_fetch: bool,
    /// Whether to automatically clean up stale repositories
    pub auto_cleanup: bool,
}

impl Default for GitClientConfig {
    fn default() -> Self {
        Self {
            base_directory: PathBuf::from("./git-repos"),
            default_sync_interval: 30, // 30 minutes
            max_state_age: 5,          // 5 minutes
            auto_fetch: true,
            auto_cleanup: false,
        }
    }
}

/// High-level Git client for managing multiple repositories
pub struct GitClient {
    /// Configuration
    config: GitClientConfig,
    /// Credential provider
    credential_provider: Arc<dyn GitCredentialProvider>,
    /// Repository instances indexed by URL
    repositories: Arc<RwLock<HashMap<String, Arc<GitRepository>>>>,
    /// State tracker
    state_tracker: Arc<GitStateTracker>,
}

impl GitClient {
    /// Create a new Git client with default configuration
    pub fn new() -> Self {
        Self::with_config(GitClientConfig::default())
    }

    /// Create a new Git client with custom configuration
    pub fn with_config(config: GitClientConfig) -> Self {
        let credential_provider = Arc::new(MemoryCredentialProvider::new());
        let state_tracker = Arc::new(GitStateTracker::new());

        Self {
            config,
            credential_provider,
            repositories: Arc::new(RwLock::new(HashMap::new())),
            state_tracker,
        }
    }

    /// Create a new Git client with custom credential provider
    pub fn with_credential_provider(
        config: GitClientConfig,
        credential_provider: Arc<dyn GitCredentialProvider>,
    ) -> Self {
        let state_tracker = Arc::new(GitStateTracker::new());

        Self {
            config,
            credential_provider,
            repositories: Arc::new(RwLock::new(HashMap::new())),
            state_tracker,
        }
    }

    /// Get the credential provider
    pub fn credential_provider(&self) -> Arc<dyn GitCredentialProvider> {
        self.credential_provider.clone()
    }

    /// Get the state tracker
    pub fn state_tracker(&self) -> Arc<GitStateTracker> {
        self.state_tracker.clone()
    }

    /// Add or open a repository
    pub async fn add_repository(
        &self,
        url: &str,
        local_name: Option<&str>,
    ) -> GitResult<Arc<GitRepository>> {
        info!("Adding repository: {}", url);

        // Check if repository is already managed
        {
            let repositories = self.repositories.read().await;
            if let Some(repo) = repositories.get(url) {
                info!("Repository already managed: {}", url);
                return Ok(repo.clone());
            }
        }

        // Determine local path
        let local_name = local_name.unwrap_or_else(|| {
            url.split('/')
                .last()
                .unwrap_or("repository")
                .trim_end_matches(".git")
        });
        let local_path = self.config.base_directory.join(local_name);

        // Create base directory if it doesn't exist
        if !self.config.base_directory.exists() {
            tokio::fs::create_dir_all(&self.config.base_directory).await?;
        }

        // Clone or open repository
        let repository = if local_path.exists() && local_path.join(".git").exists() {
            info!("Opening existing repository at: {}", local_path.display());
            GitRepository::open(&local_path, self.credential_provider.clone())?
        } else {
            info!("Cloning repository to: {}", local_path.display());
            GitRepository::clone(url, &local_path, self.credential_provider.clone())?
        };

        // Register with state tracker
        self.state_tracker
            .register_repository(repository.info().clone())
            .await?;

        // Update state
        self.state_tracker
            .update_repository_state(url, &repository)
            .await?;

        // Auto-fetch if configured
        if self.config.auto_fetch {
            if let Err(e) = repository.fetch(None) {
                warn!("Auto-fetch failed for {}: {}", url, e);
            }
        }

        // Store repository
        let repository = Arc::new(repository);
        {
            let mut repositories = self.repositories.write().await;
            repositories.insert(url.to_string(), repository.clone());
        }

        info!("Successfully added repository: {}", url);
        Ok(repository)
    }

    /// Get a repository by URL
    pub async fn get_repository(&self, url: &str) -> Option<Arc<GitRepository>> {
        let repositories = self.repositories.read().await;
        repositories.get(url).cloned()
    }

    /// List all managed repositories (URLs only)
    pub async fn list_repositories(&self) -> Vec<String> {
        let repositories = self.repositories.read().await;
        repositories.keys().cloned().collect()
    }

    /// List all managed repositories with detailed information
    pub async fn list_repository_info(&self) -> Vec<crate::git::types::RepositoryInfo> {
        let repositories = self.repositories.read().await;
        repositories
            .values()
            .map(|repo| repo.info().clone())
            .collect()
    }

    /// Remove a repository from management (does not delete files)
    pub async fn remove_repository(&self, url: &str) -> GitResult<()> {
        info!("Removing repository from management: {}", url);

        {
            let mut repositories = self.repositories.write().await;
            repositories.remove(url);
        }

        self.state_tracker.unregister_repository(url).await?;

        info!("Successfully removed repository from management: {}", url);
        Ok(())
    }

    /// Sync a specific repository (fetch + pull)
    pub async fn sync_repository(&self, url: &str) -> GitResult<()> {
        info!("Syncing repository: {}", url);

        let repository =
            self.get_repository(url)
                .await
                .ok_or_else(|| GitError::RepositoryOperation {
                    message: format!("Repository not managed: {}", url),
                })?;

        match repository.pull(None, None) {
            Ok(()) => {
                self.state_tracker
                    .mark_sync_performed(url, true, None)
                    .await?;

                // Update state after sync
                self.state_tracker
                    .update_repository_state(url, &repository)
                    .await?;

                info!("Successfully synced repository: {}", url);
                Ok(())
            }
            Err(e) => {
                let error_msg = e.to_string();
                self.state_tracker
                    .mark_sync_performed(url, false, Some(error_msg.clone()))
                    .await?;

                error!("Failed to sync repository {}: {}", url, error_msg);
                Err(e)
            }
        }
    }

    /// Sync all repositories that need syncing
    pub async fn sync_all_repositories(&self) -> GitResult<HashMap<String, Result<(), String>>> {
        info!("Syncing all repositories that need updates");

        let repositories_to_sync = self
            .state_tracker
            .get_repositories_needing_sync(self.config.default_sync_interval)
            .await;

        let mut results = HashMap::new();

        for url in repositories_to_sync {
            match self.sync_repository(&url).await {
                Ok(()) => {
                    results.insert(url, Ok(()));
                }
                Err(e) => {
                    results.insert(url, Err(e.to_string()));
                }
            }
        }

        info!("Completed syncing repositories");
        Ok(results)
    }

    /// Refresh state for all repositories
    pub async fn refresh_all_state(&self) -> GitResult<()> {
        info!("Refreshing state for all repositories");

        let repositories = {
            let repos = self.repositories.read().await;
            repos.clone()
        };

        for (url, repository) in repositories {
            if let Err(e) = self
                .state_tracker
                .update_repository_state(&url, &repository)
                .await
            {
                warn!("Failed to update state for repository {}: {}", url, e);
            }
        }

        info!("Completed state refresh for all repositories");
        Ok(())
    }

    /// Get repositories with stale state
    pub async fn get_repositories_with_stale_state(&self) -> Vec<String> {
        self.state_tracker
            .get_repositories_with_stale_state(self.config.max_state_age)
            .await
    }

    /// Get state for a specific repository
    pub async fn get_repository_state(&self, url: &str) -> Option<GitState> {
        self.state_tracker.get_repository_state(url).await
    }

    /// Get state for all repositories
    pub async fn get_all_repository_states(&self) -> HashMap<String, GitState> {
        self.state_tracker.get_all_states().await
    }

    /// Clone a repository to a specific path (without adding to management)
    pub async fn clone_to_path<P: AsRef<Path>>(
        &self,
        url: &str,
        path: P,
    ) -> GitResult<GitRepository> {
        info!("Cloning {} to {}", url, path.as_ref().display());
        GitRepository::clone(url, path, self.credential_provider.clone())
    }

    /// Open an existing repository (without adding to management)
    pub async fn open_repository<P: AsRef<Path>>(&self, path: P) -> GitResult<GitRepository> {
        GitRepository::open(path, self.credential_provider.clone())
    }

    /// Check if a repository URL is already managed
    pub async fn is_repository_managed(&self, url: &str) -> bool {
        let repositories = self.repositories.read().await;
        repositories.contains_key(url)
    }

    /// Get configuration
    pub fn config(&self) -> &GitClientConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: GitClientConfig) {
        self.config = config;
    }

    /// Cleanup stale repositories (if auto_cleanup is enabled)
    pub async fn cleanup_stale_repositories(&self) -> GitResult<Vec<String>> {
        if !self.config.auto_cleanup {
            return Ok(Vec::new());
        }

        info!("Cleaning up stale repositories");

        let stale_repositories = self.get_repositories_with_stale_state().await;
        let mut cleaned_up = Vec::new();

        for url in stale_repositories {
            // Only remove from management, don't delete files
            if let Err(e) = self.remove_repository(&url).await {
                warn!("Failed to cleanup repository {}: {}", url, e);
            } else {
                cleaned_up.push(url);
            }
        }

        info!("Cleaned up {} stale repositories", cleaned_up.len());
        Ok(cleaned_up)
    }
}

impl Default for GitClient {
    fn default() -> Self {
        Self::new()
    }
}
