//! Secure credential storage and management module for μNet Core

pub mod external;
pub mod key_management;
pub mod rotation;

// Re-export main secrets module
pub use main::*;

mod main;
