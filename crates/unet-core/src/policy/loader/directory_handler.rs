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

        if let Some(ref local_dir) = self.git_config.local_directory {
            return Ok(PathBuf::from(local_dir));
        }

        if let Some(ref repo_url) = self.git_config.repository_url {
            return Err(PolicyError::NotImplemented {
                feature: format!("Git policy repository sync is not implemented for {repo_url}"),
            });
        }

        if let Some(ref repo_url) = self.git_config.policies_repo {
            return Err(PolicyError::NotImplemented {
                feature: format!("Git policy repository sync is not implemented for {repo_url}"),
            });
        }

        Err(PolicyError::Evaluation {
            message: "No policies source configured (neither local directory nor Git repository)"
                .to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::DirectoryHandler;
    use crate::config::GitConfig;
    use crate::policy::PolicyError;
    use std::path::PathBuf;

    fn create_git_config() -> GitConfig {
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
    fn test_get_policies_directory_uses_configured_local_directory() {
        let mut git_config = create_git_config();
        git_config.local_directory = Some("./policies".to_string());

        let handler = DirectoryHandler::new(git_config);
        let result = handler.get_policies_directory().unwrap();

        assert_eq!(result, PathBuf::from("./policies"));
    }

    #[test]
    fn test_get_policies_directory_rejects_repository_url_until_git_sync_exists() {
        let mut git_config = create_git_config();
        git_config.repository_url = Some("https://github.com/example/policies.git".to_string());

        let handler = DirectoryHandler::new(git_config);
        let result = handler.get_policies_directory();

        assert!(result.is_err());
        match result.unwrap_err() {
            PolicyError::NotImplemented { feature } => {
                assert!(feature.contains("Git policy repository sync"));
            }
            other => panic!("Expected NotImplemented error, got {other:?}"),
        }
    }

    #[test]
    fn test_get_policies_directory_rejects_backward_compatible_git_repo_until_supported() {
        let mut git_config = create_git_config();
        git_config.policies_repo = Some("https://github.com/example/policies.git".to_string());

        let handler = DirectoryHandler::new(git_config);
        let result = handler.get_policies_directory();

        assert!(result.is_err());
        match result.unwrap_err() {
            PolicyError::NotImplemented { feature } => {
                assert!(feature.contains("Git policy repository sync"));
            }
            other => panic!("Expected NotImplemented error, got {other:?}"),
        }
    }
}
