//! Manual conflict resolution tools and interfaces
//!
//! This module provides tools and interfaces for manual conflict resolution,
//! including interactive resolution tools, diff viewers, and merge tools integration.

use crate::git::conflict_resolution::{
    ConflictInfo, ConflictResolutionResult, ConflictType, ResolutionStrategy,
};
use crate::git::types::{GitError, GitResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tracing::{info, warn};

/// Configuration for external merge tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeToolConfig {
    /// Name of the merge tool
    pub name: String,
    /// Command to execute the merge tool
    pub command: String,
    /// Arguments to pass to the merge tool
    pub args: Vec<String>,
    /// Whether this tool supports three-way merge
    pub supports_three_way: bool,
    /// Whether this tool can be used in batch mode
    pub supports_batch: bool,
    /// File extensions this tool is optimized for
    pub file_extensions: Vec<String>,
}

impl Default for MergeToolConfig {
    fn default() -> Self {
        Self {
            name: "vimdiff".to_string(),
            command: "vim".to_string(),
            args: vec!["-d".to_string()],
            supports_three_way: true,
            supports_batch: false,
            file_extensions: vec!["*".to_string()],
        }
    }
}

impl MergeToolConfig {
    /// Create configuration for common merge tools
    pub fn vimdiff() -> Self {
        Self {
            name: "vimdiff".to_string(),
            command: "vim".to_string(),
            args: vec!["-d".to_string()],
            supports_three_way: true,
            supports_batch: false,
            file_extensions: vec!["*".to_string()],
        }
    }

    pub fn code() -> Self {
        Self {
            name: "vscode".to_string(),
            command: "code".to_string(),
            args: vec!["--wait".to_string(), "--diff".to_string()],
            supports_three_way: false,
            supports_batch: false,
            file_extensions: vec!["*".to_string()],
        }
    }

    pub fn meld() -> Self {
        Self {
            name: "meld".to_string(),
            command: "meld".to_string(),
            args: vec![],
            supports_three_way: true,
            supports_batch: false,
            file_extensions: vec!["*".to_string()],
        }
    }

    pub fn kdiff3() -> Self {
        Self {
            name: "kdiff3".to_string(),
            command: "kdiff3".to_string(),
            args: vec![],
            supports_three_way: true,
            supports_batch: true,
            file_extensions: vec!["*".to_string()],
        }
    }

    /// Check if this tool is available on the system
    pub fn is_available(&self) -> bool {
        Command::new(&self.command)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    /// Check if this tool supports the given file extension
    pub fn supports_file(&self, file_path: &Path) -> bool {
        if self.file_extensions.contains(&"*".to_string()) {
            return true;
        }

        if let Some(extension) = file_path.extension().and_then(|s| s.to_str()) {
            self.file_extensions
                .iter()
                .any(|ext| ext.eq_ignore_ascii_case(extension))
        } else {
            false
        }
    }
}

/// Interactive conflict resolution session
#[derive(Debug)]
pub struct ConflictResolutionSession {
    /// Conflicts to resolve
    conflicts: Vec<ConflictInfo>,
    /// Current conflict index
    current_index: usize,
    /// Resolution results
    results: Vec<ConflictResolutionResult>,
    /// Available merge tools
    merge_tools: Vec<MergeToolConfig>,
    /// Default merge tool
    default_tool: Option<String>,
}

impl ConflictResolutionSession {
    /// Create a new resolution session
    pub fn new(conflicts: Vec<ConflictInfo>) -> Self {
        let merge_tools = vec![
            MergeToolConfig::vimdiff(),
            MergeToolConfig::code(),
            MergeToolConfig::meld(),
            MergeToolConfig::kdiff3(),
        ];

        Self {
            conflicts,
            current_index: 0,
            results: Vec::new(),
            merge_tools,
            default_tool: None,
        }
    }

    /// Get the current conflict
    pub fn current_conflict(&self) -> Option<&ConflictInfo> {
        self.conflicts.get(self.current_index)
    }

    /// Move to the next conflict
    pub fn next_conflict(&mut self) -> Option<&ConflictInfo> {
        if self.current_index + 1 < self.conflicts.len() {
            self.current_index += 1;
            self.current_conflict()
        } else {
            None
        }
    }

    /// Move to the previous conflict
    pub fn previous_conflict(&mut self) -> Option<&ConflictInfo> {
        if self.current_index > 0 {
            self.current_index -= 1;
            self.current_conflict()
        } else {
            None
        }
    }

    /// Get session statistics
    pub fn get_statistics(&self) -> ConflictSessionStatistics {
        let total = self.conflicts.len();
        let resolved = self.results.len();
        let successful = self.results.iter().filter(|r| r.success).count();
        let failed = resolved - successful;

        ConflictSessionStatistics {
            total_conflicts: total,
            resolved_conflicts: resolved,
            successful_resolutions: successful,
            failed_resolutions: failed,
            remaining_conflicts: total - resolved,
            completion_percentage: if total > 0 {
                (resolved as f32 / total as f32) * 100.0
            } else {
                100.0
            },
        }
    }

    /// Get available merge tools for the current conflict
    pub fn get_available_tools(&self) -> Vec<&MergeToolConfig> {
        if let Some(conflict) = self.current_conflict() {
            self.merge_tools
                .iter()
                .filter(|tool| tool.is_available() && tool.supports_file(&conflict.file_path))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Set the default merge tool
    pub fn set_default_tool(&mut self, tool_name: &str) {
        if self.merge_tools.iter().any(|t| t.name == tool_name) {
            self.default_tool = Some(tool_name.to_string());
        }
    }

    /// Add a resolution result
    pub fn add_result(&mut self, result: ConflictResolutionResult) {
        self.results.push(result);
    }

    /// Check if all conflicts are resolved
    pub fn is_complete(&self) -> bool {
        self.results.len() >= self.conflicts.len()
    }

    /// Get all resolution results
    pub fn get_results(&self) -> &[ConflictResolutionResult] {
        &self.results
    }
}

/// Statistics for a conflict resolution session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictSessionStatistics {
    /// Total number of conflicts
    pub total_conflicts: usize,
    /// Number of resolved conflicts
    pub resolved_conflicts: usize,
    /// Number of successful resolutions
    pub successful_resolutions: usize,
    /// Number of failed resolutions
    pub failed_resolutions: usize,
    /// Number of remaining conflicts
    pub remaining_conflicts: usize,
    /// Completion percentage
    pub completion_percentage: f32,
}

/// Diff viewer for conflict analysis
#[derive(Debug)]
pub struct ConflictDiffViewer {
    /// Configuration for diff display
    config: DiffViewerConfig,
}

/// Configuration for diff viewer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffViewerConfig {
    /// Number of context lines to show
    pub context_lines: usize,
    /// Whether to show line numbers
    pub show_line_numbers: bool,
    /// Whether to highlight syntax
    pub syntax_highlighting: bool,
    /// Width for side-by-side diff
    pub side_by_side_width: usize,
    /// Color scheme for diff display
    pub color_scheme: DiffColorScheme,
}

impl Default for DiffViewerConfig {
    fn default() -> Self {
        Self {
            context_lines: 3,
            show_line_numbers: true,
            syntax_highlighting: false,
            side_by_side_width: 80,
            color_scheme: DiffColorScheme::default(),
        }
    }
}

/// Color scheme for diff display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffColorScheme {
    /// Color for added lines
    pub added_color: String,
    /// Color for removed lines
    pub removed_color: String,
    /// Color for context lines
    pub context_color: String,
    /// Color for conflict markers
    pub conflict_color: String,
}

impl Default for DiffColorScheme {
    fn default() -> Self {
        Self {
            added_color: "green".to_string(),
            removed_color: "red".to_string(),
            context_color: "white".to_string(),
            conflict_color: "yellow".to_string(),
        }
    }
}

impl ConflictDiffViewer {
    /// Create a new diff viewer
    pub fn new() -> Self {
        Self {
            config: DiffViewerConfig::default(),
        }
    }

    /// Create a diff viewer with custom configuration
    pub fn with_config(config: DiffViewerConfig) -> Self {
        Self { config }
    }

    /// Generate a unified diff view
    pub fn generate_unified_diff(&self, conflict: &ConflictInfo) -> GitResult<String> {
        if let (Some(ours), Some(theirs)) = (&conflict.our_content, &conflict.their_content) {
            let mut diff = String::new();

            diff.push_str(&format!(
                "--- a/{}\n+++ b/{}\n",
                conflict.file_path.display(),
                conflict.file_path.display()
            ));

            let our_lines: Vec<&str> = ours.lines().collect();
            let their_lines: Vec<&str> = theirs.lines().collect();

            // Simple unified diff generation
            diff.push_str(&format!(
                "@@ -1,{} +1,{} @@\n",
                our_lines.len(),
                their_lines.len()
            ));

            // Show our version with - prefix
            for line in our_lines {
                diff.push_str(&format!("-{}\n", line));
            }

            // Show their version with + prefix
            for line in their_lines {
                diff.push_str(&format!("+{}\n", line));
            }

            Ok(diff)
        } else {
            Err(GitError::RepositoryOperation {
                message: "Insufficient content for diff generation".to_string(),
            })
        }
    }

    /// Generate a side-by-side diff view
    pub fn generate_side_by_side_diff(&self, conflict: &ConflictInfo) -> GitResult<String> {
        if let (Some(ours), Some(theirs)) = (&conflict.our_content, &conflict.their_content) {
            let mut diff = String::new();
            let width = self.config.side_by_side_width;

            // Header
            diff.push_str(&format!(
                "{:width$} | {:width$}\n",
                "OURS (current branch)",
                "THEIRS (incoming branch)",
                width = width
            ));
            diff.push_str(&format!("{}\n", "-".repeat(width * 2 + 3)));

            let our_lines: Vec<&str> = ours.lines().collect();
            let their_lines: Vec<&str> = theirs.lines().collect();
            let max_lines = our_lines.len().max(their_lines.len());

            for i in 0..max_lines {
                let our_line = our_lines.get(i).unwrap_or(&"");
                let their_line = their_lines.get(i).unwrap_or(&"");

                diff.push_str(&format!(
                    "{:width$} | {:width$}\n",
                    our_line,
                    their_line,
                    width = width
                ));
            }

            Ok(diff)
        } else {
            Err(GitError::RepositoryOperation {
                message: "Insufficient content for diff generation".to_string(),
            })
        }
    }

    /// Generate a three-way diff view (base, ours, theirs)
    pub fn generate_three_way_diff(&self, conflict: &ConflictInfo) -> GitResult<String> {
        let mut diff = String::new();
        let width = self.config.side_by_side_width / 3;

        // Header
        diff.push_str(&format!(
            "{:width$} | {:width$} | {:width$}\n",
            "BASE (common)",
            "OURS (current)",
            "THEIRS (incoming)",
            width = width
        ));
        diff.push_str(&format!("{}\n", "-".repeat(width * 3 + 6)));

        let base_lines: Vec<&str> = conflict
            .base_content
            .as_ref()
            .map(|s| s.lines().collect())
            .unwrap_or_default();
        let our_lines: Vec<&str> = conflict
            .our_content
            .as_ref()
            .map(|s| s.lines().collect())
            .unwrap_or_default();
        let their_lines: Vec<&str> = conflict
            .their_content
            .as_ref()
            .map(|s| s.lines().collect())
            .unwrap_or_default();

        let max_lines = base_lines.len().max(our_lines.len()).max(their_lines.len());

        for i in 0..max_lines {
            let base_line = base_lines.get(i).unwrap_or(&"");
            let our_line = our_lines.get(i).unwrap_or(&"");
            let their_line = their_lines.get(i).unwrap_or(&"");

            diff.push_str(&format!(
                "{:width$} | {:width$} | {:width$}\n",
                base_line,
                our_line,
                their_line,
                width = width
            ));
        }

        Ok(diff)
    }
}

/// Merge tool integration manager
#[derive(Debug)]
pub struct MergeToolManager {
    /// Available merge tools
    tools: HashMap<String, MergeToolConfig>,
    /// Default tool preference order
    preference_order: Vec<String>,
}

impl MergeToolManager {
    /// Create a new merge tool manager
    pub fn new() -> Self {
        let mut tools = HashMap::new();

        // Add common merge tools
        tools.insert("vimdiff".to_string(), MergeToolConfig::vimdiff());
        tools.insert("vscode".to_string(), MergeToolConfig::code());
        tools.insert("meld".to_string(), MergeToolConfig::meld());
        tools.insert("kdiff3".to_string(), MergeToolConfig::kdiff3());

        let preference_order = vec![
            "vscode".to_string(),
            "meld".to_string(),
            "kdiff3".to_string(),
            "vimdiff".to_string(),
        ];

        Self {
            tools,
            preference_order,
        }
    }

    /// Add a custom merge tool
    pub fn add_tool(&mut self, tool: MergeToolConfig) {
        self.tools.insert(tool.name.clone(), tool);
    }

    /// Get available merge tools
    pub fn get_available_tools(&self) -> Vec<&MergeToolConfig> {
        self.tools
            .values()
            .filter(|tool| tool.is_available())
            .collect()
    }

    /// Get the best tool for a specific file
    pub fn get_best_tool_for_file(&self, file_path: &Path) -> Option<&MergeToolConfig> {
        for tool_name in &self.preference_order {
            if let Some(tool) = self.tools.get(tool_name) {
                if tool.is_available() && tool.supports_file(file_path) {
                    return Some(tool);
                }
            }
        }
        None
    }

    /// Launch external merge tool for conflict resolution
    pub fn launch_merge_tool(
        &self,
        tool_name: &str,
        conflict: &ConflictInfo,
    ) -> GitResult<MergeToolResult> {
        let tool = self
            .tools
            .get(tool_name)
            .ok_or_else(|| GitError::RepositoryOperation {
                message: format!("Merge tool '{}' not found", tool_name),
            })?;

        if !tool.is_available() {
            return Err(GitError::RepositoryOperation {
                message: format!("Merge tool '{}' is not available", tool_name),
            });
        }

        info!(
            "Launching merge tool '{}' for '{}'",
            tool_name,
            conflict.file_path.display()
        );

        // Create temporary files for the merge
        let temp_dir = std::env::temp_dir();
        let file_name = conflict
            .file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        let base_file = temp_dir.join(format!("base_{}", file_name));
        let our_file = temp_dir.join(format!("ours_{}", file_name));
        let their_file = temp_dir.join(format!("theirs_{}", file_name));
        let merged_file = temp_dir.join(format!("merged_{}", file_name));

        // Write temporary files
        if let Some(base_content) = &conflict.base_content {
            std::fs::write(&base_file, base_content).map_err(|e| {
                GitError::RepositoryOperation {
                    message: format!("Failed to write base file: {}", e),
                }
            })?;
        }

        if let Some(our_content) = &conflict.our_content {
            std::fs::write(&our_file, our_content).map_err(|e| GitError::RepositoryOperation {
                message: format!("Failed to write our file: {}", e),
            })?;
        }

        if let Some(their_content) = &conflict.their_content {
            std::fs::write(&their_file, their_content).map_err(|e| {
                GitError::RepositoryOperation {
                    message: format!("Failed to write their file: {}", e),
                }
            })?;
        }

        // Prepare command arguments
        let mut cmd = Command::new(&tool.command);
        cmd.args(&tool.args);

        if tool.supports_three_way {
            cmd.args(&[
                base_file.to_str().unwrap(),
                our_file.to_str().unwrap(),
                their_file.to_str().unwrap(),
                merged_file.to_str().unwrap(),
            ]);
        } else {
            cmd.args(&[our_file.to_str().unwrap(), their_file.to_str().unwrap()]);
        }

        // Execute the merge tool
        let status = cmd.status().map_err(|e| GitError::RepositoryOperation {
            message: format!("Failed to launch merge tool: {}", e),
        })?;

        let result = if status.success() {
            // Read the merged result if available
            let merged_content = if merged_file.exists() {
                Some(std::fs::read_to_string(&merged_file).map_err(|e| {
                    GitError::RepositoryOperation {
                        message: format!("Failed to read merged file: {}", e),
                    }
                })?)
            } else {
                None
            };

            MergeToolResult {
                success: true,
                merged_content,
                exit_code: status.code(),
                error: None,
            }
        } else {
            MergeToolResult {
                success: false,
                merged_content: None,
                exit_code: status.code(),
                error: Some(format!(
                    "Merge tool exited with non-zero status: {:?}",
                    status.code()
                )),
            }
        };

        // Clean up temporary files
        let _ = std::fs::remove_file(base_file);
        let _ = std::fs::remove_file(our_file);
        let _ = std::fs::remove_file(their_file);
        let _ = std::fs::remove_file(merged_file);

        Ok(result)
    }
}

impl Default for MergeToolManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Result from launching an external merge tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeToolResult {
    /// Whether the merge was successful
    pub success: bool,
    /// Merged content (if available)
    pub merged_content: Option<String>,
    /// Exit code from the merge tool
    pub exit_code: Option<i32>,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Conflict resolution assistant that provides guidance
#[derive(Debug)]
pub struct ConflictResolutionAssistant {
    /// Knowledge base for resolution strategies
    knowledge_base: ConflictKnowledgeBase,
}

/// Knowledge base for conflict resolution strategies
#[derive(Debug)]
pub struct ConflictKnowledgeBase {
    /// File type specific strategies
    file_type_strategies: HashMap<String, Vec<ResolutionStrategy>>,
    /// Common conflict patterns and solutions
    pattern_solutions: HashMap<String, String>,
}

impl Default for ConflictKnowledgeBase {
    fn default() -> Self {
        let mut file_type_strategies = HashMap::new();

        // JSON files
        file_type_strategies.insert(
            "json".to_string(),
            vec![ResolutionStrategy::AutoMerge, ResolutionStrategy::Manual],
        );

        // YAML files
        file_type_strategies.insert(
            "yaml".to_string(),
            vec![ResolutionStrategy::AutoMerge, ResolutionStrategy::Manual],
        );

        // Documentation files
        file_type_strategies.insert(
            "md".to_string(),
            vec![ResolutionStrategy::AutoMerge, ResolutionStrategy::UseTheirs],
        );

        // Configuration files
        file_type_strategies.insert(
            "conf".to_string(),
            vec![ResolutionStrategy::Manual, ResolutionStrategy::UseOurs],
        );

        let mut pattern_solutions = HashMap::new();
        pattern_solutions.insert(
            "version_conflict".to_string(),
            "Consider using the newer version or manually merging version numbers".to_string(),
        );
        pattern_solutions.insert(
            "dependency_conflict".to_string(),
            "Review dependency requirements and choose compatible versions".to_string(),
        );

        Self {
            file_type_strategies,
            pattern_solutions,
        }
    }
}

impl ConflictResolutionAssistant {
    /// Create a new resolution assistant
    pub fn new() -> Self {
        Self {
            knowledge_base: ConflictKnowledgeBase::default(),
        }
    }

    /// Get resolution suggestions for a conflict
    pub fn get_suggestions(&self, conflict: &ConflictInfo) -> Vec<ResolutionSuggestion> {
        let mut suggestions = Vec::new();

        // File type based suggestions
        if let Some(extension) = conflict.file_path.extension().and_then(|s| s.to_str()) {
            if let Some(strategies) = self.knowledge_base.file_type_strategies.get(extension) {
                for strategy in strategies {
                    suggestions.push(ResolutionSuggestion {
                        strategy: *strategy,
                        confidence: 0.8,
                        reasoning: format!(
                            "Recommended strategy for {} files",
                            extension.to_uppercase()
                        ),
                        steps: self.get_strategy_steps(*strategy, conflict),
                    });
                }
            }
        }

        // Pattern-based suggestions
        if self.detect_version_conflict(conflict) {
            suggestions.push(ResolutionSuggestion {
                strategy: ResolutionStrategy::Manual,
                confidence: 0.9,
                reasoning: "Version conflict detected - manual review recommended".to_string(),
                steps: vec![
                    "Compare version numbers".to_string(),
                    "Choose the appropriate version".to_string(),
                    "Update dependent configurations".to_string(),
                ],
            });
        }

        suggestions
    }

    /// Analyze conflict complexity
    pub fn analyze_complexity(&self, conflict: &ConflictInfo) -> ConflictComplexity {
        let mut score = 0.0;
        let mut factors = Vec::new();

        // File size factor
        if let Some(content) = &conflict.our_content {
            let lines = content.lines().count();
            if lines > 100 {
                score += 0.3;
                factors.push("Large file size".to_string());
            }
        }

        // Content similarity factor
        if let (Some(ours), Some(theirs)) = (&conflict.our_content, &conflict.their_content) {
            let similarity = self.calculate_similarity(ours, theirs);
            if similarity < 0.5 {
                score += 0.4;
                factors.push("Low content similarity".to_string());
            }
        }

        // Conflict type factor
        match conflict.conflict_type {
            ConflictType::Content => score += 0.2,
            ConflictType::RenameRename => score += 0.5,
            ConflictType::Binary => score += 0.8,
            _ => {}
        }

        let level = if score < 0.3 {
            ComplexityLevel::Low
        } else if score < 0.6 {
            ComplexityLevel::Medium
        } else {
            ComplexityLevel::High
        };

        ConflictComplexity::new(level, score, factors)
    }

    // Private helper methods

    fn get_strategy_steps(
        &self,
        strategy: ResolutionStrategy,
        conflict: &ConflictInfo,
    ) -> Vec<String> {
        match strategy {
            ResolutionStrategy::UseOurs => vec![
                "Select 'Use Ours' option".to_string(),
                "Verify the content is correct".to_string(),
                "Stage the resolved file".to_string(),
            ],
            ResolutionStrategy::UseTheirs => vec![
                "Select 'Use Theirs' option".to_string(),
                "Review the incoming changes".to_string(),
                "Stage the resolved file".to_string(),
            ],
            ResolutionStrategy::Manual => vec![
                "Open the file in a merge tool".to_string(),
                "Manually edit the conflict regions".to_string(),
                "Remove conflict markers".to_string(),
                "Test the resolved content".to_string(),
                "Stage the resolved file".to_string(),
            ],
            ResolutionStrategy::AutoMerge => {
                if conflict.auto_resolvable {
                    vec![
                        "Run automatic merge resolution".to_string(),
                        "Verify the result".to_string(),
                        "Stage the resolved file".to_string(),
                    ]
                } else {
                    vec!["Automatic merge not recommended for this conflict".to_string()]
                }
            }
            _ => vec!["Custom resolution steps required".to_string()],
        }
    }

    fn detect_version_conflict(&self, conflict: &ConflictInfo) -> bool {
        if let (Some(ours), Some(theirs)) = (&conflict.our_content, &conflict.their_content) {
            // Simple pattern matching for version conflicts
            let version_patterns = [
                r#""version""#,
                r#"version ="#,
                r#"version:"#,
                r#"<version>"#,
            ];

            for pattern in &version_patterns {
                if ours.contains(pattern) && theirs.contains(pattern) {
                    return true;
                }
            }
        }
        false
    }

    fn calculate_similarity(&self, text1: &str, text2: &str) -> f32 {
        // Simple similarity calculation based on common lines
        let lines1: Vec<&str> = text1.lines().collect();
        let lines2: Vec<&str> = text2.lines().collect();

        let common_lines = lines1.iter().filter(|line| lines2.contains(line)).count();
        let total_lines = lines1.len().max(lines2.len());

        if total_lines > 0 {
            common_lines as f32 / total_lines as f32
        } else {
            1.0
        }
    }
}

impl Default for ConflictResolutionAssistant {
    fn default() -> Self {
        Self::new()
    }
}

/// Resolution suggestion from the assistant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionSuggestion {
    /// Suggested strategy
    pub strategy: ResolutionStrategy,
    /// Confidence in this suggestion (0.0 to 1.0)
    pub confidence: f32,
    /// Reasoning for this suggestion
    pub reasoning: String,
    /// Step-by-step instructions
    pub steps: Vec<String>,
}

/// Conflict complexity analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictComplexity {
    /// Complexity level
    pub level: ComplexityLevel,
    /// Complexity score (0.0 to 1.0)
    pub score: f32,
    /// Factors contributing to complexity
    pub factors: Vec<String>,
}

impl ConflictComplexity {
    pub fn new(level: ComplexityLevel, score: f32, factors: Vec<String>) -> Self {
        Self {
            level,
            score,
            factors,
        }
    }
}

/// Complexity levels for conflicts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplexityLevel {
    /// Low complexity - can be easily resolved
    Low,
    /// Medium complexity - requires some attention
    Medium,
    /// High complexity - requires expert review
    High,
}

impl std::fmt::Display for ComplexityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComplexityLevel::Low => write!(f, "low"),
            ComplexityLevel::Medium => write!(f, "medium"),
            ComplexityLevel::High => write!(f, "high"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_tool_config() {
        let vimdiff = MergeToolConfig::vimdiff();
        assert_eq!(vimdiff.name, "vimdiff");
        assert!(vimdiff.supports_three_way);
        assert!(vimdiff.supports_file(&PathBuf::from("test.txt")));
    }

    #[test]
    fn test_conflict_session_statistics() {
        let conflicts = vec![
            ConflictInfo::new(PathBuf::from("file1.txt"), ConflictType::Content),
            ConflictInfo::new(PathBuf::from("file2.txt"), ConflictType::AddAdd),
        ];

        let mut session = ConflictResolutionSession::new(conflicts);
        let stats = session.get_statistics();

        assert_eq!(stats.total_conflicts, 2);
        assert_eq!(stats.resolved_conflicts, 0);
        assert_eq!(stats.completion_percentage, 0.0);

        // Add a resolution result
        session.add_result(ConflictResolutionResult {
            file_path: PathBuf::from("file1.txt"),
            success: true,
            strategy_used: ResolutionStrategy::UseOurs,
            resolved_content: Some("resolved".to_string()),
            error: None,
            requires_manual_intervention: false,
        });

        let updated_stats = session.get_statistics();
        assert_eq!(updated_stats.resolved_conflicts, 1);
        assert_eq!(updated_stats.completion_percentage, 50.0);
    }

    #[test]
    fn test_diff_viewer_config() {
        let config = DiffViewerConfig::default();
        assert_eq!(config.context_lines, 3);
        assert!(config.show_line_numbers);
        assert_eq!(config.side_by_side_width, 80);
    }

    #[test]
    fn test_complexity_analysis() {
        let assistant = ConflictResolutionAssistant::new();
        let conflict = ConflictInfo::new(PathBuf::from("test.json"), ConflictType::Content);

        let complexity = assistant.analyze_complexity(&conflict);
        assert!(matches!(
            complexity.level,
            ComplexityLevel::Low | ComplexityLevel::Medium
        ));
    }
}
