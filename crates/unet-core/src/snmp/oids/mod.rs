//! Standard and vendor-specific OID definitions
//!
//! This module provides organized access to SNMP Object Identifiers (OIDs)
//! used for network device monitoring and management.

pub use map::OidMap;
pub use standard::StandardOid;
pub use vendor::VendorOid;

mod map;
mod standard;
mod vendor;
