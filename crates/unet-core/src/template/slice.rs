//! Configuration slice extraction for template matching
//!
//! This module provides functionality to extract specific sections from network
//! device configurations based on template headers and vendor-specific parsing rules.

use anyhow::{Context, Result, anyhow};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::header::{HeaderParser, TemplateHeader};

/// Configuration slice extracted from a device configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigSlice {
    /// The configuration path that matched the template header
    pub path: String,
    /// The extracted configuration content
    pub content: String,
    /// Line numbers in the original configuration
    pub line_range: (usize, usize),
    /// Vendor-specific metadata
    pub metadata: SliceMetadata,
}

/// Metadata about the extracted configuration slice
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SliceMetadata {
    /// The network device vendor (cisco, juniper, arista, etc.)
    pub vendor: String,
    /// Configuration context (global, interface, routing, etc.)
    pub context: String,
    /// Whether this is a leaf or container configuration
    pub config_type: ConfigType,
    /// Additional vendor-specific attributes
    pub attributes: HashMap<String, String>,
}

/// Type of configuration element
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConfigType {
    /// Leaf configuration (single value)
    Leaf,
    /// Container configuration (has children)
    Container,
    /// List configuration (array of items)
    List,
}

/// Vendor-specific configuration parser
pub trait ConfigParser {
    /// Parse a configuration string and extract hierarchical structure
    fn parse(&self, config: &str) -> Result<ConfigTree>;

    /// Extract a specific slice based on a path
    fn extract_slice(&self, config: &str, path: &str) -> Result<Option<ConfigSlice>>;

    /// Get the vendor name this parser handles
    fn vendor(&self) -> &str;

    /// Validate configuration syntax
    fn validate_syntax(&self, config: &str) -> Result<ValidationResult>;
}

/// Hierarchical configuration tree representation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigTree {
    /// Root configuration nodes
    pub nodes: Vec<ConfigNode>,
    /// Total line count
    pub line_count: usize,
}

/// A node in the configuration tree
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigNode {
    /// Node name/key
    pub name: String,
    /// Node value (if leaf)
    pub value: Option<String>,
    /// Child nodes (if container)
    pub children: Vec<ConfigNode>,
    /// Line number in original config
    pub line_number: usize,
    /// Node type
    pub node_type: ConfigType,
    /// Full path from root
    pub path: String,
}

/// Configuration syntax validation result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the configuration is syntactically valid
    pub is_valid: bool,
    /// Validation errors found
    pub errors: Vec<ValidationError>,
    /// Warnings (non-fatal issues)
    pub warnings: Vec<String>,
}

/// Configuration validation error
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error message
    pub message: String,
    /// Line number where error occurred
    pub line_number: usize,
    /// Error severity
    pub severity: ErrorSeverity,
}

/// Error severity levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Fatal error - configuration cannot be processed
    Fatal,
    /// Error - significant issue but processing can continue
    Error,
    /// Warning - minor issue
    Warning,
}

/// Configuration slice extractor
pub struct SliceExtractor {
    /// Registered vendor-specific parsers
    parsers: HashMap<String, Box<dyn ConfigParser + Send + Sync>>,
    /// Header parser for template matching
    header_parser: HeaderParser,
}

impl SliceExtractor {
    /// Create a new slice extractor
    pub fn new() -> Self {
        let mut extractor = Self {
            parsers: HashMap::new(),
            header_parser: HeaderParser::new(),
        };

        // Register default parsers
        extractor.register_parser(Box::new(CiscoParser::new()));
        extractor.register_parser(Box::new(JuniperParser::new()));
        extractor.register_parser(Box::new(AristaParser::new()));
        extractor.register_parser(Box::new(GenericParser::new()));

        extractor
    }

    /// Register a vendor-specific parser
    pub fn register_parser(&mut self, parser: Box<dyn ConfigParser + Send + Sync>) {
        self.parsers.insert(parser.vendor().to_string(), parser);
    }

    /// Extract configuration slices based on template header
    pub fn extract_slices(
        &mut self,
        config: &str,
        vendor: &str,
        template_header: &TemplateHeader,
    ) -> Result<Vec<ConfigSlice>> {
        let parser = self
            .parsers
            .get(vendor)
            .ok_or_else(|| anyhow!("No parser registered for vendor: {}", vendor))?;

        let config_tree = parser
            .parse(config)
            .with_context(|| format!("Failed to parse {} configuration", vendor))?;

        let mut slices = Vec::new();

        // Find all paths that match the template header
        for node in &config_tree.nodes {
            self.collect_matching_slices(&node, template_header, config, vendor, &mut slices)?;
        }

        Ok(slices)
    }

    /// Recursively collect slices that match the template header
    fn collect_matching_slices(
        &mut self,
        node: &ConfigNode,
        template_header: &TemplateHeader,
        config: &str,
        vendor: &str,
        slices: &mut Vec<ConfigSlice>,
    ) -> Result<()> {
        // Check if this node's path matches the template header
        if self.header_parser.matches(template_header, &node.path)? {
            let parser = self.parsers.get(vendor).unwrap();
            if let Some(slice) = parser.extract_slice(config, &node.path)? {
                slices.push(slice);
            }
        }

        // Recursively check child nodes
        for child in &node.children {
            self.collect_matching_slices(child, template_header, config, vendor, slices)?;
        }

        Ok(())
    }

    /// Get supported vendors
    pub fn supported_vendors(&self) -> Vec<&str> {
        self.parsers.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for SliceExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Cisco IOS/IOS-XE configuration parser
#[derive(Debug)]
pub struct CiscoParser {
    /// Regex patterns for Cisco config parsing
    patterns: HashMap<String, Regex>,
}

impl CiscoParser {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();

        // Common Cisco configuration patterns
        patterns.insert(
            "interface".to_string(),
            Regex::new(r"^interface\s+(.+)$").expect("Valid regex"),
        );
        patterns.insert(
            "router".to_string(),
            Regex::new(r"^router\s+(\w+)(?:\s+(.+))?$").expect("Valid regex"),
        );
        patterns.insert(
            "vlan".to_string(),
            Regex::new(r"^vlan\s+(\d+)$").expect("Valid regex"),
        );

        Self { patterns }
    }
}

impl ConfigParser for CiscoParser {
    fn parse(&self, config: &str) -> Result<ConfigTree> {
        let mut nodes = Vec::new();
        let lines: Vec<&str> = config.lines().collect();
        let mut current_context: Vec<ConfigNode> = Vec::new();
        let mut line_number = 0;

        for line in &lines {
            line_number += 1;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('!') {
                continue;
            }

            // Determine indentation level
            let indent_level = line.len() - line.trim_start().len();

            // Parse the configuration line
            let node = self.parse_cisco_line(trimmed, line_number, indent_level)?;

            // Handle context changes based on indentation
            if indent_level == 0 {
                // Top-level configuration
                if !current_context.is_empty() {
                    nodes.push(current_context.remove(0));
                    current_context.clear();
                }
                current_context.push(node);
            } else {
                // Sub-configuration - add to current context
                if let Some(parent) = current_context.last_mut() {
                    parent.children.push(node);
                }
            }
        }

        // Add any remaining context
        if !current_context.is_empty() {
            nodes.extend(current_context);
        }

        Ok(ConfigTree {
            nodes,
            line_count: lines.len(),
        })
    }

    fn extract_slice(&self, config: &str, path: &str) -> Result<Option<ConfigSlice>> {
        let tree = self.parse(config)?;

        // Find the node matching the path
        if let Some(node) = self.find_node_by_path(&tree.nodes, path) {
            let content = self.extract_node_content(config, &node)?;

            let slice = ConfigSlice {
                path: path.to_string(),
                content,
                line_range: (node.line_number, node.line_number), // Simplified for now
                metadata: SliceMetadata {
                    vendor: "cisco".to_string(),
                    context: self.determine_context(path),
                    config_type: node.node_type.clone(),
                    attributes: HashMap::new(),
                },
            };

            return Ok(Some(slice));
        }

        Ok(None)
    }

    fn vendor(&self) -> &str {
        "cisco"
    }

    fn validate_syntax(&self, config: &str) -> Result<ValidationResult> {
        let mut errors = Vec::new();
        let warnings = Vec::new();
        let lines: Vec<&str> = config.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('!') {
                continue;
            }

            // Basic Cisco syntax validation
            if !self.is_valid_cisco_line(trimmed) {
                errors.push(ValidationError {
                    message: format!("Invalid Cisco configuration syntax: {}", trimmed),
                    line_number: line_num + 1,
                    severity: ErrorSeverity::Error,
                });
            }
        }

        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        })
    }
}

impl CiscoParser {
    fn parse_cisco_line(
        &self,
        line: &str,
        line_number: usize,
        _indent_level: usize,
    ) -> Result<ConfigNode> {
        // Determine the configuration path and type
        let (name, value, node_type, path) =
            if let Some(caps) = self.patterns.get("interface").unwrap().captures(line) {
                let interface_name = caps.get(1).unwrap().as_str();
                (
                    "interface".to_string(),
                    Some(interface_name.to_string()),
                    ConfigType::Container,
                    format!(
                        "interface.{}",
                        interface_name.replace(' ', "_").to_lowercase()
                    ),
                )
            } else if let Some(caps) = self.patterns.get("router").unwrap().captures(line) {
                let protocol = caps.get(1).unwrap().as_str();
                let instance = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                (
                    "router".to_string(),
                    Some(format!("{} {}", protocol, instance).trim().to_string()),
                    ConfigType::Container,
                    format!("router.{}.{}", protocol, instance.replace(' ', "_")),
                )
            } else if let Some(caps) = self.patterns.get("vlan").unwrap().captures(line) {
                let vlan_id = caps.get(1).unwrap().as_str();
                (
                    "vlan".to_string(),
                    Some(vlan_id.to_string()),
                    ConfigType::Container,
                    format!("vlan.{}", vlan_id),
                )
            } else {
                // Generic line parsing
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.is_empty() {
                    return Err(anyhow!("Empty configuration line"));
                }

                (
                    parts[0].to_string(),
                    if parts.len() > 1 {
                        Some(parts[1..].join(" "))
                    } else {
                        None
                    },
                    if parts.len() == 1 {
                        ConfigType::Leaf
                    } else {
                        ConfigType::Leaf
                    },
                    parts[0].to_string(),
                )
            };

        Ok(ConfigNode {
            name,
            value,
            children: Vec::new(),
            line_number,
            node_type,
            path,
        })
    }

    fn find_node_by_path<'a>(&self, nodes: &'a [ConfigNode], path: &str) -> Option<&'a ConfigNode> {
        for node in nodes {
            if node.path == path {
                return Some(node);
            }
            if let Some(found) = self.find_node_by_path(&node.children, path) {
                return Some(found);
            }
        }
        None
    }

    fn extract_node_content(&self, config: &str, node: &ConfigNode) -> Result<String> {
        let lines: Vec<&str> = config.lines().collect();
        if node.line_number == 0 || node.line_number > lines.len() {
            return Err(anyhow!("Invalid line number: {}", node.line_number));
        }

        // For now, just return the single line
        // In a full implementation, this would extract the entire block
        Ok(lines[node.line_number - 1].to_string())
    }

    fn determine_context(&self, path: &str) -> String {
        if path.starts_with("interface.") {
            "interface".to_string()
        } else if path.starts_with("router.") {
            "routing".to_string()
        } else if path.starts_with("vlan.") {
            "vlan".to_string()
        } else {
            "global".to_string()
        }
    }

    fn is_valid_cisco_line(&self, line: &str) -> bool {
        // Basic validation - check for common Cisco commands
        let valid_prefixes = [
            "interface",
            "router",
            "vlan",
            "ip",
            "no",
            "shutdown",
            "description",
            "switchport",
            "access-list",
            "hostname",
            "enable",
            "line",
            "username",
        ];

        valid_prefixes
            .iter()
            .any(|&prefix| line.starts_with(prefix))
            || line.starts_with(' ') // Indented lines are generally valid
    }
}

/// Juniper JunOS configuration parser
#[derive(Debug)]
pub struct JuniperParser;

impl JuniperParser {
    pub fn new() -> Self {
        Self
    }
}

impl ConfigParser for JuniperParser {
    fn parse(&self, _config: &str) -> Result<ConfigTree> {
        // Placeholder implementation for Juniper
        Ok(ConfigTree {
            nodes: Vec::new(),
            line_count: 0,
        })
    }

    fn extract_slice(&self, _config: &str, _path: &str) -> Result<Option<ConfigSlice>> {
        // Placeholder - would implement Juniper-specific extraction
        Ok(None)
    }

    fn vendor(&self) -> &str {
        "juniper"
    }

    fn validate_syntax(&self, _config: &str) -> Result<ValidationResult> {
        // Placeholder implementation
        Ok(ValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        })
    }
}

/// Arista EOS configuration parser
#[derive(Debug)]
pub struct AristaParser;

impl AristaParser {
    pub fn new() -> Self {
        Self
    }
}

impl ConfigParser for AristaParser {
    fn parse(&self, _config: &str) -> Result<ConfigTree> {
        // Placeholder implementation for Arista
        Ok(ConfigTree {
            nodes: Vec::new(),
            line_count: 0,
        })
    }

    fn extract_slice(&self, _config: &str, _path: &str) -> Result<Option<ConfigSlice>> {
        // Placeholder - would implement Arista-specific extraction
        Ok(None)
    }

    fn vendor(&self) -> &str {
        "arista"
    }

    fn validate_syntax(&self, _config: &str) -> Result<ValidationResult> {
        // Placeholder implementation
        Ok(ValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        })
    }
}

/// Generic line-based configuration parser
#[derive(Debug)]
pub struct GenericParser;

impl GenericParser {
    pub fn new() -> Self {
        Self
    }
}

impl ConfigParser for GenericParser {
    fn parse(&self, config: &str) -> Result<ConfigTree> {
        let mut nodes = Vec::new();
        let lines: Vec<&str> = config.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let node = ConfigNode {
                name: parts[0].to_string(),
                value: if parts.len() > 1 {
                    Some(parts[1..].join(" "))
                } else {
                    None
                },
                children: Vec::new(),
                line_number: line_num + 1,
                node_type: ConfigType::Leaf,
                path: parts[0].to_string(),
            };

            nodes.push(node);
        }

        Ok(ConfigTree {
            nodes,
            line_count: lines.len(),
        })
    }

    fn extract_slice(&self, config: &str, path: &str) -> Result<Option<ConfigSlice>> {
        let tree = self.parse(config)?;

        for node in &tree.nodes {
            if node.path == path {
                let lines: Vec<&str> = config.lines().collect();
                let content = lines[node.line_number - 1].to_string();

                let slice = ConfigSlice {
                    path: path.to_string(),
                    content,
                    line_range: (node.line_number, node.line_number),
                    metadata: SliceMetadata {
                        vendor: "generic".to_string(),
                        context: "global".to_string(),
                        config_type: node.node_type.clone(),
                        attributes: HashMap::new(),
                    },
                };

                return Ok(Some(slice));
            }
        }

        Ok(None)
    }

    fn vendor(&self) -> &str {
        "generic"
    }

    fn validate_syntax(&self, _config: &str) -> Result<ValidationResult> {
        // Generic validation - just check for empty lines
        Ok(ValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::header::{HeaderParser, MatchPattern};

    #[test]
    fn test_slice_extractor_creation() {
        let extractor = SliceExtractor::new();
        let vendors = extractor.supported_vendors();

        assert!(vendors.contains(&"cisco"));
        assert!(vendors.contains(&"juniper"));
        assert!(vendors.contains(&"arista"));
        assert!(vendors.contains(&"generic"));
    }

    #[test]
    fn test_cisco_parser_basic() {
        let parser = CiscoParser::new();
        let config = r#"
interface GigabitEthernet0/0
 description WAN interface
 ip address 192.168.1.1 255.255.255.0
!
router ospf 1
 network 192.168.1.0 0.0.0.255 area 0
"#;

        let tree = parser.parse(config).unwrap();
        assert!(!tree.nodes.is_empty());
    }

    #[test]
    fn test_cisco_interface_parsing() {
        let parser = CiscoParser::new();
        let config = "interface GigabitEthernet0/0\n description Test interface";

        let tree = parser.parse(config).unwrap();
        assert_eq!(tree.nodes.len(), 1);
        assert_eq!(tree.nodes[0].name, "interface");
        assert_eq!(tree.nodes[0].value, Some("GigabitEthernet0/0".to_string()));
        assert_eq!(tree.nodes[0].path, "interface.gigabitethernet0/0");
    }

    #[test]
    fn test_slice_extraction() {
        let mut extractor = SliceExtractor::new();
        let config = "interface GigabitEthernet0/0\n description Test";

        let mut header_parser = HeaderParser::new();
        let header = header_parser
            .parse("template-match: interface.gigabitethernet0/0")
            .unwrap();

        let slices = extractor.extract_slices(config, "cisco", &header).unwrap();
        assert!(!slices.is_empty());
        assert_eq!(slices[0].path, "interface.gigabitethernet0/0");
        assert_eq!(slices[0].metadata.vendor, "cisco");
        assert_eq!(slices[0].metadata.context, "interface");
    }

    #[test]
    fn test_generic_parser() {
        let parser = GenericParser::new();
        let config = "hostname router1\nip route 0.0.0.0 0.0.0.0 192.168.1.1";

        let tree = parser.parse(config).unwrap();
        assert_eq!(tree.nodes.len(), 2);
        assert_eq!(tree.nodes[0].name, "hostname");
        assert_eq!(tree.nodes[0].value, Some("router1".to_string()));
    }

    #[test]
    fn test_cisco_syntax_validation() {
        let parser = CiscoParser::new();

        let valid_config = "interface GigabitEthernet0/0\nip address 192.168.1.1 255.255.255.0";
        let result = parser.validate_syntax(valid_config).unwrap();
        assert!(result.is_valid);

        let invalid_config = "invalid-command with bad syntax";
        let result = parser.validate_syntax(invalid_config).unwrap();
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }
}
