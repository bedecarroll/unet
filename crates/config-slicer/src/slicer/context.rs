//! Context-aware slicing for configuration extraction
//!
//! This module provides context-aware matching capabilities, allowing slices to be
//! extracted based on configuration context (interfaces, VLANs, routing, etc.) rather
//! than just text patterns.

use crate::parser::{ConfigContext, ConfigNode, NodeType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Context-based matching for configuration slicing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SliceContext {
    /// Required configuration contexts
    pub required_contexts: Vec<ConfigContext>,
    /// Forbidden configuration contexts
    pub forbidden_contexts: Vec<ConfigContext>,
    /// Required node types
    pub required_node_types: Vec<NodeType>,
    /// Forbidden node types
    pub forbidden_node_types: Vec<NodeType>,
    /// Minimum indentation level
    pub min_indent_level: Option<usize>,
    /// Maximum indentation level
    pub max_indent_level: Option<usize>,
    /// Required metadata keys and values
    pub required_metadata: HashMap<String, String>,
    /// Additional filtering criteria
    pub filters: Vec<ContextFilter>,
}

/// Additional filtering criteria for context matching
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContextFilter {
    /// Line number range filter
    LineRange { start: usize, end: usize },
    /// Command starts with filter
    CommandStartsWith(String),
    /// Command ends with filter
    CommandEndsWith(String),
    /// Command contains filter
    CommandContains(String),
    /// Custom filter function name
    Custom(String),
}

/// Context matcher for matching configuration contexts
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextMatcher {
    /// Context type to match
    pub context_type: ContextType,
    /// Optional specific value to match
    pub value: Option<String>,
    /// Whether to match exact values or use patterns
    pub exact_match: bool,
}

/// Types of configuration contexts that can be matched
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContextType {
    /// Any global configuration
    Global,
    /// Interface configurations
    Interface,
    /// Routing protocol configurations
    Routing,
    /// VLAN configurations
    Vlan,
    /// Access control list configurations
    AccessList,
    /// BGP configurations
    Bgp,
    /// OSPF configurations
    Ospf,
    /// Line configurations (console, vty, etc.)
    Line,
    /// Custom context type
    Custom(String),
}

impl SliceContext {
    /// Create a new slice context
    #[must_use]
    pub fn new() -> Self {
        Self {
            required_contexts: Vec::new(),
            forbidden_contexts: Vec::new(),
            required_node_types: Vec::new(),
            forbidden_node_types: Vec::new(),
            min_indent_level: None,
            max_indent_level: None,
            required_metadata: HashMap::new(),
            filters: Vec::new(),
        }
    }

    /// Add a required context
    #[must_use]
    pub fn require_context(mut self, context: ConfigContext) -> Self {
        self.required_contexts.push(context);
        self
    }

    /// Add a forbidden context
    #[must_use]
    pub fn forbid_context(mut self, context: ConfigContext) -> Self {
        self.forbidden_contexts.push(context);
        self
    }

    /// Add a required node type
    #[must_use]
    pub fn require_node_type(mut self, node_type: NodeType) -> Self {
        self.required_node_types.push(node_type);
        self
    }

    /// Add a forbidden node type
    #[must_use]
    pub fn forbid_node_type(mut self, node_type: NodeType) -> Self {
        self.forbidden_node_types.push(node_type);
        self
    }

    /// Set indentation level range
    #[must_use]
    pub const fn indent_range(mut self, min: Option<usize>, max: Option<usize>) -> Self {
        self.min_indent_level = min;
        self.max_indent_level = max;
        self
    }

    /// Add required metadata
    #[must_use]
    pub fn require_metadata(mut self, key: String, value: String) -> Self {
        self.required_metadata.insert(key, value);
        self
    }

    /// Add a context filter
    #[must_use]
    pub fn add_filter(mut self, filter: ContextFilter) -> Self {
        self.filters.push(filter);
        self
    }

    /// Check if a node matches this context
    #[must_use]
    pub fn matches_node(&self, node: &ConfigNode) -> bool {
        // Check required contexts
        if !self.required_contexts.is_empty() {
            let mut context_matched = false;
            for required_context in &self.required_contexts {
                if contexts_match(&node.context, required_context) {
                    context_matched = true;
                    break;
                }
            }
            if !context_matched {
                return false;
            }
        }

        // Check forbidden contexts
        for forbidden_context in &self.forbidden_contexts {
            if contexts_match(&node.context, forbidden_context) {
                return false;
            }
        }

        // Check required node types
        if !self.required_node_types.is_empty()
            && !self.required_node_types.contains(&node.node_type)
        {
            return false;
        }

        // Check forbidden node types
        if self.forbidden_node_types.contains(&node.node_type) {
            return false;
        }

        // Check indentation levels
        if let Some(min_indent) = self.min_indent_level {
            if node.indent_level < min_indent {
                return false;
            }
        }

        if let Some(max_indent) = self.max_indent_level {
            if node.indent_level > max_indent {
                return false;
            }
        }

        // Check required metadata
        for (key, value) in &self.required_metadata {
            if node.metadata.get(key) != Some(value) {
                return false;
            }
        }

        // Check additional filters
        for filter in &self.filters {
            if !self.matches_filter(node, filter) {
                return false;
            }
        }

        true
    }

    /// Check if a node matches a specific filter
    fn matches_filter(&self, node: &ConfigNode, filter: &ContextFilter) -> bool {
        match filter {
            ContextFilter::LineRange { start, end } => {
                node.line_number >= *start && node.line_number <= *end
            }
            ContextFilter::CommandStartsWith(prefix) => node.command.starts_with(prefix),
            ContextFilter::CommandEndsWith(suffix) => node.command.ends_with(suffix),
            ContextFilter::CommandContains(substring) => node.command.contains(substring),
            ContextFilter::Custom(_name) => {
                // Custom filters would be handled by a registry or plugin system
                true
            }
        }
    }
}

impl Default for SliceContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextMatcher {
    /// Create a new context matcher
    #[must_use]
    pub const fn new(context_type: ContextType) -> Self {
        Self {
            context_type,
            value: None,
            exact_match: true,
        }
    }

    /// Create a context matcher with a specific value
    #[must_use]
    pub const fn with_value(context_type: ContextType, value: String) -> Self {
        Self {
            context_type,
            value: Some(value),
            exact_match: true,
        }
    }

    /// Create a context matcher with pattern matching
    #[must_use]
    pub const fn with_pattern(context_type: ContextType, pattern: String) -> Self {
        Self {
            context_type,
            value: Some(pattern),
            exact_match: false,
        }
    }

    /// Check if a context matches this matcher
    #[must_use]
    pub fn matches(&self, context: &ConfigContext) -> bool {
        match (&self.context_type, context) {
            (ContextType::Global, ConfigContext::Global) => true,
            (ContextType::Interface, ConfigContext::Interface(name)) => self.matches_value(name),
            (ContextType::Routing, ConfigContext::Routing(protocol)) => {
                self.matches_value(protocol)
            }
            (ContextType::Vlan, ConfigContext::Vlan(id)) => self.matches_value(&id.to_string()),
            (ContextType::AccessList, ConfigContext::AccessList(name)) => self.matches_value(name),
            (ContextType::Bgp, ConfigContext::Bgp) => true,
            (ContextType::Ospf, ConfigContext::Ospf) => true,
            (ContextType::Line, ConfigContext::Line(line_type)) => self.matches_value(line_type),
            (ContextType::Custom(expected), ConfigContext::Custom(actual)) => {
                if self.exact_match {
                    expected == actual
                } else {
                    actual.contains(expected)
                }
            }
            _ => false,
        }
    }

    /// Check if a value matches the matcher's value/pattern
    fn matches_value(&self, value: &str) -> bool {
        match &self.value {
            Some(expected) => {
                if self.exact_match {
                    value == expected
                } else {
                    value.contains(expected)
                }
            }
            None => true, // No specific value required
        }
    }
}

/// Helper function to check if two contexts match
fn contexts_match(actual: &ConfigContext, expected: &ConfigContext) -> bool {
    match (actual, expected) {
        (ConfigContext::Global, ConfigContext::Global) => true,
        (ConfigContext::Interface(a), ConfigContext::Interface(e)) => a == e,
        (ConfigContext::Routing(a), ConfigContext::Routing(e)) => a == e,
        (ConfigContext::Vlan(a), ConfigContext::Vlan(e)) => a == e,
        (ConfigContext::AccessList(a), ConfigContext::AccessList(e)) => a == e,
        (ConfigContext::Bgp, ConfigContext::Bgp) => true,
        (ConfigContext::Ospf, ConfigContext::Ospf) => true,
        (ConfigContext::Line(a), ConfigContext::Line(e)) => a == e,
        (ConfigContext::Custom(a), ConfigContext::Custom(e)) => a == e,
        _ => false,
    }
}

/// Builder for creating slice contexts
pub struct SliceContextBuilder {
    context: SliceContext,
}

impl SliceContextBuilder {
    /// Start building a new slice context
    #[must_use]
    pub fn new() -> Self {
        Self {
            context: SliceContext::new(),
        }
    }

    /// Add interface context requirement
    #[must_use]
    pub fn interfaces_only(mut self) -> Self {
        self.context = self.context.forbid_context(ConfigContext::Global);
        self
    }

    /// Add global context requirement
    #[must_use]
    pub fn global_only(mut self) -> Self {
        self.context = self.context.require_context(ConfigContext::Global);
        self
    }

    /// Add commands only (no comments, no empty lines)
    #[must_use]
    pub fn commands_only(mut self) -> Self {
        self.context = self
            .context
            .require_node_type(NodeType::Command)
            .forbid_node_type(NodeType::Comment)
            .forbid_node_type(NodeType::Empty);
        self
    }

    /// Add specific indentation level
    #[must_use]
    pub fn at_indent_level(mut self, level: usize) -> Self {
        self.context = self.context.indent_range(Some(level), Some(level));
        self
    }

    /// Add minimum indentation level
    #[must_use]
    pub fn min_indent_level(mut self, level: usize) -> Self {
        let max_level = self.context.max_indent_level;
        self.context = self.context.indent_range(Some(level), max_level);
        self
    }

    /// Add maximum indentation level
    #[must_use]
    pub fn max_indent_level(mut self, level: usize) -> Self {
        let min_level = self.context.min_indent_level;
        self.context = self.context.indent_range(min_level, Some(level));
        self
    }

    /// Build the slice context
    #[must_use]
    pub fn build(self) -> SliceContext {
        self.context
    }
}

impl Default for SliceContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::NodeType;

    fn create_test_node(
        command: &str,
        context: ConfigContext,
        node_type: NodeType,
        indent_level: usize,
        line_number: usize,
    ) -> ConfigNode {
        ConfigNode {
            command: command.to_string(),
            raw_line: command.to_string(),
            line_number,
            indent_level,
            children: vec![],
            context,
            node_type,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_slice_context_basic_matching() {
        let context = SliceContext::new()
            .require_context(ConfigContext::Interface("GigabitEthernet0/1".to_string()));

        let node = create_test_node(
            "ip address 192.168.1.1 255.255.255.0",
            ConfigContext::Interface("GigabitEthernet0/1".to_string()),
            NodeType::Command,
            1,
            10,
        );

        assert!(context.matches_node(&node));
    }

    #[test]
    fn test_slice_context_forbidden_context() {
        let context = SliceContext::new().forbid_context(ConfigContext::Global);

        let global_node = create_test_node(
            "hostname router1",
            ConfigContext::Global,
            NodeType::Command,
            0,
            1,
        );

        let interface_node = create_test_node(
            "ip address 192.168.1.1 255.255.255.0",
            ConfigContext::Interface("GigabitEthernet0/1".to_string()),
            NodeType::Command,
            1,
            10,
        );

        assert!(!context.matches_node(&global_node));
        assert!(context.matches_node(&interface_node));
    }

    #[test]
    fn test_slice_context_node_type_filtering() {
        let context = SliceContext::new()
            .require_node_type(NodeType::Command)
            .forbid_node_type(NodeType::Comment);

        let command_node = create_test_node(
            "ip address 192.168.1.1 255.255.255.0",
            ConfigContext::Global,
            NodeType::Command,
            0,
            1,
        );

        let comment_node = create_test_node(
            "! This is a comment",
            ConfigContext::Global,
            NodeType::Comment,
            0,
            2,
        );

        assert!(context.matches_node(&command_node));
        assert!(!context.matches_node(&comment_node));
    }

    #[test]
    fn test_slice_context_indent_level_filtering() {
        let context = SliceContext::new().indent_range(Some(1), Some(2));

        let level_0_node = create_test_node(
            "interface GigabitEthernet0/1",
            ConfigContext::Global,
            NodeType::Section,
            0,
            1,
        );

        let level_1_node = create_test_node(
            "ip address 192.168.1.1 255.255.255.0",
            ConfigContext::Interface("GigabitEthernet0/1".to_string()),
            NodeType::Command,
            1,
            2,
        );

        let level_3_node = create_test_node(
            "nested command",
            ConfigContext::Interface("GigabitEthernet0/1".to_string()),
            NodeType::Command,
            3,
            3,
        );

        assert!(!context.matches_node(&level_0_node));
        assert!(context.matches_node(&level_1_node));
        assert!(!context.matches_node(&level_3_node));
    }

    #[test]
    fn test_context_matcher() {
        let interface_matcher =
            ContextMatcher::with_value(ContextType::Interface, "GigabitEthernet0/1".to_string());

        let vlan_matcher = ContextMatcher::new(ContextType::Vlan);

        assert!(
            interface_matcher.matches(&ConfigContext::Interface("GigabitEthernet0/1".to_string()))
        );
        assert!(
            !interface_matcher.matches(&ConfigContext::Interface("FastEthernet0/1".to_string()))
        );
        assert!(!interface_matcher.matches(&ConfigContext::Global));

        assert!(vlan_matcher.matches(&ConfigContext::Vlan(100)));
        assert!(vlan_matcher.matches(&ConfigContext::Vlan(200)));
        assert!(!vlan_matcher.matches(&ConfigContext::Global));
    }

    #[test]
    fn test_context_filter_matching() {
        let context = SliceContext::new()
            .add_filter(ContextFilter::CommandStartsWith("ip".to_string()))
            .add_filter(ContextFilter::LineRange { start: 10, end: 20 });

        let matching_node = create_test_node(
            "ip address 192.168.1.1 255.255.255.0",
            ConfigContext::Global,
            NodeType::Command,
            0,
            15,
        );

        let non_matching_command = create_test_node(
            "hostname router1",
            ConfigContext::Global,
            NodeType::Command,
            0,
            15,
        );

        let non_matching_line = create_test_node(
            "ip address 192.168.1.1 255.255.255.0",
            ConfigContext::Global,
            NodeType::Command,
            0,
            25,
        );

        assert!(context.matches_node(&matching_node));
        assert!(!context.matches_node(&non_matching_command));
        assert!(!context.matches_node(&non_matching_line));
    }

    #[test]
    fn test_slice_context_builder() {
        let context = SliceContextBuilder::new()
            .interfaces_only()
            .commands_only()
            .min_indent_level(1)
            .build();

        let valid_node = create_test_node(
            "ip address 192.168.1.1 255.255.255.0",
            ConfigContext::Interface("GigabitEthernet0/1".to_string()),
            NodeType::Command,
            1,
            10,
        );

        let invalid_context = create_test_node(
            "hostname router1",
            ConfigContext::Global,
            NodeType::Command,
            1,
            5,
        );

        let invalid_type = create_test_node(
            "! Comment",
            ConfigContext::Interface("GigabitEthernet0/1".to_string()),
            NodeType::Comment,
            1,
            8,
        );

        let invalid_indent = create_test_node(
            "ip address 192.168.1.1 255.255.255.0",
            ConfigContext::Interface("GigabitEthernet0/1".to_string()),
            NodeType::Command,
            0,
            10,
        );

        assert!(context.matches_node(&valid_node));
        assert!(!context.matches_node(&invalid_context));
        assert!(!context.matches_node(&invalid_type));
        assert!(!context.matches_node(&invalid_indent));
    }
}
