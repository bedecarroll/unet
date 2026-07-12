//! Test modules for all model types
//!
//! This module organizes tests for different model components:
//! - `enums`: Tests for `Lifecycle`, `DeviceRole`, and `Vendor` enums
//! - `node`: Tests for `Node` model and `NodeBuilder`
//! - `link`: Tests for `Link` model and `LinkBuilder`  
//! - `location`: Tests for `Location` model and `LocationBuilder`

#[cfg(test)]
mod enums;

#[cfg(test)]
mod node;

#[cfg(test)]
mod link;

#[cfg(test)]
mod location;
