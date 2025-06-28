//! Policy file loading with Git integration
//!
//! This module provides functionality to load policy files from local directories
//! or Git repositories, with validation, caching, and hot-reloading capabilities.

use crate::config::GitConfig;
use crate::policy::{PolicyError, PolicyParser, PolicyResult, PolicyRule};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tracing::{debug, info, warn};
use walkdir::WalkDir;

/// Policy file loader with Git integration and caching
#[derive(Debug, Clone)]
pub struct PolicyLoader {
    /// Git configuration
    git_config: GitConfig,
    /// Cached policies by file path
    policy_cache: HashMap<PathBuf, CachedPolicy>,
    /// Local policies directory
    local_dir: Option<PathBuf>,
    /// Cache expiry duration
    cache_ttl: Duration,
}

/// Cached policy with metadata
#[derive(Debug, Clone)]
struct CachedPolicy {
    /// Parsed policy rules
    rules: Vec<PolicyRule>,
    /// File modification time when cached
    mtime: SystemTime,
    /// Cache timestamp
    cached_at: SystemTime,
    /// File content hash for validation
    content_hash: u64,
}

/// Policy file metadata
#[derive(Debug, Clone)]
pub struct PolicyFile {
    /// File path relative to policies directory
    pub path: PathBuf,
    /// Parsed policy rules
    pub rules: Vec<PolicyRule>,
    /// File modification time
    pub modified: SystemTime,
    /// File size in bytes
    pub size: u64,
}

/// Policy loading results
#[derive(Debug)]
pub struct LoadResult {
    /// Successfully loaded policy files
    pub loaded: Vec<PolicyFile>,
    /// Files with parsing errors
    pub errors: Vec<(PathBuf, PolicyError)>,
    /// Total files processed
    pub total_files: usize,
}

impl PolicyLoader {
    /// Create a new policy loader with Git configuration
    pub fn new(git_config: GitConfig) -> Self {
        Self {
            git_config,
            policy_cache: HashMap::new(),
            local_dir: None,
            cache_ttl: Duration::from_secs(300), // 5 minutes default
        }
    }

    /// Set local policies directory (alternative to Git)
    pub fn with_local_dir<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.local_dir = Some(dir.into());
        self
    }

    /// Set cache TTL duration
    pub fn with_cache_ttl(mut self, ttl: Duration) -> Self {
        self.cache_ttl = ttl;
        self
    }

    /// Load all policy files from configured source
    pub async fn load_policies(&mut self) -> PolicyResult<LoadResult> {
        info!("Loading policy files");

        let policies_dir = self.get_policies_directory().await?;
        self.load_policies_from_directory(&policies_dir).await
    }

    /// Load policies from a specific directory
    pub async fn load_policies_from_directory(&mut self, dir: &Path) -> PolicyResult<LoadResult> {
        if !dir.exists() {
            return Err(PolicyError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Policies directory not found: {}", dir.display()),
            )));
        }

        debug!("Loading policies from directory: {}", dir.display());

        let mut loaded = Vec::new();
        let mut errors = Vec::new();
        let mut total_files = 0;

        // Walk the directory tree looking for .policy files
        for entry in WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "policy"))
        {
            total_files += 1;
            let file_path = entry.path();

            match self.load_policy_file(file_path).await {
                Ok(policy_file) => {
                    debug!("Loaded policy file: {}", file_path.display());
                    loaded.push(policy_file);
                }
                Err(e) => {
                    warn!("Failed to load policy file {}: {}", file_path.display(), e);
                    errors.push((file_path.to_path_buf(), e));
                }
            }
        }

        info!(
            "Policy loading complete: {} loaded, {} errors, {} total files",
            loaded.len(),
            errors.len(),
            total_files
        );

        Ok(LoadResult {
            loaded,
            errors,
            total_files,
        })
    }

    /// Load a single policy file with caching
    pub async fn load_policy_file(&mut self, path: &Path) -> PolicyResult<PolicyFile> {
        let metadata = std::fs::metadata(path).map_err(|e| PolicyError::Io(e))?;

        let mtime = metadata.modified().map_err(|e| PolicyError::Io(e))?;

        // Check cache first
        if let Some(cached) = self.policy_cache.get(path) {
            if self.is_cache_valid(cached, mtime) {
                debug!("Using cached policy: {}", path.display());
                return Ok(PolicyFile {
                    path: path.to_path_buf(),
                    rules: cached.rules.clone(),
                    modified: mtime,
                    size: metadata.len(),
                });
            }
        }

        // Load and parse the file
        debug!("Loading policy file from disk: {}", path.display());
        let content = std::fs::read_to_string(path).map_err(|e| PolicyError::Io(e))?;

        // Validate file format and parse policies
        let rules = self.parse_policy_file(&content)?;

        // Calculate content hash for validation
        let content_hash = self.calculate_hash(&content);

        // Update cache
        let cached_policy = CachedPolicy {
            rules: rules.clone(),
            mtime,
            cached_at: SystemTime::now(),
            content_hash,
        };
        self.policy_cache.insert(path.to_path_buf(), cached_policy);

        Ok(PolicyFile {
            path: path.to_path_buf(),
            rules,
            modified: mtime,
            size: metadata.len(),
        })
    }

    /// Parse policy file content into rules
    fn parse_policy_file(&self, content: &str) -> PolicyResult<Vec<PolicyRule>> {
        let mut rules = Vec::new();
        let mut line_number = 0;

        for line in content.lines() {
            line_number += 1;
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
                continue;
            }

            // Parse the policy rule
            match PolicyParser::parse_rule(line) {
                Ok(rule) => rules.push(rule),
                Err(e) => {
                    return Err(PolicyError::Parse(crate::policy::ParseError {
                        message: format!("Line {}: {}", line_number, e.message),
                        location: Some((line_number, 1)),
                    }));
                }
            }
        }

        if rules.is_empty() {
            warn!("No valid policy rules found in file");
        }

        Ok(rules)
    }

    /// Check if cached policy is still valid
    fn is_cache_valid(&self, cached: &CachedPolicy, current_mtime: SystemTime) -> bool {
        // Check if file has been modified
        if cached.mtime != current_mtime {
            return false;
        }

        // Check if cache has expired
        if let Ok(elapsed) = cached.cached_at.elapsed() {
            if elapsed > self.cache_ttl {
                return false;
            }
        }

        true
    }

    /// Calculate simple hash of content for validation
    fn calculate_hash(&self, content: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }

    /// Get the policies directory (from Git or local)
    async fn get_policies_directory(&self) -> PolicyResult<PathBuf> {
        if let Some(ref local_dir) = self.local_dir {
            return Ok(local_dir.clone());
        }

        if let Some(ref repo_url) = self.git_config.policies_repo {
            return self.sync_policies_from_git(repo_url).await;
        }

        Err(PolicyError::Evaluation {
            message: "No policies source configured (neither local directory nor Git repository)"
                .to_string(),
        })
    }

    /// Sync policies from Git repository
    async fn sync_policies_from_git(&self, repo_url: &str) -> PolicyResult<PathBuf> {
        use crate::git::{GitClient, GitClientConfig, GitRepository};

        debug!("Syncing policies from Git repository: {}", repo_url);

        // Clone for use in closure and logging
        let repo_url_for_closure = repo_url.to_string();
        let repo_url_for_logging = repo_url.to_string();
        let branch = self.git_config.branch.clone();

        let local_path = tokio::task::spawn_blocking(move || {
            // Create a new GitClient for this operation
            let git_config = GitClientConfig {
                base_directory: std::path::PathBuf::from("./git-repos"),
                default_sync_interval: 30,
                max_state_age: 5,
                auto_fetch: true,
                auto_cleanup: false,
            };
            let git_client = GitClient::with_config(git_config);

            // Determine local path for policies
            let local_path = std::path::PathBuf::from("./git-repos/policies");

            // Create base directory if it doesn't exist
            if !local_path.parent().unwrap().exists() {
                std::fs::create_dir_all(local_path.parent().unwrap())
                    .map_err(|e| format!("Failed to create git-repos directory: {}", e))?;
            }

            // Clone or open repository
            let credential_provider = git_client.credential_provider();
            let repository = if local_path.exists() && local_path.join(".git").exists() {
                debug!(
                    "Opening existing policies repository at: {}",
                    local_path.display()
                );
                GitRepository::open(&local_path, credential_provider)
                    .map_err(|e| format!("Failed to open existing repository: {}", e))?
            } else {
                debug!("Cloning policies repository to: {}", local_path.display());
                GitRepository::clone(&repo_url_for_closure, &local_path, credential_provider)
                    .map_err(|e| format!("Failed to clone repository: {}", e))?
            };

            // Fetch latest changes and pull
            repository
                .fetch(None)
                .map_err(|e| format!("Failed to fetch from repository: {}", e))?;

            repository
                .pull(None, Some(&branch))
                .map_err(|e| format!("Failed to pull changes: {}", e))?;

            debug!("Successfully synced policies repository");
            Ok::<PathBuf, String>(local_path)
        })
        .await
        .map_err(|e| PolicyError::Evaluation {
            message: format!("Task join error: {}", e),
        })?
        .map_err(|e| PolicyError::Evaluation { message: e })?;

        info!(
            "Policies synced from Git repository: {}",
            repo_url_for_logging
        );
        Ok(local_path)
    }

    /// Force reload policies from source (clears cache first)
    pub async fn reload_policies(&mut self) -> PolicyResult<LoadResult> {
        info!("Force reloading policies from source");
        self.clear_cache();
        self.load_policies().await
    }

    /// Sync policies from Git and reload them with validation
    pub async fn sync_and_reload(&mut self) -> PolicyResult<LoadResult> {
        info!("Syncing and reloading policies with validation");

        // Track currently cached files for removal detection
        let cached_files: HashSet<PathBuf> = self.policy_cache.keys().cloned().collect();

        // Clear cache to force reload
        self.clear_cache();

        // Sync from Git if configured
        if self.git_config.policies_repo.is_some() {
            let policies_dir = self.get_policies_directory().await?;

            // Validate all policy files after sync
            let validation_result = self.validate_policies_directory(&policies_dir).await?;

            if !validation_result.is_valid() {
                warn!(
                    "Policy validation found {} errors after sync from {}",
                    validation_result.error_count(),
                    self.git_config.policies_repo.as_ref().unwrap()
                );

                // Log validation errors
                for error in &validation_result.errors {
                    warn!(
                        "Policy validation error in {}: line {}: {}",
                        error.file_path.display(),
                        error.line,
                        error.message
                    );
                }
            } else {
                info!(
                    "Policy validation passed: {} files, {} rules validated",
                    validation_result.total_files, validation_result.total_valid_rules
                );
            }
        }

        // Load policies (this will also trigger Git sync if needed)
        let load_result = self.load_policies().await?;

        // Detect removed files by comparing with cached files
        let loaded_files: HashSet<PathBuf> =
            load_result.loaded.iter().map(|f| f.path.clone()).collect();

        let removed_files: Vec<&PathBuf> = cached_files.difference(&loaded_files).collect();

        if !removed_files.is_empty() {
            info!(
                "Detected {} policy file(s) removed from repository:",
                removed_files.len()
            );
            for removed_file in &removed_files {
                info!("  - Policy file removed: {}", removed_file.display());
            }
        }

        // Detect new files
        let new_files: Vec<&PathBuf> = loaded_files.difference(&cached_files).collect();

        if !new_files.is_empty() {
            info!(
                "Detected {} new policy file(s) in repository:",
                new_files.len()
            );
            for new_file in &new_files {
                info!("  - New policy file: {}", new_file.display());
            }
        }

        Ok(load_result)
    }

    /// Clear the policy cache
    pub fn clear_cache(&mut self) {
        self.policy_cache.clear();
        debug!("Policy cache cleared");
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        let total_entries = self.policy_cache.len();
        let expired_entries = self
            .policy_cache
            .values()
            .filter(|cached| {
                cached
                    .cached_at
                    .elapsed()
                    .map(|elapsed| elapsed > self.cache_ttl)
                    .unwrap_or(true)
            })
            .count();

        CacheStats {
            total_entries,
            expired_entries,
            valid_entries: total_entries - expired_entries,
        }
    }

    /// Validate all policy files in a directory
    pub async fn validate_policies_directory(
        &self,
        dir: &Path,
    ) -> PolicyResult<DirectoryValidationResult> {
        if !dir.exists() {
            return Err(PolicyError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Policies directory not found: {}", dir.display()),
            )));
        }

        debug!("Validating policies in directory: {}", dir.display());

        let mut total_files = 0;
        let mut total_valid_rules = 0;
        let mut errors = Vec::new();

        // Walk the directory tree looking for .policy files
        for entry in WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "policy"))
        {
            total_files += 1;
            let file_path = entry.path();

            // Read and validate the file
            match std::fs::read_to_string(file_path) {
                Ok(content) => {
                    match self.validate_policy_file(&content) {
                        Ok(file_result) => {
                            total_valid_rules += file_result.valid_rules;

                            // Convert file-level errors to directory-level errors
                            for file_error in file_result.errors {
                                errors.push(DirectoryValidationError {
                                    file_path: file_path.to_path_buf(),
                                    line: file_error.line,
                                    message: file_error.message,
                                    content: file_error.content,
                                });
                            }
                        }
                        Err(e) => {
                            errors.push(DirectoryValidationError {
                                file_path: file_path.to_path_buf(),
                                line: 0,
                                message: format!("Failed to validate file: {}", e),
                                content: String::new(),
                            });
                        }
                    }
                }
                Err(e) => {
                    errors.push(DirectoryValidationError {
                        file_path: file_path.to_path_buf(),
                        line: 0,
                        message: format!("Failed to read file: {}", e),
                        content: String::new(),
                    });
                }
            }
        }

        Ok(DirectoryValidationResult {
            total_files,
            total_valid_rules,
            errors,
        })
    }

    /// Validate a policy file format without parsing all rules
    pub fn validate_policy_file(&self, content: &str) -> PolicyResult<ValidationResult> {
        let mut total_lines = 0;
        let mut valid_rules = 0;
        let mut errors = Vec::new();

        for (line_number, line) in content.lines().enumerate() {
            let line_num = line_number + 1;
            total_lines += 1;
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
                continue;
            }

            // Try parsing the rule
            match PolicyParser::parse_rule(line) {
                Ok(_) => valid_rules += 1,
                Err(e) => {
                    errors.push(ValidationError {
                        line: line_num,
                        message: e.message,
                        content: line.to_string(),
                    });
                }
            }
        }

        Ok(ValidationResult {
            total_lines,
            valid_rules,
            errors,
        })
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub expired_entries: usize,
    pub valid_entries: usize,
}

/// Policy file validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub total_lines: usize,
    pub valid_rules: usize,
    pub errors: Vec<ValidationError>,
}

/// Validation error details
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub line: usize,
    pub message: String,
    pub content: String,
}

/// Directory validation result
#[derive(Debug, Clone)]
pub struct DirectoryValidationResult {
    pub total_files: usize,
    pub total_valid_rules: usize,
    pub errors: Vec<DirectoryValidationError>,
}

/// Directory validation error details
#[derive(Debug, Clone)]
pub struct DirectoryValidationError {
    pub file_path: PathBuf,
    pub line: usize,
    pub message: String,
    pub content: String,
}

impl ValidationResult {
    /// Check if validation passed (no errors)
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get error count
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }
}

impl DirectoryValidationResult {
    /// Check if validation passed (no errors)
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get error count
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Get files with errors
    pub fn files_with_errors(&self) -> Vec<&PathBuf> {
        self.errors.iter().map(|e| &e.file_path).collect()
    }

    /// Get error count by file
    pub fn errors_by_file(&self) -> std::collections::HashMap<&PathBuf, usize> {
        let mut error_counts = std::collections::HashMap::new();
        for error in &self.errors {
            *error_counts.entry(&error.file_path).or_insert(0) += 1;
        }
        error_counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GitConfig;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_policy_loader_local_directory() {
        let temp_dir = TempDir::new().unwrap();
        let policies_dir = temp_dir.path().join("policies");
        fs::create_dir_all(&policies_dir).unwrap();

        // Create a test policy file
        let policy_content = r#"# Test policy file
WHEN node.vendor == "cisco" THEN ASSERT node.version IS "15.1"
WHEN node.role == "router" THEN SET custom_data.managed TO true
"#;
        let policy_file = policies_dir.join("test.policy");
        fs::write(&policy_file, policy_content).unwrap();

        // Create policy loader
        let git_config = GitConfig {
            policies_repo: None,
            templates_repo: None,
            branch: "main".to_string(),
            sync_interval: 300,
        };

        let mut loader = PolicyLoader::new(git_config).with_local_dir(policies_dir);

        // Load policies
        let result = loader.load_policies().await.unwrap();

        assert_eq!(result.loaded.len(), 1);
        assert_eq!(result.errors.len(), 0);
        assert_eq!(result.total_files, 1);

        let policy_file = &result.loaded[0];
        assert_eq!(policy_file.rules.len(), 2);
    }

    #[tokio::test]
    async fn test_policy_file_validation() {
        let git_config = GitConfig {
            policies_repo: None,
            templates_repo: None,
            branch: "main".to_string(),
            sync_interval: 300,
        };

        let loader = PolicyLoader::new(git_config);

        let valid_content = r#"# Valid policy file
WHEN node.vendor == "cisco" THEN ASSERT node.version IS "15.1"
WHEN node.role == "router" THEN SET custom_data.managed TO true
"#;

        let result = loader.validate_policy_file(valid_content).unwrap();
        assert!(result.is_valid());
        assert_eq!(result.valid_rules, 2);
        assert_eq!(result.error_count(), 0);

        let invalid_content = r#"# Invalid policy file
WHEN node.vendor == "cisco" THEN ASSERT node.version IS "15.1"
INVALID SYNTAX HERE
WHEN node.role == "router" THEN SET custom_data.managed TO true
"#;

        let result = loader.validate_policy_file(invalid_content).unwrap();
        assert!(!result.is_valid());
        assert_eq!(result.valid_rules, 2);
        assert_eq!(result.error_count(), 1);
    }

    #[tokio::test]
    async fn test_policy_caching() {
        let temp_dir = TempDir::new().unwrap();
        let policy_file = temp_dir.path().join("test.policy");

        let policy_content = r#"WHEN node.vendor == "cisco" THEN ASSERT node.version IS "15.1""#;
        fs::write(&policy_file, policy_content).unwrap();

        let git_config = GitConfig {
            policies_repo: None,
            templates_repo: None,
            branch: "main".to_string(),
            sync_interval: 300,
        };

        let mut loader = PolicyLoader::new(git_config).with_cache_ttl(Duration::from_secs(60));

        // Load file first time
        let result1 = loader.load_policy_file(&policy_file).await.unwrap();
        assert_eq!(result1.rules.len(), 1);

        // Check cache stats
        let stats = loader.cache_stats();
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.valid_entries, 1);

        // Load file second time (should use cache)
        let result2 = loader.load_policy_file(&policy_file).await.unwrap();
        assert_eq!(result2.rules.len(), 1);

        // Modify file to invalidate cache
        std::thread::sleep(Duration::from_millis(10)); // Ensure different mtime
        fs::write(&policy_file, policy_content).unwrap();

        // Load file third time (should reload from disk)
        let result3 = loader.load_policy_file(&policy_file).await.unwrap();
        assert_eq!(result3.rules.len(), 1);
    }

    #[tokio::test]
    async fn test_policy_loader_with_real_policies_directory() {
        // Test with the actual policies directory
        let policies_path = std::path::PathBuf::from("/home/bc/unet/policies");

        if !policies_path.exists() {
            // Skip test if policies directory doesn't exist
            println!(
                "Skipping test: policies directory not found at {:?}",
                policies_path
            );
            return;
        }

        let git_config = GitConfig {
            policies_repo: None,
            templates_repo: None,
            branch: "main".to_string(),
            sync_interval: 300,
        };

        let mut loader = PolicyLoader::new(git_config).with_local_dir(&policies_path);

        // Load policies from real directory
        let result = loader.load_policies().await.unwrap();

        // Print results for debugging
        println!(
            "Loaded {} policies from {:?}",
            result.loaded.len(),
            policies_path
        );
        println!("Total files found: {}", result.total_files);
        println!("Errors: {}", result.errors.len());

        for policy_file in &result.loaded {
            println!(
                "  - {}: {} rules",
                policy_file.path.display(),
                policy_file.rules.len()
            );
        }

        for (path, error) in &result.errors {
            println!("  ERROR in {}: {}", path.display(), error);
        }

        // Should have at least one policy file (cisco-compliance.policy)
        if result.total_files == 0 {
            println!("No policy files found. Directory contents:");
            if let Ok(entries) = std::fs::read_dir(&policies_path) {
                for entry in entries.flatten() {
                    println!("  {:?}", entry.path());
                }
            }
            panic!("Expected at least 1 policy file");
        }

        if result.loaded.len() == 0 && result.errors.len() > 0 {
            // There were parsing errors, so let's see what they are
            for (path, error) in &result.errors {
                println!("Policy file {} failed to parse: {}", path.display(), error);
            }
            panic!("All policy files failed to parse");
        }

        assert!(
            result.loaded.len() >= 1,
            "Expected at least 1 loaded policy, found {}",
            result.loaded.len()
        );

        // Check that our cisco-compliance.policy was loaded
        let cisco_policy = result
            .loaded
            .iter()
            .find(|p| p.path.file_name().unwrap() == "cisco-compliance.policy");
        assert!(
            cisco_policy.is_some(),
            "Expected to find cisco-compliance.policy"
        );

        let cisco_policy = cisco_policy.unwrap();
        assert_eq!(
            cisco_policy.rules.len(),
            4,
            "Expected 4 rules in cisco-compliance.policy, found {}",
            cisco_policy.rules.len()
        );
    }
}
