//! Configuration slicing algorithms
//!
//! This module provides various algorithms for extracting specific sections (slices)
//! from hierarchical configuration trees. It supports glob pattern matching,
//! regex-based extraction, hierarchical slicing, and context-aware selection.

use crate::error::{Error, Result};
use crate::parser::ConfigNode;
use crate::slicer::algorithms::{SliceAlgorithm, SliceResult};
use crate::slicer::context::{ContextMatcher, SliceContext};
use crate::slicer::patterns::{GlobPattern, RegexPattern, SlicePattern};
use regex::Regex;
use std::collections::HashMap;
use tracing::{debug, warn};

/// Main interface for configuration slicing operations
pub struct ConfigSlicer {
    extractors: HashMap<String, Box<dyn SliceAlgorithm>>,
}

impl ConfigSlicer {
    /// Create a new ConfigSlicer with default extractors
    pub fn new() -> Self {
        let mut slicer = Self {
            extractors: HashMap::new(),
        };

        // Register default extractors
        slicer.register_extractor("glob", Box::new(GlobExtractor::new()));
        slicer.register_extractor("regex", Box::new(RegexExtractor::new()));
        slicer.register_extractor("hierarchical", Box::new(HierarchicalExtractor::new()));
        slicer.register_extractor("context", Box::new(ContextExtractor::new()));

        slicer
    }

    /// Register a custom slice extractor
    pub fn register_extractor(&mut self, name: &str, extractor: Box<dyn SliceAlgorithm>) {
        self.extractors.insert(name.to_string(), extractor);
    }

    /// Extract configuration slices using the specified algorithm
    pub fn extract_slice(
        &self,
        config_tree: &ConfigNode,
        pattern: &SlicePattern,
        context: Option<&SliceContext>,
    ) -> Result<SliceResult> {
        let extractor_name = match pattern {
            SlicePattern::Glob(_) => "glob",
            SlicePattern::Regex(_) => "regex",
            SlicePattern::Hierarchical(_) => "hierarchical",
            SlicePattern::Context(_) => "context",
        };

        let extractor = self
            .extractors
            .get(extractor_name)
            .ok_or_else(|| Error::UnsupportedPattern(extractor_name.to_string()))?;

        debug!("Extracting slice using {} algorithm", extractor_name);
        extractor.extract(config_tree, pattern, context)
    }

    /// Extract multiple slices with different patterns
    pub fn extract_multiple_slices(
        &self,
        config_tree: &ConfigNode,
        patterns: &[SlicePattern],
        context: Option<&SliceContext>,
    ) -> Result<Vec<SliceResult>> {
        let mut results = Vec::new();

        for pattern in patterns {
            match self.extract_slice(config_tree, pattern, context) {
                Ok(result) => results.push(result),
                Err(e) => {
                    warn!("Failed to extract slice with pattern {:?}: {}", pattern, e);
                    // Continue with other patterns instead of failing completely
                }
            }
        }

        Ok(results)
    }

    /// Get available extractor names
    pub fn available_extractors(&self) -> Vec<String> {
        self.extractors.keys().cloned().collect()
    }
}

impl Default for ConfigSlicer {
    fn default() -> Self {
        Self::new()
    }
}

/// Glob pattern-based slice extractor
pub struct GlobExtractor;

impl GlobExtractor {
    pub fn new() -> Self {
        Self
    }
}

impl SliceAlgorithm for GlobExtractor {
    fn extract(
        &self,
        config_tree: &ConfigNode,
        pattern: &SlicePattern,
        context: Option<&SliceContext>,
    ) -> Result<SliceResult> {
        let glob_pattern = match pattern {
            SlicePattern::Glob(p) => p,
            _ => return Err(Error::InvalidPattern("Expected glob pattern".to_string())),
        };

        let mut matches = Vec::new();
        extract_glob_matches(config_tree, glob_pattern, &mut matches, "")?;

        // Apply context filtering if provided
        if let Some(ctx) = context {
            let filtered_matches: Vec<_> = matches
                .into_iter()
                .filter(|node| ctx.matches_node(node))
                .collect();
            matches = filtered_matches;
        }

        Ok(SliceResult {
            matches,
            pattern: pattern.clone(),
            metadata: HashMap::new(),
        })
    }
}

/// Regex pattern-based slice extractor
pub struct RegexExtractor;

impl RegexExtractor {
    pub fn new() -> Self {
        Self
    }
}

impl SliceAlgorithm for RegexExtractor {
    fn extract(
        &self,
        config_tree: &ConfigNode,
        pattern: &SlicePattern,
        context: Option<&SliceContext>,
    ) -> Result<SliceResult> {
        let regex_pattern = match pattern {
            SlicePattern::Regex(p) => p,
            _ => return Err(Error::InvalidPattern("Expected regex pattern".to_string())),
        };

        let mut matches = Vec::new();
        extract_regex_matches(config_tree, regex_pattern, &mut matches)?;

        // Apply context filtering if provided
        if let Some(ctx) = context {
            let filtered_matches: Vec<_> = matches
                .into_iter()
                .filter(|node| ctx.matches_node(node))
                .collect();
            matches = filtered_matches;
        }

        Ok(SliceResult {
            matches,
            pattern: pattern.clone(),
            metadata: HashMap::new(),
        })
    }
}

/// Hierarchical slice extractor
pub struct HierarchicalExtractor;

impl HierarchicalExtractor {
    pub fn new() -> Self {
        Self
    }
}

impl SliceAlgorithm for HierarchicalExtractor {
    fn extract(
        &self,
        config_tree: &ConfigNode,
        pattern: &SlicePattern,
        context: Option<&SliceContext>,
    ) -> Result<SliceResult> {
        let hierarchical_pattern = match pattern {
            SlicePattern::Hierarchical(p) => p,
            _ => {
                return Err(Error::InvalidPattern(
                    "Expected hierarchical pattern".to_string(),
                ));
            }
        };

        let mut matches = Vec::new();
        extract_hierarchical_matches(config_tree, hierarchical_pattern, &mut matches)?;

        // Apply context filtering if provided
        if let Some(ctx) = context {
            let filtered_matches: Vec<_> = matches
                .into_iter()
                .filter(|node| ctx.matches_node(node))
                .collect();
            matches = filtered_matches;
        }

        Ok(SliceResult {
            matches,
            pattern: pattern.clone(),
            metadata: HashMap::new(),
        })
    }
}

/// Context-aware slice extractor
pub struct ContextExtractor;

impl ContextExtractor {
    pub fn new() -> Self {
        Self
    }
}

impl SliceAlgorithm for ContextExtractor {
    fn extract(
        &self,
        config_tree: &ConfigNode,
        pattern: &SlicePattern,
        _context: Option<&SliceContext>,
    ) -> Result<SliceResult> {
        let context_pattern = match pattern {
            SlicePattern::Context(p) => p,
            _ => {
                return Err(Error::InvalidPattern(
                    "Expected context pattern".to_string(),
                ));
            }
        };

        let mut matches = Vec::new();
        extract_context_matches(config_tree, context_pattern, &mut matches)?;

        Ok(SliceResult {
            matches,
            pattern: pattern.clone(),
            metadata: HashMap::new(),
        })
    }
}

// Helper functions for pattern matching

fn extract_glob_matches(
    node: &ConfigNode,
    pattern: &GlobPattern,
    matches: &mut Vec<ConfigNode>,
    current_path: &str,
) -> Result<()> {
    let node_path = if current_path.is_empty() {
        node.command.clone()
    } else {
        format!("{}/{}", current_path, node.command)
    };

    // Check both full path and just the command
    if pattern.matches(&node_path) || pattern.matches(&node.command) {
        matches.push(node.clone());
    }

    // Recursively check children
    for child in &node.children {
        extract_glob_matches(child, pattern, matches, &node_path)?;
    }

    Ok(())
}

fn extract_regex_matches(
    node: &ConfigNode,
    pattern: &RegexPattern,
    matches: &mut Vec<ConfigNode>,
) -> Result<()> {
    if pattern.matches(&node.command) {
        matches.push(node.clone());
    }

    // Recursively check children
    for child in &node.children {
        extract_regex_matches(child, pattern, matches)?;
    }

    Ok(())
}

fn extract_hierarchical_matches(
    node: &ConfigNode,
    pattern: &HierarchicalPattern,
    matches: &mut Vec<ConfigNode>,
) -> Result<()> {
    if pattern.matches_node(node) {
        matches.push(node.clone());
    }

    // Recursively check children
    for child in &node.children {
        extract_hierarchical_matches(child, pattern, matches)?;
    }

    Ok(())
}

fn extract_context_matches(
    node: &ConfigNode,
    pattern: &ContextMatcher,
    matches: &mut Vec<ConfigNode>,
) -> Result<()> {
    if pattern.matches(&node.context) {
        matches.push(node.clone());
    }

    // Recursively check children
    for child in &node.children {
        extract_context_matches(child, pattern, matches)?;
    }

    Ok(())
}

/// Hierarchical pattern for complex tree matching
#[derive(Debug, Clone, PartialEq)]
pub struct HierarchicalPattern {
    /// Path segments for hierarchical matching
    pub path_segments: Vec<PathSegment>,
    /// Whether to match exact depth
    pub exact_depth: bool,
    /// Maximum depth to search
    pub max_depth: Option<usize>,
}

/// A segment in a hierarchical path
#[derive(Debug, Clone, PartialEq)]
pub struct PathSegment {
    /// Pattern to match the segment
    pub pattern: String,
    /// Whether this segment is optional
    pub optional: bool,
    /// Whether to match exactly or use wildcards
    pub exact_match: bool,
}

impl HierarchicalPattern {
    pub fn new(path: &str) -> Result<Self> {
        let segments = parse_hierarchical_path(path)?;
        Ok(Self {
            path_segments: segments,
            exact_depth: false,
            max_depth: None,
        })
    }

    pub fn with_exact_depth(mut self) -> Self {
        self.exact_depth = true;
        self
    }

    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }

    pub fn matches_node(&self, node: &ConfigNode) -> bool {
        // Implementation for hierarchical node matching
        // This would involve building a path from root to this node
        // and matching against the path_segments
        matches_hierarchical_path(node, &self.path_segments, 0)
    }
}

fn parse_hierarchical_path(path: &str) -> Result<Vec<PathSegment>> {
    let mut segments = Vec::new();

    for segment_str in path.split('/') {
        if segment_str.is_empty() {
            continue;
        }

        let optional = segment_str.starts_with('[') && segment_str.ends_with(']');
        let pattern = if optional {
            segment_str[1..segment_str.len() - 1].to_string()
        } else {
            segment_str.to_string()
        };

        let exact_match = !pattern.contains('*') && !pattern.contains('?');

        segments.push(PathSegment {
            pattern,
            optional,
            exact_match,
        });
    }

    Ok(segments)
}

fn matches_hierarchical_path(
    node: &ConfigNode,
    segments: &[PathSegment],
    segment_index: usize,
) -> bool {
    if segment_index >= segments.len() {
        return true; // All segments matched
    }

    let segment = &segments[segment_index];

    // Check if current node matches the current segment
    let node_matches = if segment.exact_match {
        node.command == segment.pattern
    } else {
        // Use glob-style matching for wildcards
        glob_match(&node.command, &segment.pattern)
    };

    if node_matches {
        // This node matches, try to match remaining segments in children
        for child in &node.children {
            if matches_hierarchical_path(child, segments, segment_index + 1) {
                return true;
            }
        }
    }

    // If segment is optional, try to match remaining segments without consuming this segment
    if segment.optional {
        for child in &node.children {
            if matches_hierarchical_path(child, segments, segment_index + 1) {
                return true;
            }
        }
    }

    false
}

fn glob_match(text: &str, pattern: &str) -> bool {
    // Simple glob matching implementation
    if pattern == "*" {
        return true;
    }

    if !pattern.contains('*') && !pattern.contains('?') {
        return text == pattern;
    }

    // Convert glob pattern to regex
    let regex_pattern = pattern
        .replace(".", r"\.")
        .replace("*", ".*")
        .replace("?", ".");

    if let Ok(regex) = Regex::new(&format!("^{}$", regex_pattern)) {
        regex.is_match(text)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{ConfigContext, NodeType};

    fn create_test_node(command: &str, children: Vec<ConfigNode>) -> ConfigNode {
        ConfigNode {
            command: command.to_string(),
            raw_line: command.to_string(),
            line_number: 1,
            indent_level: 0,
            children,
            context: ConfigContext::Global,
            node_type: NodeType::Command,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_glob_pattern_matching() {
        let slicer = ConfigSlicer::new();

        let root = create_test_node(
            "root",
            vec![
                create_test_node("interface GigabitEthernet0/1", vec![]),
                create_test_node("interface GigabitEthernet0/2", vec![]),
                create_test_node("hostname test", vec![]),
            ],
        );

        let pattern = SlicePattern::Glob(GlobPattern::new("interface*").unwrap());
        let result = slicer.extract_slice(&root, &pattern, None).unwrap();

        assert_eq!(result.matches.len(), 2);
        assert!(result.matches[0].command.starts_with("interface"));
        assert!(result.matches[1].command.starts_with("interface"));
    }

    #[test]
    fn test_hierarchical_pattern() {
        let pattern = HierarchicalPattern::new("interface/[description]/ip").unwrap();
        assert_eq!(pattern.path_segments.len(), 3);
        assert_eq!(pattern.path_segments[0].pattern, "interface");
        assert!(!pattern.path_segments[0].optional);
        assert_eq!(pattern.path_segments[1].pattern, "description");
        assert!(pattern.path_segments[1].optional);
        assert_eq!(pattern.path_segments[2].pattern, "ip");
        assert!(!pattern.path_segments[2].optional);
    }

    #[test]
    fn test_glob_matching() {
        assert!(glob_match("interface GigabitEthernet0/1", "interface*"));
        assert!(glob_match("test", "*"));
        assert!(glob_match("test", "test"));
        assert!(!glob_match("test", "other"));
        assert!(glob_match("test123", "test???"));
    }
}
