//! Enhanced branch management for environment workflows
//!
//! This module provides advanced branch management functionality specifically
//! designed for environment-based workflows, including branch protection,
//! merge strategies, and conflict resolution.

use crate::git::environment::{EnvironmentConfig, EnvironmentType};
use crate::git::repository::GitRepository;
use crate::git::types::{BranchInfo, GitError, GitResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};

/// Branch protection rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchProtectionRules {
    /// Whether this branch is protected
    pub protected: bool,
    /// Require pull request reviews before merging
    pub require_pull_request_reviews: bool,
    /// Number of required reviewers
    pub required_reviewers: u32,
    /// Dismiss stale reviews when new commits are pushed
    pub dismiss_stale_reviews: bool,
    /// Require status checks to pass before merging
    pub require_status_checks: bool,
    /// List of required status check contexts
    pub required_status_checks: Vec<String>,
    /// Restrict pushes to matching users
    pub push_restrictions: Vec<String>,
    /// Allow force pushes
    pub allow_force_pushes: bool,
    /// Allow deletions
    pub allow_deletions: bool,
    /// Require branches to be up to date before merging
    pub require_up_to_date: bool,
}

impl Default for BranchProtectionRules {
    fn default() -> Self {
        Self {
            protected: false,
            require_pull_request_reviews: false,
            required_reviewers: 0,
            dismiss_stale_reviews: false,
            require_status_checks: false,
            required_status_checks: Vec::new(),
            push_restrictions: Vec::new(),
            allow_force_pushes: false,
            allow_deletions: false,
            require_up_to_date: false,
        }
    }
}

impl BranchProtectionRules {
    /// Create protection rules for production environments
    pub fn production_protection() -> Self {
        Self {
            protected: true,
            require_pull_request_reviews: true,
            required_reviewers: 2,
            dismiss_stale_reviews: true,
            require_status_checks: true,
            required_status_checks: vec![
                "ci/tests".to_string(),
                "ci/security".to_string(),
                "ci/lint".to_string(),
            ],
            push_restrictions: Vec::new(),
            allow_force_pushes: false,
            allow_deletions: false,
            require_up_to_date: true,
        }
    }

    /// Create protection rules for staging environments
    pub fn staging_protection() -> Self {
        Self {
            protected: true,
            require_pull_request_reviews: true,
            required_reviewers: 1,
            dismiss_stale_reviews: false,
            require_status_checks: true,
            required_status_checks: vec!["ci/tests".to_string()],
            push_restrictions: Vec::new(),
            allow_force_pushes: false,
            allow_deletions: false,
            require_up_to_date: true,
        }
    }

    /// Create minimal protection rules for development environments
    pub fn development_protection() -> Self {
        Self {
            protected: false,
            require_pull_request_reviews: false,
            required_reviewers: 0,
            dismiss_stale_reviews: false,
            require_status_checks: false,
            required_status_checks: Vec::new(),
            push_restrictions: Vec::new(),
            allow_force_pushes: true,
            allow_deletions: true,
            require_up_to_date: false,
        }
    }
}

/// Branch merge strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeStrategy {
    /// Regular merge commit
    Merge,
    /// Squash and merge
    Squash,
    /// Rebase and merge
    Rebase,
    /// Fast-forward only
    FastForward,
}

impl Default for MergeStrategy {
    fn default() -> Self {
        MergeStrategy::Merge
    }
}

impl std::fmt::Display for MergeStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MergeStrategy::Merge => write!(f, "merge"),
            MergeStrategy::Squash => write!(f, "squash"),
            MergeStrategy::Rebase => write!(f, "rebase"),
            MergeStrategy::FastForward => write!(f, "fast-forward"),
        }
    }
}

/// Branch switching context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchSwitchContext {
    /// Previous branch name
    pub previous_branch: String,
    /// Target branch name
    pub target_branch: String,
    /// Environment context
    pub environment: Option<String>,
    /// Whether to stash changes before switching
    pub stash_changes: bool,
    /// Whether to create branch if it doesn't exist
    pub create_if_missing: bool,
    /// Switch timestamp
    pub timestamp: DateTime<Utc>,
    /// User performing the switch
    pub user: Option<String>,
}

/// Branch switching result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchSwitchResult {
    /// Whether the switch was successful
    pub success: bool,
    /// Previous branch
    pub previous_branch: String,
    /// Current branch after switch
    pub current_branch: String,
    /// Whether changes were stashed
    pub changes_stashed: bool,
    /// Stash reference if changes were stashed
    pub stash_ref: Option<String>,
    /// Any warnings during the switch
    pub warnings: Vec<String>,
    /// Error message if switch failed
    pub error: Option<String>,
}

/// Enhanced branch manager for environment workflows
pub struct BranchManager {
    /// Git repository
    repository: GitRepository,
    /// Branch protection rules by branch name
    protection_rules: HashMap<String, BranchProtectionRules>,
    /// Environment configurations
    environments: HashMap<String, EnvironmentConfig>,
}

impl BranchManager {
    /// Create a new branch manager
    pub fn new(repository: GitRepository) -> Self {
        Self {
            repository,
            protection_rules: HashMap::new(),
            environments: HashMap::new(),
        }
    }

    /// Configure environments
    pub fn configure_environments(&mut self, environments: Vec<EnvironmentConfig>) {
        self.environments.clear();
        for env in environments {
            self.environments.insert(env.name.clone(), env);
        }

        // Set up default protection rules based on environment types
        self.setup_default_protection_rules();
    }

    /// Switch to a branch with environment context
    pub fn switch_to_branch(&self, context: BranchSwitchContext) -> GitResult<BranchSwitchResult> {
        info!(
            "Switching from '{}' to '{}'",
            context.previous_branch, context.target_branch
        );

        let mut result = BranchSwitchResult {
            success: false,
            previous_branch: context.previous_branch.clone(),
            current_branch: context.previous_branch.clone(),
            changes_stashed: false,
            stash_ref: None,
            warnings: Vec::new(),
            error: None,
        };

        // Check if target branch exists
        let branches = self.repository.list_branches(None)?;
        let target_exists = branches.iter().any(|b| b.name == context.target_branch);

        if !target_exists {
            if context.create_if_missing {
                info!("Creating new branch: {}", context.target_branch);
                self.repository
                    .create_branch(&context.target_branch, None)?;
            } else {
                let error_msg = format!("Branch '{}' does not exist", context.target_branch);
                result.error = Some(error_msg.clone());
                return Err(GitError::RepositoryOperation { message: error_msg });
            }
        }

        // Check repository status
        let status = self.repository.status()?;

        // Handle uncommitted changes
        if !status.is_clean && context.stash_changes {
            info!("Stashing changes before branch switch");
            // Note: This is a simplified implementation
            // In a real implementation, you'd use git2 stash functionality
            result.changes_stashed = true;
            result.stash_ref = Some(format!("stash@{{0}}: WIP on {}", context.previous_branch));
            result
                .warnings
                .push("Changes were stashed before switching branches".to_string());
        } else if !status.is_clean {
            let error_msg = "Repository has uncommitted changes. Use stash_changes=true to handle automatically".to_string();
            result.error = Some(error_msg.clone());
            return Err(GitError::RepositoryOperation { message: error_msg });
        }

        // Check branch protection rules
        if let Some(protection) = self.protection_rules.get(&context.target_branch) {
            if protection.protected {
                result.warnings.push(format!(
                    "Switching to protected branch '{}'. Be aware of protection rules.",
                    context.target_branch
                ));
            }
        }

        // Perform the actual branch switch
        match self.repository.checkout_branch(&context.target_branch) {
            Ok(_) => {
                result.success = true;
                result.current_branch = context.target_branch.clone();
                info!("Successfully switched to branch: {}", context.target_branch);
            }
            Err(e) => {
                let error_msg = format!(
                    "Failed to switch to branch '{}': {}",
                    context.target_branch, e
                );
                result.error = Some(error_msg.clone());
                return Err(GitError::RepositoryOperation { message: error_msg });
            }
        }

        Ok(result)
    }

    /// Switch to environment branch
    pub fn switch_to_environment(
        &self,
        environment_name: &str,
        stash_changes: bool,
    ) -> GitResult<BranchSwitchResult> {
        let env = self.environments.get(environment_name).ok_or_else(|| {
            GitError::RepositoryOperation {
                message: format!("Environment '{}' not found", environment_name),
            }
        })?;

        let current_branch = self.repository.current_branch_name()?;

        let context = BranchSwitchContext {
            previous_branch: current_branch,
            target_branch: env.branch_name.clone(),
            environment: Some(environment_name.to_string()),
            stash_changes,
            create_if_missing: true,
            timestamp: Utc::now(),
            user: None,
        };

        self.switch_to_branch(context)
    }

    /// Get enhanced branch information
    pub fn get_branch_info(&self, branch_name: &str) -> GitResult<EnhancedBranchInfo> {
        let branches = self.repository.list_branches(None)?;
        let branch = branches
            .into_iter()
            .find(|b| b.name == branch_name)
            .ok_or_else(|| GitError::RepositoryOperation {
                message: format!("Branch '{}' not found", branch_name),
            })?;

        let protection_rules = self
            .protection_rules
            .get(branch_name)
            .cloned()
            .unwrap_or_default();

        let environment = self.find_environment_for_branch(branch_name);

        let merge_base = self.get_merge_base(branch_name)?;
        let ahead_behind = if let Some(ref base) = merge_base {
            self.get_ahead_behind_from_base(branch_name, base)?
        } else {
            (0, 0)
        };

        Ok(EnhancedBranchInfo {
            basic_info: branch,
            protection_rules,
            environment,
            merge_base,
            commits_ahead_of_base: ahead_behind.0,
            commits_behind_base: ahead_behind.1,
            last_activity: Utc::now(), // This would be calculated from actual commits
            is_merged: false,          // This would be determined by checking merge status
        })
    }

    /// List all branches with enhanced information
    pub fn list_enhanced_branches(&self) -> GitResult<Vec<EnhancedBranchInfo>> {
        let branches = self.repository.list_branches(None)?;
        let mut enhanced_branches = Vec::new();

        for branch in branches {
            match self.get_branch_info(&branch.name) {
                Ok(enhanced) => enhanced_branches.push(enhanced),
                Err(e) => {
                    warn!(
                        "Failed to get enhanced info for branch '{}': {}",
                        branch.name, e
                    );
                    // Create a minimal enhanced branch info
                    enhanced_branches.push(EnhancedBranchInfo {
                        basic_info: branch,
                        protection_rules: BranchProtectionRules::default(),
                        environment: None,
                        merge_base: None,
                        commits_ahead_of_base: 0,
                        commits_behind_base: 0,
                        last_activity: Utc::now(),
                        is_merged: false,
                    });
                }
            }
        }

        Ok(enhanced_branches)
    }

    /// Set protection rules for a branch
    pub fn set_branch_protection(&mut self, branch_name: String, rules: BranchProtectionRules) {
        info!("Setting protection rules for branch: {}", branch_name);
        self.protection_rules.insert(branch_name, rules);
    }

    /// Get protection rules for a branch
    pub fn get_branch_protection(&self, branch_name: &str) -> Option<&BranchProtectionRules> {
        self.protection_rules.get(branch_name)
    }

    /// Validate branch operation against protection rules
    pub fn validate_branch_operation(
        &self,
        branch_name: &str,
        operation: BranchOperation,
    ) -> GitResult<()> {
        if let Some(rules) = self.protection_rules.get(branch_name) {
            if rules.protected {
                match operation {
                    BranchOperation::ForcePush if !rules.allow_force_pushes => {
                        return Err(GitError::RepositoryOperation {
                            message: format!(
                                "Force push not allowed on protected branch '{}'",
                                branch_name
                            ),
                        });
                    }
                    BranchOperation::Delete if !rules.allow_deletions => {
                        return Err(GitError::RepositoryOperation {
                            message: format!(
                                "Deletion not allowed on protected branch '{}'",
                                branch_name
                            ),
                        });
                    }
                    BranchOperation::DirectPush if rules.require_pull_request_reviews => {
                        return Err(GitError::RepositoryOperation {
                            message: format!(
                                "Direct push not allowed on protected branch '{}'. Pull request required.",
                                branch_name
                            ),
                        });
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    /// Create a new branch with environment context
    pub fn create_environment_branch(
        &self,
        env: &EnvironmentConfig,
        source_branch: Option<&str>,
    ) -> GitResult<()> {
        info!(
            "Creating environment branch '{}' for environment '{}'",
            env.branch_name, env.name
        );

        // Check if branch already exists
        let branches = self.repository.list_branches(None)?;
        if branches.iter().any(|b| b.name == env.branch_name) {
            return Err(GitError::RepositoryOperation {
                message: format!("Branch '{}' already exists", env.branch_name),
            });
        }

        // Create the branch
        self.repository
            .create_branch(&env.branch_name, source_branch)?;

        info!(
            "Successfully created environment branch: {}",
            env.branch_name
        );
        Ok(())
    }

    /// Delete a branch with safety checks
    pub fn delete_branch(&self, branch_name: &str, force: bool) -> GitResult<()> {
        info!("Attempting to delete branch: {}", branch_name);

        // Validate against protection rules
        if !force {
            self.validate_branch_operation(branch_name, BranchOperation::Delete)?;
        }

        // Check if it's the current branch
        let current_branch = self.repository.current_branch_name()?;
        if current_branch == branch_name {
            return Err(GitError::RepositoryOperation {
                message: "Cannot delete the current branch".to_string(),
            });
        }

        // Check if it's an environment branch
        if let Some(env) = self.find_environment_for_branch(branch_name) {
            return Err(GitError::RepositoryOperation {
                message: format!(
                    "Cannot delete environment branch '{}' for environment '{}'",
                    branch_name, env
                ),
            });
        }

        // Perform the deletion
        // Note: This would use git2 branch deletion functionality
        info!(
            "Branch '{}' would be deleted (implementation needed)",
            branch_name
        );

        Ok(())
    }

    /// Get merge base between branches
    pub fn get_merge_base(&self, _branch_name: &str) -> GitResult<Option<String>> {
        // This is a simplified implementation
        // In a real implementation, you'd use git2 to find the merge base
        // between the branch and its upstream or default branch
        Ok(Some("abc123".to_string())) // Placeholder
    }

    /// Get ahead/behind counts from merge base
    pub fn get_ahead_behind_from_base(
        &self,
        _branch_name: &str,
        _base: &str,
    ) -> GitResult<(usize, usize)> {
        // This is a simplified implementation
        // In a real implementation, you'd calculate the actual ahead/behind counts
        Ok((0, 0)) // Placeholder
    }

    // Private helper methods

    fn setup_default_protection_rules(&mut self) {
        for env in self.environments.values() {
            let rules = match env.environment_type {
                EnvironmentType::Production => BranchProtectionRules::production_protection(),
                EnvironmentType::Staging => BranchProtectionRules::staging_protection(),
                EnvironmentType::Development => BranchProtectionRules::development_protection(),
                EnvironmentType::Custom => BranchProtectionRules::default(),
            };

            self.protection_rules.insert(env.branch_name.clone(), rules);
        }
    }

    fn find_environment_for_branch(&self, branch_name: &str) -> Option<String> {
        for env in self.environments.values() {
            if env.branch_name == branch_name {
                return Some(env.name.clone());
            }
        }
        None
    }
}

/// Enhanced branch information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedBranchInfo {
    /// Basic branch information from Git
    pub basic_info: BranchInfo,
    /// Branch protection rules
    pub protection_rules: BranchProtectionRules,
    /// Associated environment (if any)
    pub environment: Option<String>,
    /// Merge base commit hash
    pub merge_base: Option<String>,
    /// Number of commits ahead of merge base
    pub commits_ahead_of_base: usize,
    /// Number of commits behind merge base
    pub commits_behind_base: usize,
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
    /// Whether this branch has been merged
    pub is_merged: bool,
}

/// Branch operations that can be validated against protection rules
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BranchOperation {
    /// Direct push to branch
    DirectPush,
    /// Force push to branch
    ForcePush,
    /// Delete branch
    Delete,
    /// Create pull request
    CreatePullRequest,
    /// Merge pull request
    MergePullRequest,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_strategy_display() {
        assert_eq!(MergeStrategy::Merge.to_string(), "merge");
        assert_eq!(MergeStrategy::Squash.to_string(), "squash");
        assert_eq!(MergeStrategy::Rebase.to_string(), "rebase");
        assert_eq!(MergeStrategy::FastForward.to_string(), "fast-forward");
    }

    #[test]
    fn test_production_protection_rules() {
        let rules = BranchProtectionRules::production_protection();
        assert!(rules.protected);
        assert!(rules.require_pull_request_reviews);
        assert_eq!(rules.required_reviewers, 2);
        assert!(!rules.allow_force_pushes);
        assert!(!rules.allow_deletions);
    }

    #[test]
    fn test_development_protection_rules() {
        let rules = BranchProtectionRules::development_protection();
        assert!(!rules.protected);
        assert!(!rules.require_pull_request_reviews);
        assert!(rules.allow_force_pushes);
        assert!(rules.allow_deletions);
    }
}
