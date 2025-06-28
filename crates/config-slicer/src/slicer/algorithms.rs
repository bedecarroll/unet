//! Core slice extraction algorithms and interfaces
//!
//! This module defines the traits and types used by all slice extraction algorithms.

use crate::error::Result;
use crate::parser::ConfigNode;
use crate::slicer::{SliceContext, SlicePattern};
use std::collections::HashMap;

/// Core trait for slice extraction algorithms
pub trait SliceAlgorithm: Send + Sync {
    /// Extract configuration slices matching the given pattern
    fn extract(
        &self,
        config_tree: &ConfigNode,
        pattern: &SlicePattern,
        context: Option<&SliceContext>,
    ) -> Result<SliceResult>;
}

/// Result of a slice extraction operation
#[derive(Debug, Clone)]
pub struct SliceResult {
    /// Matching configuration nodes
    pub matches: Vec<ConfigNode>,
    /// Pattern that was used for extraction
    pub pattern: SlicePattern,
    /// Additional metadata about the extraction
    pub metadata: HashMap<String, String>,
}

impl SliceResult {
    /// Create a new slice result
    #[must_use]
    pub fn new(matches: Vec<ConfigNode>, pattern: SlicePattern) -> Self {
        Self {
            matches,
            pattern,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the result
    #[must_use]
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Get the number of matches
    #[must_use]
    pub fn match_count(&self) -> usize {
        self.matches.len()
    }

    /// Check if any matches were found
    #[must_use]
    pub fn has_matches(&self) -> bool {
        !self.matches.is_empty()
    }

    /// Get matches as a reference
    #[must_use]
    pub fn matches(&self) -> &[ConfigNode] {
        &self.matches
    }

    /// Get pattern as a reference
    #[must_use]
    pub const fn pattern(&self) -> &SlicePattern {
        &self.pattern
    }

    /// Get metadata as a reference
    #[must_use]
    pub const fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
}

/// Interface for extracting slices from configuration trees
pub trait SliceExtractor {
    /// Extract a single slice using the specified pattern
    fn extract_slice(
        &self,
        config_tree: &ConfigNode,
        pattern: &SlicePattern,
        context: Option<&SliceContext>,
    ) -> Result<SliceResult>;

    /// Extract multiple slices using different patterns
    fn extract_multiple_slices(
        &self,
        config_tree: &ConfigNode,
        patterns: &[SlicePattern],
        context: Option<&SliceContext>,
    ) -> Result<Vec<SliceResult>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{ConfigContext, NodeType};
    use crate::slicer::patterns::GlobPattern;

    fn create_test_node(command: &str) -> ConfigNode {
        ConfigNode {
            command: command.to_string(),
            raw_line: command.to_string(),
            line_number: 1,
            indent_level: 0,
            children: vec![],
            context: ConfigContext::Global,
            node_type: NodeType::Command,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_slice_result_creation() {
        let node = create_test_node("test command");
        let pattern = SlicePattern::Glob(GlobPattern::new("test*").unwrap());

        let result = SliceResult::new(vec![node], pattern.clone());

        assert_eq!(result.match_count(), 1);
        assert!(result.has_matches());
        assert_eq!(result.matches().len(), 1);
        assert_eq!(result.pattern(), &pattern);
        assert!(result.metadata().is_empty());
    }

    #[test]
    fn test_slice_result_with_metadata() {
        let node = create_test_node("test command");
        let pattern = SlicePattern::Glob(GlobPattern::new("test*").unwrap());

        let result = SliceResult::new(vec![node], pattern)
            .with_metadata("algorithm".to_string(), "glob".to_string())
            .with_metadata("execution_time".to_string(), "10ms".to_string());

        assert_eq!(result.metadata().len(), 2);
        assert_eq!(
            result.metadata().get("algorithm"),
            Some(&"glob".to_string())
        );
        assert_eq!(
            result.metadata().get("execution_time"),
            Some(&"10ms".to_string())
        );
    }
}
