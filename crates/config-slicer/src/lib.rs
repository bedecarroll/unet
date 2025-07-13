//! Configuration Slicing and Diffing Library
//!
//! This library provides tools for parsing network device configurations,
//! extracting specific sections (slices), and computing diffs between configurations.

#![warn(missing_docs)]
#![deny(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

pub mod diff;
pub mod error;
pub mod parser;
pub mod slicer;

pub use error::{ConfigSlicerError, Result};
