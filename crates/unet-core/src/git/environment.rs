//! Git-based environment and branch management for μNet
//!
//! This module provides functionality for managing multiple environments
//! (dev, staging, production) using Git branches, including environment-specific
//! configurations and promotion workflows.

use crate::git::branch_management::{BranchManager, BranchOperation};
use crate::git::repository::GitRepository;
use crate::git::types::{BranchInfo, GitError, GitResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{info, warn};

/// Environment types supported by μNet
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnvironmentType {
    /// Development environment
    Development,
    /// Staging/testing environment  
    Staging,
    /// Production environment
    Production,
    /// Custom environment
    Custom,
}

impl EnvironmentType {
    /// Get the default branch name for this environment type
    pub fn default_branch_name(&self) -> &'static str {
        match self {
            EnvironmentType::Development => "dev",
            EnvironmentType::Staging => "staging",
            EnvironmentType::Production => "main",
            EnvironmentType::Custom => "custom",
        }
    }

    /// Parse environment type from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "dev" | "development" => Some(EnvironmentType::Development),
            "staging" | "stage" => Some(EnvironmentType::Staging),
            "prod" | "production" | "main" => Some(EnvironmentType::Production),
            "custom" => Some(EnvironmentType::Custom),
            _ => None,
        }
    }
}

impl std::fmt::Display for EnvironmentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnvironmentType::Development => write!(f, "development"),
            EnvironmentType::Staging => write!(f, "staging"),
            EnvironmentType::Production => write!(f, "production"),
            EnvironmentType::Custom => write!(f, "custom"),
        }
    }
}

/// Environment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Environment name
    pub name: String,
    /// Environment type
    pub environment_type: EnvironmentType,
    /// Associated Git branch
    pub branch_name: String,
    /// Environment-specific configuration overrides
    pub config_overrides: HashMap<String, serde_json::Value>,
    /// Environment description
    pub description: Option<String>,
    /// Whether this environment requires approval for changes
    pub requires_approval: bool,
    /// Allowed source environments for promotion
    pub allowed_sources: Vec<String>,
    /// Environment protection rules
    pub protection_rules: EnvironmentProtection,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Environment protection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentProtection {
    /// Require pull request reviews
    pub require_pull_request: bool,
    /// Number of required reviewers
    pub required_reviewers: u32,
    /// Dismiss stale reviews when new commits are pushed
    pub dismiss_stale_reviews: bool,
    /// Require status checks to pass
    pub require_status_checks: bool,
    /// Required status check contexts
    pub required_status_checks: Vec<String>,
    /// Restrict pushes to this environment
    pub restrict_pushes: bool,
    /// Allow force pushes
    pub allow_force_pushes: bool,
    /// Allow deletions
    pub allow_deletions: bool,
}

impl Default for EnvironmentProtection {
    fn default() -> Self {
        Self {
            require_pull_request: false,
            required_reviewers: 0,
            dismiss_stale_reviews: false,
            require_status_checks: false,
            required_status_checks: Vec::new(),
            restrict_pushes: false,
            allow_force_pushes: false,
            allow_deletions: false,
        }
    }
}

/// Environment promotion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionRequest {
    /// Unique promotion ID
    pub id: String,
    /// Source environment
    pub source_environment: String,
    /// Target environment
    pub target_environment: String,
    /// Source commit hash
    pub source_commit: String,
    /// Target commit hash (after promotion)
    pub target_commit: Option<String>,
    /// Promotion status
    pub status: PromotionStatus,
    /// Promotion message/description
    pub message: String,
    /// Requestor information
    pub requested_by: String,
    /// Approved by (if applicable)
    pub approved_by: Option<String>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Completed timestamp
    pub completed_at: Option<DateTime<Utc>>,
    /// Promotion metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Promotion status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromotionStatus {
    /// Promotion is pending
    Pending,
    /// Promotion is approved and ready
    Approved,
    /// Promotion is in progress
    InProgress,
    /// Promotion completed successfully
    Completed,
    /// Promotion failed
    Failed,
    /// Promotion was cancelled
    Cancelled,
    /// Promotion was rejected
    Rejected,
}

impl std::fmt::Display for PromotionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PromotionStatus::Pending => write!(f, "pending"),
            PromotionStatus::Approved => write!(f, "approved"),
            PromotionStatus::InProgress => write!(f, "in_progress"),
            PromotionStatus::Completed => write!(f, "completed"),
            PromotionStatus::Failed => write!(f, "failed"),
            PromotionStatus::Cancelled => write!(f, "cancelled"),
            PromotionStatus::Rejected => write!(f, "rejected"),
        }
    }
}

/// Environment promotion result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionResult {
    /// Promotion request
    pub request: PromotionRequest,
    /// Whether the promotion was successful
    pub success: bool,
    /// Error message if promotion failed
    pub error: Option<String>,
    /// Files that were changed during promotion
    pub changed_files: Vec<PathBuf>,
    /// Conflicts that occurred during promotion
    pub conflicts: Vec<String>,
}

/// Promotion workflow status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionWorkflowStatus {
    /// Promotion request ID
    pub promotion_id: String,
    /// Current status
    pub status: PromotionStatus,
    /// Source environment
    pub source_environment: String,
    /// Target environment
    pub target_environment: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Completion timestamp
    pub completed_at: Option<DateTime<Utc>>,
    /// Estimated duration
    pub estimated_duration: Option<std::time::Duration>,
    /// Required approvals
    pub required_approvals: Vec<String>,
    /// Current approvals
    pub current_approvals: Vec<String>,
    /// Blockers preventing promotion
    pub blockers: Vec<String>,
}

/// Git-based environment manager
pub struct EnvironmentManager {
    /// Git repository
    repository: GitRepository,
    /// Branch manager for advanced operations
    branch_manager: Option<BranchManager>,
    /// Environment configurations
    environments: HashMap<String, EnvironmentConfig>,
    /// Active promotion requests
    promotions: HashMap<String, PromotionRequest>,
}

impl EnvironmentManager {
    /// Create a new environment manager
    pub fn new(repository: GitRepository) -> Self {
        Self {
            repository,
            branch_manager: None,
            environments: HashMap::new(),
            promotions: HashMap::new(),
        }
    }

    /// Create a new environment manager with branch management integration
    pub fn with_branch_manager(repository: GitRepository, branch_manager: BranchManager) -> Self {
        Self {
            repository,
            branch_manager: Some(branch_manager),
            environments: HashMap::new(),
            promotions: HashMap::new(),
        }
    }

    /// Initialize default environments
    pub fn initialize_default_environments(&mut self) -> GitResult<()> {
        info!("Initializing default environments");

        // Create default environment configurations
        let default_envs = vec![
            EnvironmentConfig {
                name: "development".to_string(),
                environment_type: EnvironmentType::Development,
                branch_name: "dev".to_string(),
                config_overrides: HashMap::new(),
                description: Some("Development environment for testing changes".to_string()),
                requires_approval: false,
                allowed_sources: vec![],
                protection_rules: EnvironmentProtection::default(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            EnvironmentConfig {
                name: "staging".to_string(),
                environment_type: EnvironmentType::Staging,
                branch_name: "staging".to_string(),
                config_overrides: HashMap::new(),
                description: Some("Staging environment for pre-production testing".to_string()),
                requires_approval: true,
                allowed_sources: vec!["development".to_string()],
                protection_rules: EnvironmentProtection {
                    require_pull_request: true,
                    required_reviewers: 1,
                    require_status_checks: true,
                    required_status_checks: vec!["ci/tests".to_string()],
                    ..Default::default()
                },
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            EnvironmentConfig {
                name: "production".to_string(),
                environment_type: EnvironmentType::Production,
                branch_name: "main".to_string(),
                config_overrides: HashMap::new(),
                description: Some("Production environment".to_string()),
                requires_approval: true,
                allowed_sources: vec!["staging".to_string()],
                protection_rules: EnvironmentProtection {
                    require_pull_request: true,
                    required_reviewers: 2,
                    dismiss_stale_reviews: true,
                    require_status_checks: true,
                    required_status_checks: vec!["ci/tests".to_string(), "ci/security".to_string()],
                    restrict_pushes: true,
                    allow_force_pushes: false,
                    allow_deletions: false,
                },
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        ];

        for env in default_envs {
            self.environments.insert(env.name.clone(), env);
        }

        info!(
            "Initialized {} default environments",
            self.environments.len()
        );
        Ok(())
    }

    /// Create a new environment
    pub fn create_environment(&mut self, config: EnvironmentConfig) -> GitResult<()> {
        info!("Creating environment: {}", config.name);

        // Validate environment doesn't already exist
        if self.environments.contains_key(&config.name) {
            return Err(GitError::RepositoryOperation {
                message: format!("Environment '{}' already exists", config.name),
            });
        }

        // Validate branch name is unique across environments
        for existing_env in self.environments.values() {
            if existing_env.branch_name == config.branch_name {
                return Err(GitError::RepositoryOperation {
                    message: format!(
                        "Branch '{}' is already used by environment '{}'",
                        config.branch_name, existing_env.name
                    ),
                });
            }
        }

        // Create the branch if it doesn't exist
        match self.repository.checkout_branch(&config.branch_name) {
            Ok(_) => {
                info!("Branch '{}' already exists", config.branch_name);
            }
            Err(_) => {
                info!("Creating new branch: {}", config.branch_name);
                self.repository.create_branch(&config.branch_name, None)?;
            }
        }

        // Store the environment configuration
        self.environments
            .insert(config.name.clone(), config.clone());

        info!("Successfully created environment: {}", config.name);
        Ok(())
    }

    /// Get environment configuration
    pub fn get_environment(&self, name: &str) -> Option<&EnvironmentConfig> {
        self.environments.get(name)
    }

    /// List all environments
    pub fn list_environments(&self) -> Vec<&EnvironmentConfig> {
        self.environments.values().collect()
    }

    /// Switch to a specific environment
    pub fn switch_environment(&self, environment_name: &str) -> GitResult<()> {
        let env = self.environments.get(environment_name).ok_or_else(|| {
            GitError::RepositoryOperation {
                message: format!("Environment '{}' not found", environment_name),
            }
        })?;

        info!("Switching to environment: {}", environment_name);
        self.repository.checkout_branch(&env.branch_name)?;
        info!("Successfully switched to environment: {}", environment_name);

        Ok(())
    }

    /// Get current environment based on active branch
    pub fn get_current_environment(&self) -> GitResult<Option<&EnvironmentConfig>> {
        let current_branch = self.repository.current_branch_name()?;

        for env in self.environments.values() {
            if env.branch_name == current_branch {
                return Ok(Some(env));
            }
        }

        Ok(None)
    }

    /// Create a promotion request
    pub fn create_promotion_request(
        &mut self,
        source_env: &str,
        target_env: &str,
        message: String,
        requested_by: String,
    ) -> GitResult<String> {
        info!(
            "Creating promotion request: {} -> {}",
            source_env, target_env
        );

        // Validate environments exist
        let source_config =
            self.environments
                .get(source_env)
                .ok_or_else(|| GitError::RepositoryOperation {
                    message: format!("Source environment '{}' not found", source_env),
                })?;

        let target_config =
            self.environments
                .get(target_env)
                .ok_or_else(|| GitError::RepositoryOperation {
                    message: format!("Target environment '{}' not found", target_env),
                })?;

        // Validate promotion is allowed
        if !target_config.allowed_sources.is_empty()
            && !target_config
                .allowed_sources
                .contains(&source_env.to_string())
        {
            return Err(GitError::RepositoryOperation {
                message: format!(
                    "Promotion from '{}' to '{}' is not allowed",
                    source_env, target_env
                ),
            });
        }

        // Get source commit hash
        let original_branch = self.repository.current_branch_name()?;
        self.repository
            .checkout_branch(&source_config.branch_name)?;
        let source_commit = self.repository.get_current_commit_hash()?;
        self.repository.checkout_branch(&original_branch)?;

        // Generate promotion ID
        let promotion_id = format!(
            "promotion-{}-{}-{}",
            source_env,
            target_env,
            Utc::now().timestamp()
        );

        // Create promotion request
        let request = PromotionRequest {
            id: promotion_id.clone(),
            source_environment: source_env.to_string(),
            target_environment: target_env.to_string(),
            source_commit,
            target_commit: None,
            status: if target_config.requires_approval {
                PromotionStatus::Pending
            } else {
                PromotionStatus::Approved
            },
            message,
            requested_by,
            approved_by: None,
            created_at: Utc::now(),
            completed_at: None,
            metadata: HashMap::new(),
        };

        self.promotions.insert(promotion_id.clone(), request);

        info!("Created promotion request: {}", promotion_id);
        Ok(promotion_id)
    }

    /// Approve a promotion request
    pub fn approve_promotion(&mut self, promotion_id: &str, approved_by: String) -> GitResult<()> {
        let request =
            self.promotions
                .get_mut(promotion_id)
                .ok_or_else(|| GitError::RepositoryOperation {
                    message: format!("Promotion request '{}' not found", promotion_id),
                })?;

        if request.status != PromotionStatus::Pending {
            return Err(GitError::RepositoryOperation {
                message: format!("Promotion request '{}' is not pending", promotion_id),
            });
        }

        request.status = PromotionStatus::Approved;
        request.approved_by = Some(approved_by);

        info!("Approved promotion request: {}", promotion_id);
        Ok(())
    }

    /// Execute a promotion
    pub fn execute_promotion(&mut self, promotion_id: &str) -> GitResult<PromotionResult> {
        // First, validate the promotion request and get environment info
        let (source_env_name, target_env_name) = {
            let request =
                self.promotions
                    .get(promotion_id)
                    .ok_or_else(|| GitError::RepositoryOperation {
                        message: format!("Promotion request '{}' not found", promotion_id),
                    })?;

            if request.status != PromotionStatus::Approved {
                return Err(GitError::RepositoryOperation {
                    message: format!("Promotion request '{}' is not approved", promotion_id),
                });
            }

            (
                request.source_environment.clone(),
                request.target_environment.clone(),
            )
        };

        // Get environment configurations
        let source_config = self
            .environments
            .get(&source_env_name)
            .ok_or_else(|| GitError::RepositoryOperation {
                message: format!("Source environment '{}' not found", source_env_name),
            })?
            .clone();

        let target_config = self
            .environments
            .get(&target_env_name)
            .ok_or_else(|| GitError::RepositoryOperation {
                message: format!("Target environment '{}' not found", target_env_name),
            })?
            .clone();

        // Update promotion status to in progress
        {
            let request = self.promotions.get_mut(promotion_id).unwrap();
            info!("Executing promotion: {}", promotion_id);
            request.status = PromotionStatus::InProgress;
        }

        // Store original branch to restore later
        let original_branch = self.repository.current_branch_name()?;

        let result = self.perform_promotion(&source_config, &target_config);

        // Restore original branch
        if let Err(e) = self.repository.checkout_branch(&original_branch) {
            warn!(
                "Failed to restore original branch '{}': {}",
                original_branch, e
            );
        }

        // Update promotion request with result
        let request = self.promotions.get_mut(promotion_id).unwrap();
        match result {
            Ok(mut promotion_result) => {
                request.status = PromotionStatus::Completed;
                request.completed_at = Some(Utc::now());
                request.target_commit = Some(self.repository.get_current_commit_hash()?);

                promotion_result.request = request.clone();
                info!("Successfully completed promotion: {}", promotion_id);
                Ok(promotion_result)
            }
            Err(e) => {
                request.status = PromotionStatus::Failed;
                request.completed_at = Some(Utc::now());

                let promotion_result = PromotionResult {
                    request: request.clone(),
                    success: false,
                    error: Some(e.to_string()),
                    changed_files: Vec::new(),
                    conflicts: Vec::new(),
                };

                warn!("Promotion failed: {}: {}", promotion_id, e);
                Ok(promotion_result)
            }
        }
    }

    /// Get promotion request
    pub fn get_promotion(&self, promotion_id: &str) -> Option<&PromotionRequest> {
        self.promotions.get(promotion_id)
    }

    /// List promotion requests
    pub fn list_promotions(&self) -> Vec<&PromotionRequest> {
        self.promotions.values().collect()
    }

    /// Create an automated promotion workflow
    pub fn create_automated_promotion_workflow(
        &mut self,
        source_env: &str,
        target_env: &str,
        auto_approve: bool,
    ) -> GitResult<String> {
        info!(
            "Creating automated promotion workflow: {} -> {}",
            source_env, target_env
        );

        // Validate the promotion path
        self.validate_promotion_path(source_env, target_env)?;

        // Create the promotion request
        let promotion_id = self.create_promotion_request(
            source_env,
            target_env,
            format!("Automated promotion from {} to {}", source_env, target_env),
            "system".to_string(),
        )?;

        // Auto-approve if requested and conditions are met
        if auto_approve {
            let (requires_approval, is_development) = {
                let target_config = self.environments.get(target_env).unwrap();
                (
                    target_config.requires_approval,
                    target_config.environment_type == EnvironmentType::Development,
                )
            };

            if !requires_approval {
                self.approve_promotion(&promotion_id, "system".to_string())?;

                // Execute immediately for development environments
                if is_development {
                    self.execute_promotion(&promotion_id)?;
                }
            }
        }

        Ok(promotion_id)
    }

    /// Validate promotion path against environment rules
    pub fn validate_promotion_path(&self, source_env: &str, target_env: &str) -> GitResult<()> {
        let source_config =
            self.environments
                .get(source_env)
                .ok_or_else(|| GitError::RepositoryOperation {
                    message: format!("Source environment '{}' not found", source_env),
                })?;

        let target_config =
            self.environments
                .get(target_env)
                .ok_or_else(|| GitError::RepositoryOperation {
                    message: format!("Target environment '{}' not found", target_env),
                })?;

        // Check if promotion is allowed
        if !target_config.allowed_sources.is_empty()
            && !target_config
                .allowed_sources
                .contains(&source_env.to_string())
        {
            return Err(GitError::RepositoryOperation {
                message: format!(
                    "Promotion from '{}' to '{}' is not allowed. Allowed sources: {:?}",
                    source_env, target_env, target_config.allowed_sources
                ),
            });
        }

        // Validate branch protection rules if branch manager is available
        if let Some(ref branch_manager) = self.branch_manager {
            // Check target branch protection
            branch_manager.validate_branch_operation(
                &target_config.branch_name,
                BranchOperation::MergePullRequest,
            )?;
        }

        // Validate environment hierarchy (dev -> staging -> prod)
        match (
            source_config.environment_type,
            target_config.environment_type,
        ) {
            (EnvironmentType::Development, EnvironmentType::Production) => {
                return Err(GitError::RepositoryOperation {
                    message: "Direct promotion from development to production is not allowed. Use staging as intermediate.".to_string(),
                });
            }
            _ => {} // Other combinations are allowed
        }

        Ok(())
    }

    /// Get promotion workflow status
    pub fn get_promotion_workflow_status(
        &self,
        promotion_id: &str,
    ) -> Option<PromotionWorkflowStatus> {
        self.promotions
            .get(promotion_id)
            .map(|request| PromotionWorkflowStatus {
                promotion_id: promotion_id.to_string(),
                status: request.status,
                source_environment: request.source_environment.clone(),
                target_environment: request.target_environment.clone(),
                created_at: request.created_at,
                completed_at: request.completed_at,
                estimated_duration: self.estimate_promotion_duration(request),
                required_approvals: self.get_required_approvals(&request.target_environment),
                current_approvals: request
                    .approved_by
                    .clone()
                    .map(|a| vec![a])
                    .unwrap_or_default(),
                blockers: self.check_promotion_blockers(request),
            })
    }

    /// Cancel a promotion request
    pub fn cancel_promotion(&mut self, promotion_id: &str, reason: String) -> GitResult<()> {
        let request =
            self.promotions
                .get_mut(promotion_id)
                .ok_or_else(|| GitError::RepositoryOperation {
                    message: format!("Promotion request '{}' not found", promotion_id),
                })?;

        if request.status == PromotionStatus::InProgress {
            return Err(GitError::RepositoryOperation {
                message: "Cannot cancel promotion that is in progress".to_string(),
            });
        }

        if request.status == PromotionStatus::Completed {
            return Err(GitError::RepositoryOperation {
                message: "Cannot cancel completed promotion".to_string(),
            });
        }

        request.status = PromotionStatus::Cancelled;
        request.completed_at = Some(Utc::now());
        request.metadata.insert(
            "cancellation_reason".to_string(),
            serde_json::Value::String(reason),
        );

        info!("Cancelled promotion request: {}", promotion_id);
        Ok(())
    }

    /// Get promotion history for an environment
    pub fn get_environment_promotion_history(
        &self,
        environment_name: &str,
    ) -> Vec<&PromotionRequest> {
        self.promotions
            .values()
            .filter(|p| {
                p.target_environment == environment_name || p.source_environment == environment_name
            })
            .collect()
    }

    /// Check if environment has pending promotions
    pub fn has_pending_promotions(&self, environment_name: &str) -> bool {
        self.promotions.values().any(|p| {
            (p.source_environment == environment_name || p.target_environment == environment_name)
                && matches!(
                    p.status,
                    PromotionStatus::Pending
                        | PromotionStatus::Approved
                        | PromotionStatus::InProgress
                )
        })
    }

    // Private helper methods for promotion workflows

    fn estimate_promotion_duration(
        &self,
        _request: &PromotionRequest,
    ) -> Option<std::time::Duration> {
        // This would calculate estimated duration based on:
        // - Size of changes
        // - Historical promotion times
        // - Environment complexity
        Some(std::time::Duration::from_secs(300)) // 5 minutes placeholder
    }

    fn get_required_approvals(&self, environment_name: &str) -> Vec<String> {
        if let Some(env) = self.environments.get(environment_name) {
            match env.environment_type {
                EnvironmentType::Production => {
                    vec!["tech-lead".to_string(), "security".to_string()]
                }
                EnvironmentType::Staging => vec!["senior-dev".to_string()],
                _ => vec![],
            }
        } else {
            vec![]
        }
    }

    fn check_promotion_blockers(&self, request: &PromotionRequest) -> Vec<String> {
        let mut blockers = Vec::new();

        // Check if target environment has pending changes
        if self.has_pending_promotions(&request.target_environment) {
            blockers.push("Target environment has pending promotions".to_string());
        }

        // Check if source branch is behind target
        // This would be implemented with actual Git operations

        // Check if required status checks are passing
        // This would integrate with CI/CD systems

        blockers
    }

    /// Get branch information for environment
    pub fn get_environment_branch_info(&self, environment_name: &str) -> GitResult<BranchInfo> {
        let env = self.environments.get(environment_name).ok_or_else(|| {
            GitError::RepositoryOperation {
                message: format!("Environment '{}' not found", environment_name),
            }
        })?;

        let branches = self.repository.list_branches(None)?;
        branches
            .into_iter()
            .find(|branch| branch.name == env.branch_name)
            .ok_or_else(|| GitError::RepositoryOperation {
                message: format!(
                    "Branch '{}' not found for environment '{}'",
                    env.branch_name, environment_name
                ),
            })
    }

    /// Sync environment with remote
    pub fn sync_environment(&self, environment_name: &str) -> GitResult<()> {
        let env = self.environments.get(environment_name).ok_or_else(|| {
            GitError::RepositoryOperation {
                message: format!("Environment '{}' not found", environment_name),
            }
        })?;

        info!("Syncing environment '{}' with remote", environment_name);

        // Store current branch
        let original_branch = self.repository.current_branch_name()?;

        // Switch to environment branch
        self.repository.checkout_branch(&env.branch_name)?;

        // Pull latest changes
        let result = self.repository.pull(None, Some(&env.branch_name));

        // Restore original branch
        if let Err(e) = self.repository.checkout_branch(&original_branch) {
            warn!(
                "Failed to restore original branch '{}': {}",
                original_branch, e
            );
        }

        result?;
        info!("Successfully synced environment '{}'", environment_name);
        Ok(())
    }

    // Private helper methods

    fn perform_promotion(
        &self,
        source_config: &EnvironmentConfig,
        target_config: &EnvironmentConfig,
    ) -> GitResult<PromotionResult> {
        info!(
            "Performing promotion from '{}' to '{}'",
            source_config.name, target_config.name
        );

        // Switch to target branch
        self.repository
            .checkout_branch(&target_config.branch_name)?;

        // Fetch latest changes
        self.repository.fetch(None)?;

        // Switch to source branch to get changes
        self.repository
            .checkout_branch(&source_config.branch_name)?;
        let source_commit = self.repository.get_current_commit_hash()?;

        // Switch back to target branch
        self.repository
            .checkout_branch(&target_config.branch_name)?;

        // Get current status before merge
        let _status_before = self.repository.status()?;

        // Attempt to merge source into target
        let merge_result = self.repository.pull(None, Some(&source_config.branch_name));

        let mut conflicts = Vec::new();
        let mut changed_files = Vec::new();

        match merge_result {
            Ok(_) => {
                // Get status after merge to see what changed
                let status_after = self.repository.status()?;
                changed_files = status_after
                    .changed_files
                    .into_iter()
                    .map(|change| change.path)
                    .collect();

                info!("Promotion merge completed successfully");
            }
            Err(GitError::MergeConflict { files }) => {
                conflicts = files;
                warn!("Merge conflicts detected during promotion");

                return Ok(PromotionResult {
                    request: PromotionRequest {
                        id: "temp".to_string(),
                        source_environment: source_config.name.clone(),
                        target_environment: target_config.name.clone(),
                        source_commit: source_commit.clone(),
                        target_commit: None,
                        status: PromotionStatus::Failed,
                        message: "Merge conflicts detected".to_string(),
                        requested_by: "system".to_string(),
                        approved_by: None,
                        created_at: Utc::now(),
                        completed_at: None,
                        metadata: HashMap::new(),
                    },
                    success: false,
                    error: Some("Merge conflicts detected".to_string()),
                    changed_files,
                    conflicts,
                });
            }
            Err(e) => {
                return Err(e);
            }
        }

        // Create promotion commit if there are changes
        if !changed_files.is_empty() {
            let commit_message = format!(
                "Promote changes from {} to {}\n\nSource commit: {}",
                source_config.name, target_config.name, source_commit
            );

            self.repository.stage_all()?;
            self.repository.commit(&commit_message, None, None)?;
        }

        Ok(PromotionResult {
            request: PromotionRequest {
                id: "temp".to_string(),
                source_environment: source_config.name.clone(),
                target_environment: target_config.name.clone(),
                source_commit,
                target_commit: None,
                status: PromotionStatus::Completed,
                message: "Promotion completed successfully".to_string(),
                requested_by: "system".to_string(),
                approved_by: None,
                created_at: Utc::now(),
                completed_at: Some(Utc::now()),
                metadata: HashMap::new(),
            },
            success: true,
            error: None,
            changed_files,
            conflicts,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_type_from_str() {
        assert_eq!(
            EnvironmentType::from_str("dev"),
            Some(EnvironmentType::Development)
        );
        assert_eq!(
            EnvironmentType::from_str("Development"),
            Some(EnvironmentType::Development)
        );
        assert_eq!(
            EnvironmentType::from_str("staging"),
            Some(EnvironmentType::Staging)
        );
        assert_eq!(
            EnvironmentType::from_str("PRODUCTION"),
            Some(EnvironmentType::Production)
        );
        assert_eq!(EnvironmentType::from_str("invalid"), None);
    }

    #[test]
    fn test_environment_type_default_branch() {
        assert_eq!(EnvironmentType::Development.default_branch_name(), "dev");
        assert_eq!(EnvironmentType::Staging.default_branch_name(), "staging");
        assert_eq!(EnvironmentType::Production.default_branch_name(), "main");
    }

    #[test]
    fn test_promotion_status_display() {
        assert_eq!(PromotionStatus::Pending.to_string(), "pending");
        assert_eq!(PromotionStatus::Completed.to_string(), "completed");
        assert_eq!(PromotionStatus::Failed.to_string(), "failed");
    }
}
