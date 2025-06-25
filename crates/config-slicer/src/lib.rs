//! Configuration Slicing and Diffing Library
//!
//! This library provides tools for parsing network device configurations,
//! extracting specific sections (slices), and computing diffs between configurations.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

pub mod api;
pub mod diff;
pub mod error;
pub mod parser;
pub mod slicer;
pub mod streaming;
pub mod validation;

pub use api::{
    ConfigSlicerApi, ErrorSeverity, StreamingConfigProcessor, ValidationError, ValidationReport,
    ValidationWarning, WarningType,
};
pub use diff::{
    ApprovalInfo, ApprovalPriority, CacheConfig, CacheStats, CachedDiffResult,
    ColoredTerminalFormatter, ConflictResolver, ConflictType, DiffChange, DiffDisplay, DiffEngine,
    DiffFormatter, DiffOptions, DiffResult, DiffType, DiffWorkflowOrchestrator, DisplayOptions,
    HierarchicalDiff, HierarchicalDiffer, HtmlFormatter, Resolution, ResolutionStrategy,
    SemanticDiff, SemanticDiffer, SideBySideFormatter, TextDiff, TextDiffer, UnifiedFormatter,
    WorkflowExecution, WorkflowHistoryEntry, WorkflowStatus,
};
pub use error::{Error, ErrorCategory, ErrorContext, Result};
pub use streaming::{ConfigChunk, MemoryMonitor, StreamingConfig, StreamingProcessor};
pub use validation::{
    ConfigValidator, ValidationReport as ValidationReportDetail, ValidationRule,
    ValidationSeverity, ValidationSummary, ValidationViolation,
};
