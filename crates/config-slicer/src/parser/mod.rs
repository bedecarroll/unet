//! Configuration parsing modules
//!
//! This module provides hierarchical parsing capabilities for network device configurations.

pub mod core;
pub mod optimized;
pub mod plugin;
pub mod vendors;

// Re-export commonly used types
pub use core::{
    ConfigContext, ConfigNode, HierarchicalParser, IndentDetection, NodeType, ParserConfig,
    TreeTraversal, ValidationReport,
};
pub use optimized::{CacheStats, OptimizedHierarchicalParser};
pub use plugin::{ConfigParserPlugin, PluginRegistry, PluginRegistryBuilder};
pub use vendors::{Vendor, VendorParser, VendorValidationReport};
