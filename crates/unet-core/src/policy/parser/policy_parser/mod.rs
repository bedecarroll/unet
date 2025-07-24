//! Main policy parser implementation

mod action_parsing;
mod condition_parsing;
pub mod core;
mod entry_points;
mod tests;
mod value_parsing;

// Re-export the main struct for backward compatibility
pub use core::PolicyParser;
