//! Policy file processing and loading logic

use crate::policy::{PolicyError, PolicyResult, PolicyRule};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tracing::{debug, info, warn};
use walkdir::WalkDir;

use super::cache::CachedPolicy;
use super::validation::PolicyValidator;

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

/// File processing functionality for policy loading
#[derive(Debug, Clone)]
pub struct FileProcessor {
    /// Cache TTL duration
    cache_ttl: Duration,
}

impl FileProcessor {
    /// Create a new file processor with cache TTL
    #[must_use]
    pub const fn new(cache_ttl: Duration) -> Self {
        Self { cache_ttl }
    }

    /// Load policies from a specific directory
    ///
    /// # Errors
    ///
    /// Returns `PolicyError` if:
    /// - The directory does not exist
    /// - Directory cannot be accessed
    /// - Policy files cannot be read or parsed
    pub fn load_policies_from_directory(
        &self,
        dir: &Path,
        policy_cache: &mut HashMap<PathBuf, CachedPolicy>,
    ) -> PolicyResult<LoadResult> {
        Self::validate_directory(dir)?;
        debug!("Loading policies from directory: {}", dir.display());

        let policy_files = Self::collect_policy_files(dir);
        let load_result = self.process_policy_files(policy_files, policy_cache);

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
    fn process_policy_files(
        &self,
        policy_files: Vec<PathBuf>,
        policy_cache: &mut HashMap<PathBuf, CachedPolicy>,
    ) -> LoadResult {
        let mut loaded = Vec::new();
        let mut errors = Vec::new();
        let total_files = policy_files.len();

        for file_path in policy_files {
            match self.load_policy_file(&file_path, policy_cache) {
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
    pub fn load_policy_file(
        &self,
        path: &Path,
        policy_cache: &mut HashMap<PathBuf, CachedPolicy>,
    ) -> PolicyResult<PolicyFile> {
        let metadata = std::fs::metadata(path).map_err(PolicyError::Io)?;
        let mtime = metadata.modified().map_err(PolicyError::Io)?;

        // Check cache first
        if let Some(cached) = policy_cache.get(path) {
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
        policy_cache.insert(path.to_path_buf(), cached_policy);

        Ok(PolicyFile {
            path: path.to_path_buf(),
            rules,
            modified: mtime,
            size: metadata.len(),
        })
    }
}
