//! Shared test helpers for filter function tests
//!
//! Provides common query creation functions and utilities used across
//! all filter test modules to reduce code duplication.

use crate::entities::{links, locations, nodes};
use sea_orm::{EntityTrait, Select};

/// Create a base node query for testing
pub fn create_node_query() -> Select<nodes::Entity> {
    nodes::Entity::find()
}

/// Create a base location query for testing
pub fn create_location_query() -> Select<locations::Entity> {
    locations::Entity::find()
}

/// Create a base link query for testing
pub fn create_link_query() -> Select<links::Entity> {
    links::Entity::find()
}
