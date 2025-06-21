//! Configuration Slicing and Diffing Library
//!
//! This library provides tools for parsing network device configurations,
//! extracting specific sections (slices), and computing diffs between configurations.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

pub mod diff;
pub mod error;
pub mod parser;
pub mod slicer;

pub use error::{Error, Result};
