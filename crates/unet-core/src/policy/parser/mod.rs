//! Parser implementation that converts Pest parse trees to AST
//!
//! This module implements the conversion from Pest's parse tree to our
//! strongly-typed AST structures.

pub use error::ParseError;
pub use policy_parser::PolicyParser;

mod error;
mod policy_parser;
mod utils;
