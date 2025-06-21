//! μNet Core Library
//!
//! This library provides the core functionality for μNet network configuration management,
//! including data models, storage abstractions, policy engine, template engine, and SNMP integration.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

pub mod config;
pub mod datastore;
pub mod error;
pub mod models;
pub mod policy;
pub mod snmp;
pub mod template;

pub use error::{Error, Result};
