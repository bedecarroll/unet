//! Background tasks for the μNet server

use chrono::{DateTime, Utc};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tokio::time::{interval, sleep};
use tracing::{debug, error, info, warn};

use unet_core::{
    config::Config,
    datastore::DataStore,
    git::{GitClient, GitClientConfig, GitError, GitRepository},
    policy_integration::PolicyService,
};

/// Git sync status information
#[derive(Debug, Clone)]
pub struct GitSyncStatus {
    pub repository_url: String,
    pub repository_type: String,
    pub last_sync_time: Option<DateTime<Utc>>,
    pub last_sync_status: SyncStatus,
    pub last_error: Option<String>,
    pub sync_count: u64,
    pub error_count: u64,
}

/// Sync status enum
#[derive(Debug, Clone)]
pub enum SyncStatus {
    Never,
    InProgress,
    Success,
    Failed,
    UpToDate,
}

/// Background task manager
#[derive(Clone)]
pub struct BackgroundTasks {
    config: Config,
    datastore: Arc<dyn DataStore + Send + Sync>,
    policy_service: PolicyService,
}

impl BackgroundTasks {
    /// Create a new background task manager
    pub fn new(
        config: Config,
        datastore: Arc<dyn DataStore + Send + Sync>,
        policy_service: PolicyService,
    ) -> Self {
        Self {
            config,
            datastore,
            policy_service,
        }
    }

    /// Sync policies from Git and update the policy engine
    pub async fn sync_policies(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting policy synchronization");

        let mut policy_service = self.policy_service.clone();

        // Sync and reload policies from Git
        match policy_service.sync_and_reload_policies().await {
            Ok(policies) => {
                info!("Successfully synced {} policies from Git", policies.len());

                // Optionally trigger immediate policy evaluation for critical updates
                if !policies.is_empty() {
                    debug!("Triggering policy evaluation after sync");
                    if let Err(e) = self.evaluate_synced_policies(&mut policy_service).await {
                        warn!("Failed to evaluate policies after sync: {}", e);
                    }
                }

                Ok(())
            }
            Err(e) => {
                error!("Failed to sync policies from Git: {}", e);
                Err(format!("Policy sync failed: {}", e).into())
            }
        }
    }

    /// Evaluate policies after sync (subset of nodes for quick validation)
    async fn evaluate_synced_policies(
        &self,
        policy_service: &mut PolicyService,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Get a small sample of nodes for validation
        let nodes = self
            .datastore
            .get_nodes_for_policy_evaluation()
            .await
            .map_err(|e| format!("Failed to get nodes: {}", e))?;

        let sample_size = std::cmp::min(nodes.len(), 5); // Evaluate max 5 nodes
        let sample_nodes = &nodes[..sample_size];

        if sample_nodes.is_empty() {
            debug!("No nodes available for policy validation");
            return Ok(());
        }

        info!(
            "Validating synced policies against {} nodes",
            sample_nodes.len()
        );

        let mut validation_errors = 0;
        for node in sample_nodes {
            match policy_service.evaluate_node(&*self.datastore, node).await {
                Ok(results) => {
                    debug!(
                        "Policy validation successful for node {} ({}): {} results",
                        node.id,
                        node.name,
                        results.len()
                    );
                }
                Err(e) => {
                    validation_errors += 1;
                    warn!(
                        "Policy validation failed for node {} ({}): {}",
                        node.id, node.name, e
                    );
                }
            }
        }

        if validation_errors > 0 {
            warn!(
                "Policy validation completed with {} errors out of {} nodes",
                validation_errors,
                sample_nodes.len()
            );
        } else {
            info!(
                "Policy validation successful for all {} nodes",
                sample_nodes.len()
            );
        }

        Ok(())
    }

    /// Start all background tasks
    pub async fn start(&self) {
        info!("Starting background tasks");

        // Start policy evaluation task
        let policy_task = PolicyEvaluationTask {
            datastore: self.datastore.clone(),
            policy_service: self.policy_service.clone(),
            interval_seconds: self.config.git.sync_interval,
        };

        tokio::spawn(async move {
            policy_task.run().await;
        });

        // Start Git sync task if repositories are configured
        if self.config.git.policies_repo.is_some() || self.config.git.templates_repo.is_some() {
            let git_sync_task = GitSyncTask {
                config: self.config.clone(),
                interval_seconds: self.config.git.sync_interval,
            };

            // Also start a policy sync task that coordinates with Git sync
            let policy_sync_task = PolicySyncTask {
                background_tasks: self.clone(),
                interval_seconds: self.config.git.sync_interval,
            };

            tokio::spawn(async move {
                git_sync_task.run().await;
            });

            tokio::spawn(async move {
                policy_sync_task.run().await;
            });
        }

        info!("Background tasks started");
    }
}

/// Background task for periodic policy evaluation
struct PolicyEvaluationTask {
    datastore: Arc<dyn DataStore + Send + Sync>,
    policy_service: PolicyService,
    interval_seconds: u64,
}

impl PolicyEvaluationTask {
    /// Run the policy evaluation task
    async fn run(&self) {
        info!(
            "Starting policy evaluation background task with interval: {}s",
            self.interval_seconds
        );

        // Wait a bit before starting the first evaluation
        sleep(Duration::from_secs(30)).await;

        let mut interval = interval(Duration::from_secs(self.interval_seconds));

        loop {
            interval.tick().await;

            debug!("Running periodic policy evaluation");

            if let Err(e) = self.evaluate_all_policies().await {
                error!("Policy evaluation failed: {}", e);
            }
        }
    }

    /// Evaluate policies for all nodes
    async fn evaluate_all_policies(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let start_time = std::time::Instant::now();

        // Get all nodes that need policy evaluation
        let nodes = self
            .datastore
            .get_nodes_for_policy_evaluation()
            .await
            .map_err(|e| format!("Failed to get nodes: {}", e))?;

        if nodes.is_empty() {
            debug!("No nodes found for policy evaluation");
            return Ok(());
        }

        info!("Evaluating policies for {} nodes", nodes.len());

        // Clone policy service for mutable operations
        let mut policy_service = self.policy_service.clone();

        // Load policies
        let policies = policy_service
            .load_policies()
            .await
            .map_err(|e| format!("Failed to load policies: {}", e))?;

        if policies.is_empty() {
            debug!("No policies loaded for evaluation");
            return Ok(());
        }

        info!("Loaded {} policies for evaluation", policies.len());

        let mut total_results = 0;
        let mut successful_evaluations = 0;
        let mut failed_evaluations = 0;

        // Evaluate policies for each node
        for node in &nodes {
            match policy_service.evaluate_node(&*self.datastore, node).await {
                Ok(results) => {
                    total_results += results.len();
                    successful_evaluations += 1;

                    // Store results in the database
                    if let Err(e) = policy_service
                        .store_results(&*self.datastore, &node.id, &results)
                        .await
                    {
                        warn!("Failed to store policy results for node {}: {}", node.id, e);
                    }

                    debug!(
                        "Evaluated {} policies for node {} ({})",
                        results.len(),
                        node.id,
                        node.name
                    );
                }
                Err(e) => {
                    failed_evaluations += 1;
                    error!(
                        "Failed to evaluate policies for node {} ({}): {}",
                        node.id, node.name, e
                    );
                }
            }
        }

        let duration = start_time.elapsed();

        info!(
            "Policy evaluation completed: {} nodes processed, {} successful, {} failed, {} total results, took {:?}",
            nodes.len(),
            successful_evaluations,
            failed_evaluations,
            total_results,
            duration
        );

        Ok(())
    }
}

/// Background task for SNMP polling (existing functionality)
pub struct SnmpPollingTask {
    datastore: Arc<dyn DataStore + Send + Sync>,
    interval_seconds: u64,
}

impl SnmpPollingTask {
    /// Create a new SNMP polling task
    pub fn new(datastore: Arc<dyn DataStore + Send + Sync>, interval_seconds: u64) -> Self {
        Self {
            datastore,
            interval_seconds,
        }
    }

    /// Run the SNMP polling task
    pub async fn run(&self) {
        info!(
            "Starting SNMP polling background task with interval: {}s",
            self.interval_seconds
        );

        // Wait a bit before starting the first poll
        sleep(Duration::from_secs(15)).await;

        let mut interval = interval(Duration::from_secs(self.interval_seconds));

        loop {
            interval.tick().await;

            debug!("Running periodic SNMP polling");

            if let Err(e) = self.poll_all_nodes().await {
                error!("SNMP polling failed: {}", e);
            }
        }
    }

    /// Poll all nodes for SNMP data
    async fn poll_all_nodes(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Get all nodes that need SNMP polling
        let nodes = self
            .datastore
            .get_nodes_for_policy_evaluation()
            .await
            .map_err(|e| format!("Failed to get nodes: {}", e))?;

        if nodes.is_empty() {
            debug!("No nodes found for SNMP polling");
            return Ok(());
        }

        info!("Polling {} nodes for SNMP data", nodes.len());

        // TODO: Implement actual SNMP polling here
        // For now, this is a placeholder for future SNMP integration

        Ok(())
    }
}

/// Global sync status store
static SYNC_STATUS_STORE: std::sync::LazyLock<
    Arc<RwLock<std::collections::HashMap<String, GitSyncStatus>>>,
> = std::sync::LazyLock::new(|| Arc::new(RwLock::new(std::collections::HashMap::new())));

/// Background task for Git repository synchronization
struct GitSyncTask {
    config: Config,
    interval_seconds: u64,
}

/// Retry configuration for Git operations
struct GitRetryConfig {
    max_retries: u32,
    initial_delay_seconds: u64,
    max_delay_seconds: u64,
    backoff_multiplier: f64,
}

impl Default for GitRetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_seconds: 5,
            max_delay_seconds: 300, // 5 minutes
            backoff_multiplier: 2.0,
        }
    }
}

impl GitSyncTask {
    /// Determine if a Git error is retryable
    fn is_retryable_error(error: &Box<dyn std::error::Error + Send + Sync>) -> bool {
        let error_str = error.to_string().to_lowercase();

        // Network-related errors are usually retryable
        if error_str.contains("network")
            || error_str.contains("timeout")
            || error_str.contains("connection")
            || error_str.contains("dns")
            || error_str.contains("resolve")
        {
            return true;
        }

        // Temporary server errors are retryable
        if error_str.contains("server error")
            || error_str.contains("503")
            || error_str.contains("502")
            || error_str.contains("504")
        {
            return true;
        }

        // Authentication and repository not found errors are not retryable
        if error_str.contains("authentication")
            || error_str.contains("not found")
            || error_str.contains("access denied")
            || error_str.contains("permission denied")
        {
            return false;
        }

        // Merge conflicts might be resolvable with retries
        if error_str.contains("merge conflict") {
            return true;
        }

        // Default to retryable for unknown errors
        true
    }

    /// Update sync status for a repository
    async fn update_sync_status(
        repo_url: &str,
        repo_type: &str,
        status: SyncStatus,
        error: Option<String>,
    ) {
        let key = format!("{}:{}", repo_type, repo_url);
        let mut store = SYNC_STATUS_STORE.write().await;

        if let Some(existing_status) = store.get_mut(&key) {
            existing_status.last_sync_time = Some(Utc::now());
            existing_status.last_sync_status = status.clone();
            existing_status.last_error = error;

            match status {
                SyncStatus::Success | SyncStatus::UpToDate => {
                    existing_status.sync_count += 1;
                }
                SyncStatus::Failed => {
                    existing_status.error_count += 1;
                }
                _ => {}
            }
        } else {
            let new_status = GitSyncStatus {
                repository_url: repo_url.to_string(),
                repository_type: repo_type.to_string(),
                last_sync_time: Some(Utc::now()),
                last_sync_status: status.clone(),
                last_error: error,
                sync_count: if matches!(status, SyncStatus::Success | SyncStatus::UpToDate) {
                    1
                } else {
                    0
                },
                error_count: if matches!(status, SyncStatus::Failed) {
                    1
                } else {
                    0
                },
            };
            store.insert(key, new_status);
        }
    }

    /// Get sync status for all repositories
    pub async fn get_sync_status() -> std::collections::HashMap<String, GitSyncStatus> {
        SYNC_STATUS_STORE.read().await.clone()
    }

    /// Get sync status for a specific repository
    pub async fn get_repository_sync_status(
        repo_url: &str,
        repo_type: &str,
    ) -> Option<GitSyncStatus> {
        let key = format!("{}:{}", repo_type, repo_url);
        SYNC_STATUS_STORE.read().await.get(&key).cloned()
    }
    /// Run the Git sync task
    async fn run(&self) {
        info!(
            "Starting Git sync background task with interval: {}s",
            self.interval_seconds
        );

        // Wait a bit before starting the first sync
        sleep(Duration::from_secs(45)).await;

        let mut interval = interval(Duration::from_secs(self.interval_seconds));

        loop {
            interval.tick().await;

            debug!("Running periodic Git sync");

            if let Err(e) = self.sync_repositories().await {
                error!("Git sync failed: {}", e);
            }
        }
    }

    /// Sync all configured repositories
    async fn sync_repositories(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let start_time = std::time::Instant::now();
        info!("Starting Git repository synchronization");

        let mut sync_count = 0;
        let mut error_count = 0;

        // Sync policies repository if configured
        if let Some(policies_repo) = &self.config.git.policies_repo {
            match self
                .sync_repository_with_retry(policies_repo, "policies")
                .await
            {
                Ok(()) => {
                    sync_count += 1;
                    info!("Successfully synced policies repository: {}", policies_repo);
                }
                Err(e) => {
                    error_count += 1;
                    error!(
                        "Failed to sync policies repository {}: {}",
                        policies_repo, e
                    );
                }
            }
        }

        // Sync templates repository if configured
        if let Some(templates_repo) = &self.config.git.templates_repo {
            match self
                .sync_repository_with_retry(templates_repo, "templates")
                .await
            {
                Ok(()) => {
                    sync_count += 1;
                    info!(
                        "Successfully synced templates repository: {}",
                        templates_repo
                    );
                }
                Err(e) => {
                    error_count += 1;
                    error!(
                        "Failed to sync templates repository {}: {}",
                        templates_repo, e
                    );
                }
            }
        }

        let duration = start_time.elapsed();
        info!(
            "Git sync completed: {} repositories synced, {} errors, took {:?}",
            sync_count, error_count, duration
        );

        if error_count > 0 {
            return Err(format!("Git sync completed with {} errors", error_count).into());
        }

        Ok(())
    }

    /// Sync a repository with retry logic and error handling
    async fn sync_repository_with_retry(
        &self,
        repo_url: &str,
        repo_type: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Mark as in progress
        Self::update_sync_status(repo_url, repo_type, SyncStatus::InProgress, None).await;

        let retry_config = GitRetryConfig::default();
        let mut attempt = 0;
        let mut last_error = None;

        while attempt <= retry_config.max_retries {
            match self.sync_repository(repo_url, repo_type).await {
                Ok(()) => {
                    Self::update_sync_status(repo_url, repo_type, SyncStatus::Success, None).await;
                    return Ok(());
                }
                Err(e) => {
                    // Check if this is an "up to date" status
                    if e.to_string().contains("UP_TO_DATE") {
                        Self::update_sync_status(repo_url, repo_type, SyncStatus::UpToDate, None)
                            .await;
                        return Ok(());
                    }
                    let is_retryable = Self::is_retryable_error(&e);
                    last_error = Some(e);
                    attempt += 1;

                    if !is_retryable {
                        error!(
                            "Non-retryable error for {} repository {}, giving up: {}",
                            repo_type,
                            repo_url,
                            last_error.as_ref().unwrap()
                        );
                        break;
                    }

                    if attempt <= retry_config.max_retries {
                        let delay = std::cmp::min(
                            retry_config.initial_delay_seconds
                                * (retry_config.backoff_multiplier.powi(attempt as i32 - 1) as u64),
                            retry_config.max_delay_seconds,
                        );

                        warn!(
                            "Git sync attempt {} failed for {} repository {}, retrying in {}s: {}",
                            attempt,
                            repo_type,
                            repo_url,
                            delay,
                            last_error.as_ref().unwrap()
                        );

                        sleep(Duration::from_secs(delay)).await;
                    } else {
                        warn!(
                            "Max retries ({}) exceeded for {} repository {}",
                            retry_config.max_retries, repo_type, repo_url
                        );
                    }
                }
            }
        }

        let final_error = format!(
            "Git sync failed after {} attempts for {} repository {}: {}",
            retry_config.max_retries + 1,
            repo_type,
            repo_url,
            last_error.as_ref().unwrap()
        );

        Self::update_sync_status(
            repo_url,
            repo_type,
            SyncStatus::Failed,
            Some(final_error.clone()),
        )
        .await;
        Err(final_error.into())
    }

    /// Sync a single repository using a temporary GitClient with incremental updates
    async fn sync_repository(
        &self,
        repo_url: &str,
        repo_type: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!("Syncing {} repository: {}", repo_type, repo_url);

        // Use tokio::task::spawn_blocking for Git operations since git2 is not async
        let repo_url = repo_url.to_string();
        let repo_type = repo_type.to_string();
        let branch = self.config.git.branch.clone();

        tokio::task::spawn_blocking(move || {
            // Create a new GitClient for this operation
            let git_config = GitClientConfig {
                base_directory: std::path::PathBuf::from("./git-repos"),
                default_sync_interval: 30,
                max_state_age: 5,
                auto_fetch: true,
                auto_cleanup: false,
            };
            let git_client = GitClient::with_config(git_config);

            // Determine local path
            let local_name = repo_url
                .split('/')
                .last()
                .unwrap_or("repository")
                .trim_end_matches(".git");
            let local_path = std::path::PathBuf::from("./git-repos").join(&repo_type);

            // Create base directory if it doesn't exist
            if !local_path.parent().unwrap().exists() {
                std::fs::create_dir_all(local_path.parent().unwrap())?;
            }

            // Clone or open repository directly using GitRepository
            let credential_provider = git_client.credential_provider();
            let repository = if local_path.exists() && local_path.join(".git").exists() {
                debug!("Opening existing repository at: {}", local_path.display());
                GitRepository::open(&local_path, credential_provider)?
            } else {
                debug!("Cloning repository to: {}", local_path.display());
                GitRepository::clone(&repo_url, &local_path, credential_provider)?
            };

            // Get current HEAD commit before fetch for incremental update detection
            let current_head = repository.get_current_commit_hash().unwrap_or_else(|_| {
                debug!("Could not get current HEAD, proceeding with full sync");
                "unknown".to_string()
            });

            // Fetch latest changes
            repository.fetch(None)?;

            // Check if there are new commits (incremental update detection)
            let latest_remote_commit = repository
                .get_latest_remote_commit_hash(&branch)
                .unwrap_or_else(|_| {
                    debug!("Could not get latest remote commit, proceeding with pull");
                    "unknown".to_string()
                });

            if current_head == latest_remote_commit && current_head != "unknown" {
                debug!(
                    "Repository {} is already up to date (commit: {})",
                    repo_type,
                    current_head[..8].to_string()
                );
                // Return special value to indicate up-to-date status
                return Err(GitError::RepositoryOperation {
                    message: "UP_TO_DATE".to_string(),
                });
            }

            debug!(
                "Repository {} has updates: {} -> {}",
                repo_type,
                &current_head[..8],
                &latest_remote_commit[..8]
            );

            // Pull changes from the configured branch
            repository.pull(None, Some(&branch)).map_err(|e| match &e {
                GitError::MergeConflict { files } => {
                    // Log merge conflicts but don't fail immediately - could be resolved
                    warn!(
                        "Merge conflict detected in repository {}: files {:?}",
                        repo_type, files
                    );
                    GitError::RepositoryOperation {
                        message: format!(
                            "Merge conflict in files: {} - manual resolution may be required",
                            files.join(", ")
                        ),
                    }
                }
                GitError::Authentication { repository } => {
                    // Authentication errors are likely persistent
                    error!(
                        "Authentication failed for repository {}: {}",
                        repo_type, repository
                    );
                    e
                }
                GitError::Network { message } => {
                    // Network errors might be transient
                    warn!("Network error for repository {}: {}", repo_type, message);
                    e
                }
                GitError::RepositoryNotFound { path } => {
                    // Repository not found is a configuration issue
                    error!("Repository not found for {}: {}", repo_type, path);
                    e
                }
                _ => {
                    // Log other errors with context
                    warn!("Git operation failed for repository {}: {}", repo_type, e);
                    e
                }
            })?;

            debug!("Successfully synced {} repository", repo_type);
            Ok::<(), GitError>(())
        })
        .await
        .map_err(|e| format!("Task join error: {}", e))?
        .map_err(|e| format!("Git sync error: {}", e))?;

        Ok(())
    }
}

/// Background task for policy synchronization
struct PolicySyncTask {
    background_tasks: BackgroundTasks,
    interval_seconds: u64,
}

impl PolicySyncTask {
    /// Run the policy sync task
    async fn run(&self) {
        info!(
            "Starting policy sync background task with interval: {}s",
            self.interval_seconds
        );

        // Wait longer before starting the first policy sync to let Git sync complete first
        sleep(Duration::from_secs(60)).await;

        let mut interval = interval(Duration::from_secs(self.interval_seconds));

        loop {
            interval.tick().await;

            debug!("Running periodic policy synchronization");

            if let Err(e) = self.background_tasks.sync_policies().await {
                error!("Policy sync failed: {}", e);
            }
        }
    }
}
