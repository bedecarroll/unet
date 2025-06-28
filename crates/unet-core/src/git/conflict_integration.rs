//! Integrated conflict resolution system
//!
//! This module provides a high-level API that integrates all conflict resolution
//! components including detection, manual tools, automated resolution, and workflow management.

use crate::git::conflict_resolution::{
    ConflictAnalysis, ConflictInfo, ConflictResolutionConfig, ConflictResolutionResult,
    ConflictResolver, ResolutionStrategy,
};
use crate::git::conflict_tools::{
    ConflictDiffViewer, ConflictResolutionAssistant, ConflictResolutionSession, MergeToolManager,
    MergeToolResult,
};
use crate::git::environment::EnvironmentConfig;
use crate::git::repository::GitRepository;
use crate::git::types::{GitError, GitResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{info, warn};
use uuid::Uuid;

/// Comprehensive conflict resolution service
pub struct ConflictResolutionService {
    /// Core conflict resolver
    resolver: ConflictResolver,
    /// Diff viewer for conflict analysis
    diff_viewer: ConflictDiffViewer,
    /// Resolution assistant for guidance
    assistant: ConflictResolutionAssistant,
    /// Merge tool manager
    merge_tool_manager: MergeToolManager,
    /// Active resolution sessions
    active_sessions: HashMap<String, ConflictResolutionSession>,
    /// Service configuration
    config: ConflictServiceConfig,
}

/// Configuration for the conflict resolution service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictServiceConfig {
    /// Whether to enable automatic resolution
    pub enable_auto_resolution: bool,
    /// Whether to create backup files before resolution
    pub create_backups: bool,
    /// Default merge tool preference
    pub default_merge_tool: Option<String>,
    /// Maximum number of active sessions
    pub max_active_sessions: usize,
    /// Session timeout in seconds
    pub session_timeout_seconds: u64,
    /// Whether to provide resolution suggestions
    pub enable_suggestions: bool,
}

impl Default for ConflictServiceConfig {
    fn default() -> Self {
        Self {
            enable_auto_resolution: true,
            create_backups: true,
            default_merge_tool: None,
            max_active_sessions: 10,
            session_timeout_seconds: 3600, // 1 hour
            enable_suggestions: true,
        }
    }
}

impl ConflictResolutionService {
    /// Create a new conflict resolution service
    pub fn new(repository: GitRepository) -> Self {
        let config = ConflictServiceConfig::default();
        let resolver_config = ConflictResolutionConfig {
            enable_auto_resolution: config.enable_auto_resolution,
            create_backups: config.create_backups,
            ..Default::default()
        };

        Self {
            resolver: ConflictResolver::with_config(repository, resolver_config),
            diff_viewer: ConflictDiffViewer::new(),
            assistant: ConflictResolutionAssistant::new(),
            merge_tool_manager: MergeToolManager::new(),
            active_sessions: HashMap::new(),
            config,
        }
    }

    /// Create a new service with custom configuration
    pub fn with_config(repository: GitRepository, config: ConflictServiceConfig) -> Self {
        let resolver_config = ConflictResolutionConfig {
            enable_auto_resolution: config.enable_auto_resolution,
            create_backups: config.create_backups,
            ..Default::default()
        };

        Self {
            resolver: ConflictResolver::with_config(repository, resolver_config),
            diff_viewer: ConflictDiffViewer::new(),
            assistant: ConflictResolutionAssistant::new(),
            merge_tool_manager: MergeToolManager::new(),
            active_sessions: HashMap::new(),
            config,
        }
    }

    /// Configure environments for context-aware resolution
    pub fn configure_environments(&mut self, environments: Vec<EnvironmentConfig>) {
        self.resolver.configure_environments(environments);
    }

    /// Start a comprehensive conflict resolution workflow
    pub fn start_resolution_workflow(&mut self) -> GitResult<ConflictResolutionWorkflow> {
        info!("Starting conflict resolution workflow");

        // Detect conflicts
        let analysis = self.resolver.detect_conflicts()?;

        if analysis.conflict_count == 0 {
            info!("No conflicts detected");
            return Ok(ConflictResolutionWorkflow {
                session_id: "no-conflicts".to_string(),
                status: WorkflowStatus::NoConflicts,
                analysis,
                suggestions: Vec::new(),
                progress: WorkflowProgress::completed(),
            });
        }

        // Create a new resolution session
        let session_id = self.create_session(analysis.conflicts.clone())?;

        // Generate suggestions for all conflicts
        let suggestions = if self.config.enable_suggestions {
            self.generate_suggestions(&analysis.conflicts)?
        } else {
            Vec::new()
        };

        // Attempt automatic resolution for eligible conflicts
        let auto_results = if self.config.enable_auto_resolution {
            self.resolver.auto_resolve_conflicts()?
        } else {
            Vec::new()
        };

        // Update session with auto-resolution results
        if let Some(session) = self.active_sessions.get_mut(&session_id) {
            for result in auto_results {
                session.add_result(result);
            }
        }

        let status = if analysis.can_auto_resolve {
            WorkflowStatus::AutoResolved
        } else {
            WorkflowStatus::AwaitingManualResolution
        };

        let progress = self.calculate_progress(&session_id)?;

        Ok(ConflictResolutionWorkflow {
            session_id,
            status,
            analysis,
            suggestions,
            progress,
        })
    }

    /// Get resolution suggestions for a specific conflict
    pub fn get_conflict_suggestions(&self, file_path: &Path) -> GitResult<Vec<ConflictSuggestion>> {
        let analysis = self.resolver.detect_conflicts()?;
        let conflict = analysis
            .conflicts
            .iter()
            .find(|c| c.file_path == file_path)
            .ok_or_else(|| GitError::RepositoryOperation {
                message: format!("No conflict found for file: {}", file_path.display()),
            })?;

        let assistant_suggestions = self.assistant.get_suggestions(conflict);
        let complexity = self.assistant.analyze_complexity(conflict);

        let mut suggestions = Vec::new();
        for suggestion in assistant_suggestions {
            suggestions.push(ConflictSuggestion {
                strategy: suggestion.strategy,
                confidence: suggestion.confidence,
                reasoning: suggestion.reasoning,
                steps: suggestion.steps,
                complexity: complexity.clone(),
                estimated_time_minutes: self
                    .estimate_resolution_time(&suggestion.strategy, &complexity),
            });
        }

        Ok(suggestions)
    }

    /// Generate diff view for a conflict
    pub fn get_conflict_diff(&self, file_path: &Path, diff_type: DiffType) -> GitResult<String> {
        let analysis = self.resolver.detect_conflicts()?;
        let conflict = analysis
            .conflicts
            .iter()
            .find(|c| c.file_path == file_path)
            .ok_or_else(|| GitError::RepositoryOperation {
                message: format!("No conflict found for file: {}", file_path.display()),
            })?;

        match diff_type {
            DiffType::Unified => self.diff_viewer.generate_unified_diff(conflict),
            DiffType::SideBySide => self.diff_viewer.generate_side_by_side_diff(conflict),
            DiffType::ThreeWay => self.diff_viewer.generate_three_way_diff(conflict),
        }
    }

    /// Launch external merge tool for a conflict
    pub fn launch_merge_tool(
        &self,
        file_path: &Path,
        tool_name: Option<&str>,
    ) -> GitResult<MergeToolResult> {
        let analysis = self.resolver.detect_conflicts()?;
        let conflict = analysis
            .conflicts
            .iter()
            .find(|c| c.file_path == file_path)
            .ok_or_else(|| GitError::RepositoryOperation {
                message: format!("No conflict found for file: {}", file_path.display()),
            })?;

        let tool_to_use = if let Some(tool) = tool_name {
            tool.to_string()
        } else if let Some(default_tool) = &self.config.default_merge_tool {
            default_tool.clone()
        } else {
            // Find the best available tool
            self.merge_tool_manager
                .get_best_tool_for_file(&conflict.file_path)
                .map(|tool| tool.name.clone())
                .ok_or_else(|| GitError::RepositoryOperation {
                    message: "No suitable merge tool found".to_string(),
                })?
        };

        self.merge_tool_manager
            .launch_merge_tool(&tool_to_use, conflict)
    }

    /// Resolve a conflict with a specific strategy
    pub fn resolve_conflict(
        &mut self,
        session_id: &str,
        file_path: &Path,
        strategy: ResolutionStrategy,
    ) -> GitResult<ConflictResolutionResult> {
        info!(
            "Resolving conflict in session '{}' for file '{}' with strategy '{}'",
            session_id,
            file_path.display(),
            strategy
        );

        let result = self.resolver.resolve_conflict(file_path, strategy)?;

        // Update the session with the result
        if let Some(session) = self.active_sessions.get_mut(session_id) {
            session.add_result(result.clone());
        }

        Ok(result)
    }

    /// Apply manual resolution content to a file
    pub fn apply_manual_resolution(
        &mut self,
        session_id: &str,
        file_path: &Path,
        resolved_content: &str,
    ) -> GitResult<ConflictResolutionResult> {
        info!(
            "Applying manual resolution in session '{}' for file '{}'",
            session_id,
            file_path.display()
        );

        let result = self
            .resolver
            .apply_resolution(file_path, resolved_content)?;

        // Update the session with the result
        if let Some(session) = self.active_sessions.get_mut(session_id) {
            session.add_result(result.clone());
        }

        Ok(result)
    }

    /// Get the status of a resolution session
    pub fn get_session_status(&self, session_id: &str) -> Option<ConflictSessionStatus> {
        self.active_sessions.get(session_id).map(|session| {
            let stats = session.get_statistics();
            let current_conflict = session.current_conflict().cloned();

            ConflictSessionStatus {
                session_id: session_id.to_string(),
                statistics: stats,
                current_conflict,
                is_complete: session.is_complete(),
                available_tools: session.get_available_tools().into_iter().cloned().collect(),
            }
        })
    }

    /// Get all available merge tools
    pub fn get_available_merge_tools(&self) -> Vec<String> {
        self.merge_tool_manager
            .get_available_tools()
            .into_iter()
            .map(|tool| tool.name.clone())
            .collect()
    }

    /// Check if all conflicts are resolved
    pub fn are_all_conflicts_resolved(&self) -> GitResult<bool> {
        self.resolver.are_all_conflicts_resolved()
    }

    /// Complete a resolution session
    pub fn complete_session(&mut self, session_id: &str) -> GitResult<ConflictResolutionSummary> {
        let session = self.active_sessions.remove(session_id).ok_or_else(|| {
            GitError::RepositoryOperation {
                message: format!("Session '{}' not found", session_id),
            }
        })?;

        let stats = session.get_statistics();
        let results = session.get_results().to_vec();

        let successful_resolutions = results.iter().filter(|r| r.success).count();
        let failed_resolutions = results.len() - successful_resolutions;

        info!(
            "Completed session '{}': {} successful, {} failed",
            session_id, successful_resolutions, failed_resolutions
        );

        Ok(ConflictResolutionSummary {
            session_id: session_id.to_string(),
            total_conflicts: stats.total_conflicts,
            successful_resolutions,
            failed_resolutions,
            resolution_results: results,
        })
    }

    /// Cancel a resolution session
    pub fn cancel_session(&mut self, session_id: &str) -> GitResult<()> {
        if self.active_sessions.remove(session_id).is_some() {
            info!("Cancelled resolution session: {}", session_id);
            Ok(())
        } else {
            Err(GitError::RepositoryOperation {
                message: format!("Session '{}' not found", session_id),
            })
        }
    }

    /// Get list of active sessions
    pub fn get_active_sessions(&self) -> Vec<String> {
        self.active_sessions.keys().cloned().collect()
    }

    // Private helper methods

    fn create_session(&mut self, conflicts: Vec<ConflictInfo>) -> GitResult<String> {
        // Clean up old sessions if we're at the limit
        if self.active_sessions.len() >= self.config.max_active_sessions {
            self.cleanup_old_sessions();
        }

        let session_id = format!("session-{}", Uuid::new_v4());
        let session = ConflictResolutionSession::new(conflicts);

        self.active_sessions.insert(session_id.clone(), session);
        info!("Created resolution session: {}", session_id);

        Ok(session_id)
    }

    fn cleanup_old_sessions(&mut self) {
        // In a real implementation, you'd track session creation times and remove old ones
        // For now, just remove one arbitrary session
        if let Some(old_session_id) = self.active_sessions.keys().next().cloned() {
            self.active_sessions.remove(&old_session_id);
            warn!("Removed old session to make room: {}", old_session_id);
        }
    }

    fn generate_suggestions(
        &self,
        conflicts: &[ConflictInfo],
    ) -> GitResult<Vec<ConflictSuggestion>> {
        let mut all_suggestions = Vec::new();

        for conflict in conflicts {
            let assistant_suggestions = self.assistant.get_suggestions(conflict);
            let complexity = self.assistant.analyze_complexity(conflict);

            for suggestion in assistant_suggestions {
                all_suggestions.push(ConflictSuggestion {
                    strategy: suggestion.strategy,
                    confidence: suggestion.confidence,
                    reasoning: suggestion.reasoning,
                    steps: suggestion.steps,
                    complexity: complexity.clone(),
                    estimated_time_minutes: self
                        .estimate_resolution_time(&suggestion.strategy, &complexity),
                });
            }
        }

        Ok(all_suggestions)
    }

    fn estimate_resolution_time(
        &self,
        strategy: &ResolutionStrategy,
        complexity: &crate::git::conflict_tools::ConflictComplexity,
    ) -> u32 {
        let base_time = match strategy {
            ResolutionStrategy::UseOurs | ResolutionStrategy::UseTheirs => 1,
            ResolutionStrategy::AutoMerge => 2,
            ResolutionStrategy::Manual => 10,
            ResolutionStrategy::Custom => 15,
            ResolutionStrategy::Abort => 0,
        };

        let complexity_multiplier = match complexity.level {
            crate::git::conflict_tools::ComplexityLevel::Low => 1.0,
            crate::git::conflict_tools::ComplexityLevel::Medium => 2.0,
            crate::git::conflict_tools::ComplexityLevel::High => 3.0,
        };

        (base_time as f32 * complexity_multiplier) as u32
    }

    fn calculate_progress(&self, session_id: &str) -> GitResult<WorkflowProgress> {
        if let Some(session) = self.active_sessions.get(session_id) {
            let stats = session.get_statistics();
            Ok(WorkflowProgress {
                total_conflicts: stats.total_conflicts,
                resolved_conflicts: stats.resolved_conflicts,
                successful_resolutions: stats.successful_resolutions,
                failed_resolutions: stats.failed_resolutions,
                completion_percentage: stats.completion_percentage,
            })
        } else {
            Err(GitError::RepositoryOperation {
                message: format!("Session '{}' not found", session_id),
            })
        }
    }
}

/// Complete conflict resolution workflow result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolutionWorkflow {
    /// Session ID for tracking the workflow
    pub session_id: String,
    /// Current workflow status
    pub status: WorkflowStatus,
    /// Conflict analysis results
    pub analysis: ConflictAnalysis,
    /// Resolution suggestions
    pub suggestions: Vec<ConflictSuggestion>,
    /// Progress information
    pub progress: WorkflowProgress,
}

/// Workflow status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowStatus {
    /// No conflicts detected
    NoConflicts,
    /// All conflicts auto-resolved
    AutoResolved,
    /// Awaiting manual resolution
    AwaitingManualResolution,
    /// Resolution in progress
    InProgress,
    /// All conflicts resolved
    Completed,
    /// Workflow cancelled
    Cancelled,
}

impl std::fmt::Display for WorkflowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkflowStatus::NoConflicts => write!(f, "no-conflicts"),
            WorkflowStatus::AutoResolved => write!(f, "auto-resolved"),
            WorkflowStatus::AwaitingManualResolution => write!(f, "awaiting-manual"),
            WorkflowStatus::InProgress => write!(f, "in-progress"),
            WorkflowStatus::Completed => write!(f, "completed"),
            WorkflowStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Enhanced conflict suggestion with additional context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictSuggestion {
    /// Suggested resolution strategy
    pub strategy: ResolutionStrategy,
    /// Confidence in this suggestion (0.0 to 1.0)
    pub confidence: f32,
    /// Reasoning for the suggestion
    pub reasoning: String,
    /// Step-by-step instructions
    pub steps: Vec<String>,
    /// Conflict complexity analysis
    pub complexity: crate::git::conflict_tools::ConflictComplexity,
    /// Estimated time to resolve in minutes
    pub estimated_time_minutes: u32,
}

/// Progress tracking for conflict resolution workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowProgress {
    /// Total number of conflicts
    pub total_conflicts: usize,
    /// Number of resolved conflicts
    pub resolved_conflicts: usize,
    /// Number of successful resolutions
    pub successful_resolutions: usize,
    /// Number of failed resolutions
    pub failed_resolutions: usize,
    /// Completion percentage
    pub completion_percentage: f32,
}

impl WorkflowProgress {
    /// Create a completed progress indicator
    pub fn completed() -> Self {
        Self {
            total_conflicts: 0,
            resolved_conflicts: 0,
            successful_resolutions: 0,
            failed_resolutions: 0,
            completion_percentage: 100.0,
        }
    }
}

/// Status of a conflict resolution session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictSessionStatus {
    /// Session ID
    pub session_id: String,
    /// Session statistics
    pub statistics: crate::git::conflict_tools::ConflictSessionStatistics,
    /// Current conflict being worked on
    pub current_conflict: Option<ConflictInfo>,
    /// Whether the session is complete
    pub is_complete: bool,
    /// Available merge tools
    pub available_tools: Vec<crate::git::conflict_tools::MergeToolConfig>,
}

/// Summary of a completed resolution session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolutionSummary {
    /// Session ID
    pub session_id: String,
    /// Total number of conflicts
    pub total_conflicts: usize,
    /// Number of successful resolutions
    pub successful_resolutions: usize,
    /// Number of failed resolutions
    pub failed_resolutions: usize,
    /// Detailed resolution results
    pub resolution_results: Vec<ConflictResolutionResult>,
}

/// Types of diff views available
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffType {
    /// Unified diff format
    Unified,
    /// Side-by-side diff format
    SideBySide,
    /// Three-way diff (base, ours, theirs)
    ThreeWay,
}

impl std::fmt::Display for DiffType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiffType::Unified => write!(f, "unified"),
            DiffType::SideBySide => write!(f, "side-by-side"),
            DiffType::ThreeWay => write!(f, "three-way"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::conflict_resolution::ConflictType as GitConflictType;

    #[test]
    fn test_workflow_status_display() {
        assert_eq!(WorkflowStatus::NoConflicts.to_string(), "no-conflicts");
        assert_eq!(WorkflowStatus::AutoResolved.to_string(), "auto-resolved");
        assert_eq!(
            WorkflowStatus::AwaitingManualResolution.to_string(),
            "awaiting-manual"
        );
    }

    #[test]
    fn test_diff_type_display() {
        assert_eq!(DiffType::Unified.to_string(), "unified");
        assert_eq!(DiffType::SideBySide.to_string(), "side-by-side");
        assert_eq!(DiffType::ThreeWay.to_string(), "three-way");
    }

    #[test]
    fn test_workflow_progress_completed() {
        let progress = WorkflowProgress::completed();
        assert_eq!(progress.completion_percentage, 100.0);
        assert_eq!(progress.total_conflicts, 0);
    }

    #[test]
    fn test_service_config_default() {
        let config = ConflictServiceConfig::default();
        assert!(config.enable_auto_resolution);
        assert!(config.create_backups);
        assert_eq!(config.max_active_sessions, 10);
        assert_eq!(config.session_timeout_seconds, 3600);
    }
}
