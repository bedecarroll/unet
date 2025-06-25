//! Configuration diffing
//!
//! This module provides comprehensive diffing capabilities for network configurations,
//! supporting both text-based and structured hierarchical diffs.

pub mod algorithms;
pub mod analysis;
pub mod conflict;
pub mod display;
pub mod engine;
pub mod types;
pub mod workflow;

// Re-export main types and functions for ease of use
pub use algorithms::{HierarchicalDiffer, SemanticDiffer, TextDiffer};
pub use analysis::{
    AnalysisSummary, ChangeCategorization, ConfigSection, DiffAnalysis, DiffAnalyzer,
    DiffStatistics, NetworkFunction, OverallAssessment, Recommendation,
};
pub use conflict::{ConflictResolver, ConflictType, Resolution, ResolutionStrategy};
pub use display::{
    ColoredTerminalFormatter, DiffDisplay, DiffFormatter, DisplayOptions, HtmlFormatter,
    SideBySideFormatter, UnifiedFormatter,
};
pub use engine::DiffEngine;
pub use types::{
    DiffChange, DiffOptions, DiffResult, DiffSection, DiffType, HierarchicalDiff, SemanticDiff,
    TextDiff,
};
pub use workflow::{
    ApprovalInfo, ApprovalPriority, CacheConfig, CacheStats, CachedDiffResult,
    DiffWorkflowOrchestrator, WorkflowExecution, WorkflowHistoryEntry, WorkflowStatus,
};
