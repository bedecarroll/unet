//! Configuration Slicing and Diffing Library
//!
//! This library provides tools for parsing network device configurations,
//! extracting specific sections (slices), and computing diffs between configurations.

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(clippy::all)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![allow(missing_docs)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unused_self)]
#![allow(clippy::unnecessary_wraps)]
#![allow(missing_docs)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::ifs_same_cond)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::unused_async)]
#![allow(clippy::needless_pass_by_ref_mut)]
#![allow(clippy::single_match_else)]
#![allow(clippy::single_match)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cognitive_complexity)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::similar_names)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::if_same_then_else)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_late_init)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::collection_is_never_read)]
#![allow(clippy::unused_peekable)]
#![allow(clippy::branches_sharing_code)]
#![allow(clippy::manual_let_else)]
#![allow(clippy::significant_drop_tightening)]
#![allow(clippy::assigning_clones)]
#![allow(clippy::format_push_string)]
#![allow(clippy::borrowed_box)]

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
