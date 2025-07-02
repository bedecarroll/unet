//! Template-driven configuration slicing
//!
//! This module provides functionality to extract slice patterns from template headers
//! and integrate with the config-slicer library for consistent configuration slicing.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::template::{HeaderParser, MatchPattern, TemplateHeader, TemplateScope};
// Remove problematic config-slicer imports for now
// pub use config_slicer::slicer::{SliceContext, SliceContextBuilder, SlicePattern, GlobPattern, RegexPattern};

/// Template-driven slice extractor that bridges template headers with config-slicer
pub struct TemplateDrivenSlicer {
    header_parser: HeaderParser,
    pattern_cache: HashMap<String, String>, // Cache pattern strings instead of SlicePattern
}

impl std::fmt::Debug for TemplateDrivenSlicer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TemplateDrivenSlicer")
            .field("pattern_cache_size", &self.pattern_cache.len())
            .finish()
    }
}

impl TemplateDrivenSlicer {
    /// Create a new template-driven slicer
    pub fn new() -> Result<Self> {
        Ok(Self {
            header_parser: HeaderParser::new(),
            pattern_cache: HashMap::new(),
        })
    }

    /// Extract slice patterns from template content
    ///
    /// Parses template headers and converts them to slice patterns
    pub fn extract_slice_patterns(&mut self, template_content: &str) -> Result<Vec<String>> {
        let headers = self.extract_template_headers(template_content)?;
        let mut patterns = Vec::new();

        for header in headers {
            let pattern_string = self.convert_header_to_pattern_string(&header)?;
            patterns.push(pattern_string);
        }

        Ok(patterns)
    }

    /// Extract template headers from template content
    fn extract_template_headers(&mut self, template_content: &str) -> Result<Vec<TemplateHeader>> {
        let mut headers = Vec::new();

        for line in template_content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("template-match:")
                || trimmed.starts_with("{#") && trimmed.contains("template-match:")
            {
                // Handle both direct headers and Jinja comment headers
                let header_line = if trimmed.starts_with("{#") {
                    // Extract from Jinja comment: {# template-match: pattern #}
                    let start = trimmed.find("template-match:").unwrap();
                    let end = trimmed.rfind("#}").unwrap_or(trimmed.len());
                    trimmed[start..end].trim()
                } else {
                    trimmed
                };

                match self.header_parser.parse(header_line) {
                    Ok(header) => headers.push(header),
                    Err(e) => {
                        // Log warning but continue processing
                        tracing::warn!(
                            error = %e,
                            line = %line,
                            "Failed to parse template header, skipping"
                        );
                    }
                }
            }
        }

        Ok(headers)
    }

    /// Convert template header pattern to pattern string
    fn convert_header_to_pattern_string(&self, header: &TemplateHeader) -> Result<String> {
        // Check cache first
        if let Some(cached_pattern) = self.pattern_cache.get(&header.raw) {
            return Ok(cached_pattern.clone());
        }

        let pattern_string = match &header.pattern {
            MatchPattern::Exact(pattern) => pattern.clone(),
            MatchPattern::Regex(pattern) => format!("regex:{}", pattern),
            MatchPattern::HierarchicalPath(pattern) => pattern.clone(),
            MatchPattern::Glob(pattern) => pattern.clone(),
        };

        Ok(pattern_string)
    }

    /// Validate that slice extraction matches template expectations
    ///
    /// This ensures consistency between what templates expect and what gets sliced
    pub fn validate_slice_extraction(
        &mut self,
        template_content: &str,
        config_content: &str,
        _vendor_hint: Option<&str>,
    ) -> Result<TemplateSliceValidationResult> {
        let patterns = self.extract_slice_patterns(template_content)?;
        let mut validation_result = TemplateSliceValidationResult::new();

        for pattern in patterns {
            // Simple validation - check if pattern exists in config
            let matches = self.simple_pattern_match(&pattern, config_content);

            if matches == 0 {
                validation_result.add_warning(format!(
                    "Pattern '{}' matched no configuration sections",
                    pattern
                ));
            } else {
                validation_result.add_successful_match(pattern.clone(), matches);
            }
        }

        Ok(validation_result)
    }

    /// Simple pattern matching for validation
    fn simple_pattern_match(&self, pattern: &str, config_content: &str) -> usize {
        if pattern.starts_with("regex:") {
            // Basic regex matching
            let regex_pattern = &pattern[6..];
            if let Ok(re) = regex::Regex::new(regex_pattern) {
                re.find_iter(config_content).count()
            } else {
                0
            }
        } else if pattern.contains('*') {
            // Basic glob matching
            config_content
                .lines()
                .filter(|line| self.glob_match(pattern, line))
                .count()
        } else {
            // Exact matching
            config_content
                .lines()
                .filter(|line| line.contains(pattern))
                .count()
        }
    }

    /// Simple glob pattern matching
    fn glob_match(&self, pattern: &str, text: &str) -> bool {
        // Very basic glob matching for validation purposes
        if pattern == "*" {
            return true;
        }

        if let Some(prefix) = pattern.strip_suffix('*') {
            text.starts_with(prefix)
        } else if let Some(suffix) = pattern.strip_prefix('*') {
            text.ends_with(suffix)
        } else {
            pattern == text
        }
    }

    /// Create template-config consistency checker
    ///
    /// Ensures that templates and configurations are consistent with each other
    pub fn create_consistency_checker(&self) -> TemplateConfigConsistencyChecker {
        TemplateConfigConsistencyChecker::new()
    }

    /// Generate slice patterns automatically from configuration analysis
    ///
    /// Analyzes configuration content and suggests patterns that could be templated
    pub fn generate_automatic_slice_patterns(
        &self,
        config_content: &str,
        _vendor_hint: Option<&str>,
    ) -> Result<Vec<AutoGeneratedPattern>> {
        let mut patterns = Vec::new();

        // Simple analysis of configuration sections
        let sections = self.analyze_config_sections(config_content);

        // Generate patterns based on common network configuration sections
        for (section_name, lines) in sections {
            if self.is_templatable_section(&section_name) {
                patterns.push(AutoGeneratedPattern {
                    pattern: format!("{}*", section_name),
                    description: format!("All {} configurations", section_name),
                    confidence: self.calculate_pattern_confidence(&section_name),
                    suggested_template_name: self.suggest_template_name(&section_name),
                    example_content: lines.into_iter().take(5).collect::<Vec<_>>().join("\n"),
                });
            }
        }

        // Sort by confidence score
        patterns.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(patterns)
    }

    /// Simple configuration analysis to identify sections
    fn analyze_config_sections(&self, config_content: &str) -> HashMap<String, Vec<String>> {
        let mut sections: HashMap<String, Vec<String>> = HashMap::new();

        for line in config_content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('!') || trimmed.starts_with('#') {
                continue;
            }

            // Identify section start patterns
            let section_name = if trimmed.starts_with("interface ") {
                "interface"
            } else if trimmed.starts_with("router ") {
                "router"
            } else if trimmed.starts_with("vlan ") {
                "vlan"
            } else if trimmed.starts_with("access-list ") {
                "access-list"
            } else if trimmed.starts_with("route-map ") {
                "route-map"
            } else if trimmed.starts_with("policy-map ") {
                "policy-map"
            } else if trimmed.starts_with("class-map ") {
                "class-map"
            } else {
                "global"
            };

            sections
                .entry(section_name.to_string())
                .or_insert_with(Vec::new)
                .push(line.to_string());
        }

        sections
    }

    /// Check if a configuration section is suitable for templating
    fn is_templatable_section(&self, section_name: &str) -> bool {
        matches!(
            section_name.to_lowercase().as_str(),
            "interface"
                | "vlan"
                | "router"
                | "access-list"
                | "route-map"
                | "policy-map"
                | "class-map"
                | "bgp"
                | "ospf"
                | "eigrp"
                | "isis"
                | "snmp-server"
                | "ntp"
                | "logging"
        )
    }

    /// Calculate confidence score for automatically generated patterns
    fn calculate_pattern_confidence(&self, section_name: &str) -> f64 {
        match section_name.to_lowercase().as_str() {
            "interface" => 0.9,
            "vlan" => 0.8,
            "router" | "bgp" | "ospf" => 0.7,
            "access-list" | "route-map" => 0.6,
            "snmp-server" | "ntp" | "logging" => 0.5,
            _ => 0.3,
        }
    }

    /// Suggest template name based on section
    fn suggest_template_name(&self, section_name: &str) -> String {
        format!(
            "{}_template.j2",
            section_name.to_lowercase().replace("-", "_")
        )
    }
}

impl Default for TemplateDrivenSlicer {
    fn default() -> Self {
        Self::new().expect("Failed to create default TemplateDrivenSlicer")
    }
}

/// Result of slice validation against template expectations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateSliceValidationResult {
    /// Whether the validation passed overall
    pub is_valid: bool,
    /// Successful pattern matches
    pub successful_matches: Vec<PatternMatch>,
    /// Validation errors encountered
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
    /// Total patterns checked
    pub patterns_checked: usize,
}

impl TemplateSliceValidationResult {
    /// Create a new validation result
    pub fn new() -> Self {
        Self {
            is_valid: true,
            successful_matches: Vec::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            patterns_checked: 0,
        }
    }

    /// Add a successful pattern match
    pub fn add_successful_match(&mut self, pattern: String, matches_count: usize) {
        self.successful_matches.push(PatternMatch {
            pattern,
            matches_count,
        });
        self.patterns_checked += 1;
    }

    /// Add an error to the validation result
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.is_valid = false;
        self.patterns_checked += 1;
    }

    /// Add a warning to the validation result
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
        self.patterns_checked += 1;
    }

    /// Check if validation has any issues
    pub fn has_issues(&self) -> bool {
        !self.errors.is_empty() || !self.warnings.is_empty()
    }

    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.patterns_checked == 0 {
            100.0
        } else {
            (self.successful_matches.len() as f64 / self.patterns_checked as f64) * 100.0
        }
    }
}

impl Default for TemplateSliceValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Pattern match information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatch {
    /// The pattern that matched
    pub pattern: String,
    /// Number of matches found
    pub matches_count: usize,
}

/// Template-configuration consistency checker
#[derive(Debug)]
pub struct TemplateConfigConsistencyChecker {
    /// Rules for consistency checking
    rules: Vec<ConsistencyRule>,
}

impl TemplateConfigConsistencyChecker {
    /// Create a new consistency checker
    pub fn new() -> Self {
        Self {
            rules: Self::default_consistency_rules(),
        }
    }

    /// Check consistency between template and configuration
    pub fn check_consistency(
        &self,
        template_content: &str,
        config_content: &str,
        context: &HashMap<String, serde_json::Value>,
    ) -> Result<ConsistencyCheckResult> {
        let mut result = ConsistencyCheckResult::new();

        for rule in &self.rules {
            match rule.check(template_content, config_content, context) {
                Ok(check_result) => {
                    if !check_result.passed {
                        result.add_violation(ConsistencyViolation {
                            rule_name: rule.name.clone(),
                            description: check_result.message,
                            severity: rule.severity,
                        });
                    }
                }
                Err(e) => {
                    result.add_violation(ConsistencyViolation {
                        rule_name: rule.name.clone(),
                        description: format!("Rule check failed: {}", e),
                        severity: ConsistencySeverity::Warning,
                    });
                }
            }
        }

        Ok(result)
    }

    /// Get default consistency rules
    fn default_consistency_rules() -> Vec<ConsistencyRule> {
        vec![
            ConsistencyRule {
                name: "interface_references".to_string(),
                description: "Template should only reference interfaces that exist in config"
                    .to_string(),
                severity: ConsistencySeverity::Error,
                check_fn: Box::new(|template, config, _context| {
                    // Simple check for interface references
                    let template_interfaces = extract_interface_references(template);
                    let config_interfaces = extract_interface_definitions(config);

                    for template_interface in &template_interfaces {
                        if !config_interfaces.contains(template_interface) {
                            return Ok(RuleCheckResult {
                                passed: false,
                                message: format!(
                                    "Template references interface '{}' not found in config",
                                    template_interface
                                ),
                            });
                        }
                    }

                    Ok(RuleCheckResult {
                        passed: true,
                        message: "All template interface references are valid".to_string(),
                    })
                }),
            },
            ConsistencyRule {
                name: "vlan_references".to_string(),
                description: "Template should only reference VLANs that exist in config"
                    .to_string(),
                severity: ConsistencySeverity::Warning,
                check_fn: Box::new(|template, config, _context| {
                    let template_vlans = extract_vlan_references(template);
                    let config_vlans = extract_vlan_definitions(config);

                    for template_vlan in &template_vlans {
                        if !config_vlans.contains(template_vlan) {
                            return Ok(RuleCheckResult {
                                passed: false,
                                message: format!(
                                    "Template references VLAN '{}' not found in config",
                                    template_vlan
                                ),
                            });
                        }
                    }

                    Ok(RuleCheckResult {
                        passed: true,
                        message: "All template VLAN references are valid".to_string(),
                    })
                }),
            },
        ]
    }

    /// Add a custom consistency rule
    pub fn add_rule(&mut self, rule: ConsistencyRule) {
        self.rules.push(rule);
    }
}

impl Default for TemplateConfigConsistencyChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Consistency rule for template-config checking
pub struct ConsistencyRule {
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Severity of violations
    pub severity: ConsistencySeverity,
    /// Function to check the rule
    pub check_fn: Box<
        dyn Fn(&str, &str, &HashMap<String, serde_json::Value>) -> Result<RuleCheckResult>
            + Send
            + Sync,
    >,
}

impl std::fmt::Debug for ConsistencyRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConsistencyRule")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("severity", &self.severity)
            .field("check_fn", &"<function>")
            .finish()
    }
}

impl ConsistencyRule {
    /// Execute the consistency check
    pub fn check(
        &self,
        template_content: &str,
        config_content: &str,
        context: &HashMap<String, serde_json::Value>,
    ) -> Result<RuleCheckResult> {
        (self.check_fn)(template_content, config_content, context)
    }
}

/// Severity level for consistency violations
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ConsistencySeverity {
    Error,
    Warning,
    Info,
}

/// Result of a single rule check
#[derive(Debug, Clone)]
pub struct RuleCheckResult {
    /// Whether the rule check passed
    pub passed: bool,
    /// Message describing the result
    pub message: String,
}

/// Result of consistency checking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyCheckResult {
    /// Whether the overall check passed
    pub is_consistent: bool,
    /// Violations found
    pub violations: Vec<ConsistencyViolation>,
    /// Summary statistics
    pub stats: ConsistencyStats,
}

impl ConsistencyCheckResult {
    /// Create a new consistency check result
    pub fn new() -> Self {
        Self {
            is_consistent: true,
            violations: Vec::new(),
            stats: ConsistencyStats::default(),
        }
    }

    /// Add a consistency violation
    pub fn add_violation(&mut self, violation: ConsistencyViolation) {
        if violation.severity == ConsistencySeverity::Error {
            self.is_consistent = false;
            self.stats.errors += 1;
        } else if violation.severity == ConsistencySeverity::Warning {
            self.stats.warnings += 1;
        } else {
            self.stats.info += 1;
        }

        self.violations.push(violation);
    }

    /// Get violations by severity
    pub fn get_violations_by_severity(
        &self,
        severity: ConsistencySeverity,
    ) -> Vec<&ConsistencyViolation> {
        self.violations
            .iter()
            .filter(|v| v.severity == severity)
            .collect()
    }
}

impl Default for ConsistencyCheckResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Consistency violation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyViolation {
    /// Name of the rule that was violated
    pub rule_name: String,
    /// Description of the violation
    pub description: String,
    /// Severity of the violation
    pub severity: ConsistencySeverity,
}

/// Statistics about consistency checking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConsistencyStats {
    /// Number of errors found
    pub errors: usize,
    /// Number of warnings found
    pub warnings: usize,
    /// Number of info messages
    pub info: usize,
}

/// Automatically generated slice pattern suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoGeneratedPattern {
    /// The suggested pattern
    pub pattern: String,
    /// Description of what the pattern matches
    pub description: String,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Suggested template name
    pub suggested_template_name: String,
    /// Example content that would be matched
    pub example_content: String,
}

/// Template assignment for nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateAssignment {
    /// Unique identifier for the assignment
    pub id: Uuid,
    /// Node ID this assignment applies to
    pub node_id: Uuid,
    /// Template name or path
    pub template_name: String,
    /// Priority of this assignment (higher = more priority)
    pub priority: u32,
    /// Whether this assignment is active
    pub active: bool,
    /// Template context variables for this assignment
    pub context_variables: HashMap<String, serde_json::Value>,
    /// Template scope restrictions
    pub scope: Option<TemplateScope>,
}

impl TemplateAssignment {
    /// Create a new template assignment
    pub fn new(node_id: Uuid, template_name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            node_id,
            template_name,
            priority: 100,
            active: true,
            context_variables: HashMap::new(),
            scope: None,
        }
    }

    /// Set priority for the assignment
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Set active state
    pub fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Add context variables
    pub fn with_variables(mut self, variables: HashMap<String, serde_json::Value>) -> Self {
        self.context_variables.extend(variables);
        self
    }

    /// Set template scope
    pub fn with_scope(mut self, scope: TemplateScope) -> Self {
        self.scope = Some(scope);
        self
    }
}

// Helper functions for consistency checking

fn extract_interface_references(template: &str) -> Vec<String> {
    let mut interfaces = Vec::new();

    // Simple regex-based extraction - this could be enhanced
    let patterns = [
        r"interface\s+(\S+)",
        r"{{.*interface.*}}",
        r"GigabitEthernet\d+/\d+/\d+",
        r"FastEthernet\d+/\d+/\d+",
        r"TenGigabitEthernet\d+/\d+/\d+",
    ];

    for pattern in &patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            for cap in re.captures_iter(template) {
                if let Some(interface) = cap.get(1) {
                    interfaces.push(interface.as_str().to_string());
                }
            }
        }
    }

    interfaces
}

fn extract_interface_definitions(config: &str) -> Vec<String> {
    let mut interfaces = Vec::new();

    // Extract actual interface definitions from config
    if let Ok(re) = regex::Regex::new(r"interface\s+(\S+)") {
        for cap in re.captures_iter(config) {
            if let Some(interface) = cap.get(1) {
                interfaces.push(interface.as_str().to_string());
            }
        }
    }

    interfaces
}

fn extract_vlan_references(template: &str) -> Vec<String> {
    let mut vlans = Vec::new();

    if let Ok(re) = regex::Regex::new(r"vlan\s+(\d+)") {
        for cap in re.captures_iter(template) {
            if let Some(vlan) = cap.get(1) {
                vlans.push(vlan.as_str().to_string());
            }
        }
    }

    vlans
}

fn extract_vlan_definitions(config: &str) -> Vec<String> {
    let mut vlans = Vec::new();

    if let Ok(re) = regex::Regex::new(r"vlan\s+(\d+)") {
        for cap in re.captures_iter(config) {
            if let Some(vlan) = cap.get(1) {
                vlans.push(vlan.as_str().to_string());
            }
        }
    }

    vlans
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_driven_slicer_creation() {
        let slicer = TemplateDrivenSlicer::new();
        assert!(slicer.is_ok());
    }

    #[test]
    fn test_extract_template_headers_from_content() {
        let mut slicer = TemplateDrivenSlicer::new().unwrap();

        let template_content = r#"
{# template-match: interface.GigabitEthernet.* #}
interface GigabitEthernet{{ interface_id }}
 description {{ description }}
 ip address {{ ip_address }} {{ subnet_mask }}

{# template-match: /^vlan\.\d+$/ #}
vlan {{ vlan_id }}
 name {{ vlan_name }}
"#;

        let headers = slicer.extract_template_headers(template_content).unwrap();
        assert_eq!(headers.len(), 2);

        // Check first header
        match &headers[0].pattern {
            MatchPattern::Glob(pattern) => {
                assert_eq!(pattern, "interface.GigabitEthernet.*");
            }
            _ => panic!("Expected glob pattern"),
        }

        // Check second header
        match &headers[1].pattern {
            MatchPattern::Regex(pattern) => {
                assert_eq!(pattern, "^vlan\\.\\d+$");
            }
            _ => panic!("Expected regex pattern"),
        }
    }

    #[test]
    fn test_extract_slice_patterns() {
        let mut slicer = TemplateDrivenSlicer::new().unwrap();

        let template_content = r#"
{# template-match: interface.* #}
interface {{ interface_name }}
 description test
"#;

        let patterns = slicer.extract_slice_patterns(template_content).unwrap();
        assert_eq!(patterns.len(), 1);
    }

    #[test]
    fn test_slice_validation_result() {
        let mut result = TemplateSliceValidationResult::new();
        assert!(result.is_valid);
        assert_eq!(result.success_rate(), 100.0);

        result.add_successful_match("interface.*".to_string(), 5);
        assert_eq!(result.patterns_checked, 1);
        assert_eq!(result.successful_matches.len(), 1);

        result.add_error("Pattern failed".to_string());
        assert!(!result.is_valid);
        assert_eq!(result.patterns_checked, 2);
        assert_eq!(result.success_rate(), 50.0);

        result.add_warning("Warning message".to_string());
        assert!(result.has_issues());
    }

    #[test]
    fn test_consistency_checker_creation() {
        let checker = TemplateConfigConsistencyChecker::new();
        assert!(!checker.rules.is_empty());
    }

    #[test]
    fn test_template_assignment_builder() {
        let node_id = Uuid::new_v4();
        let assignment = TemplateAssignment::new(node_id, "test_template.j2".to_string())
            .with_priority(200)
            .with_active(true);

        assert_eq!(assignment.node_id, node_id);
        assert_eq!(assignment.template_name, "test_template.j2");
        assert_eq!(assignment.priority, 200);
        assert!(assignment.active);
    }

    #[test]
    fn test_extract_interface_references() {
        let template = r#"
interface GigabitEthernet0/0/1
interface {{ interface_name }}
interface FastEthernet1/0/1
"#;

        let interfaces = extract_interface_references(template);
        assert!(interfaces.contains(&"GigabitEthernet0/0/1".to_string()));
        assert!(interfaces.len() >= 2);
    }

    #[test]
    fn test_extract_vlan_references() {
        let template = r#"
vlan 100
vlan {{ vlan_id }}
vlan 200
"#;

        let vlans = extract_vlan_references(template);
        assert!(vlans.contains(&"100".to_string()));
        assert!(vlans.contains(&"200".to_string()));
    }

    #[test]
    fn test_is_templatable_section() {
        let slicer = TemplateDrivenSlicer::new().unwrap();

        assert!(slicer.is_templatable_section("interface"));
        assert!(slicer.is_templatable_section("vlan"));
        assert!(slicer.is_templatable_section("router"));
        assert!(!slicer.is_templatable_section("unknown_section"));
    }

    #[test]
    fn test_calculate_pattern_confidence() {
        let slicer = TemplateDrivenSlicer::new().unwrap();

        assert_eq!(slicer.calculate_pattern_confidence("interface"), 0.9);
        assert_eq!(slicer.calculate_pattern_confidence("vlan"), 0.8);
        assert!(slicer.calculate_pattern_confidence("unknown") < 0.5);
    }

    #[test]
    fn test_consistency_severity_ordering() {
        use std::mem::discriminant;

        // Test that we can distinguish between severities
        assert!(
            discriminant(&ConsistencySeverity::Error)
                != discriminant(&ConsistencySeverity::Warning)
        );
        assert!(
            discriminant(&ConsistencySeverity::Warning) != discriminant(&ConsistencySeverity::Info)
        );
    }

    #[test]
    fn test_auto_generated_pattern_creation() {
        let pattern = AutoGeneratedPattern {
            pattern: "interface.*".to_string(),
            description: "All interface configurations".to_string(),
            confidence: 0.9,
            suggested_template_name: "interface_template.j2".to_string(),
            example_content: "interface GigabitEthernet0/0/1\n description test".to_string(),
        };

        assert_eq!(pattern.confidence, 0.9);
        assert!(pattern.pattern.starts_with("interface"));
    }
}
