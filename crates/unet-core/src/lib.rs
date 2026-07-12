//! μNet Core Library
//!
//! This library provides the core functionality for μNet network configuration management,
//! including data models, storage abstractions, policy engine, template engine, and SNMP integration.
//!
//! # Quick Start
//!
//! ```rust
//! use unet_core::{models::*, datastore::*};
//!
//! // Create a new node using the builder pattern
//! let node = NodeBuilder::new()
//!     .name("router-01".to_string())
//!     .domain("example.com".to_string())
//!     .vendor(Vendor::Cisco)
//!     .model("ISR4431".to_string())
//!     .role(DeviceRole::Router)
//!     .lifecycle(Lifecycle::Live)
//!     .build()
//!     .unwrap();
//! ```
//!
//! # Architecture
//!
//! The library is organized into several modules:
//!
//! - [`models`] - Core data models (Node, Link, Location)
//! - [`datastore`] - Storage abstraction layer with multiple backends
//! - [`error`] - Unified error types and handling
//! - [`config`] - Configuration management (Milestone 1.3.3)
//! - [`policy`] - Policy engine (Milestone 3)
//! - [`snmp`] - SNMP integration (Milestone 2)
//! - [`template`] - Template rendering (Milestone 4)

#![warn(missing_docs)]

// Public modules
pub mod config;
pub mod datastore;
pub mod entities;
pub mod error;
pub mod logging;
pub mod models;
pub mod policy;
pub mod policy_integration;
pub mod snmp;
pub mod template;

// Re-exports for convenience
pub use error::{Error, Result};

/// Prelude module for commonly used types
///
/// Common imports for μNet Core
///
/// This module provides convenient re-exports of the most commonly used types.
pub mod prelude {

    // Core error types
    pub use crate::error::{Error, Result};

    // Data models
    pub use crate::models::{
        DeviceRole, Lifecycle, Link, LinkBuilder, Location, LocationBuilder, Node, NodeBuilder,
        Vendor,
    };

    // Derived state models
    pub use crate::models::derived::{
        EnvironmentalMetrics, InterfaceAdminStatus, InterfaceOperStatus, InterfaceStats,
        InterfaceStatus, NodeStatus, PerformanceMetrics, SystemInfo,
    };

    // DataStore trait and common types
    pub use crate::datastore::{
        BatchOperation,
        BatchResult,
        DataStore,
        DataStoreError,
        DataStoreResult,
        Filter,
        FilterOperation,
        FilterValue,
        PagedResult,
        Pagination,
        QueryOptions,
        Sort,
        SortDirection,
        // Helper functions
        filter_contains,
        filter_equals_string,
        filter_equals_uuid,
        sort_asc,
        sort_desc,
        // Transaction helpers
        transaction_helpers,
    };

    // Configuration and logging
    pub use crate::config::Config;
    pub use crate::logging::{init_default_tracing, init_tracing};

    // SNMP types
    pub use crate::snmp::{
        OidMap, PollingConfig, PollingHandle, PollingResult, PollingScheduler, PollingTask,
        SessionConfig, SnmpClient, SnmpClientConfig, SnmpCredentials, SnmpError, SnmpResult,
        SnmpValue, StandardOid, VendorOid,
    };

    // Policy integration types
    pub use crate::policy_integration::{
        DefaultPolicyEvaluationEngine, PolicyEvaluationEngine, PolicyService,
    };
}
