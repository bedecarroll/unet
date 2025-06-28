//! Git conflict resolution system for μNet
//!
//! This module provides comprehensive conflict detection, analysis, and resolution
//! capabilities for Git operations, with both automated and manual resolution options.

use crate::git::environment::{EnvironmentConfig, EnvironmentType};
use crate::git::repository::GitRepository;
use crate::git::types::{FileChange, FileStatus, GitError, GitResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Types of merge conflicts that can occur
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictType {
    /// Content conflict - both sides modified the same lines
    Content,
    /// Add/Add conflict - both sides added the same file
    AddAdd,
    /// Delete/Modify conflict - one side deleted, other modified
    DeleteModify,
    /// Modify/Delete conflict - one side modified, other deleted
    ModifyDelete,
    /// Rename/Add conflict - renamed file conflicts with added file
    RenameAdd,
    /// Rename/Rename conflict - both sides renamed to different names
    RenameRename,
    /// Submodule conflict - submodule pointer conflicts
    Submodule,
    /// Binary conflict - binary files conflict
    Binary,
}

impl std::fmt::Display for ConflictType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConflictType::Content => write!(f, "content"),
            ConflictType::AddAdd => write!(f, "add/add"),
            ConflictType::DeleteModify => write!(f, "delete/modify"),
            ConflictType::ModifyDelete => write!(f, "modify/delete"),
            ConflictType::RenameAdd => write!(f, "rename/add"),
            ConflictType::RenameRename => write!(f, "rename/rename"),
            ConflictType::Submodule => write!(f, "submodule"),
            ConflictType::Binary => write!(f, "binary"),
        }
    }
}

/// Conflict resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionStrategy {
    /// Use the version from the current branch (ours)
    UseOurs,
    /// Use the version from the incoming branch (theirs)
    UseTheirs,
    /// Attempt automatic merge using Git's default strategy
    AutoMerge,
    /// Require manual resolution
    Manual,
    /// Use a custom resolution strategy
    Custom,
    /// Abort the merge operation
    Abort,
}

impl std::fmt::Display for ResolutionStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolutionStrategy::UseOurs => write!(f, "use-ours"),
            ResolutionStrategy::UseTheirs => write!(f, "use-theirs"),
            ResolutionStrategy::AutoMerge => write!(f, "auto-merge"),
            ResolutionStrategy::Manual => write!(f, "manual"),
            ResolutionStrategy::Custom => write!(f, "custom"),
            ResolutionStrategy::Abort => write!(f, "abort"),
        }
    }
}

/// Information about a specific merge conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictInfo {
    /// Path to the conflicted file
    pub file_path: PathBuf,
    /// Type of conflict
    pub conflict_type: ConflictType,
    /// Content from the current branch (ours)
    pub our_content: Option<String>,
    /// Content from the incoming branch (theirs)  
    pub their_content: Option<String>,
    /// Base content (common ancestor)
    pub base_content: Option<String>,
    /// Suggested resolution strategy
    pub suggested_strategy: ResolutionStrategy,
    /// Whether this conflict can be automatically resolved
    pub auto_resolvable: bool,
    /// Additional metadata about the conflict
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ConflictInfo {
    /// Create a new conflict info
    pub fn new(file_path: PathBuf, conflict_type: ConflictType) -> Self {
        let auto_resolvable = matches!(
            conflict_type,
            ConflictType::DeleteModify | ConflictType::ModifyDelete | ConflictType::AddAdd
        );

        let suggested_strategy = if auto_resolvable {
            ResolutionStrategy::AutoMerge
        } else {
            ResolutionStrategy::Manual
        };

        Self {
            file_path,
            conflict_type,
            our_content: None,
            their_content: None,
            base_content: None,
            suggested_strategy,
            auto_resolvable,
            metadata: HashMap::new(),
        }
    }

    /// Check if this is a safe file type for automatic resolution
    pub fn is_safe_for_auto_resolution(&self) -> bool {
        if !self.auto_resolvable {
            return false;
        }

        // Check file extension for safe auto-resolution
        if let Some(extension) = self.file_path.extension().and_then(|s| s.to_str()) {
            matches!(
                extension.to_lowercase().as_str(),
                "json" | "yaml" | "yml" | "toml" | "ini" | "cfg" | "conf" | "txt" | "md"
            )
        } else {
            false
        }
    }

    /// Get conflict markers for manual resolution display
    pub fn get_conflict_markers(&self) -> Option<String> {
        if let (Some(ours), Some(theirs)) = (&self.our_content, &self.their_content) {
            Some(format!(
                "<<<<<<< HEAD\n{}\n=======\n{}\n>>>>>>> incoming",
                ours, theirs
            ))
        } else {
            None
        }
    }
}

/// Result of a conflict resolution attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolutionResult {
    /// Path to the resolved file
    pub file_path: PathBuf,
    /// Whether the resolution was successful
    pub success: bool,
    /// Strategy used for resolution
    pub strategy_used: ResolutionStrategy,
    /// Resolved content (if successful)
    pub resolved_content: Option<String>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Whether manual intervention is still required
    pub requires_manual_intervention: bool,
}

/// Conflict detection and resolution results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictAnalysis {
    /// List of detected conflicts
    pub conflicts: Vec<ConflictInfo>,
    /// Total number of conflicts
    pub conflict_count: usize,
    /// Number of automatically resolvable conflicts
    pub auto_resolvable_count: usize,
    /// Number of conflicts requiring manual resolution
    pub manual_resolution_count: usize,
    /// Whether the merge can proceed automatically
    pub can_auto_resolve: bool,
    /// Recommended resolution approach
    pub recommended_approach: ResolutionStrategy,
    /// Analysis timestamp
    pub analyzed_at: DateTime<Utc>,
}

impl ConflictAnalysis {
    /// Create a new conflict analysis from a list of conflicts
    pub fn new(conflicts: Vec<ConflictInfo>) -> Self {
        let conflict_count = conflicts.len();
        let auto_resolvable_count = conflicts.iter().filter(|c| c.auto_resolvable).count();
        let manual_resolution_count = conflict_count - auto_resolvable_count;

        let can_auto_resolve = manual_resolution_count == 0;
        let recommended_approach = if can_auto_resolve {
            ResolutionStrategy::AutoMerge
        } else if auto_resolvable_count > manual_resolution_count {
            ResolutionStrategy::Manual // Mixed approach
        } else {
            ResolutionStrategy::Manual
        };

        Self {
            conflicts,
            conflict_count,
            auto_resolvable_count,
            manual_resolution_count,
            can_auto_resolve,
            recommended_approach,
            analyzed_at: Utc::now(),
        }
    }

    /// Get conflicts by type
    pub fn get_conflicts_by_type(&self, conflict_type: ConflictType) -> Vec<&ConflictInfo> {
        self.conflicts
            .iter()
            .filter(|c| c.conflict_type == conflict_type)
            .collect()
    }

    /// Get auto-resolvable conflicts
    pub fn get_auto_resolvable_conflicts(&self) -> Vec<&ConflictInfo> {
        self.conflicts
            .iter()
            .filter(|c| c.auto_resolvable)
            .collect()
    }

    /// Get manual resolution conflicts
    pub fn get_manual_resolution_conflicts(&self) -> Vec<&ConflictInfo> {
        self.conflicts
            .iter()
            .filter(|c| !c.auto_resolvable)
            .collect()
    }
}

/// Configuration for conflict resolution behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolutionConfig {
    /// Default strategy for content conflicts
    pub default_content_strategy: ResolutionStrategy,
    /// Default strategy for add/add conflicts
    pub default_add_add_strategy: ResolutionStrategy,
    /// Default strategy for delete/modify conflicts
    pub default_delete_modify_strategy: ResolutionStrategy,
    /// Whether to attempt automatic resolution
    pub enable_auto_resolution: bool,
    /// File patterns to always resolve manually
    pub manual_resolution_patterns: Vec<String>,
    /// File patterns safe for automatic resolution
    pub auto_resolution_patterns: Vec<String>,
    /// Maximum file size for automatic resolution (bytes)
    pub max_auto_resolution_file_size: usize,
    /// Whether to create backup files before resolution
    pub create_backups: bool,
}

impl Default for ConflictResolutionConfig {
    fn default() -> Self {
        Self {
            default_content_strategy: ResolutionStrategy::Manual,
            default_add_add_strategy: ResolutionStrategy::Manual,
            default_delete_modify_strategy: ResolutionStrategy::UseOurs,
            enable_auto_resolution: true,
            manual_resolution_patterns: vec![
                "*.key".to_string(),
                "*.pem".to_string(),
                "*.cert".to_string(),
                "Cargo.lock".to_string(),
                "package-lock.json".to_string(),
            ],
            auto_resolution_patterns: vec![
                "*.json".to_string(),
                "*.yaml".to_string(),
                "*.yml".to_string(),
                "*.toml".to_string(),
                "*.md".to_string(),
                "*.txt".to_string(),
            ],
            max_auto_resolution_file_size: 1_048_576, // 1 MB
            create_backups: true,
        }
    }
}

/// Git conflict resolution manager
pub struct ConflictResolver {
    /// Git repository
    repository: GitRepository,
    /// Configuration for conflict resolution
    config: ConflictResolutionConfig,
    /// Environment configurations for context
    environments: HashMap<String, EnvironmentConfig>,
}

impl ConflictResolver {
    /// Create a new conflict resolver
    pub fn new(repository: GitRepository) -> Self {
        Self {
            repository,
            config: ConflictResolutionConfig::default(),
            environments: HashMap::new(),
        }
    }

    /// Create a new conflict resolver with custom configuration
    pub fn with_config(repository: GitRepository, config: ConflictResolutionConfig) -> Self {
        Self {
            repository,
            config,
            environments: HashMap::new(),
        }
    }

    /// Configure environments for context-aware resolution
    pub fn configure_environments(&mut self, environments: Vec<EnvironmentConfig>) {
        self.environments.clear();
        for env in environments {
            self.environments.insert(env.name.clone(), env);
        }
    }

    /// Detect conflicts in the current repository state
    pub fn detect_conflicts(&self) -> GitResult<ConflictAnalysis> {
        info!("Detecting merge conflicts in repository");

        // Get repository status to find conflicted files
        let status = self.repository.status()?;
        let mut conflicts = Vec::new();

        for change in &status.changed_files {
            if matches!(change.status, FileStatus::Conflicted) {
                let conflict_type = self.determine_conflict_type(&change)?;
                let mut conflict_info = ConflictInfo::new(change.path.clone(), conflict_type);

                // Load file contents for analysis
                self.load_conflict_content(&mut conflict_info)?;

                // Apply configuration-based resolution suggestions
                self.apply_resolution_config(&mut conflict_info);

                conflicts.push(conflict_info);
            }
        }

        let analysis = ConflictAnalysis::new(conflicts);
        info!(
            "Detected {} conflicts ({} auto-resolvable, {} manual)",
            analysis.conflict_count,
            analysis.auto_resolvable_count,
            analysis.manual_resolution_count
        );

        Ok(analysis)
    }

    /// Automatically resolve conflicts where safe to do so
    pub fn auto_resolve_conflicts(&self) -> GitResult<Vec<ConflictResolutionResult>> {
        if !self.config.enable_auto_resolution {
            return Ok(Vec::new());
        }

        info!("Attempting automatic conflict resolution");
        let analysis = self.detect_conflicts()?;
        let mut results = Vec::new();

        for conflict in analysis.get_auto_resolvable_conflicts() {
            if conflict.is_safe_for_auto_resolution() {
                let result = self.resolve_conflict_auto(conflict)?;
                results.push(result);
            }
        }

        info!("Automatically resolved {} conflicts", results.len());
        Ok(results)
    }

    /// Resolve a specific conflict with a given strategy
    pub fn resolve_conflict(
        &self,
        file_path: &Path,
        strategy: ResolutionStrategy,
    ) -> GitResult<ConflictResolutionResult> {
        info!(
            "Resolving conflict in '{}' with strategy '{}'",
            file_path.display(),
            strategy
        );

        let analysis = self.detect_conflicts()?;
        let conflict = analysis
            .conflicts
            .iter()
            .find(|c| c.file_path == file_path)
            .ok_or_else(|| GitError::RepositoryOperation {
                message: format!("No conflict found for file: {}", file_path.display()),
            })?;

        match strategy {
            ResolutionStrategy::UseOurs => self.resolve_use_ours(conflict),
            ResolutionStrategy::UseTheirs => self.resolve_use_theirs(conflict),
            ResolutionStrategy::AutoMerge => self.resolve_conflict_auto(conflict),
            ResolutionStrategy::Manual => self.prepare_manual_resolution(conflict),
            ResolutionStrategy::Custom => self.resolve_custom(conflict),
            ResolutionStrategy::Abort => self.abort_resolution(conflict),
        }
    }

    /// Get conflict resolution interface for manual resolution
    pub fn get_manual_resolution_interface(
        &self,
        file_path: &Path,
    ) -> GitResult<ManualResolutionInterface> {
        let analysis = self.detect_conflicts()?;
        let conflict = analysis
            .conflicts
            .iter()
            .find(|c| c.file_path == file_path)
            .ok_or_else(|| GitError::RepositoryOperation {
                message: format!("No conflict found for file: {}", file_path.display()),
            })?;

        Ok(ManualResolutionInterface::new(conflict.clone()))
    }

    /// Apply resolved content to a conflicted file
    pub fn apply_resolution(
        &self,
        file_path: &Path,
        resolved_content: &str,
    ) -> GitResult<ConflictResolutionResult> {
        info!("Applying manual resolution to: {}", file_path.display());

        // Create backup if configured
        if self.config.create_backups {
            self.create_backup_file(file_path)?;
        }

        // Write the resolved content
        std::fs::write(file_path, resolved_content).map_err(|e| GitError::RepositoryOperation {
            message: format!("Failed to write resolved content: {}", e),
        })?;

        // Stage the resolved file
        // Note: This would use git2 staging functionality
        info!("Staging resolved file: {}", file_path.display());

        Ok(ConflictResolutionResult {
            file_path: file_path.to_path_buf(),
            success: true,
            strategy_used: ResolutionStrategy::Manual,
            resolved_content: Some(resolved_content.to_string()),
            error: None,
            requires_manual_intervention: false,
        })
    }

    /// Check if all conflicts have been resolved
    pub fn are_all_conflicts_resolved(&self) -> GitResult<bool> {
        let analysis = self.detect_conflicts()?;
        Ok(analysis.conflict_count == 0)
    }

    /// Get environment-specific resolution recommendations
    pub fn get_environment_resolution_recommendations(
        &self,
        environment_name: &str,
    ) -> GitResult<Vec<EnvironmentResolutionRecommendation>> {
        let env = self.environments.get(environment_name).ok_or_else(|| {
            GitError::RepositoryOperation {
                message: format!("Environment '{}' not found", environment_name),
            }
        })?;

        let analysis = self.detect_conflicts()?;
        let mut recommendations = Vec::new();

        for conflict in &analysis.conflicts {
            let recommendation = self.get_environment_specific_recommendation(env, conflict);
            recommendations.push(recommendation);
        }

        Ok(recommendations)
    }

    // Private helper methods

    fn determine_conflict_type(&self, change: &FileChange) -> GitResult<ConflictType> {
        // This is a simplified implementation
        // In a real implementation, you'd analyze the Git index to determine the exact conflict type
        match change.status {
            FileStatus::Conflicted => {
                // Default to content conflict - would be determined by actual Git analysis
                Ok(ConflictType::Content)
            }
            _ => Err(GitError::RepositoryOperation {
                message: "File is not in conflicted state".to_string(),
            }),
        }
    }

    fn load_conflict_content(&self, conflict: &mut ConflictInfo) -> GitResult<()> {
        let file_path = &conflict.file_path;

        // In a real implementation, you'd use git2 to get the different versions
        // For now, this is a simplified implementation
        if file_path.exists() {
            let content =
                std::fs::read_to_string(file_path).map_err(|e| GitError::RepositoryOperation {
                    message: format!("Failed to read conflicted file: {}", e),
                })?;

            // Parse conflict markers to extract different versions
            if content.contains("<<<<<<<") && content.contains(">>>>>>>") {
                let (ours, theirs) = self.parse_conflict_markers(&content)?;
                conflict.our_content = Some(ours);
                conflict.their_content = Some(theirs);
            }
        }

        Ok(())
    }

    fn parse_conflict_markers(&self, content: &str) -> GitResult<(String, String)> {
        let lines: Vec<&str> = content.lines().collect();
        let mut ours = Vec::new();
        let mut theirs = Vec::new();
        let mut in_ours = false;
        let mut in_theirs = false;

        for line in lines {
            if line.starts_with("<<<<<<<") {
                in_ours = true;
                continue;
            } else if line.starts_with("=======") {
                in_ours = false;
                in_theirs = true;
                continue;
            } else if line.starts_with(">>>>>>>") {
                in_theirs = false;
                continue;
            }

            if in_ours {
                ours.push(line);
            } else if in_theirs {
                theirs.push(line);
            }
        }

        Ok((ours.join("\n"), theirs.join("\n")))
    }

    fn apply_resolution_config(&self, conflict: &mut ConflictInfo) {
        let file_path_str = conflict.file_path.to_string_lossy();

        // Check if file matches manual resolution patterns
        for pattern in &self.config.manual_resolution_patterns {
            if self.matches_pattern(&file_path_str, pattern) {
                conflict.auto_resolvable = false;
                conflict.suggested_strategy = ResolutionStrategy::Manual;
                return;
            }
        }

        // Check if file matches auto resolution patterns
        for pattern in &self.config.auto_resolution_patterns {
            if self.matches_pattern(&file_path_str, pattern) {
                conflict.auto_resolvable = true;
                conflict.suggested_strategy = match conflict.conflict_type {
                    ConflictType::Content => self.config.default_content_strategy,
                    ConflictType::AddAdd => self.config.default_add_add_strategy,
                    ConflictType::DeleteModify | ConflictType::ModifyDelete => {
                        self.config.default_delete_modify_strategy
                    }
                    _ => ResolutionStrategy::Manual,
                };
                return;
            }
        }
    }

    fn matches_pattern(&self, file_path: &str, pattern: &str) -> bool {
        // Simple glob-like pattern matching
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                file_path.starts_with(parts[0]) && file_path.ends_with(parts[1])
            } else {
                false
            }
        } else {
            file_path == pattern
        }
    }

    fn resolve_conflict_auto(
        &self,
        conflict: &ConflictInfo,
    ) -> GitResult<ConflictResolutionResult> {
        match conflict.conflict_type {
            ConflictType::DeleteModify => {
                // Use the modified version (ours)
                self.resolve_use_ours(conflict)
            }
            ConflictType::ModifyDelete => {
                // Use the existing version (ours)
                self.resolve_use_ours(conflict)
            }
            ConflictType::AddAdd => {
                // For add/add conflicts, attempt to merge if both are text files
                if let (Some(ours), Some(theirs)) = (&conflict.our_content, &conflict.their_content)
                {
                    // Simple merge strategy - append both contents
                    let merged = format!("{}\n{}", ours, theirs);
                    Ok(ConflictResolutionResult {
                        file_path: conflict.file_path.clone(),
                        success: true,
                        strategy_used: ResolutionStrategy::AutoMerge,
                        resolved_content: Some(merged),
                        error: None,
                        requires_manual_intervention: false,
                    })
                } else {
                    self.prepare_manual_resolution(conflict)
                }
            }
            _ => self.prepare_manual_resolution(conflict),
        }
    }

    fn resolve_use_ours(&self, conflict: &ConflictInfo) -> GitResult<ConflictResolutionResult> {
        if let Some(content) = &conflict.our_content {
            Ok(ConflictResolutionResult {
                file_path: conflict.file_path.clone(),
                success: true,
                strategy_used: ResolutionStrategy::UseOurs,
                resolved_content: Some(content.clone()),
                error: None,
                requires_manual_intervention: false,
            })
        } else {
            Err(GitError::RepositoryOperation {
                message: "No 'ours' content available for resolution".to_string(),
            })
        }
    }

    fn resolve_use_theirs(&self, conflict: &ConflictInfo) -> GitResult<ConflictResolutionResult> {
        if let Some(content) = &conflict.their_content {
            Ok(ConflictResolutionResult {
                file_path: conflict.file_path.clone(),
                success: true,
                strategy_used: ResolutionStrategy::UseTheirs,
                resolved_content: Some(content.clone()),
                error: None,
                requires_manual_intervention: false,
            })
        } else {
            Err(GitError::RepositoryOperation {
                message: "No 'theirs' content available for resolution".to_string(),
            })
        }
    }

    fn resolve_custom(&self, conflict: &ConflictInfo) -> GitResult<ConflictResolutionResult> {
        // Placeholder for custom resolution logic
        // This would integrate with external tools or custom resolution strategies
        Ok(ConflictResolutionResult {
            file_path: conflict.file_path.clone(),
            success: false,
            strategy_used: ResolutionStrategy::Custom,
            resolved_content: None,
            error: Some("Custom resolution not implemented".to_string()),
            requires_manual_intervention: true,
        })
    }

    fn prepare_manual_resolution(
        &self,
        conflict: &ConflictInfo,
    ) -> GitResult<ConflictResolutionResult> {
        Ok(ConflictResolutionResult {
            file_path: conflict.file_path.clone(),
            success: false,
            strategy_used: ResolutionStrategy::Manual,
            resolved_content: None,
            error: Some("Manual resolution required".to_string()),
            requires_manual_intervention: true,
        })
    }

    fn abort_resolution(&self, conflict: &ConflictInfo) -> GitResult<ConflictResolutionResult> {
        Ok(ConflictResolutionResult {
            file_path: conflict.file_path.clone(),
            success: false,
            strategy_used: ResolutionStrategy::Abort,
            resolved_content: None,
            error: Some("Resolution aborted".to_string()),
            requires_manual_intervention: false,
        })
    }

    fn create_backup_file(&self, file_path: &Path) -> GitResult<()> {
        let backup_path = file_path.with_extension(format!(
            "{}.backup.{}",
            file_path.extension().and_then(|s| s.to_str()).unwrap_or(""),
            Utc::now().timestamp()
        ));

        std::fs::copy(file_path, &backup_path).map_err(|e| GitError::RepositoryOperation {
            message: format!("Failed to create backup file: {}", e),
        })?;

        info!("Created backup file: {}", backup_path.display());
        Ok(())
    }

    fn get_environment_specific_recommendation(
        &self,
        env: &EnvironmentConfig,
        conflict: &ConflictInfo,
    ) -> EnvironmentResolutionRecommendation {
        let strategy = match env.environment_type {
            EnvironmentType::Production => {
                // Production environments are conservative
                match conflict.conflict_type {
                    ConflictType::DeleteModify | ConflictType::ModifyDelete => {
                        ResolutionStrategy::Manual // Always require manual review in production
                    }
                    _ => ResolutionStrategy::Manual,
                }
            }
            EnvironmentType::Staging => {
                // Staging can be more permissive but still careful
                match conflict.conflict_type {
                    ConflictType::DeleteModify => ResolutionStrategy::UseOurs,
                    ConflictType::AddAdd => ResolutionStrategy::AutoMerge,
                    _ => ResolutionStrategy::Manual,
                }
            }
            EnvironmentType::Development => {
                // Development can use more aggressive auto-resolution
                if conflict.auto_resolvable {
                    ResolutionStrategy::AutoMerge
                } else {
                    ResolutionStrategy::Manual
                }
            }
            EnvironmentType::Custom => conflict.suggested_strategy,
        };

        EnvironmentResolutionRecommendation {
            environment: env.name.clone(),
            file_path: conflict.file_path.clone(),
            recommended_strategy: strategy,
            reasoning: self.get_strategy_reasoning(env, conflict, strategy),
            confidence: self.calculate_confidence(env, conflict, strategy),
        }
    }

    fn get_strategy_reasoning(
        &self,
        env: &EnvironmentConfig,
        conflict: &ConflictInfo,
        strategy: ResolutionStrategy,
    ) -> String {
        match (env.environment_type, strategy) {
            (EnvironmentType::Production, ResolutionStrategy::Manual) => {
                "Production environment requires manual review for all conflicts".to_string()
            }
            (EnvironmentType::Staging, ResolutionStrategy::UseOurs) => {
                "Staging environment: prefer current branch changes".to_string()
            }
            (EnvironmentType::Development, ResolutionStrategy::AutoMerge) => {
                format!(
                    "Development environment: auto-merge {} conflict",
                    conflict.conflict_type
                )
            }
            _ => format!("Default strategy for {} conflict", conflict.conflict_type),
        }
    }

    fn calculate_confidence(
        &self,
        env: &EnvironmentConfig,
        conflict: &ConflictInfo,
        _strategy: ResolutionStrategy,
    ) -> f32 {
        let mut confidence: f32 = 0.5; // Base confidence

        // Higher confidence for auto-resolvable conflicts
        if conflict.auto_resolvable {
            confidence += 0.3;
        }

        // Environment-specific adjustments
        match env.environment_type {
            EnvironmentType::Production => confidence *= 0.7, // Lower confidence in prod
            EnvironmentType::Development => confidence += 0.2, // Higher confidence in dev
            _ => {}
        }

        // File type adjustments
        if conflict.is_safe_for_auto_resolution() {
            confidence += 0.2;
        }

        confidence.min(1.0)
    }
}

/// Manual conflict resolution interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualResolutionInterface {
    /// Conflict information
    pub conflict: ConflictInfo,
    /// Available resolution options
    pub resolution_options: Vec<ResolutionOption>,
    /// Diff view of the conflict
    pub diff_view: Option<String>,
    /// Suggested resolution
    pub suggested_resolution: Option<String>,
}

impl ManualResolutionInterface {
    /// Create a new manual resolution interface
    pub fn new(conflict: ConflictInfo) -> Self {
        let resolution_options = vec![
            ResolutionOption {
                strategy: ResolutionStrategy::UseOurs,
                description: "Use the version from the current branch".to_string(),
                preview: conflict.our_content.clone(),
            },
            ResolutionOption {
                strategy: ResolutionStrategy::UseTheirs,
                description: "Use the version from the incoming branch".to_string(),
                preview: conflict.their_content.clone(),
            },
            ResolutionOption {
                strategy: ResolutionStrategy::Manual,
                description: "Manually edit the file to resolve conflicts".to_string(),
                preview: conflict.get_conflict_markers(),
            },
        ];

        let diff_view = conflict.get_conflict_markers();

        Self {
            conflict,
            resolution_options,
            diff_view,
            suggested_resolution: None,
        }
    }

    /// Get the conflict markers for display
    pub fn get_conflict_display(&self) -> String {
        self.diff_view.clone().unwrap_or_else(|| {
            format!(
                "Conflict in file: {}\nType: {}",
                self.conflict.file_path.display(),
                self.conflict.conflict_type
            )
        })
    }
}

/// Resolution option for manual conflict resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionOption {
    /// Resolution strategy
    pub strategy: ResolutionStrategy,
    /// Human-readable description
    pub description: String,
    /// Preview of what the resolution would look like
    pub preview: Option<String>,
}

/// Environment-specific resolution recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentResolutionRecommendation {
    /// Environment name
    pub environment: String,
    /// File path
    pub file_path: PathBuf,
    /// Recommended strategy
    pub recommended_strategy: ResolutionStrategy,
    /// Reasoning for the recommendation
    pub reasoning: String,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conflict_type_display() {
        assert_eq!(ConflictType::Content.to_string(), "content");
        assert_eq!(ConflictType::AddAdd.to_string(), "add/add");
        assert_eq!(ConflictType::DeleteModify.to_string(), "delete/modify");
    }

    #[test]
    fn test_resolution_strategy_display() {
        assert_eq!(ResolutionStrategy::UseOurs.to_string(), "use-ours");
        assert_eq!(ResolutionStrategy::UseTheirs.to_string(), "use-theirs");
        assert_eq!(ResolutionStrategy::AutoMerge.to_string(), "auto-merge");
        assert_eq!(ResolutionStrategy::Manual.to_string(), "manual");
    }

    #[test]
    fn test_conflict_info_creation() {
        let path = PathBuf::from("test.json");
        let conflict = ConflictInfo::new(path.clone(), ConflictType::Content);

        assert_eq!(conflict.file_path, path);
        assert_eq!(conflict.conflict_type, ConflictType::Content);
        assert!(!conflict.auto_resolvable);
        assert_eq!(conflict.suggested_strategy, ResolutionStrategy::Manual);
    }

    #[test]
    fn test_safe_auto_resolution() {
        let json_conflict = ConflictInfo::new(PathBuf::from("config.json"), ConflictType::AddAdd);
        let key_conflict = ConflictInfo::new(PathBuf::from("secret.key"), ConflictType::AddAdd);

        assert!(json_conflict.is_safe_for_auto_resolution());
        assert!(!key_conflict.is_safe_for_auto_resolution());
    }

    #[test]
    fn test_conflict_analysis() {
        let conflicts = vec![
            ConflictInfo::new(PathBuf::from("safe.json"), ConflictType::AddAdd),
            ConflictInfo::new(PathBuf::from("manual.key"), ConflictType::Content),
        ];

        let analysis = ConflictAnalysis::new(conflicts);

        assert_eq!(analysis.conflict_count, 2);
        assert_eq!(analysis.auto_resolvable_count, 1);
        assert_eq!(analysis.manual_resolution_count, 1);
        assert!(!analysis.can_auto_resolve);
    }
}
