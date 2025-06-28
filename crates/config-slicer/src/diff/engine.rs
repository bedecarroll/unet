//! Main diff engine orchestrating all diff algorithms
//!
//! This module provides the main `DiffEngine` that coordinates text-based,
//! hierarchical, and semantic diff algorithms to provide comprehensive
//! configuration comparison capabilities.

use crate::diff::algorithms::{HierarchicalDiffer, SemanticDiffer, TextDiffer};
use crate::diff::conflict::{ConflictResolver, ResolutionStrategy};
use crate::diff::types::{
    ChangeComplexity, DiffOptions, DiffResult, DiffSummary, HierarchicalDiff, RiskLevel,
    SemanticDiff, TextDiff,
};
use crate::parser::{HierarchicalParser, ParserConfig};
use anyhow::{Context, Result};
use tracing::{debug, info, warn};

/// Main diff engine that orchestrates all diff algorithms
pub struct DiffEngine {
    /// Options controlling diff behavior
    options: DiffOptions,
    /// Text-based differ
    text_differ: TextDiffer,
    /// Hierarchical differ
    hierarchical_differ: HierarchicalDiffer,
    /// Semantic differ
    semantic_differ: SemanticDiffer,
    /// Conflict resolver
    conflict_resolver: ConflictResolver,
    /// Parser for hierarchical analysis
    parser: HierarchicalParser,
}

impl DiffEngine {
    /// Create a new diff engine with default options
    pub fn new() -> Result<Self> {
        Self::with_options(DiffOptions::default())
    }

    /// Create a new diff engine with specified options
    pub fn with_options(options: DiffOptions) -> Result<Self> {
        let text_differ = TextDiffer::new(options.clone());
        let hierarchical_differ = HierarchicalDiffer::new(options.clone());
        let semantic_differ = SemanticDiffer::new(options.clone())?;
        let conflict_resolver = ConflictResolver::new(ResolutionStrategy::Manual);
        let parser = HierarchicalParser::new()?;

        Ok(Self {
            options,
            text_differ,
            hierarchical_differ,
            semantic_differ,
            conflict_resolver,
            parser,
        })
    }

    /// Set the conflict resolution strategy
    #[must_use]
    pub fn with_conflict_strategy(mut self, strategy: ResolutionStrategy) -> Self {
        self.conflict_resolver = ConflictResolver::new(strategy);
        self
    }

    /// Perform a comprehensive diff between two configurations
    pub fn diff(&self, old_config: &str, new_config: &str) -> Result<DiffResult> {
        info!("Starting comprehensive configuration diff");

        // Step 1: Text-based diff
        debug!("Performing text-based diff");
        let text_diff = self
            .text_differ
            .diff(old_config, new_config)
            .context("Failed to perform text diff")?;

        // Step 2: Hierarchical diff (if enabled)
        let hierarchical_diff = if self.should_perform_hierarchical_diff() {
            debug!("Performing hierarchical diff");
            self.perform_hierarchical_diff(old_config, new_config)?
        } else {
            debug!("Skipping hierarchical diff");
            HierarchicalDiff {
                sections: Vec::new(),
                structure_changes: Vec::new(),
                path_changes: std::collections::HashMap::new(),
            }
        };

        // Step 3: Semantic diff (if enabled)
        let semantic_diff = if self.options.include_semantic {
            debug!("Performing semantic diff");
            self.semantic_differ
                .diff(old_config, new_config, &text_diff)
                .context("Failed to perform semantic diff")?
        } else {
            debug!("Skipping semantic diff");
            SemanticDiff {
                functional_changes: Vec::new(),
                impact_analysis: Vec::new(),
                change_groups: Vec::new(),
            }
        };

        // Step 4: Calculate summary
        let summary = self.calculate_summary(&text_diff, &hierarchical_diff, &semantic_diff);

        let mut diff_result = DiffResult {
            text_diff,
            hierarchical_diff,
            semantic_diff,
            summary,
            options: self.options.clone(),
        };

        // Step 5: Conflict detection and resolution (if enabled)
        if self.should_detect_conflicts() {
            debug!("Detecting and resolving conflicts");
            self.handle_conflicts(&mut diff_result)?;
        }

        info!(
            "Diff analysis complete: {} total changes",
            diff_result.summary.total_changes
        );
        Ok(diff_result)
    }

    /// Perform a quick text-only diff for simple comparisons
    pub fn quick_diff(&self, old_config: &str, new_config: &str) -> Result<TextDiff> {
        debug!("Performing quick text-only diff");
        self.text_differ.diff(old_config, new_config)
    }

    /// Perform only hierarchical diff
    pub fn hierarchical_diff(
        &self,
        old_config: &str,
        new_config: &str,
    ) -> Result<HierarchicalDiff> {
        debug!("Performing hierarchical-only diff");
        self.perform_hierarchical_diff(old_config, new_config)
    }

    /// Perform only semantic diff
    pub fn semantic_diff(&self, old_config: &str, new_config: &str) -> Result<SemanticDiff> {
        debug!("Performing semantic-only diff");
        let text_diff = self.text_differ.diff(old_config, new_config)?;
        self.semantic_differ
            .diff(old_config, new_config, &text_diff)
    }

    /// Update diff options
    pub fn set_options(&mut self, options: DiffOptions) {
        self.options = options.clone();
        self.text_differ = TextDiffer::new(options.clone());
        self.hierarchical_differ = HierarchicalDiffer::new(options);
    }

    /// Get current diff options
    #[must_use]
    pub const fn options(&self) -> &DiffOptions {
        &self.options
    }

    const fn should_perform_hierarchical_diff(&self) -> bool {
        // For now, always perform hierarchical diff if we have the capability
        // In the future, this could be controlled by options
        true
    }

    const fn should_detect_conflicts(&self) -> bool {
        // Enable conflict detection for all diffs by default
        // This could be made configurable in the future
        true
    }

    fn perform_hierarchical_diff(
        &self,
        old_config: &str,
        new_config: &str,
    ) -> Result<HierarchicalDiff> {
        // Parse both configurations into trees
        let old_tree = self
            .parser
            .parse(old_config)
            .context("Failed to parse old configuration")?;
        let new_tree = self
            .parser
            .parse(new_config)
            .context("Failed to parse new configuration")?;

        // Compare the trees
        self.hierarchical_differ.diff(&old_tree, &new_tree)
    }

    fn calculate_summary(
        &self,
        text_diff: &TextDiff,
        hierarchical_diff: &HierarchicalDiff,
        semantic_diff: &SemanticDiff,
    ) -> DiffSummary {
        let total_changes = text_diff.additions + text_diff.deletions + text_diff.modifications;
        let sections_affected = hierarchical_diff.sections.len();

        // Determine highest risk level from semantic analysis
        let highest_risk = semantic_diff
            .impact_analysis
            .iter()
            .map(|impact| &impact.risk_level)
            .max()
            .cloned()
            .unwrap_or(RiskLevel::Low);

        // Assess complexity based on various factors
        let complexity = self.assess_complexity(total_changes, sections_affected, &highest_risk);

        DiffSummary {
            total_changes,
            additions: text_diff.additions,
            deletions: text_diff.deletions,
            modifications: text_diff.modifications,
            sections_affected,
            highest_risk,
            complexity,
        }
    }

    const fn assess_complexity(
        &self,
        total_changes: usize,
        sections_affected: usize,
        highest_risk: &RiskLevel,
    ) -> ChangeComplexity {
        // Simple heuristic for complexity assessment
        match (total_changes, sections_affected, highest_risk) {
            (0..=5, 0..=2, RiskLevel::Low) => ChangeComplexity::Simple,
            (6..=20, 3..=10, RiskLevel::Low | RiskLevel::Medium) => ChangeComplexity::Moderate,
            (21..=50, 11..=25, _) | (_, _, RiskLevel::High) => ChangeComplexity::Complex,
            _ => ChangeComplexity::VeryComplex,
        }
    }

    fn handle_conflicts(&self, diff_result: &mut DiffResult) -> Result<()> {
        // Detect conflicts
        let conflicts = self
            .conflict_resolver
            .detect_conflicts(diff_result)
            .context("Failed to detect conflicts")?;

        if conflicts.is_empty() {
            debug!("No conflicts detected");
            return Ok(());
        }

        warn!("Detected {} conflicts in diff", conflicts.len());

        // Resolve conflicts
        let resolutions = self
            .conflict_resolver
            .resolve_conflicts(diff_result, &conflicts)
            .context("Failed to resolve conflicts")?;

        // Store conflict information in metadata
        // In a more complete implementation, you might want to add conflict
        // information directly to the DiffResult structure
        debug!("Resolved {} conflicts", resolutions.len());

        Ok(())
    }
}

impl Default for DiffEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create default DiffEngine")
    }
}

/// Builder for creating a `DiffEngine` with custom configuration
pub struct DiffEngineBuilder {
    options: DiffOptions,
    resolution_strategy: ResolutionStrategy,
    parser_config: ParserConfig,
}

impl DiffEngineBuilder {
    /// Create a new builder with default settings
    #[must_use]
    pub fn new() -> Self {
        Self {
            options: DiffOptions::default(),
            resolution_strategy: ResolutionStrategy::Manual,
            parser_config: ParserConfig::default(),
        }
    }

    /// Set diff options
    #[must_use]
    pub const fn with_options(mut self, options: DiffOptions) -> Self {
        self.options = options;
        self
    }

    /// Set conflict resolution strategy
    #[must_use]
    pub const fn with_resolution_strategy(mut self, strategy: ResolutionStrategy) -> Self {
        self.resolution_strategy = strategy;
        self
    }

    /// Set parser configuration
    #[must_use]
    pub const fn with_parser_config(mut self, config: ParserConfig) -> Self {
        self.parser_config = config;
        self
    }

    /// Enable semantic analysis
    #[must_use]
    pub const fn enable_semantic(mut self) -> Self {
        self.options.include_semantic = true;
        self
    }

    /// Enable impact analysis
    #[must_use]
    pub const fn enable_impact_analysis(mut self) -> Self {
        self.options.include_impact = true;
        self
    }

    /// Set context lines
    #[must_use]
    pub const fn with_context_lines(mut self, lines: usize) -> Self {
        self.options.context_lines = lines;
        self
    }

    /// Ignore whitespace differences
    #[must_use]
    pub const fn ignore_whitespace(mut self) -> Self {
        self.options.ignore_whitespace = true;
        self
    }

    /// Build the `DiffEngine`
    pub fn build(self) -> Result<DiffEngine> {
        let text_differ = TextDiffer::new(self.options.clone());
        let hierarchical_differ = HierarchicalDiffer::new(self.options.clone());
        let semantic_differ = SemanticDiffer::new(self.options.clone())?;
        let conflict_resolver = ConflictResolver::new(self.resolution_strategy);
        let parser = HierarchicalParser::new()?;

        Ok(DiffEngine {
            options: self.options,
            text_differ,
            hierarchical_differ,
            semantic_differ,
            conflict_resolver,
            parser,
        })
    }
}

impl Default for DiffEngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_engine_creation() {
        let engine = DiffEngine::new().unwrap();
        assert_eq!(engine.options().context_lines, 3);
    }

    #[test]
    fn test_diff_engine_builder() {
        let engine = DiffEngineBuilder::new()
            .with_context_lines(5)
            .enable_semantic()
            .ignore_whitespace()
            .build()
            .unwrap();

        assert_eq!(engine.options().context_lines, 5);
        assert!(engine.options().include_semantic);
        assert!(engine.options().ignore_whitespace);
    }

    #[test]
    fn test_quick_diff() {
        let engine = DiffEngine::new().unwrap();
        let old_config = "interface GigabitEthernet0/1\n ip address 192.168.1.1 255.255.255.0";
        let new_config = "interface GigabitEthernet0/1\n ip address 192.168.1.2 255.255.255.0";

        let result = engine.quick_diff(old_config, new_config).unwrap();
        assert!(!result.changes.is_empty());
    }

    #[test]
    fn test_comprehensive_diff() {
        let engine = DiffEngine::new().unwrap();
        let old_config = "interface GigabitEthernet0/1\n ip address 192.168.1.1 255.255.255.0\n!\nvlan 100\n name production";
        let new_config = "interface GigabitEthernet0/1\n ip address 192.168.1.2 255.255.255.0\n!\nvlan 100\n name staging\nvlan 200\n name development";

        let result = engine.diff(old_config, new_config).unwrap();
        assert!(result.summary.total_changes > 0);
        assert!(!result.text_diff.changes.is_empty());
    }

    #[test]
    fn test_complexity_assessment() {
        let engine = DiffEngine::new().unwrap();

        // Test simple complexity
        let complexity = engine.assess_complexity(3, 1, &RiskLevel::Low);
        assert_eq!(complexity, ChangeComplexity::Simple);

        // Test complex complexity
        let complexity = engine.assess_complexity(30, 15, &RiskLevel::High);
        assert_eq!(complexity, ChangeComplexity::Complex);
    }
}
