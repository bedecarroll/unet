//! Vendor-specific configuration parsers
//!
//! This module provides specialized parsers for different network device vendors,
//! building upon the core hierarchical parser with vendor-specific patterns and behaviors.

use super::core::{
    ConfigContext, ConfigNode, HierarchicalParser, IndentDetection, NodeType, ParserConfig,
    TreeTraversal,
};
use anyhow::Result;
use regex::Regex;
use tracing::{debug, info};

/// Supported network device vendors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Vendor {
    /// Cisco IOS/IOS-XE/NX-OS
    Cisco,
    /// Juniper JunOS
    Juniper,
    /// Arista EOS
    Arista,
    /// Generic line-based configuration
    Generic,
}

/// Vendor-specific parser that extends the core hierarchical parser
pub struct VendorParser {
    /// Core hierarchical parser
    core_parser: HierarchicalParser,
    /// Vendor type
    vendor: Vendor,
    /// Vendor-specific patterns
    vendor_patterns: VendorPatterns,
}

/// Vendor-specific parsing patterns and behaviors
#[derive(Debug, Clone)]
struct VendorPatterns {
    /// Comment line patterns
    comment_patterns: Vec<Regex>,
    /// Section header patterns with contexts
    section_patterns: Vec<SectionPattern>,
    /// Command validation patterns
    command_patterns: Vec<CommandPattern>,
    /// Indentation characteristics
    indent_characteristics: IndentCharacteristics,
    /// End-of-section markers
    end_markers: Vec<Regex>,
}

/// Section pattern for vendor-specific contexts
#[derive(Debug, Clone)]
struct SectionPattern {
    /// Regex pattern to match section headers
    pattern: Regex,
    /// Context type to assign
    context_factory: fn(&str) -> ConfigContext,
    /// Whether this section can have children
    has_children: bool,
    /// Typical indentation for children
    child_indent: usize,
}

/// Command validation pattern
#[derive(Debug, Clone)]
struct CommandPattern {
    /// Pattern to match commands
    pattern: Regex,
    /// Command description
    description: String,
    /// Whether this is a deprecated command
    deprecated: bool,
}

/// Vendor-specific indentation characteristics
#[derive(Debug, Clone)]
struct IndentCharacteristics {
    /// Preferred indentation type
    preferred_indent: IndentDetection,
    /// Typical section indentation size
    section_indent: usize,
    /// Whether strict indentation is enforced
    strict_indentation: bool,
}

impl VendorParser {
    /// Create a new vendor-specific parser
    pub fn new(vendor: Vendor) -> Result<Self> {
        let config = ParserConfig::default();
        Self::with_config(vendor, config)
    }

    /// Create a new vendor-specific parser with custom configuration
    pub fn with_config(vendor: Vendor, config: ParserConfig) -> Result<Self> {
        let core_parser = HierarchicalParser::with_config(config)?;
        let vendor_patterns = Self::create_vendor_patterns(&vendor)?;

        Ok(Self {
            core_parser,
            vendor,
            vendor_patterns,
        })
    }

    /// Parse vendor-specific configuration
    pub fn parse(&self, config_text: &str) -> Result<ConfigNode> {
        info!("Parsing configuration for vendor: {:?}", self.vendor);

        // Pre-process the configuration text for vendor-specific handling
        let preprocessed = self.preprocess_config(config_text)?;

        // Parse using core parser
        let mut tree = self.core_parser.parse(&preprocessed)?;

        // Apply vendor-specific post-processing
        self.postprocess_tree(&mut tree)?;

        debug!("Vendor-specific parsing completed for {:?}", self.vendor);
        Ok(tree)
    }

    /// Pre-process configuration text for vendor-specific handling
    fn preprocess_config(&self, config_text: &str) -> Result<String> {
        let mut processed = config_text.to_string();

        match self.vendor {
            Vendor::Cisco => {
                // Handle Cisco-specific preprocessing
                processed = self.preprocess_cisco_config(&processed)?;
            }
            Vendor::Juniper => {
                // Handle Juniper-specific preprocessing
                processed = self.preprocess_juniper_config(&processed)?;
            }
            Vendor::Arista => {
                // Handle Arista-specific preprocessing
                processed = self.preprocess_arista_config(&processed)?;
            }
            Vendor::Generic => {
                // Generic preprocessing - minimal changes
            }
        }

        Ok(processed)
    }

    /// Apply vendor-specific post-processing to the parsed tree
    fn postprocess_tree(&self, tree: &mut ConfigNode) -> Result<()> {
        match self.vendor {
            Vendor::Cisco => self.postprocess_cisco_tree(tree),
            Vendor::Juniper => self.postprocess_juniper_tree(tree),
            Vendor::Arista => self.postprocess_arista_tree(tree),
            Vendor::Generic => Ok(()),
        }
    }

    /// Create vendor-specific patterns
    fn create_vendor_patterns(vendor: &Vendor) -> Result<VendorPatterns> {
        match vendor {
            Vendor::Cisco => Self::create_cisco_patterns(),
            Vendor::Juniper => Self::create_juniper_patterns(),
            Vendor::Arista => Self::create_arista_patterns(),
            Vendor::Generic => Self::create_generic_patterns(),
        }
    }

    /// Get vendor type
    pub fn vendor(&self) -> &Vendor {
        &self.vendor
    }

    /// Validate vendor-specific configuration syntax
    pub fn validate_vendor_syntax(&self, tree: &ConfigNode) -> Result<VendorValidationReport> {
        let mut report = VendorValidationReport::new(self.vendor.clone());

        // Perform vendor-specific validation
        self.validate_vendor_tree(tree, &mut report)?;

        Ok(report)
    }

    /// Perform vendor-specific tree validation
    fn validate_vendor_tree(
        &self,
        node: &ConfigNode,
        report: &mut VendorValidationReport,
    ) -> Result<()> {
        // Validate node against vendor patterns
        self.validate_node_against_patterns(node, report)?;

        // Recursively validate children
        for child in &node.children {
            self.validate_vendor_tree(child, report)?;
        }

        Ok(())
    }

    /// Validate a node against vendor-specific patterns
    fn validate_node_against_patterns(
        &self,
        node: &ConfigNode,
        report: &mut VendorValidationReport,
    ) -> Result<()> {
        if node.node_type == NodeType::Command {
            let mut command_recognized = false;

            for pattern in &self.vendor_patterns.command_patterns {
                if pattern.pattern.is_match(&node.command) {
                    command_recognized = true;
                    if pattern.deprecated {
                        report.warnings.push(format!(
                            "Deprecated command at line {}: {} ({})",
                            node.line_number, node.command, pattern.description
                        ));
                    }
                    break;
                }
            }

            if !command_recognized {
                report.warnings.push(format!(
                    "Unrecognized command at line {}: {}",
                    node.line_number, node.command
                ));
            }
        }

        Ok(())
    }
}

/// Validation report for vendor-specific syntax
#[derive(Debug, Clone)]
pub struct VendorValidationReport {
    /// Vendor being validated
    pub vendor: Vendor,
    /// Validation errors
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
    /// Vendor-specific recommendations
    pub recommendations: Vec<String>,
}

impl VendorValidationReport {
    /// Create a new vendor validation report
    pub fn new(vendor: Vendor) -> Self {
        Self {
            vendor,
            errors: Vec::new(),
            warnings: Vec::new(),
            recommendations: Vec::new(),
        }
    }

    /// Check if validation passed without errors
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

// Vendor-specific implementation methods will be added in separate files
// to keep this file manageable and allow for focused development

impl VendorParser {
    /// Cisco-specific preprocessing
    fn preprocess_cisco_config(&self, config: &str) -> Result<String> {
        let mut processed = config.to_string();

        // Handle Cisco line continuations (lines ending with backslash)
        let continuation_regex = Regex::new(r"\\\s*\n\s*")?;
        processed = continuation_regex.replace_all(&processed, " ").to_string();

        // Normalize interface names (e.g., "Gi0/1" -> "GigabitEthernet0/1")
        let interface_regex = Regex::new(r"\b(Gi|Fa|Te|Eth)(\d+/\d+)")?;
        processed = interface_regex
            .replace_all(&processed, |caps: &regex::Captures| {
                let prefix = match &caps[1] {
                    "Gi" => "GigabitEthernet",
                    "Fa" => "FastEthernet",
                    "Te" => "TenGigabitEthernet",
                    "Eth" => "Ethernet",
                    _ => &caps[1],
                };
                format!("{}{}", prefix, &caps[2])
            })
            .to_string();

        Ok(processed)
    }

    /// Juniper-specific preprocessing
    fn preprocess_juniper_config(&self, config: &str) -> Result<String> {
        let mut processed = config.to_string();

        // Handle Juniper brace-style configuration
        // Convert from brace style to indented style for consistent parsing
        let brace_regex = Regex::new(r"^(\s*\w[^{]*)\s*\{\s*$")?;
        let lines: Vec<&str> = processed.lines().collect();
        let mut converted_lines = Vec::new();
        let mut brace_stack = Vec::new();

        for line in lines {
            if brace_regex.is_match(line) {
                // Opening brace - store the section header
                let header = brace_regex
                    .captures(line)
                    .and_then(|cap| cap.get(1))
                    .map(|m| m.as_str().trim())
                    .unwrap_or(line.trim());
                converted_lines.push(header.to_string());
                brace_stack.push(converted_lines.len() - 1);
            } else if line.trim() == "}" {
                // Closing brace - pop from stack
                brace_stack.pop();
            } else if !line.trim().is_empty() {
                // Regular line - add appropriate indentation
                let indent_level = brace_stack.len();
                let indent = "    ".repeat(indent_level);
                converted_lines.push(format!("{}{}", indent, line.trim()));
            } else {
                // Empty line
                converted_lines.push(line.to_string());
            }
        }

        processed = converted_lines.join("\n");
        Ok(processed)
    }

    /// Arista-specific preprocessing
    fn preprocess_arista_config(&self, config: &str) -> Result<String> {
        let mut processed = config.to_string();

        // Handle Arista-specific interface naming
        let interface_regex = Regex::new(r"\b(Et|Ma)(\d+/\d+)")?;
        processed = interface_regex
            .replace_all(&processed, |caps: &regex::Captures| {
                let prefix = match &caps[1] {
                    "Et" => "Ethernet",
                    "Ma" => "Management",
                    _ => &caps[1],
                };
                format!("{}{}", prefix, &caps[2])
            })
            .to_string();

        Ok(processed)
    }

    /// Cisco-specific post-processing
    fn postprocess_cisco_tree(&self, tree: &mut ConfigNode) -> Result<()> {
        // Add Cisco-specific context enrichment
        TreeTraversal::depth_first(tree, |node| {
            if let ConfigContext::Interface(ref name) = node.context {
                // Add interface type metadata
                let mut metadata = node.metadata.clone();
                metadata.insert("vendor".to_string(), "cisco".to_string());
                if name.starts_with("GigabitEthernet") {
                    metadata.insert("interface_type".to_string(), "gigabit".to_string());
                } else if name.starts_with("FastEthernet") {
                    metadata.insert("interface_type".to_string(), "fast".to_string());
                }
                // Note: We can't modify the node here due to borrowing rules
                // This would need to be implemented differently in a real scenario
            }
            Ok(())
        })?;

        Ok(())
    }

    /// Juniper-specific post-processing
    fn postprocess_juniper_tree(&self, _tree: &mut ConfigNode) -> Result<()> {
        // Add Juniper-specific processing
        Ok(())
    }

    /// Arista-specific post-processing
    fn postprocess_arista_tree(&self, _tree: &mut ConfigNode) -> Result<()> {
        // Add Arista-specific processing
        Ok(())
    }

    /// Create Cisco-specific patterns
    fn create_cisco_patterns() -> Result<VendorPatterns> {
        let comment_patterns = vec![
            Regex::new(r"^\s*!")?, // Cisco comments start with !
        ];

        let section_patterns = vec![
            SectionPattern {
                pattern: Regex::new(r"^\s*interface\s+(\S+)")?,
                context_factory: |name| ConfigContext::Interface(name.to_string()),
                has_children: true,
                child_indent: 1,
            },
            SectionPattern {
                pattern: Regex::new(r"^\s*router\s+bgp\s+(\d+)")?,
                context_factory: |_| ConfigContext::Bgp,
                has_children: true,
                child_indent: 1,
            },
            SectionPattern {
                pattern: Regex::new(r"^\s*router\s+ospf\s+(\d+)")?,
                context_factory: |_| ConfigContext::Ospf,
                has_children: true,
                child_indent: 1,
            },
            SectionPattern {
                pattern: Regex::new(r"^\s*line\s+(\S+)")?,
                context_factory: |line_type| ConfigContext::Line(line_type.to_string()),
                has_children: true,
                child_indent: 1,
            },
        ];

        let command_patterns = vec![
            CommandPattern {
                pattern: Regex::new(r"^\s*hostname\s+\S+")?,
                description: "Set device hostname".to_string(),
                deprecated: false,
            },
            CommandPattern {
                pattern: Regex::new(r"^\s*ip\s+route\s+")?,
                description: "Static IP route".to_string(),
                deprecated: false,
            },
            CommandPattern {
                pattern: Regex::new(r"^\s*no\s+ip\s+domain-lookup")?,
                description: "Disable DNS lookups".to_string(),
                deprecated: false,
            },
        ];

        Ok(VendorPatterns {
            comment_patterns,
            section_patterns,
            command_patterns,
            indent_characteristics: IndentCharacteristics {
                preferred_indent: IndentDetection::Spaces(1),
                section_indent: 1,
                strict_indentation: true,
            },
            end_markers: vec![Regex::new(r"^\s*exit")?],
        })
    }

    /// Create Juniper-specific patterns
    fn create_juniper_patterns() -> Result<VendorPatterns> {
        let comment_patterns = vec![
            Regex::new(r"^\s*#")?,   // Juniper comments start with #
            Regex::new(r"^\s*/\*")?, // Juniper also supports /* */ comments
        ];

        let section_patterns = vec![
            SectionPattern {
                pattern: Regex::new(r"^\s*interfaces\s+(\S+)")?,
                context_factory: |name| ConfigContext::Interface(name.to_string()),
                has_children: true,
                child_indent: 4,
            },
            SectionPattern {
                pattern: Regex::new(r"^\s*protocols\s+bgp")?,
                context_factory: |_| ConfigContext::Bgp,
                has_children: true,
                child_indent: 4,
            },
            SectionPattern {
                pattern: Regex::new(r"^\s*protocols\s+ospf")?,
                context_factory: |_| ConfigContext::Ospf,
                has_children: true,
                child_indent: 4,
            },
        ];

        let command_patterns = vec![
            CommandPattern {
                pattern: Regex::new(r"^\s*set\s+system\s+host-name\s+\S+")?,
                description: "Set system hostname".to_string(),
                deprecated: false,
            },
            CommandPattern {
                pattern: Regex::new(r"^\s*set\s+routing-options\s+static\s+route")?,
                description: "Static route configuration".to_string(),
                deprecated: false,
            },
        ];

        Ok(VendorPatterns {
            comment_patterns,
            section_patterns,
            command_patterns,
            indent_characteristics: IndentCharacteristics {
                preferred_indent: IndentDetection::Spaces(4),
                section_indent: 4,
                strict_indentation: false,
            },
            end_markers: vec![],
        })
    }

    /// Create Arista-specific patterns
    fn create_arista_patterns() -> Result<VendorPatterns> {
        let comment_patterns = vec![
            Regex::new(r"^\s*!")?, // Arista uses ! for comments like Cisco
        ];

        let section_patterns = vec![
            SectionPattern {
                pattern: Regex::new(r"^\s*interface\s+(\S+)")?,
                context_factory: |name| ConfigContext::Interface(name.to_string()),
                has_children: true,
                child_indent: 3,
            },
            SectionPattern {
                pattern: Regex::new(r"^\s*router\s+bgp\s+(\d+)")?,
                context_factory: |_| ConfigContext::Bgp,
                has_children: true,
                child_indent: 3,
            },
        ];

        let command_patterns = vec![
            CommandPattern {
                pattern: Regex::new(r"^\s*hostname\s+\S+")?,
                description: "Set device hostname".to_string(),
                deprecated: false,
            },
            CommandPattern {
                pattern: Regex::new(r"^\s*ip\s+route\s+")?,
                description: "Static IP route".to_string(),
                deprecated: false,
            },
        ];

        Ok(VendorPatterns {
            comment_patterns,
            section_patterns,
            command_patterns,
            indent_characteristics: IndentCharacteristics {
                preferred_indent: IndentDetection::Spaces(3),
                section_indent: 3,
                strict_indentation: true,
            },
            end_markers: vec![],
        })
    }

    /// Create generic patterns
    fn create_generic_patterns() -> Result<VendorPatterns> {
        let comment_patterns = vec![
            Regex::new(r"^\s*[!#]")?, // Support both ! and # comments
        ];

        let section_patterns = vec![SectionPattern {
            pattern: Regex::new(r"^\s*\[(\w+)\]")?, // INI-style sections
            context_factory: |name| ConfigContext::Custom(name.to_string()),
            has_children: true,
            child_indent: 2,
        }];

        let command_patterns = vec![CommandPattern {
            pattern: Regex::new(r"^\s*\w+\s*=")?, // Key-value pairs
            description: "Configuration parameter".to_string(),
            deprecated: false,
        }];

        Ok(VendorPatterns {
            comment_patterns,
            section_patterns,
            command_patterns,
            indent_characteristics: IndentCharacteristics {
                preferred_indent: IndentDetection::Spaces(2),
                section_indent: 2,
                strict_indentation: false,
            },
            end_markers: vec![],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cisco_parser_creation() {
        let parser = VendorParser::new(Vendor::Cisco);
        assert!(parser.is_ok());
        assert_eq!(parser.unwrap().vendor(), &Vendor::Cisco);
    }

    #[test]
    fn test_juniper_parser_creation() {
        let parser = VendorParser::new(Vendor::Juniper);
        assert!(parser.is_ok());
        assert_eq!(parser.unwrap().vendor(), &Vendor::Juniper);
    }

    #[test]
    fn test_arista_parser_creation() {
        let parser = VendorParser::new(Vendor::Arista);
        assert!(parser.is_ok());
        assert_eq!(parser.unwrap().vendor(), &Vendor::Arista);
    }

    #[test]
    fn test_generic_parser_creation() {
        let parser = VendorParser::new(Vendor::Generic);
        assert!(parser.is_ok());
        assert_eq!(parser.unwrap().vendor(), &Vendor::Generic);
    }

    #[test]
    fn test_cisco_config_parsing() -> Result<()> {
        let parser = VendorParser::new(Vendor::Cisco)?;
        let config = r#"
hostname test-router
!
interface GigabitEthernet0/1
 description Uplink
 no shutdown
!
ip route 0.0.0.0 0.0.0.0 192.168.1.1
"#;

        let tree = parser.parse(config)?;
        assert_eq!(tree.node_type, NodeType::Root);

        // Find interface node
        let interface_nodes = TreeTraversal::find_by_command_pattern(&tree, r"interface\s+")?;
        assert!(!interface_nodes.is_empty());

        Ok(())
    }

    #[test]
    fn test_juniper_config_parsing() -> Result<()> {
        let parser = VendorParser::new(Vendor::Juniper)?;
        let config = r#"
system {
    host-name test-router;
}
interfaces {
    ge-0/0/1 {
        description "Uplink";
        unit 0 {
            family inet {
                address 192.168.1.1/24;
            }
        }
    }
}
"#;

        let tree = parser.parse(config)?;
        assert_eq!(tree.node_type, NodeType::Root);

        Ok(())
    }

    #[test]
    fn test_cisco_interface_normalization() -> Result<()> {
        let parser = VendorParser::new(Vendor::Cisco)?;
        let config = r#"
interface Gi0/1
 description Test
"#;

        let tree = parser.parse(config)?;
        let interface_nodes = TreeTraversal::find_by_command_pattern(&tree, r"interface\s+")?;
        assert!(!interface_nodes.is_empty());

        // The interface name should be normalized to GigabitEthernet
        let interface_node = interface_nodes[0];
        assert!(interface_node.command.contains("GigabitEthernet"));

        Ok(())
    }

    #[test]
    fn test_vendor_validation() -> Result<()> {
        let parser = VendorParser::new(Vendor::Cisco)?;
        let config = r#"
hostname valid-router
invalid-command-here
interface GigabitEthernet0/1
 description Valid
"#;

        let tree = parser.parse(config)?;
        let report = parser.validate_vendor_syntax(&tree)?;

        assert!(report.is_valid()); // No errors, just warnings
        assert!(!report.warnings.is_empty()); // Should warn about unrecognized command

        Ok(())
    }
}
