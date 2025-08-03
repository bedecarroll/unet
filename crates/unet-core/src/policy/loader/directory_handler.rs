//! Directory handling and Git integration logic

use crate::config::GitConfig;
use crate::policy::{PolicyError, PolicyResult};
use std::path::PathBuf;

/// Directory handler for policy loading from Git or local sources
#[derive(Debug, Clone)]
pub struct DirectoryHandler {
    /// Git configuration
    git_config: GitConfig,
    /// Local policies directory
    local_dir: Option<PathBuf>,
}

impl DirectoryHandler {
    /// Create a new directory handler with Git configuration
    #[must_use]
    pub const fn new(git_config: GitConfig) -> Self {
        Self {
            git_config,
            local_dir: None,
        }
    }

    /// Set local policies directory (alternative to Git)
    #[must_use]
    pub fn with_local_dir<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.local_dir = Some(dir.into());
        self
    }

    /// Get the policies directory (from Git or local)
    pub fn get_policies_directory(&self) -> PolicyResult<PathBuf> {
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
}
