//! Error logging and reporting functionality for μNet Core

pub mod core;
mod tests;

// Re-export the main trait for backward compatibility
pub use core::ErrorReporting;
