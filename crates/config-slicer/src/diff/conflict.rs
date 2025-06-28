//! Conflict resolution for configuration diffs
//!
//! This module provides conflict detection and resolution capabilities for
//! configuration changes that may have conflicts or require special handling.

use crate::diff::types::{
    ChangeSeverity, DiffChange, DiffResult, DiffSection, DiffType, FunctionalChangeType,
    HierarchicalDiff, SemanticDiff, TextDiff,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

/// Conflict resolver for handling configuration conflicts
pub struct ConflictResolver {
    /// Resolution strategy to use
    strategy: ResolutionStrategy,
    /// Custom resolution rules
    custom_rules: HashMap<String, ResolutionRule>,
}

/// Types of conflicts that can occur in configuration diffs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictType {
    /// Duplicate configuration commands
    Duplicate,
    /// Conflicting IP address assignments
    IpConflict,
    /// VLAN ID conflicts
    VlanConflict,
    /// Interface assignment conflicts
    InterfaceConflict,
    /// Routing protocol conflicts
    RoutingConflict,
    /// Overlapping access control rules
    AccessControlConflict,
    /// Resource allocation conflicts (bandwidth, queues, etc.)
    ResourceConflict,
    /// Dependency conflicts (missing dependencies)
    DependencyConflict,
    /// Custom conflict type
    Custom(String),
}

/// Resolution strategies for handling conflicts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionStrategy {
    /// Prefer the older configuration (conservative)
    PreferOld,
    /// Prefer the newer configuration (progressive)
    PreferNew,
    /// Merge configurations where possible
    Merge,
    /// Stop and require manual resolution
    Manual,
    /// Use custom resolution rules
    Custom,
}

/// Resolution result for a conflict
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resolution {
    /// Type of conflict that was resolved
    pub conflict_type: ConflictType,
    /// Resolution strategy used
    pub strategy: ResolutionStrategy,
    /// Resolved configuration lines
    pub resolved_lines: Vec<String>,
    /// Explanation of the resolution
    pub explanation: String,
    /// Whether manual review is recommended
    pub requires_review: bool,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Custom resolution rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionRule {
    /// Pattern to match against configuration lines
    pub pattern: String,
    /// Action to take when pattern matches
    pub action: ResolutionAction,
    /// Priority of this rule (higher = more important)
    pub priority: u32,
    /// Description of what this rule does
    pub description: String,
}

/// Actions that can be taken for conflict resolution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionAction {
    /// Keep the old configuration line
    KeepOld,
    /// Keep the new configuration line
    KeepNew,
    /// Merge both lines with a separator
    Merge(String),
    /// Replace with a custom line
    Replace(String),
    /// Remove the conflicting lines
    Remove,
    /// Mark for manual review
    Manual,
}

impl ConflictResolver {
    /// Create a new conflict resolver with the specified strategy
    #[must_use]
    pub fn new(strategy: ResolutionStrategy) -> Self {
        Self {
            strategy,
            custom_rules: HashMap::new(),
        }
    }

    /// Add a custom resolution rule
    pub fn add_rule(&mut self, name: String, rule: ResolutionRule) {
        self.custom_rules.insert(name, rule);
    }

    /// Detect conflicts in a diff result
    pub fn detect_conflicts(&self, diff_result: &DiffResult) -> Result<Vec<ConflictType>> {
        debug!("Detecting conflicts in diff result");
        let mut conflicts = Vec::new();

        // Check text diff for conflicts
        conflicts.extend(self.detect_text_conflicts(&diff_result.text_diff)?);

        // Check hierarchical diff for conflicts
        conflicts.extend(self.detect_hierarchical_conflicts(&diff_result.hierarchical_diff)?);

        // Check semantic diff for conflicts
        conflicts.extend(self.detect_semantic_conflicts(&diff_result.semantic_diff)?);

        Ok(conflicts)
    }

    /// Resolve conflicts in a diff result
    pub fn resolve_conflicts(
        &self,
        diff_result: &DiffResult,
        conflicts: &[ConflictType],
    ) -> Result<Vec<Resolution>> {
        debug!("Resolving {} conflicts", conflicts.len());
        let mut resolutions = Vec::new();

        for conflict in conflicts {
            let resolution = self.resolve_conflict(diff_result, conflict)?;
            resolutions.push(resolution);
        }

        Ok(resolutions)
    }

    fn detect_text_conflicts(&self, text_diff: &TextDiff) -> Result<Vec<ConflictType>> {
        let mut conflicts = Vec::new();
        let mut seen_commands = HashMap::new();

        for change in &text_diff.changes {
            if change.change_type == DiffType::Unchanged {
                continue;
            }

            // Check for duplicate commands
            if let Some(line) = &change.new_line {
                let command = self.extract_command(line);
                if let Some(existing_line) = seen_commands.get(&command) {
                    if existing_line != line {
                        conflicts.push(ConflictType::Duplicate);
                    }
                } else {
                    seen_commands.insert(command, line.clone());
                }
            }

            // Check for specific conflict types
            conflicts.extend(self.detect_line_conflicts(change)?);
        }

        Ok(conflicts)
    }

    fn detect_hierarchical_conflicts(
        &self,
        hierarchical_diff: &HierarchicalDiff,
    ) -> Result<Vec<ConflictType>> {
        let mut conflicts = Vec::new();

        for section in &hierarchical_diff.sections {
            // Check for interface conflicts
            if section.path.contains("interface") {
                conflicts.extend(self.detect_interface_conflicts(section)?);
            }

            // Check for VLAN conflicts
            if section.path.contains("vlan") {
                conflicts.push(ConflictType::VlanConflict);
            }

            // Check for routing conflicts
            if section.path.contains("router") || section.path.contains("routing") {
                conflicts.push(ConflictType::RoutingConflict);
            }
        }

        Ok(conflicts)
    }

    fn detect_semantic_conflicts(&self, semantic_diff: &SemanticDiff) -> Result<Vec<ConflictType>> {
        let mut conflicts = Vec::new();

        for change in &semantic_diff.functional_changes {
            let conflict_type = match change.change_type {
                FunctionalChangeType::IpAddressing => ConflictType::IpConflict,
                FunctionalChangeType::VlanConfig => ConflictType::VlanConflict,
                FunctionalChangeType::InterfaceState => ConflictType::InterfaceConflict,
                FunctionalChangeType::Routing => ConflictType::RoutingConflict,
                FunctionalChangeType::AccessControl => ConflictType::AccessControlConflict,
                FunctionalChangeType::QualityOfService => ConflictType::ResourceConflict,
                _ => continue,
            };

            // Only add conflicts for high-severity changes
            if matches!(
                change.severity,
                ChangeSeverity::Error | ChangeSeverity::Critical
            ) {
                conflicts.push(conflict_type);
            }
        }

        Ok(conflicts)
    }

    fn detect_line_conflicts(&self, change: &DiffChange) -> Result<Vec<ConflictType>> {
        let mut conflicts = Vec::new();

        if let Some(line) = &change.new_line {
            let line_lower = line.to_lowercase();

            // IP address conflicts
            if line_lower.contains("ip address") {
                conflicts.push(ConflictType::IpConflict);
            }

            // VLAN conflicts
            if line_lower.contains("vlan") {
                conflicts.push(ConflictType::VlanConflict);
            }

            // Interface conflicts
            if line_lower.starts_with("interface") {
                conflicts.push(ConflictType::InterfaceConflict);
            }

            // Access control conflicts
            if line_lower.contains("access-list")
                || line_lower.contains("permit")
                || line_lower.contains("deny")
            {
                conflicts.push(ConflictType::AccessControlConflict);
            }
        }

        Ok(conflicts)
    }

    fn detect_interface_conflicts(&self, section: &DiffSection) -> Result<Vec<ConflictType>> {
        let mut conflicts = Vec::new();

        // Check if interface is being configured in multiple incompatible ways
        if section.change_type == DiffType::Modification {
            if let (Some(old_section), Some(new_section)) =
                (&section.old_section, &section.new_section)
            {
                // Look for conflicting interface configurations
                let old_has_ip = self.has_ip_config(&old_section.raw_line);
                let new_has_ip = self.has_ip_config(&new_section.raw_line);

                if old_has_ip && new_has_ip {
                    conflicts.push(ConflictType::IpConflict);
                }
            }
        }

        Ok(conflicts)
    }

    fn resolve_conflict(
        &self,
        diff_result: &DiffResult,
        conflict: &ConflictType,
    ) -> Result<Resolution> {
        let resolution = match (&self.strategy, conflict) {
            (ResolutionStrategy::PreferOld, _) => self.resolve_prefer_old(diff_result, conflict)?,
            (ResolutionStrategy::PreferNew, _) => self.resolve_prefer_new(diff_result, conflict)?,
            (ResolutionStrategy::Merge, _) => self.resolve_merge(diff_result, conflict)?,
            (ResolutionStrategy::Manual, _) => self.resolve_manual(diff_result, conflict)?,
            (ResolutionStrategy::Custom, _) => self.resolve_custom(diff_result, conflict)?,
        };

        Ok(resolution)
    }

    fn resolve_prefer_old(
        &self,
        diff_result: &DiffResult,
        conflict: &ConflictType,
    ) -> Result<Resolution> {
        let resolved_lines = self.extract_old_lines(diff_result, conflict);

        Ok(Resolution {
            conflict_type: conflict.clone(),
            strategy: ResolutionStrategy::PreferOld,
            resolved_lines,
            explanation: "Conflict resolved by preferring the original configuration".to_string(),
            requires_review: false,
            metadata: HashMap::new(),
        })
    }

    fn resolve_prefer_new(
        &self,
        diff_result: &DiffResult,
        conflict: &ConflictType,
    ) -> Result<Resolution> {
        let resolved_lines = self.extract_new_lines(diff_result, conflict);

        Ok(Resolution {
            conflict_type: conflict.clone(),
            strategy: ResolutionStrategy::PreferNew,
            resolved_lines,
            explanation: "Conflict resolved by preferring the new configuration".to_string(),
            requires_review: false,
            metadata: HashMap::new(),
        })
    }

    fn resolve_merge(
        &self,
        diff_result: &DiffResult,
        conflict: &ConflictType,
    ) -> Result<Resolution> {
        let mut resolved_lines = Vec::new();
        let explanation: String;

        match conflict {
            ConflictType::Duplicate => {
                // For duplicates, keep only unique lines
                let old_lines = self.extract_old_lines(diff_result, conflict);
                let new_lines = self.extract_new_lines(diff_result, conflict);

                resolved_lines.extend(old_lines);
                for line in new_lines {
                    if !resolved_lines.contains(&line) {
                        resolved_lines.push(line);
                    }
                }
                explanation = "Duplicate lines merged by keeping unique entries".to_string();
            }
            ConflictType::IpConflict => {
                // For IP conflicts, this usually requires manual review
                explanation = "IP address conflict requires manual review".to_string();
                return Ok(Resolution {
                    conflict_type: conflict.clone(),
                    strategy: ResolutionStrategy::Manual,
                    resolved_lines: Vec::new(),
                    explanation,
                    requires_review: true,
                    metadata: HashMap::new(),
                });
            }
            _ => {
                // Default merge behavior
                resolved_lines.extend(self.extract_old_lines(diff_result, conflict));
                resolved_lines.extend(self.extract_new_lines(diff_result, conflict));
                explanation = "Conflict resolved by merging both configurations".to_string();
            }
        }

        Ok(Resolution {
            conflict_type: conflict.clone(),
            strategy: ResolutionStrategy::Merge,
            resolved_lines,
            explanation,
            requires_review: true,
            metadata: HashMap::new(),
        })
    }

    fn resolve_manual(
        &self,
        _diff_result: &DiffResult,
        conflict: &ConflictType,
    ) -> Result<Resolution> {
        Ok(Resolution {
            conflict_type: conflict.clone(),
            strategy: ResolutionStrategy::Manual,
            resolved_lines: Vec::new(),
            explanation: "Conflict marked for manual resolution".to_string(),
            requires_review: true,
            metadata: HashMap::new(),
        })
    }

    fn resolve_custom(
        &self,
        diff_result: &DiffResult,
        conflict: &ConflictType,
    ) -> Result<Resolution> {
        // Look for matching custom rules
        if let Some((name, rule)) = self.custom_rules.iter().next() {
            // This is a simplified custom resolution - in practice, you'd want
            // more sophisticated pattern matching
            let resolution = match rule.action {
                ResolutionAction::KeepOld => self.resolve_prefer_old(diff_result, conflict)?,
                ResolutionAction::KeepNew => self.resolve_prefer_new(diff_result, conflict)?,
                ResolutionAction::Manual => self.resolve_manual(diff_result, conflict)?,
                _ => {
                    // For other actions, create a custom resolution
                    Resolution {
                        conflict_type: conflict.clone(),
                        strategy: ResolutionStrategy::Custom,
                        resolved_lines: Vec::new(),
                        explanation: format!(
                            "Custom rule '{}' applied: {}",
                            name, rule.description
                        ),
                        requires_review: true,
                        metadata: HashMap::new(),
                    }
                }
            };
            return Ok(resolution);
        }

        // If no custom rule matches, fall back to manual
        self.resolve_manual(diff_result, conflict)
    }

    fn extract_command(&self, line: &str) -> String {
        // Extract the main command from a configuration line
        line.split_whitespace().next().unwrap_or("").to_string()
    }

    fn extract_old_lines(&self, diff_result: &DiffResult, _conflict: &ConflictType) -> Vec<String> {
        diff_result
            .text_diff
            .changes
            .iter()
            .filter_map(|change| change.old_line.clone())
            .collect()
    }

    fn extract_new_lines(&self, diff_result: &DiffResult, _conflict: &ConflictType) -> Vec<String> {
        diff_result
            .text_diff
            .changes
            .iter()
            .filter_map(|change| change.new_line.clone())
            .collect()
    }

    fn has_ip_config(&self, line: &str) -> bool {
        line.to_lowercase().contains("ip address")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::types::*;

    #[test]
    fn test_conflict_detection() {
        let resolver = ConflictResolver::new(ResolutionStrategy::Manual);

        let text_diff = TextDiff {
            changes: vec![DiffChange {
                change_type: DiffType::Addition,
                old_line: None,
                new_line: Some("ip address 192.168.1.1 255.255.255.0".to_string()),
                old_line_number: None,
                new_line_number: Some(1),
                context: None,
            }],
            additions: 1,
            deletions: 0,
            modifications: 0,
            context_lines: 3,
        };

        let conflicts = resolver.detect_text_conflicts(&text_diff).unwrap();
        assert!(conflicts.contains(&ConflictType::IpConflict));
    }

    #[test]
    fn test_prefer_new_resolution() {
        let resolver = ConflictResolver::new(ResolutionStrategy::PreferNew);

        let diff_result = DiffResult {
            text_diff: TextDiff {
                changes: vec![DiffChange {
                    change_type: DiffType::Modification,
                    old_line: Some("old config".to_string()),
                    new_line: Some("new config".to_string()),
                    old_line_number: Some(1),
                    new_line_number: Some(1),
                    context: None,
                }],
                additions: 0,
                deletions: 0,
                modifications: 1,
                context_lines: 3,
            },
            hierarchical_diff: HierarchicalDiff {
                sections: Vec::new(),
                structure_changes: Vec::new(),
                path_changes: HashMap::new(),
            },
            semantic_diff: SemanticDiff {
                functional_changes: Vec::new(),
                impact_analysis: Vec::new(),
                change_groups: Vec::new(),
            },
            summary: DiffSummary {
                total_changes: 1,
                additions: 0,
                deletions: 0,
                modifications: 1,
                sections_affected: 1,
                highest_risk: RiskLevel::Low,
                complexity: ChangeComplexity::Simple,
            },
            options: DiffOptions::default(),
        };

        let resolution = resolver
            .resolve_prefer_new(&diff_result, &ConflictType::Duplicate)
            .unwrap();
        assert_eq!(resolution.strategy, ResolutionStrategy::PreferNew);
        assert!(!resolution.resolved_lines.is_empty());
    }
}
