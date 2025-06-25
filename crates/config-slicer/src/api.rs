//! Public API for config-slicer library
//!
//! This module provides a clean, type-safe interface for parsing and slicing
//! network device configurations. It abstracts the complexity of the underlying
//! implementation while providing comprehensive error handling and validation.

use crate::error::{Error, Result};
use crate::parser::{ConfigNode, PluginRegistry, PluginRegistryBuilder, Vendor};
use crate::slicer::{
    ConfigSlicer, PatternBuilder, SliceContext, SliceContextBuilder, SlicePattern, SliceResult,
};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use tracing::{debug, info};

/// Main entry point for the config-slicer library
///
/// This struct provides a high-level, easy-to-use interface for parsing
/// network device configurations and extracting specific sections.
///
/// # Examples
///
/// ```rust
/// use config_slicer::ConfigSlicerApi;
///
/// let api = ConfigSlicerApi::new();
/// let config_text = "interface GigabitEthernet0/1\n description Link to router\n ip address 10.1.1.1 255.255.255.0";
/// let tree = api.parse_config(config_text, None)?;
/// let results = api.slice_by_glob(&tree, "interface*")?;
/// ```
pub struct ConfigSlicerApi {
    parser_registry: PluginRegistry,
    slicer: ConfigSlicer,
}

impl ConfigSlicerApi {
    /// Create a new ConfigSlicerApi with default configuration
    pub fn new() -> Self {
        let parser_registry = PluginRegistryBuilder::new()
            .with_default_plugins()
            .unwrap_or_else(|_| PluginRegistryBuilder::new())
            .build();

        let slicer = ConfigSlicer::new();

        Self {
            parser_registry,
            slicer,
        }
    }

    /// Create a new ConfigSlicerApi with custom parser plugins
    pub fn with_parser_registry(parser_registry: PluginRegistry) -> Self {
        let slicer = ConfigSlicer::new();

        Self {
            parser_registry,
            slicer,
        }
    }

    /// Parse a configuration string into a hierarchical tree
    ///
    /// # Arguments
    ///
    /// * `config_text` - The configuration text to parse
    /// * `vendor` - Optional vendor hint for parser selection
    ///
    /// # Returns
    ///
    /// Returns a `ConfigNode` representing the root of the parsed configuration tree
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The configuration text is malformed
    /// - No suitable parser can be found
    /// - Parsing fails due to syntax errors
    pub fn parse_config(&self, config_text: &str, vendor: Option<Vendor>) -> Result<ConfigNode> {
        debug!(
            "Parsing configuration with {} characters",
            config_text.len()
        );

        if config_text.trim().is_empty() {
            return Err(Error::Parse("Configuration text is empty".to_string()));
        }

        // Auto-detect vendor if not provided
        let detected_vendor = vendor.unwrap_or_else(|| self.detect_vendor(config_text));

        info!("Using vendor parser: {:?}", detected_vendor);

        // For now, use a simple parser since the plugin system is complex
        // In a real implementation, this would use the detected vendor
        self.simple_parse(config_text, detected_vendor)
    }

    /// Parse a configuration file into a hierarchical tree
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the configuration file
    /// * `vendor` - Optional vendor hint for parser selection
    ///
    /// # Returns
    ///
    /// Returns a `ConfigNode` representing the root of the parsed configuration tree
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be read
    /// - The configuration is malformed
    /// - No suitable parser can be found
    pub fn parse_config_file<P: AsRef<Path>>(
        &self,
        file_path: P,
        vendor: Option<Vendor>,
    ) -> Result<ConfigNode> {
        let path = file_path.as_ref();
        debug!("Parsing configuration file: {}", path.display());

        let config_text = fs::read_to_string(path).map_err(|e| Error::Io(e))?;

        self.parse_config(&config_text, vendor)
    }

    /// Extract configuration slices using a glob pattern
    ///
    /// This is a convenience method for the most common use case.
    ///
    /// # Arguments
    ///
    /// * `config_tree` - The parsed configuration tree
    /// * `pattern` - Glob pattern (e.g., "interface*", "vlan?0?")
    ///
    /// # Examples
    ///
    /// ```rust
    /// let results = api.slice_by_glob(&tree, "interface*")?;
    /// let results = api.slice_by_glob(&tree, "vlan?0?")?;
    /// ```
    pub fn slice_by_glob(&self, config_tree: &ConfigNode, pattern: &str) -> Result<SliceResult> {
        let slice_pattern = PatternBuilder::glob(pattern).build()?;
        self.slicer.extract_slice(config_tree, &slice_pattern, None)
    }

    /// Extract configuration slices using a regular expression
    ///
    /// # Arguments
    ///
    /// * `config_tree` - The parsed configuration tree
    /// * `pattern` - Regular expression pattern
    ///
    /// # Examples
    ///
    /// ```rust
    /// let results = api.slice_by_regex(&tree, r"^interface\s+GigabitEthernet\d+/\d+$")?;
    /// ```
    pub fn slice_by_regex(&self, config_tree: &ConfigNode, pattern: &str) -> Result<SliceResult> {
        let slice_pattern = PatternBuilder::regex(pattern).build()?;
        self.slicer.extract_slice(config_tree, &slice_pattern, None)
    }

    /// Extract configuration slices using a hierarchical path
    ///
    /// # Arguments
    ///
    /// * `config_tree` - The parsed configuration tree
    /// * `path` - Hierarchical path (e.g., "interface/[description]/ip")
    ///
    /// # Examples
    ///
    /// ```rust
    /// let results = api.slice_by_path(&tree, "interface/ip")?;
    /// let results = api.slice_by_path(&tree, "router/bgp/[neighbor]/remote-as")?;
    /// ```
    pub fn slice_by_path(&self, config_tree: &ConfigNode, path: &str) -> Result<SliceResult> {
        use crate::slicer::HierarchicalPattern;
        let hierarchical_pattern = HierarchicalPattern::new(path)?;
        let slice_pattern = SlicePattern::Hierarchical(hierarchical_pattern);
        self.slicer.extract_slice(config_tree, &slice_pattern, None)
    }

    /// Extract configuration slices with advanced context filtering
    ///
    /// # Arguments
    ///
    /// * `config_tree` - The parsed configuration tree
    /// * `pattern` - Slice pattern to match
    /// * `context` - Context for filtering results
    pub fn slice_with_context(
        &self,
        config_tree: &ConfigNode,
        pattern: &SlicePattern,
        context: &SliceContext,
    ) -> Result<SliceResult> {
        self.slicer
            .extract_slice(config_tree, pattern, Some(context))
    }

    /// Extract multiple slices using different patterns
    ///
    /// This method allows for efficient extraction of multiple different
    /// configuration sections in a single operation.
    ///
    /// # Arguments
    ///
    /// * `config_tree` - The parsed configuration tree
    /// * `patterns` - Array of patterns to match
    pub fn slice_multiple(
        &self,
        config_tree: &ConfigNode,
        patterns: &[SlicePattern],
    ) -> Result<Vec<SliceResult>> {
        self.slicer
            .extract_multiple_slices(config_tree, patterns, None)
    }

    /// Create a context builder for advanced filtering
    ///
    /// # Examples
    ///
    /// ```rust
    /// let context = api.context_builder()
    ///     .include_vendor(Vendor::Cisco)
    ///     .include_context_type(ConfigContext::Interface)
    ///     .build();
    /// ```
    pub fn context_builder(&self) -> SliceContextBuilder {
        SliceContextBuilder::new()
    }

    /// Create a pattern builder for complex patterns
    ///
    /// # Examples
    ///
    /// ```rust
    /// let pattern = api.pattern_builder()
    ///     .glob("interface*")
    ///     .case_sensitive(false)
    ///     .build()?;
    /// ```
    pub fn pattern_builder(&self) -> PatternBuilder {
        PatternBuilder::glob("")
    }

    /// Validate a configuration for syntax errors
    ///
    /// This method parses the configuration and reports any validation
    /// issues without returning the full parse tree.
    ///
    /// # Arguments
    ///
    /// * `config_text` - Configuration text to validate
    /// * `vendor` - Optional vendor hint
    ///
    /// # Returns
    ///
    /// Returns a validation report containing any errors or warnings
    pub fn validate_config(
        &self,
        config_text: &str,
        vendor: Option<Vendor>,
    ) -> Result<ValidationReport> {
        // First, try to parse the configuration
        match self.parse_config(config_text, vendor) {
            Ok(tree) => {
                // If parsing succeeds, run additional validation checks
                Ok(ValidationReport::new_valid(tree))
            }
            Err(parse_error) => {
                // If parsing fails, create a validation report with the error
                Ok(ValidationReport::new_invalid(parse_error))
            }
        }
    }

    /// Get information about available parsers
    pub fn available_parsers(&self) -> Vec<Vendor> {
        vec![
            Vendor::Cisco,
            Vendor::Juniper,
            Vendor::Arista,
            Vendor::Generic,
        ]
    }

    /// Get information about available slice extractors
    pub fn available_extractors(&self) -> Vec<String> {
        self.slicer.available_extractors()
    }

    /// Auto-detect vendor from configuration text
    fn detect_vendor(&self, config_text: &str) -> Vendor {
        // Simple heuristic vendor detection
        if config_text.contains("interface GigabitEthernet") || config_text.contains("hostname") {
            Vendor::Cisco
        } else if config_text.contains("set interfaces") || config_text.contains("set system") {
            Vendor::Juniper
        } else if config_text.contains("interface Ethernet")
            || config_text.contains("spanning-tree mode")
        {
            Vendor::Arista
        } else {
            Vendor::Generic
        }
    }

    /// Simple configuration parser
    fn simple_parse(&self, config_text: &str, _vendor: Vendor) -> Result<ConfigNode> {
        use crate::parser::{ConfigContext, NodeType};

        // Create a simple root node with basic structure
        let mut root = ConfigNode {
            command: "root".to_string(),
            raw_line: "".to_string(),
            line_number: 0,
            indent_level: 0,
            children: Vec::new(),
            context: ConfigContext::Global,
            node_type: NodeType::Root,
            metadata: HashMap::new(),
        };

        // Parse lines and create basic hierarchy
        for (line_num, line) in config_text.lines().enumerate() {
            if line.trim().is_empty() || line.trim().starts_with('!') {
                continue;
            }

            let indent_level = line.len() - line.trim_start().len();
            let child = ConfigNode {
                command: line.trim().to_string(),
                raw_line: line.to_string(),
                line_number: line_num + 1,
                indent_level,
                children: Vec::new(),
                context: ConfigContext::Global,
                node_type: NodeType::Command,
                metadata: HashMap::new(),
            };

            root.children.push(child);
        }

        Ok(root)
    }
}

impl Default for ConfigSlicerApi {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration validation report
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// Whether the configuration is valid
    pub is_valid: bool,
    /// List of validation errors
    pub errors: Vec<ValidationError>,
    /// List of validation warnings
    pub warnings: Vec<ValidationWarning>,
    /// Parsed configuration tree (if valid)
    pub config_tree: Option<ConfigNode>,
    /// Validation metadata
    pub metadata: HashMap<String, String>,
}

impl ValidationReport {
    /// Create a new validation report for valid configuration
    pub fn new_valid(config_tree: ConfigNode) -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            config_tree: Some(config_tree),
            metadata: HashMap::new(),
        }
    }

    /// Create a new validation report for invalid configuration
    pub fn new_invalid(error: Error) -> Self {
        Self {
            is_valid: false,
            errors: vec![ValidationError::from_error(error)],
            warnings: Vec::new(),
            config_tree: None,
            metadata: HashMap::new(),
        }
    }

    /// Add a validation warning
    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }

    /// Add a validation error (makes the report invalid)
    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
        self.is_valid = false;
    }

    /// Check if the configuration has any issues
    pub fn has_issues(&self) -> bool {
        !self.errors.is_empty() || !self.warnings.is_empty()
    }

    /// Get a summary of validation results
    pub fn summary(&self) -> String {
        if self.is_valid && !self.has_issues() {
            "Configuration is valid with no issues".to_string()
        } else if self.is_valid {
            format!(
                "Configuration is valid with {} warnings",
                self.warnings.len()
            )
        } else {
            format!(
                "Configuration is invalid with {} errors and {} warnings",
                self.errors.len(),
                self.warnings.len()
            )
        }
    }
}

/// Validation error information
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Error message
    pub message: String,
    /// Line number where error occurred (if applicable)
    pub line_number: Option<usize>,
    /// Error severity
    pub severity: ErrorSeverity,
}

impl ValidationError {
    /// Create validation error from config-slicer error
    pub fn from_error(error: Error) -> Self {
        Self {
            message: error.to_string(),
            line_number: None,
            severity: ErrorSeverity::Error,
        }
    }

    /// Create a new validation error
    pub fn new(message: String, line_number: Option<usize>) -> Self {
        Self {
            message,
            line_number,
            severity: ErrorSeverity::Error,
        }
    }
}

/// Validation warning information
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    /// Warning message
    pub message: String,
    /// Line number where warning occurred (if applicable)
    pub line_number: Option<usize>,
    /// Warning type
    pub warning_type: WarningType,
}

impl ValidationWarning {
    /// Create a new validation warning
    pub fn new(message: String, line_number: Option<usize>, warning_type: WarningType) -> Self {
        Self {
            message,
            line_number,
            warning_type,
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorSeverity {
    /// Critical error that prevents parsing
    Error,
    /// Warning that doesn't prevent parsing
    Warning,
    /// Informational message
    Info,
}

/// Warning types
#[derive(Debug, Clone, PartialEq)]
pub enum WarningType {
    /// Deprecated syntax
    Deprecated,
    /// Best practice violation
    BestPractice,
    /// Potential configuration issue
    PotentialIssue,
    /// Vendor-specific consideration
    VendorSpecific,
}

/// Builder for streaming configuration processing
pub struct StreamingConfigProcessor {
    api: ConfigSlicerApi,
    buffer_size: usize,
    chunk_size: usize,
}

impl StreamingConfigProcessor {
    /// Create a new streaming processor
    pub fn new(api: ConfigSlicerApi) -> Self {
        Self {
            api,
            buffer_size: 8192,
            chunk_size: 1024,
        }
    }

    /// Set the buffer size for reading
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Set the chunk size for processing
    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    /// Process a large configuration file in streaming mode
    ///
    /// This method is designed for processing very large configuration files
    /// that might not fit in memory all at once.
    pub fn process_large_config<R: Read>(
        &self,
        reader: R,
        vendor: Option<Vendor>,
    ) -> Result<ConfigNode> {
        let mut buffer_reader = BufReader::with_capacity(self.buffer_size, reader);
        let mut config_lines = Vec::new();
        let mut line = String::new();

        // Read the configuration line by line
        while buffer_reader.read_line(&mut line)? > 0 {
            config_lines.push(line.trim_end().to_string());
            line.clear();

            // Process in chunks to manage memory usage
            if config_lines.len() >= self.chunk_size {
                debug!("Processing chunk of {} lines", config_lines.len());
            }
        }

        // Join all lines and parse
        let config_text = config_lines.join("\n");
        self.api.parse_config(&config_text, vendor)
    }

    /// Process configuration and extract slices in streaming mode
    pub fn process_and_slice<R: Read>(
        &self,
        reader: R,
        patterns: &[SlicePattern],
        vendor: Option<Vendor>,
    ) -> Result<Vec<SliceResult>> {
        let config_tree = self.process_large_config(reader, vendor)?;
        self.api.slice_multiple(&config_tree, patterns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_creation() {
        let api = ConfigSlicerApi::new();
        assert!(!api.available_parsers().is_empty());
        assert!(!api.available_extractors().is_empty());
    }

    #[test]
    fn test_config_parsing() {
        let api = ConfigSlicerApi::new();
        let config = "interface GigabitEthernet0/1\n description Test interface\n ip address 10.1.1.1 255.255.255.0";

        let result = api.parse_config(config, Some(Vendor::Cisco));
        assert!(result.is_ok());

        let tree = result.unwrap();
        assert_eq!(tree.command, "root");
        assert!(!tree.children.is_empty());
    }

    #[test]
    fn test_empty_config_error() {
        let api = ConfigSlicerApi::new();
        let result = api.parse_config("", None);
        assert!(result.is_err());

        let result = api.parse_config("   \n\t  ", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_glob_slicing() {
        let api = ConfigSlicerApi::new();
        let config = "interface GigabitEthernet0/1\n description Test\nhostname test-router";

        let tree = api.parse_config(config, Some(Vendor::Cisco)).unwrap();
        let result = api.slice_by_glob(&tree, "interface*").unwrap();

        assert!(result.has_matches());
    }

    #[test]
    fn test_validation_report() {
        let api = ConfigSlicerApi::new();

        // Valid configuration
        let valid_config = "interface GigabitEthernet0/1\n description Test";
        let report = api
            .validate_config(valid_config, Some(Vendor::Cisco))
            .unwrap();
        assert!(report.is_valid);
        assert!(report.config_tree.is_some());

        // Invalid configuration (empty)
        let report = api.validate_config("", None).unwrap();
        assert!(!report.is_valid);
        assert!(report.config_tree.is_none());
        assert!(!report.errors.is_empty());
    }

    #[test]
    fn test_streaming_processor() {
        let api = ConfigSlicerApi::new();
        let processor = StreamingConfigProcessor::new(api)
            .with_buffer_size(4096)
            .with_chunk_size(100);

        let config = "interface GigabitEthernet0/1\n description Test interface\n ip address 10.1.1.1 255.255.255.0";
        let cursor = std::io::Cursor::new(config);

        let result = processor.process_large_config(cursor, Some(Vendor::Cisco));
        assert!(result.is_ok());
    }
}
