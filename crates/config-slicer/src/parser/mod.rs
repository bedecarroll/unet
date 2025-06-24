//! Configuration parsing modules
//!
//! This module provides hierarchical parsing capabilities for network device configurations.

pub mod core;

// Re-export commonly used types
pub use core::{
    ConfigContext, ConfigNode, HierarchicalParser, IndentDetection, NodeType, ParserConfig,
    TreeTraversal, ValidationReport,
};
