//! Node model module
//!
//! This module has been reorganized into focused submodules:
//! - `core`: Core Node struct definition and basic operations
//! - `methods`: Utility methods and custom data manipulation
//! - `tests`: Comprehensive test suite

pub mod core;
pub mod methods;

#[cfg(test)]
mod tests;

// Re-export the main struct for backward compatibility
pub use core::Node;
