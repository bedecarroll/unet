//! Policy file loading with Git integration
//!
//! This module provides functionality to load policy files from local directories
//! or Git repositories, with validation, caching, and hot-reloading capabilities.

use crate::config::GitConfig;
use crate::policy::{PolicyError, PolicyResult, PolicyRule};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tracing::{debug, info, warn};
use walkdir::WalkDir;

// Re-export all public types
pub use self::cache::{CacheManager, CacheStats, CachedPolicy};
// Git integration is not yet implemented - exports removed to avoid dead code warnings
pub use self::validation::{PolicyValidator, ValidationError, ValidationResult};

mod cache;
mod git;
mod validation;

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
    #[must_use]
    pub fn new(git_config: GitConfig) -> Self {
        Self {
            git_config,
            policy_cache: HashMap::new(),
            local_dir: None,
            cache_ttl: Duration::from_secs(300), // 5 minutes default
        }
    }

    /// Set local policies directory (alternative to Git)
    #[must_use]
    pub fn with_local_dir<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.local_dir = Some(dir.into());
        self
    }

    /// Set cache TTL duration
    #[must_use]
    pub const fn with_cache_ttl(mut self, ttl: Duration) -> Self {
        self.cache_ttl = ttl;
        self
    }

    /// Load all policy files from configured source
    ///
    /// # Errors
    ///
    /// Returns `PolicyError` if:
    /// - The policies directory cannot be accessed
    /// - Policy files cannot be read or parsed
    /// - Invalid policy syntax is encountered
    pub fn load_policies(&mut self) -> PolicyResult<LoadResult> {
        info!("Loading policy files");

        let policies_dir = self.get_policies_directory()?;
        self.load_policies_from_directory(&policies_dir)
    }

    /// Load policies from a specific directory
    ///
    /// # Errors
    ///
    /// Returns `PolicyError` if:
    /// - The directory does not exist
    /// - Directory cannot be accessed
    /// - Policy files cannot be read or parsed
    pub fn load_policies_from_directory(&mut self, dir: &Path) -> PolicyResult<LoadResult> {
        Self::validate_directory(dir)?;
        debug!("Loading policies from directory: {}", dir.display());

        let policy_files = Self::collect_policy_files(dir);
        let load_result = self.process_policy_files(policy_files);

        info!(
            "Policy loading complete: {} loaded, {} errors, {} total files",
            load_result.loaded.len(),
            load_result.errors.len(),
            load_result.total_files
        );

        Ok(load_result)
    }

    /// Validate that the policies directory exists
    fn validate_directory(dir: &Path) -> PolicyResult<()> {
        if !dir.exists() {
            return Err(PolicyError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Policies directory not found: {}", dir.display()),
            )));
        }
        Ok(())
    }

    /// Collect all .policy files from the directory tree
    fn collect_policy_files(dir: &Path) -> Vec<PathBuf> {
        WalkDir::new(dir)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "policy"))
            .map(|e| e.path().to_path_buf())
            .collect()
    }

    /// Process all collected policy files
    fn process_policy_files(&mut self, policy_files: Vec<PathBuf>) -> LoadResult {
        let mut loaded = Vec::new();
        let mut errors = Vec::new();
        let total_files = policy_files.len();

        for file_path in policy_files {
            match self.load_policy_file(&file_path) {
                Ok(policy_file) => {
                    debug!("Loaded policy file: {}", file_path.display());
                    loaded.push(policy_file);
                }
                Err(e) => {
                    warn!("Failed to load policy file {}: {}", file_path.display(), e);
                    errors.push((file_path, e));
                }
            }
        }

        LoadResult {
            loaded,
            errors,
            total_files,
        }
    }

    /// Load a single policy file with caching
    ///
    /// # Errors
    ///
    /// Returns `PolicyError` if:
    /// - The file cannot be read
    /// - The file contains invalid policy syntax
    /// - File metadata cannot be accessed
    pub fn load_policy_file(&mut self, path: &Path) -> PolicyResult<PolicyFile> {
        let metadata = std::fs::metadata(path).map_err(PolicyError::Io)?;
        let mtime = metadata.modified().map_err(PolicyError::Io)?;

        // Check cache first
        if let Some(cached) = self.policy_cache.get(path) {
            if cached.is_valid(self.cache_ttl, mtime) {
                debug!("Using cached policy: {}", path.display());
                return Ok(PolicyFile {
                    path: path.to_path_buf(),
                    rules: cached.rules.clone(),
                    modified: mtime,
                    size: metadata.len(),
                });
            }
        }

        // Cache miss or expired - load from file
        debug!("Loading policy file from disk: {}", path.display());
        let content = std::fs::read_to_string(path).map_err(PolicyError::Io)?;

        // Validate and parse policy content
        let rules = PolicyValidator::validate_and_parse(&content)?;

        // Cache the parsed policy
        let cached_policy = CachedPolicy::new(rules.clone(), mtime);
        self.policy_cache.insert(path.to_path_buf(), cached_policy);

        Ok(PolicyFile {
            path: path.to_path_buf(),
            rules,
            modified: mtime,
            size: metadata.len(),
        })
    }

    /// Get the policies directory (from Git or local)
    fn get_policies_directory(&self) -> PolicyResult<PathBuf> {
        if let Some(ref local_dir) = self.local_dir {
            return Ok(local_dir.clone());
        }

        if let Some(ref repo_url) = self.git_config.policies_repo {
            // TODO: Implement Git repository cloning/syncing
            // For now, return an error to indicate Git integration is needed
            return Err(PolicyError::Evaluation {
                message: format!("Git repository integration not yet implemented for: {repo_url}"),
            });
        }

        Err(PolicyError::Evaluation {
            message: "No policies source configured (neither local directory nor Git repository)"
                .to_string(),
        })
    }

    /// Clear the policy cache
    pub fn clear_cache(&mut self) {
        self.policy_cache.clear_cache();
        debug!("Policy cache cleared");
    }

    /// Get cache statistics
    #[must_use]
    pub fn cache_stats(&self) -> CacheStats {
        self.policy_cache.get_cache_stats(self.cache_ttl)
    }

    /// Validate a policy file format without parsing all rules
    #[must_use]
    pub fn validate_policy_file(&self, content: &str) -> ValidationResult {
        PolicyValidator::validate_policy_file(content)
    }

    /// Clear expired cache entries
    pub fn clear_expired_cache(&mut self) -> usize {
        self.policy_cache.clear_expired_cache(self.cache_ttl)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GitConfig;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_git_config() -> GitConfig {
        GitConfig {
            repository_url: None,
            local_directory: None,
            branch: "main".to_string(),
            auth_token: None,
            sync_interval: 300,
            policies_repo: None,
            templates_repo: None,
        }
    }

    #[test]
    fn test_policy_loader_local_directory() {
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
        let git_config = create_test_git_config();
        let mut loader = PolicyLoader::new(git_config).with_local_dir(policies_dir);

        // Load policies
        let result = loader.load_policies().unwrap();

        assert_eq!(result.loaded.len(), 1);
        assert_eq!(result.errors.len(), 0);
        assert_eq!(result.total_files, 1);

        let policy_file = &result.loaded[0];
        assert_eq!(policy_file.rules.len(), 2);
    }

    #[test]
    fn test_policy_file_validation() {
        let git_config = create_test_git_config();
        let loader = PolicyLoader::new(git_config);

        let valid_content = r#"# Valid policy file
WHEN node.vendor == "cisco" THEN ASSERT node.version IS "15.1"
WHEN node.role == "router" THEN SET custom_data.managed TO true
"#;

        let result = loader.validate_policy_file(valid_content);
        assert!(result.is_valid());
        assert_eq!(result.valid_rules, 2);
        assert_eq!(result.error_count(), 0);

        let invalid_content = r#"# Invalid policy file
WHEN node.vendor == "cisco" THEN ASSERT node.version IS "15.1"
INVALID SYNTAX HERE
WHEN node.role == "router" THEN SET custom_data.managed TO true
"#;

        let result = loader.validate_policy_file(invalid_content);
        assert!(!result.is_valid());
        assert_eq!(result.valid_rules, 2);
        assert_eq!(result.error_count(), 1);
    }

    #[test]
    fn test_policy_caching() {
        let temp_dir = TempDir::new().unwrap();
        let policy_file = temp_dir.path().join("test.policy");

        let policy_content = r#"WHEN node.vendor == "cisco" THEN ASSERT node.version IS "15.1""#;
        fs::write(&policy_file, policy_content).unwrap();

        let git_config = create_test_git_config();
        let mut loader = PolicyLoader::new(git_config).with_cache_ttl(Duration::from_secs(60));

        // Load file first time
        let result1 = loader.load_policy_file(&policy_file).unwrap();
        assert_eq!(result1.rules.len(), 1);

        // Check cache stats
        let stats = loader.cache_stats();
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.valid_entries, 1);

        // Load file second time (should use cache)
        let result2 = loader.load_policy_file(&policy_file).unwrap();
        assert_eq!(result2.rules.len(), 1);

        // Modify file to invalidate cache
        std::thread::sleep(Duration::from_millis(10)); // Ensure different mtime
        fs::write(&policy_file, policy_content).unwrap();

        // Load file third time (should reload from disk)
        let result3 = loader.load_policy_file(&policy_file).unwrap();
        assert_eq!(result3.rules.len(), 1);
    }

    #[test]
    fn test_policy_loader_directory_not_found() {
        let git_config = create_test_git_config();
        let non_existent_dir = Path::new("/non/existent/directory");
        let mut loader = PolicyLoader::new(git_config).with_local_dir(non_existent_dir);

        let result = loader.load_policies();
        assert!(result.is_err());

        if let Err(PolicyError::Io(io_error)) = result {
            assert_eq!(io_error.kind(), std::io::ErrorKind::NotFound);
        } else {
            panic!("Expected IO error for non-existent directory");
        }
    }

    #[test]
    fn test_policy_loader_clear_expired_cache() {
        let temp_dir = TempDir::new().unwrap();
        let policy_file = temp_dir.path().join("test.policy");

        let policy_content = r#"WHEN node.vendor == "cisco" THEN ASSERT node.version IS "15.1""#;
        fs::write(&policy_file, policy_content).unwrap();

        let git_config = create_test_git_config();
        let mut loader = PolicyLoader::new(git_config).with_cache_ttl(Duration::from_millis(1));

        // Load file to add to cache
        let _result = loader.load_policy_file(&policy_file).unwrap();

        // Wait for cache to expire
        std::thread::sleep(Duration::from_millis(10));

        // Clear expired cache entries
        let cleared_count = loader.clear_expired_cache();
        assert_eq!(cleared_count, 1);

        // Check cache is now empty
        let stats = loader.cache_stats();
        assert_eq!(stats.total_entries, 0);
    }

    #[test]
    fn test_policy_loader_invalid_policy_file() {
        let temp_dir = TempDir::new().unwrap();
        let policies_dir = temp_dir.path().join("policies");
        fs::create_dir_all(&policies_dir).unwrap();

        // Create an invalid policy file
        let invalid_policy = policies_dir.join("invalid.policy");
        fs::write(&invalid_policy, "COMPLETELY INVALID SYNTAX").unwrap();

        let git_config = create_test_git_config();
        let mut loader = PolicyLoader::new(git_config).with_local_dir(policies_dir);

        let result = loader.load_policies().unwrap();
        assert_eq!(result.loaded.len(), 0);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.total_files, 1);
    }

    #[test]
    fn test_policy_loader_mixed_valid_invalid_files() {
        let temp_dir = TempDir::new().unwrap();
        let policies_dir = temp_dir.path().join("policies");
        fs::create_dir_all(&policies_dir).unwrap();

        // Create valid policy file
        let valid_policy = r#"WHEN node.vendor == "cisco" THEN ASSERT node.version IS "15.1""#;
        fs::write(policies_dir.join("valid.policy"), valid_policy).unwrap();

        // Create invalid policy file
        fs::write(policies_dir.join("invalid.policy"), "INVALID SYNTAX").unwrap();

        let git_config = create_test_git_config();
        let mut loader = PolicyLoader::new(git_config).with_local_dir(policies_dir);

        let result = loader.load_policies().unwrap();
        assert_eq!(result.loaded.len(), 1);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.total_files, 2);
    }
}
