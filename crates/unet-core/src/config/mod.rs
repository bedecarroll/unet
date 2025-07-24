//! Configuration management for μNet Core
//!
//! This module provides TOML-based configuration management with hierarchical
//! configuration support and environment variable overrides.

// Re-export submodules
pub mod core;
pub mod defaults;
pub mod network;
pub mod types;
pub mod utils;

#[cfg(test)]
mod core_tests;

// Re-export the main Config struct and commonly used items
pub use core::Config;
pub use defaults::*;
pub use network::*;
pub use types::*;

// Re-export specific constants for backward compatibility
pub use defaults::network::LOCALHOST_SNMP;
