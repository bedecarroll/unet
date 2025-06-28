//! Core hierarchical configuration parser
//!
//! This module provides vendor-agnostic parsing capabilities for network device configurations,
//! building hierarchical trees that can be traversed and analyzed.

use anyhow::{Result, anyhow};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use tracing::{debug, info};

/// A hierarchical configuration tree node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigNode {
    /// The command or configuration line
    pub command: String,
    /// Raw line content including indentation
    pub raw_line: String,
    /// Line number in the original configuration
    pub line_number: usize,
    /// Indentation level (number of spaces/tabs)
    pub indent_level: usize,
    /// Child configuration nodes
    pub children: Vec<ConfigNode>,
    /// Configuration context (e.g., "interface", "routing", "vlan")
    pub context: ConfigContext,
    /// Node type classification
    pub node_type: NodeType,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Configuration context types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigContext {
    /// Global configuration context
    Global,
    /// Interface configuration context
    Interface(String),
    /// Routing protocol context
    Routing(String),
    /// VLAN configuration context
    Vlan(u16),
    /// Access control list context
    AccessList(String),
    /// BGP configuration context
    Bgp,
    /// OSPF configuration context
    Ospf,
    /// Line configuration context (console, vty, etc.)
    Line(String),
    /// Custom context for vendor-specific configurations
    Custom(String),
}

/// Node types for configuration classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    /// Root of the configuration tree
    Root,
    /// Section header (e.g., "interface GigabitEthernet0/1")
    Section,
    /// Configuration command
    Command,
    /// Comment line
    Comment,
    /// Empty line
    Empty,
    /// Unknown or unclassified line
    Unknown,
}

/// Parser configuration options
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Whether to preserve comments in the tree
    pub preserve_comments: bool,
    /// Whether to preserve empty lines
    pub preserve_empty_lines: bool,
    /// Indentation detection method
    pub indent_detection: IndentDetection,
    /// Maximum parsing depth to prevent infinite recursion
    pub max_depth: usize,
    /// Whether to normalize whitespace
    pub normalize_whitespace: bool,
}

/// Indentation detection methods
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndentDetection {
    /// Auto-detect indentation (spaces vs tabs)
    Auto,
    /// Use spaces for indentation counting
    Spaces(usize),
    /// Use tabs for indentation counting
    Tabs,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            preserve_comments: true,
            preserve_empty_lines: false,
            indent_detection: IndentDetection::Auto,
            max_depth: 50,
            normalize_whitespace: true,
        }
    }
}

/// Core hierarchical configuration parser
pub struct HierarchicalParser {
    /// Parser configuration
    config: ParserConfig,
    /// Indentation regex patterns
    indent_regex: Regex,
    /// Comment regex patterns
    comment_regex: Regex,
    /// Context detection patterns
    context_patterns: Vec<ContextPattern>,
}

/// Pattern for detecting configuration contexts
#[derive(Debug, Clone)]
struct ContextPattern {
    /// Regex pattern to match
    pattern: Regex,
    /// Context type to assign
    context_type: ConfigContext,
    /// Node type to assign
    node_type: NodeType,
}

impl HierarchicalParser {
    /// Create a new hierarchical parser with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(ParserConfig::default())
    }

    /// Create a new hierarchical parser with custom configuration
    pub fn with_config(config: ParserConfig) -> Result<Self> {
        let indent_regex = Regex::new(r"^(\s*)")?;
        let comment_regex = Regex::new(r"^\s*[!#]")?;

        let context_patterns = vec![
            // Interface patterns
            ContextPattern {
                pattern: Regex::new(r"^\s*interface\s+(\S+)")?,
                context_type: ConfigContext::Interface(String::new()),
                node_type: NodeType::Section,
            },
            // VLAN patterns
            ContextPattern {
                pattern: Regex::new(r"^\s*vlan\s+(\d+)")?,
                context_type: ConfigContext::Vlan(0),
                node_type: NodeType::Section,
            },
            // BGP patterns (more specific, should come before general routing)
            ContextPattern {
                pattern: Regex::new(r"^\s*router\s+bgp\s+(\d+)")?,
                context_type: ConfigContext::Bgp,
                node_type: NodeType::Section,
            },
            // OSPF patterns (more specific, should come before general routing)
            ContextPattern {
                pattern: Regex::new(r"^\s*router\s+ospf\s+(\d+)")?,
                context_type: ConfigContext::Ospf,
                node_type: NodeType::Section,
            },
            // General routing patterns (less specific, comes last)
            ContextPattern {
                pattern: Regex::new(r"^\s*router\s+(\w+)")?,
                context_type: ConfigContext::Routing(String::new()),
                node_type: NodeType::Section,
            },
            // Access list patterns
            ContextPattern {
                pattern: Regex::new(r"^\s*(?:ip\s+)?access-list\s+(\S+)")?,
                context_type: ConfigContext::AccessList(String::new()),
                node_type: NodeType::Section,
            },
            // Line configuration patterns
            ContextPattern {
                pattern: Regex::new(r"^\s*line\s+(\S+)")?,
                context_type: ConfigContext::Line(String::new()),
                node_type: NodeType::Section,
            },
        ];

        Ok(Self {
            config,
            indent_regex,
            comment_regex,
            context_patterns,
        })
    }

    /// Parse a configuration string into a hierarchical tree
    pub fn parse(&self, config_text: &str) -> Result<ConfigNode> {
        info!("Starting hierarchical configuration parsing");

        let lines: Vec<&str> = config_text.lines().collect();
        let mut root = ConfigNode {
            command: "root".to_string(),
            raw_line: String::new(),
            line_number: 0,
            indent_level: 0,
            children: Vec::new(),
            context: ConfigContext::Global,
            node_type: NodeType::Root,
            metadata: HashMap::new(),
        };

        // Detect indentation style
        let indent_style = self.detect_indentation(&lines)?;
        debug!("Detected indentation style: {:?}", indent_style);

        // Parse lines into hierarchical structure
        let mut current_path: Vec<usize> = Vec::new();
        let mut line_number = 0;

        for line in lines {
            line_number += 1;

            if let Some(node) = self.parse_line(line, line_number, &indent_style)? {
                self.insert_node(&mut root, node, &mut current_path)?;
            }
        }

        info!("Parsed {} lines into hierarchical tree", line_number);
        Ok(root)
    }

    /// Parse a single line into a configuration node
    fn parse_line(
        &self,
        line: &str,
        line_number: usize,
        indent_style: &IndentDetection,
    ) -> Result<Option<ConfigNode>> {
        // Handle empty lines
        if line.trim().is_empty() {
            if self.config.preserve_empty_lines {
                return Ok(Some(ConfigNode {
                    command: String::new(),
                    raw_line: line.to_string(),
                    line_number,
                    indent_level: 0,
                    children: Vec::new(),
                    context: ConfigContext::Global,
                    node_type: NodeType::Empty,
                    metadata: HashMap::new(),
                }));
            }
            return Ok(None);
        }

        // Handle comments
        if self.comment_regex.is_match(line) {
            if self.config.preserve_comments {
                let indent_level = self.calculate_indent_level(line, indent_style);
                return Ok(Some(ConfigNode {
                    command: line.trim().to_string(),
                    raw_line: line.to_string(),
                    line_number,
                    indent_level,
                    children: Vec::new(),
                    context: ConfigContext::Global,
                    node_type: NodeType::Comment,
                    metadata: HashMap::new(),
                }));
            }
            return Ok(None);
        }

        // Parse regular configuration line
        let indent_level = self.calculate_indent_level(line, indent_style);
        let command = if self.config.normalize_whitespace {
            line.trim().to_string()
        } else {
            line.to_string()
        };

        // Detect context and node type
        let (context, node_type) = self.detect_context(&command);

        let mut metadata = HashMap::new();
        metadata.insert("original_line".to_string(), line.to_string());

        Ok(Some(ConfigNode {
            command,
            raw_line: line.to_string(),
            line_number,
            indent_level,
            children: Vec::new(),
            context,
            node_type,
            metadata,
        }))
    }

    /// Detect indentation style from configuration lines
    fn detect_indentation(&self, lines: &[&str]) -> Result<IndentDetection> {
        match &self.config.indent_detection {
            IndentDetection::Auto => {
                let mut space_count = 0;
                let mut tab_count = 0;
                let mut space_sizes = HashMap::new();

                for line in lines {
                    if line.trim().is_empty() {
                        continue;
                    }

                    let leading_whitespace = self
                        .indent_regex
                        .captures(line)
                        .and_then(|cap| cap.get(1))
                        .map_or("", |m| m.as_str());

                    if leading_whitespace.contains('\t') {
                        tab_count += 1;
                    } else if !leading_whitespace.is_empty() {
                        space_count += 1;
                        let size = leading_whitespace.len();
                        *space_sizes.entry(size).or_insert(0) += 1;
                    }
                }

                if tab_count > space_count {
                    Ok(IndentDetection::Tabs)
                } else if space_count > 0 {
                    // Find most common space count
                    let most_common_size = space_sizes
                        .iter()
                        .max_by_key(|&(_, count)| count)
                        .map_or(2, |(&size, _)| size);
                    Ok(IndentDetection::Spaces(most_common_size))
                } else {
                    Ok(IndentDetection::Spaces(2)) // Default fallback
                }
            }
            other => Ok(other.clone()),
        }
    }

    /// Calculate indentation level for a line
    fn calculate_indent_level(&self, line: &str, indent_style: &IndentDetection) -> usize {
        let leading_whitespace = self
            .indent_regex
            .captures(line)
            .and_then(|cap| cap.get(1))
            .map_or("", |m| m.as_str());

        match indent_style {
            IndentDetection::Tabs => leading_whitespace.chars().filter(|&c| c == '\t').count(),
            IndentDetection::Spaces(size) => {
                let space_count = leading_whitespace.chars().filter(|&c| c == ' ').count();
                space_count / size
            }
            IndentDetection::Auto => {
                // This shouldn't happen as auto should be resolved by now
                leading_whitespace.len() / 2 // Fallback
            }
        }
    }

    /// Detect configuration context from command
    fn detect_context(&self, command: &str) -> (ConfigContext, NodeType) {
        for pattern in &self.context_patterns {
            if let Some(captures) = pattern.pattern.captures(command) {
                let context = match &pattern.context_type {
                    ConfigContext::Interface(_) => {
                        let interface_name = captures
                            .get(1)
                            .map(|m| m.as_str().to_string())
                            .unwrap_or_default();
                        ConfigContext::Interface(interface_name)
                    }
                    ConfigContext::Vlan(_) => {
                        let vlan_id = captures
                            .get(1)
                            .and_then(|m| m.as_str().parse::<u16>().ok())
                            .unwrap_or(0);
                        ConfigContext::Vlan(vlan_id)
                    }
                    ConfigContext::Routing(_) => {
                        let protocol = captures
                            .get(1)
                            .map(|m| m.as_str().to_string())
                            .unwrap_or_default();
                        ConfigContext::Routing(protocol)
                    }
                    ConfigContext::AccessList(_) => {
                        let acl_name = captures
                            .get(1)
                            .map(|m| m.as_str().to_string())
                            .unwrap_or_default();
                        ConfigContext::AccessList(acl_name)
                    }
                    ConfigContext::Line(_) => {
                        let line_type = captures
                            .get(1)
                            .map(|m| m.as_str().to_string())
                            .unwrap_or_default();
                        ConfigContext::Line(line_type)
                    }
                    other => other.clone(),
                };
                return (context, pattern.node_type.clone());
            }
        }

        (ConfigContext::Global, NodeType::Command)
    }

    /// Insert a node into the hierarchical tree at the appropriate location
    fn insert_node(
        &self,
        root: &mut ConfigNode,
        node: ConfigNode,
        current_path: &mut Vec<usize>,
    ) -> Result<()> {
        // Find the correct parent based on indentation level
        let target_level = node.indent_level;

        // Adjust current path based on indentation
        while current_path.len() > target_level {
            current_path.pop();
        }

        // Navigate to the insertion point
        let mut current = root;
        for &index in current_path.iter() {
            if index >= current.children.len() {
                return Err(anyhow!("Invalid path index: {}", index));
            }
            current = &mut current.children[index];
        }

        // Insert the node
        let new_index = current.children.len();
        current.children.push(node);

        // Update path if this is a section that can have children
        if current.children[new_index].node_type == NodeType::Section {
            current_path.push(new_index);
        }

        Ok(())
    }

    /// Validate the parsed configuration tree
    pub fn validate_tree(&self, root: &ConfigNode) -> Result<ValidationReport> {
        let mut report = ValidationReport::new();
        self.validate_node_recursive(root, 0, &mut report)?;
        Ok(report)
    }

    /// Recursively validate nodes in the tree
    fn validate_node_recursive(
        &self,
        node: &ConfigNode,
        depth: usize,
        report: &mut ValidationReport,
    ) -> Result<()> {
        // Check maximum depth
        if depth > self.config.max_depth {
            report.errors.push(format!(
                "Maximum parsing depth exceeded at line {}: {}",
                node.line_number, node.command
            ));
            return Ok(());
        }

        // Validate node structure
        if node.command.is_empty()
            && node.node_type != NodeType::Empty
            && node.node_type != NodeType::Root
        {
            report
                .warnings
                .push(format!("Empty command at line {}", node.line_number));
        }

        // Check for orphaned children (children with higher indentation than expected)
        for child in &node.children {
            if child.indent_level <= node.indent_level && node.node_type != NodeType::Root {
                report.warnings.push(format!(
                    "Unexpected indentation at line {}: child has same or lower indentation than parent",
                    child.line_number
                ));
            }

            self.validate_node_recursive(child, depth + 1, report)?;
        }

        report.total_nodes += 1;
        Ok(())
    }
}

/// Tree traversal utilities
pub struct TreeTraversal;

impl TreeTraversal {
    /// Perform depth-first traversal of the configuration tree
    pub fn depth_first<F>(root: &ConfigNode, mut visitor: F) -> Result<()>
    where
        F: FnMut(&ConfigNode) -> Result<()>,
    {
        Self::depth_first_recursive(root, &mut visitor)
    }

    /// Recursive helper for depth-first traversal
    fn depth_first_recursive<F>(node: &ConfigNode, visitor: &mut F) -> Result<()>
    where
        F: FnMut(&ConfigNode) -> Result<()>,
    {
        visitor(node)?;

        for child in &node.children {
            Self::depth_first_recursive(child, visitor)?;
        }

        Ok(())
    }

    /// Perform breadth-first traversal of the configuration tree
    pub fn breadth_first<F>(root: &ConfigNode, mut visitor: F) -> Result<()>
    where
        F: FnMut(&ConfigNode) -> Result<()>,
    {
        use std::collections::VecDeque;

        let mut queue = VecDeque::new();
        queue.push_back(root);

        while let Some(node) = queue.pop_front() {
            visitor(node)?;

            for child in &node.children {
                queue.push_back(child);
            }
        }

        Ok(())
    }

    /// Find nodes matching a predicate
    pub fn find_nodes<F>(root: &ConfigNode, predicate: F) -> Result<Vec<&ConfigNode>>
    where
        F: Fn(&ConfigNode) -> bool,
    {
        let mut matching_nodes = Vec::new();
        Self::find_nodes_recursive(root, &predicate, &mut matching_nodes);
        Ok(matching_nodes)
    }

    /// Recursive helper for finding nodes
    fn find_nodes_recursive<'a, F>(
        node: &'a ConfigNode,
        predicate: &F,
        matching_nodes: &mut Vec<&'a ConfigNode>,
    ) where
        F: Fn(&ConfigNode) -> bool,
    {
        if predicate(node) {
            matching_nodes.push(node);
        }

        for child in &node.children {
            Self::find_nodes_recursive(child, predicate, matching_nodes);
        }
    }

    /// Find nodes by context type
    pub fn find_by_context<'a>(
        root: &'a ConfigNode,
        context: &ConfigContext,
    ) -> Result<Vec<&'a ConfigNode>> {
        Self::find_nodes(root, |node| {
            std::mem::discriminant(&node.context) == std::mem::discriminant(context)
        })
    }

    /// Find nodes by command pattern
    pub fn find_by_command_pattern<'a>(
        root: &'a ConfigNode,
        pattern: &str,
    ) -> Result<Vec<&'a ConfigNode>> {
        let regex = Regex::new(pattern)?;
        Self::find_nodes(root, |node| regex.is_match(&node.command))
    }

    /// Get the path from root to a specific node
    #[must_use]
    pub fn get_path_to_node(root: &ConfigNode, target_line: usize) -> Option<Vec<usize>> {
        let mut path = Vec::new();
        if Self::find_path_recursive(root, target_line, &mut path) {
            Some(path)
        } else {
            None
        }
    }

    /// Recursive helper for finding path to node
    fn find_path_recursive(node: &ConfigNode, target_line: usize, path: &mut Vec<usize>) -> bool {
        if node.line_number == target_line {
            return true;
        }

        for (index, child) in node.children.iter().enumerate() {
            path.push(index);
            if Self::find_path_recursive(child, target_line, path) {
                return true;
            }
            path.pop();
        }

        false
    }
}

/// Validation report for parsed configuration
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// Total number of nodes processed
    pub total_nodes: usize,
    /// Validation errors
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationReport {
    /// Create a new validation report
    #[must_use]
    pub const fn new() -> Self {
        Self {
            total_nodes: 0,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Check if the validation passed without errors
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

impl fmt::Display for ConfigContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Global => write!(f, "global"),
            Self::Interface(name) => write!(f, "interface:{name}"),
            Self::Routing(protocol) => write!(f, "routing:{protocol}"),
            Self::Vlan(id) => write!(f, "vlan:{id}"),
            Self::AccessList(name) => write!(f, "acl:{name}"),
            Self::Bgp => write!(f, "bgp"),
            Self::Ospf => write!(f, "ospf"),
            Self::Line(line_type) => write!(f, "line:{line_type}"),
            Self::Custom(custom) => write!(f, "custom:{custom}"),
        }
    }
}

impl fmt::Display for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Root => write!(f, "root"),
            Self::Section => write!(f, "section"),
            Self::Command => write!(f, "command"),
            Self::Comment => write!(f, "comment"),
            Self::Empty => write!(f, "empty"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = HierarchicalParser::new();
        assert!(parser.is_ok());
    }

    #[test]
    fn test_simple_config_parsing() -> Result<()> {
        let parser = HierarchicalParser::new()?;
        let config = r"
hostname test-router
!
interface GigabitEthernet0/1
 description Uplink
 no shutdown
!
ip route 0.0.0.0 0.0.0.0 192.168.1.1
";

        let tree = parser.parse(config)?;
        assert_eq!(tree.node_type, NodeType::Root);
        assert!(!tree.children.is_empty());
        Ok(())
    }

    #[test]
    fn test_hierarchical_structure() -> Result<()> {
        let parser = HierarchicalParser::new()?;
        let config = r"
interface GigabitEthernet0/1
 description Test Interface
 ip address 192.168.1.1 255.255.255.0
 no shutdown
";

        let tree = parser.parse(config)?;

        // Find interface node
        let interface_nodes = TreeTraversal::find_by_command_pattern(&tree, r"interface\s+")?;
        assert_eq!(interface_nodes.len(), 1);

        let interface_node = interface_nodes[0];
        assert!(interface_node.children.len() >= 3); // description, ip address, no shutdown

        Ok(())
    }

    #[test]
    fn test_context_detection() -> Result<()> {
        let parser = HierarchicalParser::new()?;
        let config = r"
interface GigabitEthernet0/1
 description Test
vlan 100
 name Data_VLAN
router ospf 1
 network 192.168.1.0 0.0.0.255 area 0
";

        let tree = parser.parse(config)?;

        // Test interface context
        let interface_nodes = TreeTraversal::find_nodes(&tree, |node| {
            matches!(node.context, ConfigContext::Interface(_))
        })?;
        assert!(!interface_nodes.is_empty());

        // Test VLAN context
        let vlan_nodes = TreeTraversal::find_nodes(&tree, |node| {
            matches!(node.context, ConfigContext::Vlan(100))
        })?;
        assert!(!vlan_nodes.is_empty());

        // Test OSPF context
        let ospf_nodes =
            TreeTraversal::find_nodes(&tree, |node| matches!(node.context, ConfigContext::Ospf))?;
        assert!(!ospf_nodes.is_empty());

        Ok(())
    }

    #[test]
    fn test_tree_traversal() -> Result<()> {
        let parser = HierarchicalParser::new()?;
        let config = r"
hostname router
interface GigabitEthernet0/1
 description Test
 no shutdown
interface GigabitEthernet0/2
 description Test2
";

        let tree = parser.parse(config)?;

        let mut visited_commands = Vec::new();
        TreeTraversal::depth_first(&tree, |node| {
            if !node.command.is_empty() && node.node_type != NodeType::Root {
                visited_commands.push(node.command.clone());
            }
            Ok(())
        })?;

        assert!(visited_commands.contains(&"hostname router".to_string()));
        assert!(visited_commands.contains(&"interface GigabitEthernet0/1".to_string()));
        assert!(visited_commands.contains(&"description Test".to_string()));

        Ok(())
    }

    #[test]
    fn test_indentation_detection() -> Result<()> {
        let parser = HierarchicalParser::new()?;

        // Test space-based indentation
        let space_config = r"
interface GigabitEthernet0/1
  description Test
  no shutdown
";
        let tree = parser.parse(space_config)?;
        let interface_nodes = TreeTraversal::find_by_command_pattern(&tree, r"interface\s+")?;
        assert!(!interface_nodes.is_empty());

        Ok(())
    }

    #[test]
    fn test_validation() -> Result<()> {
        let parser = HierarchicalParser::new()?;
        let config = r"
hostname test
interface GigabitEthernet0/1
 description Test
";

        let tree = parser.parse(config)?;
        let report = parser.validate_tree(&tree)?;

        assert!(report.is_valid());
        assert!(report.total_nodes > 0);

        Ok(())
    }
}
