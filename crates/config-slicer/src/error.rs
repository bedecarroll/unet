//! Error types for config-slicer

use thiserror::Error;

/// Config-slicer error type
#[derive(Error, Debug)]
pub enum Error {
    /// Parsing error with detailed context
    #[error("Parsing error at line {line}: {message}")]
    ParseWithLine { line: usize, message: String },

    /// General parsing error
    #[error("Parsing error: {0}")]
    Parse(String),

    /// Syntax error in configuration
    #[error("Syntax error at line {line}, column {column}: {message}")]
    SyntaxError {
        line: usize,
        column: usize,
        message: String,
    },

    /// Malformed configuration structure
    #[error("Malformed configuration: {0}")]
    MalformedConfig(String),

    /// Invalid indentation
    #[error("Invalid indentation at line {line}: expected {expected}, found {found}")]
    InvalidIndentation {
        line: usize,
        expected: usize,
        found: usize,
    },

    /// Unclosed configuration block
    #[error("Unclosed configuration block starting at line {start_line}")]
    UnclosedBlock { start_line: usize },

    /// Invalid vendor configuration
    #[error("Invalid configuration for vendor {vendor}: {message}")]
    InvalidVendorConfig { vendor: String, message: String },

    /// Slicing error
    #[error("Slicing error: {0}")]
    Slice(String),

    /// Slicing error with context
    #[error("Slicing error in {context}: {message}")]
    SliceWithContext { context: String, message: String },

    /// Diff error
    #[error("Diff error: {0}")]
    Diff(String),

    /// Invalid pattern error
    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),

    /// Unsupported pattern error
    #[error("Unsupported pattern type: {0}")]
    UnsupportedPattern(String),

    /// Pattern compilation error
    #[error("Pattern compilation error: {0}")]
    PatternCompilation(String),

    /// Configuration validation error
    #[error("Configuration validation error: {0}")]
    Validation(String),

    /// Validation error with line number
    #[error("Validation error at line {line}: {message}")]
    ValidationWithLine { line: usize, message: String },

    /// File format error
    #[error("Unsupported file format: {format}")]
    UnsupportedFormat { format: String },

    /// Configuration too large for processing
    #[error("Configuration too large: {size} bytes exceeds limit of {limit} bytes")]
    ConfigurationTooLarge { size: usize, limit: usize },

    /// Memory allocation error
    #[error("Memory allocation error: {0}")]
    Memory(String),

    /// Timeout error
    #[error("Operation timed out after {seconds} seconds")]
    Timeout { seconds: u64 },

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Regex compilation error
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    /// UTF-8 encoding error
    #[error("UTF-8 encoding error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    /// Workflow execution error
    #[error("Workflow execution error: {0}")]
    DiffWorkflowError(String),

    /// Workflow not found error
    #[error("Workflow not found: {0}")]
    WorkflowNotFound(String),

    /// Approval not required error
    #[error("Approval not required for workflow: {0}")]
    ApprovalNotRequired(String),

    /// Unauthorized approver error
    #[error("Unauthorized approver: {0}")]
    UnauthorizedApprover(String),

    /// Cache error
    #[error("Cache error: {0}")]
    Cache(String),

    /// Other error
    #[error("Other error: {0}")]
    Other(String),
}

impl Error {
    /// Create a parsing error with line context
    pub fn parse_error_at_line(line: usize, message: impl Into<String>) -> Self {
        Self::ParseWithLine {
            line,
            message: message.into(),
        }
    }

    /// Create a syntax error with position
    pub fn syntax_error(line: usize, column: usize, message: impl Into<String>) -> Self {
        Self::SyntaxError {
            line,
            column,
            message: message.into(),
        }
    }

    /// Create an indentation error
    #[must_use]
    pub const fn indentation_error(line: usize, expected: usize, found: usize) -> Self {
        Self::InvalidIndentation {
            line,
            expected,
            found,
        }
    }

    /// Create an unclosed block error
    #[must_use]
    pub const fn unclosed_block_error(start_line: usize) -> Self {
        Self::UnclosedBlock { start_line }
    }

    /// Create a vendor-specific error
    pub fn vendor_error(vendor: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InvalidVendorConfig {
            vendor: vendor.into(),
            message: message.into(),
        }
    }

    /// Create a slicing error with context
    pub fn slice_error_with_context(
        context: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::SliceWithContext {
            context: context.into(),
            message: message.into(),
        }
    }

    /// Create a validation error with line number
    pub fn validation_error_at_line(line: usize, message: impl Into<String>) -> Self {
        Self::ValidationWithLine {
            line,
            message: message.into(),
        }
    }

    /// Create a timeout error
    #[must_use]
    pub const fn timeout_error(seconds: u64) -> Self {
        Self::Timeout { seconds }
    }

    /// Create a size limit error
    #[must_use]
    pub const fn size_limit_error(size: usize, limit: usize) -> Self {
        Self::ConfigurationTooLarge { size, limit }
    }

    /// Check if this error is recoverable
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::Parse(_)
                | Self::ParseWithLine { .. }
                | Self::InvalidPattern(_)
                | Self::Validation(_)
                | Self::ValidationWithLine { .. }
        )
    }

    /// Get the line number associated with this error, if any
    #[must_use]
    pub const fn line_number(&self) -> Option<usize> {
        match self {
            Self::ParseWithLine { line, .. }
            | Self::SyntaxError { line, .. }
            | Self::InvalidIndentation { line, .. }
            | Self::ValidationWithLine { line, .. } => Some(*line),
            Self::UnclosedBlock { start_line } => Some(*start_line),
            _ => None,
        }
    }

    /// Get the error category for grouping related errors
    #[must_use]
    pub const fn category(&self) -> ErrorCategory {
        match self {
            Self::Parse(_)
            | Self::ParseWithLine { .. }
            | Self::SyntaxError { .. }
            | Self::MalformedConfig(_)
            | Self::InvalidIndentation { .. }
            | Self::UnclosedBlock { .. }
            | Self::InvalidVendorConfig { .. } => ErrorCategory::Parsing,

            Self::Slice(_) | Self::SliceWithContext { .. } => ErrorCategory::Slicing,

            Self::Diff(_) | Self::DiffWorkflowError(_) => ErrorCategory::Diffing,

            Self::InvalidPattern(_) | Self::UnsupportedPattern(_) | Self::PatternCompilation(_) => {
                ErrorCategory::Pattern
            }

            Self::Validation(_) | Self::ValidationWithLine { .. } => ErrorCategory::Validation,

            Self::UnsupportedFormat { .. } => ErrorCategory::Format,

            Self::ConfigurationTooLarge { .. } | Self::Memory(_) => ErrorCategory::Resource,

            Self::Timeout { .. } => ErrorCategory::Performance,

            Self::Io(_) => ErrorCategory::Io,

            Self::Regex(_) | Self::Utf8(_) => ErrorCategory::External,

            Self::WorkflowNotFound(_)
            | Self::ApprovalNotRequired(_)
            | Self::UnauthorizedApprover(_) => ErrorCategory::Workflow,

            Self::Cache(_) => ErrorCategory::Cache,

            Self::Other(_) => ErrorCategory::Other,
        }
    }
}

/// Error categories for grouping related errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Errors related to configuration parsing
    Parsing,
    /// Errors related to configuration slicing
    Slicing,
    /// Errors related to configuration diffing
    Diffing,
    /// Errors related to pattern matching
    Pattern,
    /// Errors related to configuration validation
    Validation,
    /// Errors related to file format
    Format,
    /// Errors related to resource limits
    Resource,
    /// Errors related to performance limits
    Performance,
    /// I/O related errors
    Io,
    /// External library errors
    External,
    /// Workflow related errors
    Workflow,
    /// Cache related errors
    Cache,
    /// Other errors
    Other,
}

impl std::fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parsing => write!(f, "Parsing"),
            Self::Slicing => write!(f, "Slicing"),
            Self::Diffing => write!(f, "Diffing"),
            Self::Pattern => write!(f, "Pattern"),
            Self::Validation => write!(f, "Validation"),
            Self::Format => write!(f, "Format"),
            Self::Resource => write!(f, "Resource"),
            Self::Performance => write!(f, "Performance"),
            Self::Io => write!(f, "I/O"),
            Self::External => write!(f, "External"),
            Self::Workflow => write!(f, "Workflow"),
            Self::Cache => write!(f, "Cache"),
            Self::Other => write!(f, "Other"),
        }
    }
}

/// Config-slicer result type
pub type Result<T> = std::result::Result<T, Error>;

/// Error context for building detailed error messages
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// File name or source identifier
    pub source: Option<String>,
    /// Line number in the source
    pub line: Option<usize>,
    /// Column number in the line
    pub column: Option<usize>,
    /// Additional context information
    pub context: Vec<String>,
}

impl ErrorContext {
    /// Create a new error context
    #[must_use]
    pub const fn new() -> Self {
        Self {
            source: None,
            line: None,
            column: None,
            context: Vec::new(),
        }
    }

    /// Set the source file or identifier
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Set the line number
    #[must_use]
    pub const fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    /// Set the column number
    #[must_use]
    pub const fn with_column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }

    /// Add context information
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context.push(context.into());
        self
    }

    /// Build a formatted error message
    #[must_use]
    pub fn format_error(&self, message: &str) -> String {
        let mut formatted = String::new();

        if let Some(ref source) = self.source {
            formatted.push_str(&format!("In {source}: "));
        }

        if let (Some(line), Some(column)) = (self.line, self.column) {
            formatted.push_str(&format!("at line {line}, column {column}: "));
        } else if let Some(line) = self.line {
            formatted.push_str(&format!("at line {line}: "));
        }

        formatted.push_str(message);

        if !self.context.is_empty() {
            formatted.push_str(&format!(" ({})", self.context.join(", ")));
        }

        formatted
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::new()
    }
}
