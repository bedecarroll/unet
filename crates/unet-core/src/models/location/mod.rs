//! Location model and implementation
//!
//! Contains the core `Location` struct representing hierarchical locations
//! and the `LocationBuilder` for creating locations with validation.

pub mod builder;
pub mod model;

// Re-export the main types for backward compatibility
pub use builder::LocationBuilder;
pub use model::Location;
