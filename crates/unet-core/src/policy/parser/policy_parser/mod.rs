//! Main policy parser implementation

mod action_parsing;
mod condition_parsing;
pub mod core;
mod entry_points;
mod tests;
mod value_parsing;

#[cfg(test)]
mod value_parsing_comprehensive_tests;

// Re-export the main struct for backward compatibility
pub use core::PolicyParser;
