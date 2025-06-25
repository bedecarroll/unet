//! Diff type definitions and data structures

use crate::parser::ConfigNode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Configuration diff result containing all diff types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiffResult {
    /// Text-based diff using line-by-line comparison
    pub text_diff: TextDiff,
    /// Hierarchical diff comparing configuration structure
    pub hierarchical_diff: HierarchicalDiff,
    /// Semantic diff understanding configuration meaning
    pub semantic_diff: SemanticDiff,
    /// Summary statistics
    pub summary: DiffSummary,
    /// Options used to generate this diff
    pub options: DiffOptions,
}

/// Text-based diff result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextDiff {
    /// Individual line changes
    pub changes: Vec<DiffChange>,
    /// Total number of additions
    pub additions: usize,
    /// Total number of deletions
    pub deletions: usize,
    /// Total number of modifications
    pub modifications: usize,
    /// Context lines around changes
    pub context_lines: usize,
}

/// Hierarchical diff result comparing configuration trees
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HierarchicalDiff {
    /// Tree-level changes (added/removed/modified sections)
    pub sections: Vec<DiffSection>,
    /// Configuration structure changes
    pub structure_changes: Vec<StructureChange>,
    /// Path-based changes for easy navigation
    pub path_changes: HashMap<String, DiffType>,
}

/// Semantic diff understanding configuration meaning
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticDiff {
    /// Functional changes (e.g., IP address changes, VLAN modifications)
    pub functional_changes: Vec<FunctionalChange>,
    /// Impact analysis of changes
    pub impact_analysis: Vec<ImpactAnalysis>,
    /// Related changes grouped together
    pub change_groups: Vec<ChangeGroup>,
}

/// Individual diff change for text comparison
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiffChange {
    /// Type of change
    pub change_type: DiffType,
    /// Old line content (None for additions)
    pub old_line: Option<String>,
    /// New line content (None for deletions)
    pub new_line: Option<String>,
    /// Line number in old configuration (None for additions)
    pub old_line_number: Option<usize>,
    /// Line number in new configuration (None for deletions)
    pub new_line_number: Option<usize>,
    /// Context information
    pub context: Option<String>,
}

/// Types of configuration changes
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiffType {
    /// Line was added in new configuration
    Addition,
    /// Line was removed from old configuration
    Deletion,
    /// Line was modified between configurations
    Modification,
    /// Line is identical (shown for context)
    Unchanged,
}

/// Configuration section-level changes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiffSection {
    /// Section path (e.g., "interface.GigabitEthernet0/1")
    pub path: String,
    /// Type of section change
    pub change_type: DiffType,
    /// Old section configuration (None for additions)
    pub old_section: Option<ConfigNode>,
    /// New section configuration (None for deletions)
    pub new_section: Option<ConfigNode>,
    /// Child changes within this section
    pub child_changes: Vec<DiffChange>,
}

/// Configuration structure changes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructureChange {
    /// Description of the structural change
    pub description: String,
    /// Path where the change occurred
    pub path: String,
    /// Severity of the structural change
    pub severity: ChangeSeverity,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Functional changes with semantic understanding
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionalChange {
    /// Type of functional change
    pub change_type: FunctionalChangeType,
    /// Description of the change
    pub description: String,
    /// Configuration path
    pub path: String,
    /// Old value
    pub old_value: Option<String>,
    /// New value
    pub new_value: Option<String>,
    /// Impact severity
    pub severity: ChangeSeverity,
}

/// Types of functional changes
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FunctionalChangeType {
    /// IP address or subnet change
    IpAddressing,
    /// VLAN configuration change
    VlanConfig,
    /// Interface state change (up/down, speed, duplex)
    InterfaceState,
    /// Routing protocol change
    Routing,
    /// Access control change
    AccessControl,
    /// Quality of Service change
    QualityOfService,
    /// Security configuration change
    Security,
    /// Other functional change
    Other(String),
}

/// Impact analysis of configuration changes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImpactAnalysis {
    /// Affected systems or components
    pub affected_components: Vec<String>,
    /// Risk level of the change
    pub risk_level: RiskLevel,
    /// Potential impact description
    pub impact_description: String,
    /// Recommended validation steps
    pub validation_steps: Vec<String>,
}

/// Risk levels for configuration changes
#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Low risk change
    Low,
    /// Medium risk change
    Medium,
    /// High risk change
    High,
    /// Critical risk change
    Critical,
}

/// Change severity levels
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChangeSeverity {
    /// Informational change
    Info,
    /// Warning level change
    Warning,
    /// Error level change
    Error,
    /// Critical change
    Critical,
}

/// Group of related changes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeGroup {
    /// Group identifier
    pub id: String,
    /// Group description
    pub description: String,
    /// Related changes in this group
    pub changes: Vec<String>, // Paths to changes
    /// Group type
    pub group_type: ChangeGroupType,
}

/// Types of change groups
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChangeGroupType {
    /// Interface configuration group
    Interface,
    /// VLAN configuration group
    Vlan,
    /// Routing configuration group
    Routing,
    /// Security configuration group
    Security,
    /// Custom group
    Custom(String),
}

/// Diff options to control comparison behavior
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiffOptions {
    /// Number of context lines to show around changes
    pub context_lines: usize,
    /// Whether to ignore whitespace differences
    pub ignore_whitespace: bool,
    /// Whether to ignore case differences
    pub ignore_case: bool,
    /// Whether to include semantic analysis
    pub include_semantic: bool,
    /// Whether to include impact analysis
    pub include_impact: bool,
    /// Whether to group related changes
    pub group_changes: bool,
    /// Minimum change severity to include
    pub min_severity: ChangeSeverity,
}

/// Summary statistics for a diff
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiffSummary {
    /// Total number of changes
    pub total_changes: usize,
    /// Number of additions
    pub additions: usize,
    /// Number of deletions
    pub deletions: usize,
    /// Number of modifications
    pub modifications: usize,
    /// Number of sections affected
    pub sections_affected: usize,
    /// Highest risk level found
    pub highest_risk: RiskLevel,
    /// Overall change complexity
    pub complexity: ChangeComplexity,
}

/// Change complexity assessment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChangeComplexity {
    /// Simple changes, low complexity
    Simple,
    /// Moderate complexity
    Moderate,
    /// Complex changes requiring careful review
    Complex,
    /// Very complex changes with high risk
    VeryComplex,
}

impl Default for DiffOptions {
    fn default() -> Self {
        Self {
            context_lines: 3,
            ignore_whitespace: false,
            ignore_case: false,
            include_semantic: true,
            include_impact: true,
            group_changes: true,
            min_severity: ChangeSeverity::Info,
        }
    }
}

impl fmt::Display for DiffType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Addition => write!(f, "addition"),
            Self::Deletion => write!(f, "deletion"),
            Self::Modification => write!(f, "modification"),
            Self::Unchanged => write!(f, "unchanged"),
        }
    }
}

impl fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

impl fmt::Display for ChangeSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Info => write!(f, "info"),
            Self::Warning => write!(f, "warning"),
            Self::Error => write!(f, "error"),
            Self::Critical => write!(f, "critical"),
        }
    }
}
