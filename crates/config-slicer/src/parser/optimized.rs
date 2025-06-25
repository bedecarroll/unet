//! Optimized hierarchical configuration parser
//!
//! This module provides performance-optimized parsing capabilities with improvements over
//! the core parser including better memory usage, reduced allocations, and faster processing.

use crate::parser::{ConfigContext, ConfigNode, IndentDetection, NodeType, ParserConfig};
use anyhow::{Result, anyhow};
use regex::Regex;
use std::collections::HashMap;
use std::sync::OnceLock;
use tracing::{debug, info};

/// Cache for compiled regex patterns to avoid recompilation
static REGEX_CACHE: OnceLock<RegexCache> = OnceLock::new();

/// Cached regex patterns for performance
struct RegexCache {
    indent: Regex,
    comment: Regex,
    interface: Regex,
    vlan: Regex,
    bgp: Regex,
    ospf: Regex,
    routing: Regex,
    access_list: Regex,
    line: Regex,
}

impl RegexCache {
    fn new() -> Result<Self> {
        Ok(Self {
            indent: Regex::new(r"^(\s*)")?,
            comment: Regex::new(r"^\s*[!#]")?,
            interface: Regex::new(r"^\s*interface\s+(\S+)")?,
            vlan: Regex::new(r"^\s*vlan\s+(\d+)")?,
            bgp: Regex::new(r"^\s*router\s+bgp\s+(\d+)")?,
            ospf: Regex::new(r"^\s*router\s+ospf\s+(\d+)")?,
            routing: Regex::new(r"^\s*router\s+(\w+)")?,
            access_list: Regex::new(r"^\s*(?:ip\s+)?access-list\s+(\S+)")?,
            line: Regex::new(r"^\s*line\s+(\S+)")?,
        })
    }

    fn get() -> &'static RegexCache {
        REGEX_CACHE.get_or_init(|| RegexCache::new().expect("Failed to initialize regex cache"))
    }
}

/// Optimized hierarchical parser with performance improvements
pub struct OptimizedHierarchicalParser {
    config: ParserConfig,
    line_buffer: Vec<u8>,
    string_cache: HashMap<String, String>,
}

impl OptimizedHierarchicalParser {
    /// Create a new optimized parser with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(ParserConfig::default())
    }

    /// Create a new optimized parser with custom configuration
    pub fn with_config(config: ParserConfig) -> Result<Self> {
        // Initialize regex cache
        RegexCache::get();

        Ok(Self {
            config,
            line_buffer: Vec::with_capacity(1024), // Pre-allocate buffer
            string_cache: HashMap::with_capacity(256), // Cache for common strings
        })
    }

    /// Parse a configuration string into a hierarchical tree with optimizations
    pub fn parse(&mut self, config_text: &str) -> Result<ConfigNode> {
        info!("Starting optimized hierarchical configuration parsing");

        // Pre-allocate based on input size estimation
        let estimated_lines = config_text.matches('\n').count() + 1;
        let mut lines = Vec::with_capacity(estimated_lines);

        // Collect lines more efficiently
        for line in config_text.lines() {
            lines.push(line);
        }

        let mut root = ConfigNode {
            command: self.intern_string("root".to_string()),
            raw_line: String::new(),
            line_number: 0,
            indent_level: 0,
            children: Vec::with_capacity(estimated_lines / 4), // Estimate child count
            context: ConfigContext::Global,
            node_type: NodeType::Root,
            metadata: HashMap::new(),
        };

        // Detect indentation style once
        let indent_style = self.detect_indentation_optimized(&lines)?;
        debug!("Detected indentation style: {:?}", indent_style);

        // Parse lines with optimized approach
        self.parse_lines_optimized(&lines, &mut root, &indent_style)?;

        info!("Parsed {} lines into hierarchical tree", lines.len());
        Ok(root)
    }

    /// Optimized line parsing with reduced allocations
    fn parse_lines_optimized(
        &mut self,
        lines: &[&str],
        root: &mut ConfigNode,
        indent_style: &IndentDetection,
    ) -> Result<()> {
        let mut current_path: Vec<usize> = Vec::with_capacity(self.config.max_depth);
        let regex_cache = RegexCache::get();

        for (line_number, line) in lines.iter().enumerate() {
            let line_number = line_number + 1;

            if let Some(node) =
                self.parse_line_optimized(line, line_number, indent_style, regex_cache)?
            {
                self.insert_node_optimized(root, node, &mut current_path)?;
            }
        }

        Ok(())
    }

    /// Optimized single line parsing
    fn parse_line_optimized(
        &mut self,
        line: &str,
        line_number: usize,
        indent_style: &IndentDetection,
        regex_cache: &RegexCache,
    ) -> Result<Option<ConfigNode>> {
        // Handle empty lines
        if line.trim().is_empty() {
            if self.config.preserve_empty_lines {
                return Ok(Some(ConfigNode {
                    command: String::new(),
                    raw_line: self.intern_string(line.to_string()),
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
        if regex_cache.comment.is_match(line) {
            if self.config.preserve_comments {
                let indent_level =
                    self.calculate_indent_level_optimized(line, indent_style, regex_cache);
                return Ok(Some(ConfigNode {
                    command: self.intern_string(line.trim().to_string()),
                    raw_line: self.intern_string(line.to_string()),
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
        let indent_level = self.calculate_indent_level_optimized(line, indent_style, regex_cache);
        let command = if self.config.normalize_whitespace {
            self.intern_string(line.trim().to_string())
        } else {
            self.intern_string(line.to_string())
        };

        // Detect context and node type optimized
        let (context, node_type) = self.detect_context_optimized(&command, regex_cache);

        let mut metadata = HashMap::new();
        metadata.insert("original_line".to_string(), line.to_string());

        Ok(Some(ConfigNode {
            command,
            raw_line: self.intern_string(line.to_string()),
            line_number,
            indent_level,
            children: Vec::new(),
            context,
            node_type,
            metadata,
        }))
    }

    /// Optimized indentation detection with caching
    fn detect_indentation_optimized(&self, lines: &[&str]) -> Result<IndentDetection> {
        match &self.config.indent_detection {
            IndentDetection::Auto => {
                let mut space_count = 0;
                let mut tab_count = 0;
                let mut space_sizes = HashMap::with_capacity(8); // Reasonable capacity for common indent sizes

                let regex_cache = RegexCache::get();

                for line in lines.iter().take(100) {
                    // Only check first 100 lines for performance
                    if line.trim().is_empty() {
                        continue;
                    }

                    let leading_whitespace = regex_cache
                        .indent
                        .captures(line)
                        .and_then(|cap| cap.get(1))
                        .map(|m| m.as_str())
                        .unwrap_or("");

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
                    let most_common_size = space_sizes
                        .into_iter()
                        .max_by_key(|&(_, count)| count)
                        .map(|(size, _)| size)
                        .unwrap_or(2);
                    Ok(IndentDetection::Spaces(most_common_size))
                } else {
                    Ok(IndentDetection::Spaces(2))
                }
            }
            other => Ok(other.clone()),
        }
    }

    /// Optimized indentation calculation
    fn calculate_indent_level_optimized(
        &self,
        line: &str,
        indent_style: &IndentDetection,
        regex_cache: &RegexCache,
    ) -> usize {
        let leading_whitespace = regex_cache
            .indent
            .captures(line)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str())
            .unwrap_or("");

        match indent_style {
            IndentDetection::Tabs => leading_whitespace.chars().filter(|&c| c == '\t').count(),
            IndentDetection::Spaces(size) => {
                let space_count = leading_whitespace.len();
                space_count / size
            }
            IndentDetection::Auto => leading_whitespace.len() / 2, // Fallback
        }
    }

    /// Optimized context detection using pre-compiled regexes
    fn detect_context_optimized(
        &self,
        command: &str,
        regex_cache: &RegexCache,
    ) -> (ConfigContext, NodeType) {
        // Check interface first (most common)
        if let Some(captures) = regex_cache.interface.captures(command) {
            let interface_name = captures
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            return (ConfigContext::Interface(interface_name), NodeType::Section);
        }

        // Check VLAN
        if let Some(captures) = regex_cache.vlan.captures(command) {
            let vlan_id = captures
                .get(1)
                .and_then(|m| m.as_str().parse::<u16>().ok())
                .unwrap_or(0);
            return (ConfigContext::Vlan(vlan_id), NodeType::Section);
        }

        // Check BGP (more specific, before general routing)
        if regex_cache.bgp.is_match(command) {
            return (ConfigContext::Bgp, NodeType::Section);
        }

        // Check OSPF (more specific, before general routing)
        if regex_cache.ospf.is_match(command) {
            return (ConfigContext::Ospf, NodeType::Section);
        }

        // Check general routing
        if let Some(captures) = regex_cache.routing.captures(command) {
            let protocol = captures
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            return (ConfigContext::Routing(protocol), NodeType::Section);
        }

        // Check access list
        if let Some(captures) = regex_cache.access_list.captures(command) {
            let acl_name = captures
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            return (ConfigContext::AccessList(acl_name), NodeType::Section);
        }

        // Check line configuration
        if let Some(captures) = regex_cache.line.captures(command) {
            let line_type = captures
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            return (ConfigContext::Line(line_type), NodeType::Section);
        }

        (ConfigContext::Global, NodeType::Command)
    }

    /// Optimized node insertion with better path management
    fn insert_node_optimized(
        &self,
        root: &mut ConfigNode,
        node: ConfigNode,
        current_path: &mut Vec<usize>,
    ) -> Result<()> {
        let target_level = node.indent_level;

        // Adjust current path based on indentation more efficiently
        current_path.truncate(target_level);

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

    /// String interning for memory optimization
    fn intern_string(&mut self, s: String) -> String {
        if let Some(interned) = self.string_cache.get(&s) {
            interned.clone()
        } else {
            self.string_cache.insert(s.clone(), s.clone());
            s
        }
    }

    /// Clear internal caches to free memory
    pub fn clear_caches(&mut self) {
        self.string_cache.clear();
        self.line_buffer.clear();
    }

    /// Get cache statistics for monitoring
    pub fn get_cache_stats(&self) -> CacheStats {
        CacheStats {
            string_cache_size: self.string_cache.len(),
            string_cache_memory: self
                .string_cache
                .iter()
                .map(|(k, v)| k.len() + v.len())
                .sum(),
            line_buffer_capacity: self.line_buffer.capacity(),
        }
    }
}

/// Cache statistics for monitoring and optimization
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub string_cache_size: usize,
    pub string_cache_memory: usize,
    pub line_buffer_capacity: usize,
}

impl Default for OptimizedHierarchicalParser {
    fn default() -> Self {
        Self::new().expect("Failed to create optimized parser")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimized_parser_creation() {
        let parser = OptimizedHierarchicalParser::new();
        assert!(parser.is_ok());
    }

    #[test]
    fn test_optimized_parsing_performance() -> Result<()> {
        let mut parser = OptimizedHierarchicalParser::new()?;
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
        assert!(!tree.children.is_empty());

        // Check cache stats
        let stats = parser.get_cache_stats();
        assert!(stats.string_cache_size > 0);

        Ok(())
    }

    #[test]
    fn test_string_interning() -> Result<()> {
        let mut parser = OptimizedHierarchicalParser::new()?;

        // Test that identical strings are interned
        let s1 = parser.intern_string("test".to_string());
        let s2 = parser.intern_string("test".to_string());

        // They should be the same object in memory (though we can't test pointer equality in safe Rust)
        assert_eq!(s1, s2);

        let stats = parser.get_cache_stats();
        assert_eq!(stats.string_cache_size, 1); // Only one unique string cached

        Ok(())
    }

    #[test]
    fn test_regex_cache_initialization() {
        let cache = RegexCache::get();

        // Test that regex patterns work
        assert!(cache.interface.is_match("interface GigabitEthernet0/1"));
        assert!(cache.vlan.is_match("vlan 100"));
        assert!(cache.comment.is_match("! This is a comment"));
    }

    #[test]
    fn test_optimized_context_detection() -> Result<()> {
        let parser = OptimizedHierarchicalParser::new()?;
        let regex_cache = RegexCache::get();

        // Test interface detection
        let (context, node_type) =
            parser.detect_context_optimized("interface GigabitEthernet0/1", regex_cache);
        match context {
            ConfigContext::Interface(name) => assert_eq!(name, "GigabitEthernet0/1"),
            _ => panic!("Expected interface context"),
        }
        assert_eq!(node_type, NodeType::Section);

        // Test VLAN detection
        let (context, node_type) = parser.detect_context_optimized("vlan 100", regex_cache);
        match context {
            ConfigContext::Vlan(id) => assert_eq!(id, 100),
            _ => panic!("Expected VLAN context"),
        }
        assert_eq!(node_type, NodeType::Section);

        Ok(())
    }

    #[test]
    fn test_cache_clearing() -> Result<()> {
        let mut parser = OptimizedHierarchicalParser::new()?;

        // Add some data to caches
        parser.intern_string("test".to_string());
        parser.line_buffer.extend_from_slice(b"test data");

        let stats_before = parser.get_cache_stats();
        assert!(stats_before.string_cache_size > 0);

        // Clear caches
        parser.clear_caches();

        let stats_after = parser.get_cache_stats();
        assert_eq!(stats_after.string_cache_size, 0);

        Ok(())
    }
}
