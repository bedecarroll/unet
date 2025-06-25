//! Configuration slicing modules
//!
//! This module provides the core components for configuration slicing operations.

pub mod algorithms;
pub mod context;
pub mod patterns;

// Include the main library implementation
mod lib;

// Re-export everything from the main library
pub use lib::*;

// Re-export key types for easier access
pub use algorithms::{SliceAlgorithm, SliceExtractor, SliceResult};
pub use context::{ContextMatcher, SliceContext, SliceContextBuilder};
pub use patterns::{GlobPattern, PatternBuilder, PatternMatcher, RegexPattern, SlicePattern};
