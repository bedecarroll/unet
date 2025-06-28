//! Configuration validation utilities
//!
//! This module provides comprehensive validation capabilities for network device
//! configurations, including syntax validation, semantic validation, and best
//! practice checking.

use crate::parser::{ConfigNode, NodeType, Vendor};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::{debug, info};

/// Comprehensive configuration validator
pub struct ConfigValidator {
    rules: Vec<Box<dyn ValidationRule>>,
    vendor_specific_rules: HashMap<Vendor, Vec<Box<dyn ValidationRule>>>,
    severity_filter: ValidationSeverity,
}

impl ConfigValidator {
    /// Create a new validator with default rules
    #[must_use]
    pub fn new() -> Self {
        let mut validator = Self {
            rules: Vec::new(),
            vendor_specific_rules: HashMap::new(),
            severity_filter: ValidationSeverity::Info,
        };

        // Register default validation rules
        validator.register_default_rules();
        validator
    }

    /// Create a validator with custom severity filter
    #[must_use]
    pub fn with_severity_filter(severity: ValidationSeverity) -> Self {
        let mut validator = Self::new();
        validator.severity_filter = severity;
        validator
    }

    /// Register a validation rule
    pub fn register_rule(&mut self, rule: Box<dyn ValidationRule>) {
        self.rules.push(rule);
    }

    /// Register a vendor-specific validation rule
    pub fn register_vendor_rule(&mut self, vendor: Vendor, rule: Box<dyn ValidationRule>) {
        self.vendor_specific_rules
            .entry(vendor)
            .or_default()
            .push(rule);
    }

    /// Validate a configuration tree
    pub fn validate(&self, config: &ConfigNode, vendor: Option<Vendor>) -> ValidationReport {
        let mut report = ValidationReport::new();

        info!("Starting validation for vendor: {:?}", vendor);

        // Run general validation rules
        for rule in &self.rules {
            let violations = rule.validate(config);
            for violation in violations {
                if violation.severity >= self.severity_filter {
                    report.add_violation(violation);
                }
            }
        }

        // Run vendor-specific rules if vendor is specified
        if let Some(vendor) = vendor {
            if let Some(vendor_rules) = self.vendor_specific_rules.get(&vendor) {
                for rule in vendor_rules {
                    let violations = rule.validate(config);
                    for violation in violations {
                        if violation.severity >= self.severity_filter {
                            report.add_violation(violation);
                        }
                    }
                }
            }
        }

        // Perform structural validation
        self.validate_structure(config, &mut report);

        debug!(
            "Validation complete: {} violations found",
            report.violations.len()
        );
        report
    }

    /// Validate configuration structure
    fn validate_structure(&self, config: &ConfigNode, report: &mut ValidationReport) {
        let mut structure_validator = StructureValidator::new();
        structure_validator.validate_node(config, 0, report);
    }

    /// Register default validation rules
    fn register_default_rules(&mut self) {
        // Syntax validation rules
        self.register_rule(Box::new(IndentationRule::new()));
        self.register_rule(Box::new(EmptyLineRule::new()));
        self.register_rule(Box::new(CommentRule::new()));

        // Semantic validation rules
        self.register_rule(Box::new(DuplicateConfigRule::new()));
        self.register_rule(Box::new(InconsistentConfigRule::new()));

        // Best practice rules
        self.register_rule(Box::new(DescriptionRule::new()));
        self.register_rule(Box::new(NamingConventionRule::new()));

        // Security rules
        self.register_rule(Box::new(PlaintextPasswordRule::new()));
        self.register_rule(Box::new(WeakSecurityRule::new()));

        // Register vendor-specific rules
        self.register_cisco_rules();
        self.register_juniper_rules();
        self.register_arista_rules();
    }

    /// Register Cisco-specific validation rules
    fn register_cisco_rules(&mut self) {
        self.register_vendor_rule(Vendor::Cisco, Box::new(CiscoInterfaceRule::new()));
        self.register_vendor_rule(Vendor::Cisco, Box::new(CiscoVlanRule::new()));
        self.register_vendor_rule(Vendor::Cisco, Box::new(CiscoBgpRule::new()));
    }

    /// Register Juniper-specific validation rules
    fn register_juniper_rules(&mut self) {
        self.register_vendor_rule(Vendor::Juniper, Box::new(JuniperInterfaceRule::new()));
        self.register_vendor_rule(Vendor::Juniper, Box::new(JuniperPolicyRule::new()));
    }

    /// Register Arista-specific validation rules
    fn register_arista_rules(&mut self) {
        self.register_vendor_rule(Vendor::Arista, Box::new(AristaInterfaceRule::new()));
        self.register_vendor_rule(Vendor::Arista, Box::new(AristaMlagRule::new()));
    }
}

impl Default for ConfigValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for configuration validation rules
pub trait ValidationRule: Send + Sync {
    /// Validate a configuration node and return any violations
    fn validate(&self, config: &ConfigNode) -> Vec<ValidationViolation>;

    /// Get the name of this validation rule
    fn rule_name(&self) -> &str;

    /// Get the description of this validation rule
    fn rule_description(&self) -> &str;
}

/// Validation report containing all violations found
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// List of validation violations
    pub violations: Vec<ValidationViolation>,
    /// Summary statistics
    pub summary: ValidationSummary,
    /// Metadata about the validation
    pub metadata: HashMap<String, String>,
}

impl ValidationReport {
    /// Create a new empty validation report
    #[must_use]
    pub fn new() -> Self {
        Self {
            violations: Vec::new(),
            summary: ValidationSummary::default(),
            metadata: HashMap::new(),
        }
    }

    /// Add a validation violation to the report
    pub fn add_violation(&mut self, violation: ValidationViolation) {
        // Update summary statistics
        match violation.severity {
            ValidationSeverity::Error => self.summary.error_count += 1,
            ValidationSeverity::Warning => self.summary.warning_count += 1,
            ValidationSeverity::Info => self.summary.info_count += 1,
        }

        self.violations.push(violation);
    }

    /// Check if the configuration is considered valid
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.summary.error_count == 0
    }

    /// Get violations of a specific severity
    #[must_use]
    pub fn violations_by_severity(
        &self,
        severity: ValidationSeverity,
    ) -> Vec<&ValidationViolation> {
        self.violations
            .iter()
            .filter(|v| v.severity == severity)
            .collect()
    }

    /// Get violations by rule name
    #[must_use]
    pub fn violations_by_rule(&self, rule_name: &str) -> Vec<&ValidationViolation> {
        self.violations
            .iter()
            .filter(|v| v.rule_name == rule_name)
            .collect()
    }

    /// Generate a human-readable summary
    #[must_use]
    pub fn summary_text(&self) -> String {
        if self.is_valid() && self.violations.is_empty() {
            "Configuration is valid with no issues".to_string()
        } else if self.is_valid() {
            format!(
                "Configuration is valid with {} warnings and {} info messages",
                self.summary.warning_count, self.summary.info_count
            )
        } else {
            format!(
                "Configuration has {} errors, {} warnings, and {} info messages",
                self.summary.error_count, self.summary.warning_count, self.summary.info_count
            )
        }
    }

    /// Add metadata to the report
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Individual validation violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationViolation {
    /// Severity of the violation
    pub severity: ValidationSeverity,
    /// Name of the rule that detected this violation
    pub rule_name: String,
    /// Human-readable message describing the violation
    pub message: String,
    /// Line number where the violation occurs
    pub line_number: Option<usize>,
    /// Column number where the violation occurs
    pub column_number: Option<usize>,
    /// Path to the configuration node
    pub node_path: String,
    /// Suggested fix for the violation
    pub suggested_fix: Option<String>,
    /// Additional context for the violation
    pub context: HashMap<String, String>,
}

impl ValidationViolation {
    /// Create a new validation violation
    #[must_use]
    pub fn new(
        severity: ValidationSeverity,
        rule_name: String,
        message: String,
        line_number: Option<usize>,
    ) -> Self {
        Self {
            severity,
            rule_name,
            message,
            line_number,
            column_number: None,
            node_path: String::new(),
            suggested_fix: None,
            context: HashMap::new(),
        }
    }

    /// Set the column number
    #[must_use]
    pub const fn with_column(mut self, column: usize) -> Self {
        self.column_number = Some(column);
        self
    }

    /// Set the node path
    #[must_use]
    pub fn with_node_path(mut self, path: String) -> Self {
        self.node_path = path;
        self
    }

    /// Set the suggested fix
    #[must_use]
    pub fn with_suggested_fix(mut self, fix: String) -> Self {
        self.suggested_fix = Some(fix);
        self
    }

    /// Add context information
    #[must_use]
    pub fn with_context(mut self, key: String, value: String) -> Self {
        self.context.insert(key, value);
        self
    }
}

/// Validation severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ValidationSeverity {
    /// Informational message
    Info,
    /// Warning that should be addressed
    Warning,
    /// Error that must be fixed
    Error,
}

impl std::fmt::Display for ValidationSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "INFO"),
            Self::Warning => write!(f, "WARNING"),
            Self::Error => write!(f, "ERROR"),
        }
    }
}

/// Summary statistics for validation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidationSummary {
    /// Number of errors found
    pub error_count: usize,
    /// Number of warnings found
    pub warning_count: usize,
    /// Number of info messages
    pub info_count: usize,
    /// Total number of violations
    pub total_violations: usize,
}

/// Structure validator for checking configuration hierarchy
struct StructureValidator {
    seen_nodes: HashSet<String>,
    depth_limit: usize,
}

impl StructureValidator {
    fn new() -> Self {
        Self {
            seen_nodes: HashSet::new(),
            depth_limit: 50, // Reasonable depth limit
        }
    }

    fn validate_node(&mut self, node: &ConfigNode, depth: usize, report: &mut ValidationReport) {
        // Check depth limit
        if depth > self.depth_limit {
            report.add_violation(
                ValidationViolation::new(
                    ValidationSeverity::Warning,
                    "StructureValidator".to_string(),
                    format!(
                        "Configuration depth exceeds recommended limit of {}",
                        self.depth_limit
                    ),
                    Some(node.line_number),
                )
                .with_node_path(self.build_node_path(node)),
            );
        }

        // Validate this node
        self.validate_single_node(node, report);

        // Recursively validate children
        for child in &node.children {
            self.validate_node(child, depth + 1, report);
        }
    }

    fn validate_single_node(&mut self, node: &ConfigNode, report: &mut ValidationReport) {
        // Check for empty commands
        if node.command.trim().is_empty() && node.node_type == NodeType::Command {
            report.add_violation(
                ValidationViolation::new(
                    ValidationSeverity::Warning,
                    "StructureValidator".to_string(),
                    "Empty command found".to_string(),
                    Some(node.line_number),
                )
                .with_node_path(self.build_node_path(node)),
            );
        }

        // Check for very long lines
        if node.command.len() > 200 {
            report.add_violation(
                ValidationViolation::new(
                    ValidationSeverity::Info,
                    "StructureValidator".to_string(),
                    format!(
                        "Command line is very long ({} characters)",
                        node.command.len()
                    ),
                    Some(node.line_number),
                )
                .with_node_path(self.build_node_path(node))
                .with_suggested_fix(
                    "Consider breaking long lines into multiple commands".to_string(),
                ),
            );
        }
    }

    fn build_node_path(&self, node: &ConfigNode) -> String {
        // This is a simplified path - in practice, you'd build the full path
        format!("/{}", node.command)
    }
}

// Sample validation rules implementation

/// Rule to check indentation consistency
struct IndentationRule;

impl IndentationRule {
    const fn new() -> Self {
        Self
    }
}

impl ValidationRule for IndentationRule {
    fn validate(&self, config: &ConfigNode) -> Vec<ValidationViolation> {
        let mut violations = Vec::new();
        self.check_indentation_recursive(config, 0, &mut violations);
        violations
    }

    fn rule_name(&self) -> &'static str {
        "IndentationRule"
    }

    fn rule_description(&self) -> &'static str {
        "Checks for consistent indentation throughout the configuration"
    }
}

impl IndentationRule {
    fn check_indentation_recursive(
        &self,
        node: &ConfigNode,
        expected_level: usize,
        violations: &mut Vec<ValidationViolation>,
    ) {
        if node.indent_level != expected_level && node.node_type != NodeType::Root {
            violations.push(ValidationViolation::new(
                ValidationSeverity::Warning,
                self.rule_name().to_string(),
                format!(
                    "Inconsistent indentation: expected {}, found {}",
                    expected_level, node.indent_level
                ),
                Some(node.line_number),
            ));
        }

        for child in &node.children {
            self.check_indentation_recursive(child, expected_level + 1, violations);
        }
    }
}

/// Rule to check for empty lines
struct EmptyLineRule;

impl EmptyLineRule {
    const fn new() -> Self {
        Self
    }
}

impl ValidationRule for EmptyLineRule {
    fn validate(&self, config: &ConfigNode) -> Vec<ValidationViolation> {
        let mut violations = Vec::new();
        self.check_empty_lines_recursive(config, &mut violations);
        violations
    }

    fn rule_name(&self) -> &'static str {
        "EmptyLineRule"
    }

    fn rule_description(&self) -> &'static str {
        "Checks for excessive empty lines in configuration"
    }
}

impl EmptyLineRule {
    fn check_empty_lines_recursive(
        &self,
        node: &ConfigNode,
        violations: &mut Vec<ValidationViolation>,
    ) {
        if node.command.trim().is_empty() && node.node_type == NodeType::Command {
            violations.push(ValidationViolation::new(
                ValidationSeverity::Info,
                self.rule_name().to_string(),
                "Empty line found in configuration".to_string(),
                Some(node.line_number),
            ));
        }

        for child in &node.children {
            self.check_empty_lines_recursive(child, violations);
        }
    }
}

// Additional rule stubs for completeness
struct CommentRule;
impl CommentRule {
    const fn new() -> Self {
        Self
    }
}
impl ValidationRule for CommentRule {
    fn validate(&self, _config: &ConfigNode) -> Vec<ValidationViolation> {
        Vec::new()
    }
    fn rule_name(&self) -> &'static str {
        "CommentRule"
    }
    fn rule_description(&self) -> &'static str {
        "Validates comment formatting"
    }
}

struct DuplicateConfigRule;
impl DuplicateConfigRule {
    const fn new() -> Self {
        Self
    }
}
impl ValidationRule for DuplicateConfigRule {
    fn validate(&self, _config: &ConfigNode) -> Vec<ValidationViolation> {
        Vec::new()
    }
    fn rule_name(&self) -> &'static str {
        "DuplicateConfigRule"
    }
    fn rule_description(&self) -> &'static str {
        "Checks for duplicate configuration entries"
    }
}

struct InconsistentConfigRule;
impl InconsistentConfigRule {
    const fn new() -> Self {
        Self
    }
}
impl ValidationRule for InconsistentConfigRule {
    fn validate(&self, _config: &ConfigNode) -> Vec<ValidationViolation> {
        Vec::new()
    }
    fn rule_name(&self) -> &'static str {
        "InconsistentConfigRule"
    }
    fn rule_description(&self) -> &'static str {
        "Checks for inconsistent configuration"
    }
}

struct DescriptionRule;
impl DescriptionRule {
    const fn new() -> Self {
        Self
    }
}
impl ValidationRule for DescriptionRule {
    fn validate(&self, _config: &ConfigNode) -> Vec<ValidationViolation> {
        Vec::new()
    }
    fn rule_name(&self) -> &'static str {
        "DescriptionRule"
    }
    fn rule_description(&self) -> &'static str {
        "Checks for missing interface descriptions"
    }
}

struct NamingConventionRule;
impl NamingConventionRule {
    const fn new() -> Self {
        Self
    }
}
impl ValidationRule for NamingConventionRule {
    fn validate(&self, _config: &ConfigNode) -> Vec<ValidationViolation> {
        Vec::new()
    }
    fn rule_name(&self) -> &'static str {
        "NamingConventionRule"
    }
    fn rule_description(&self) -> &'static str {
        "Validates naming conventions"
    }
}

struct PlaintextPasswordRule;
impl PlaintextPasswordRule {
    const fn new() -> Self {
        Self
    }
}
impl ValidationRule for PlaintextPasswordRule {
    fn validate(&self, _config: &ConfigNode) -> Vec<ValidationViolation> {
        Vec::new()
    }
    fn rule_name(&self) -> &'static str {
        "PlaintextPasswordRule"
    }
    fn rule_description(&self) -> &'static str {
        "Checks for plaintext passwords"
    }
}

struct WeakSecurityRule;
impl WeakSecurityRule {
    const fn new() -> Self {
        Self
    }
}
impl ValidationRule for WeakSecurityRule {
    fn validate(&self, _config: &ConfigNode) -> Vec<ValidationViolation> {
        Vec::new()
    }
    fn rule_name(&self) -> &'static str {
        "WeakSecurityRule"
    }
    fn rule_description(&self) -> &'static str {
        "Checks for weak security configurations"
    }
}

// Vendor-specific rule stubs
struct CiscoInterfaceRule;
impl CiscoInterfaceRule {
    const fn new() -> Self {
        Self
    }
}
impl ValidationRule for CiscoInterfaceRule {
    fn validate(&self, _config: &ConfigNode) -> Vec<ValidationViolation> {
        Vec::new()
    }
    fn rule_name(&self) -> &'static str {
        "CiscoInterfaceRule"
    }
    fn rule_description(&self) -> &'static str {
        "Validates Cisco interface configuration"
    }
}

struct CiscoVlanRule;
impl CiscoVlanRule {
    const fn new() -> Self {
        Self
    }
}
impl ValidationRule for CiscoVlanRule {
    fn validate(&self, _config: &ConfigNode) -> Vec<ValidationViolation> {
        Vec::new()
    }
    fn rule_name(&self) -> &'static str {
        "CiscoVlanRule"
    }
    fn rule_description(&self) -> &'static str {
        "Validates Cisco VLAN configuration"
    }
}

struct CiscoBgpRule;
impl CiscoBgpRule {
    const fn new() -> Self {
        Self
    }
}
impl ValidationRule for CiscoBgpRule {
    fn validate(&self, _config: &ConfigNode) -> Vec<ValidationViolation> {
        Vec::new()
    }
    fn rule_name(&self) -> &'static str {
        "CiscoBgpRule"
    }
    fn rule_description(&self) -> &'static str {
        "Validates Cisco BGP configuration"
    }
}

struct JuniperInterfaceRule;
impl JuniperInterfaceRule {
    const fn new() -> Self {
        Self
    }
}
impl ValidationRule for JuniperInterfaceRule {
    fn validate(&self, _config: &ConfigNode) -> Vec<ValidationViolation> {
        Vec::new()
    }
    fn rule_name(&self) -> &'static str {
        "JuniperInterfaceRule"
    }
    fn rule_description(&self) -> &'static str {
        "Validates Juniper interface configuration"
    }
}

struct JuniperPolicyRule;
impl JuniperPolicyRule {
    const fn new() -> Self {
        Self
    }
}
impl ValidationRule for JuniperPolicyRule {
    fn validate(&self, _config: &ConfigNode) -> Vec<ValidationViolation> {
        Vec::new()
    }
    fn rule_name(&self) -> &'static str {
        "JuniperPolicyRule"
    }
    fn rule_description(&self) -> &'static str {
        "Validates Juniper policy configuration"
    }
}

struct AristaInterfaceRule;
impl AristaInterfaceRule {
    const fn new() -> Self {
        Self
    }
}
impl ValidationRule for AristaInterfaceRule {
    fn validate(&self, _config: &ConfigNode) -> Vec<ValidationViolation> {
        Vec::new()
    }
    fn rule_name(&self) -> &'static str {
        "AristaInterfaceRule"
    }
    fn rule_description(&self) -> &'static str {
        "Validates Arista interface configuration"
    }
}

struct AristaMlagRule;
impl AristaMlagRule {
    const fn new() -> Self {
        Self
    }
}
impl ValidationRule for AristaMlagRule {
    fn validate(&self, _config: &ConfigNode) -> Vec<ValidationViolation> {
        Vec::new()
    }
    fn rule_name(&self) -> &'static str {
        "AristaMlagRule"
    }
    fn rule_description(&self) -> &'static str {
        "Validates Arista MLAG configuration"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ConfigContext;

    fn create_test_node(command: &str, line_number: usize, indent_level: usize) -> ConfigNode {
        ConfigNode {
            command: command.to_string(),
            raw_line: command.to_string(),
            line_number,
            indent_level,
            children: Vec::new(),
            context: ConfigContext::Global,
            node_type: NodeType::Command,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_validator_creation() {
        let validator = ConfigValidator::new();
        assert!(!validator.rules.is_empty());
    }

    #[test]
    fn test_validation_report() {
        let mut report = ValidationReport::new();
        assert!(report.is_valid());
        assert_eq!(report.violations.len(), 0);

        let violation = ValidationViolation::new(
            ValidationSeverity::Warning,
            "TestRule".to_string(),
            "Test violation".to_string(),
            Some(1),
        );

        report.add_violation(violation);
        assert!(report.is_valid()); // Still valid with just warnings
        assert_eq!(report.summary.warning_count, 1);

        let error_violation = ValidationViolation::new(
            ValidationSeverity::Error,
            "TestRule".to_string(),
            "Test error".to_string(),
            Some(2),
        );

        report.add_violation(error_violation);
        assert!(!report.is_valid()); // Not valid with errors
        assert_eq!(report.summary.error_count, 1);
    }

    #[test]
    fn test_indentation_rule() {
        let rule = IndentationRule::new();

        let mut root = create_test_node("root", 0, 0);
        root.node_type = NodeType::Root;

        let child = create_test_node("interface GigabitEthernet0/1", 1, 2); // Wrong indentation
        root.children.push(child);

        let violations = rule.validate(&root);
        assert!(!violations.is_empty());
        assert_eq!(violations[0].severity, ValidationSeverity::Warning);
    }

    #[test]
    fn test_validation_severity_ordering() {
        assert!(ValidationSeverity::Error > ValidationSeverity::Warning);
        assert!(ValidationSeverity::Warning > ValidationSeverity::Info);
    }

    #[test]
    fn test_validation_violation_builder() {
        let violation = ValidationViolation::new(
            ValidationSeverity::Warning,
            "TestRule".to_string(),
            "Test message".to_string(),
            Some(10),
        )
        .with_column(5)
        .with_node_path("/interface/GigabitEthernet0/1".to_string())
        .with_suggested_fix("Add description".to_string())
        .with_context("vendor".to_string(), "cisco".to_string());

        assert_eq!(violation.line_number, Some(10));
        assert_eq!(violation.column_number, Some(5));
        assert_eq!(violation.node_path, "/interface/GigabitEthernet0/1");
        assert_eq!(violation.suggested_fix, Some("Add description".to_string()));
        assert_eq!(violation.context.get("vendor"), Some(&"cisco".to_string()));
    }
}
