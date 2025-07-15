//! Data models for μNet Core
//!
//! This module contains all the core data structures used throughout μNet,
//! including nodes, locations, links, and their associated types.

pub mod derived;
pub mod link;
pub mod location;
pub mod node;
pub mod node_builder;
pub mod tests;
pub mod validation;

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;

// Re-export all public types for backward compatibility
pub use link::{Link, LinkBuilder};
pub use location::{Location, LocationBuilder};
pub use node::Node;
pub use node_builder::NodeBuilder;
pub use validation::*;

/// Lifecycle state of a network device or configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Lifecycle {
    /// Device is planned but not yet deployed
    Planned,
    /// Device is currently being implemented/deployed
    Implementing,
    /// Device is live and operational
    Live,
    /// Device is being decommissioned or is decommissioned
    Decommissioned,
}

impl Display for Lifecycle {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Planned => write!(f, "planned"),
            Self::Implementing => write!(f, "implementing"),
            Self::Live => write!(f, "live"),
            Self::Decommissioned => write!(f, "decommissioned"),
        }
    }
}

impl FromStr for Lifecycle {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "planned" => Ok(Self::Planned),
            "implementing" => Ok(Self::Implementing),
            "live" => Ok(Self::Live),
            "decommissioned" => Ok(Self::Decommissioned),
            _ => Err(format!("Invalid lifecycle state: {s}")),
        }
    }
}

impl From<String> for Lifecycle {
    fn from(s: String) -> Self {
        s.parse().unwrap_or(Self::Planned)
    }
}

/// Role/type of network device
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceRole {
    /// Network router
    Router,
    /// Network switch
    Switch,
    /// Firewall device
    Firewall,
    /// Load balancer
    LoadBalancer,
    /// Wireless access point
    AccessPoint,
    /// Network security appliance
    SecurityAppliance,
    /// Network monitoring device
    Monitor,
    /// Generic server
    Server,
    /// Storage device
    Storage,
    /// Other/unspecified device type
    Other,
}

impl Display for DeviceRole {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Router => write!(f, "router"),
            Self::Switch => write!(f, "switch"),
            Self::Firewall => write!(f, "firewall"),
            Self::LoadBalancer => write!(f, "loadbalancer"),
            Self::AccessPoint => write!(f, "accesspoint"),
            Self::SecurityAppliance => write!(f, "securityappliance"),
            Self::Monitor => write!(f, "monitor"),
            Self::Server => write!(f, "server"),
            Self::Storage => write!(f, "storage"),
            Self::Other => write!(f, "other"),
        }
    }
}

impl FromStr for DeviceRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "router" => Ok(Self::Router),
            "switch" => Ok(Self::Switch),
            "firewall" => Ok(Self::Firewall),
            "loadbalancer" => Ok(Self::LoadBalancer),
            "accesspoint" => Ok(Self::AccessPoint),
            "securityappliance" => Ok(Self::SecurityAppliance),
            "monitor" => Ok(Self::Monitor),
            "server" => Ok(Self::Server),
            "storage" => Ok(Self::Storage),
            "other" => Ok(Self::Other),
            _ => Err(format!("Invalid device role: {s}")),
        }
    }
}

impl From<String> for DeviceRole {
    fn from(s: String) -> Self {
        s.parse().unwrap_or(Self::Other)
    }
}

/// Network equipment vendor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Vendor {
    /// Cisco Systems
    Cisco,
    /// Juniper Networks
    Juniper,
    /// Arista Networks
    Arista,
    /// Palo Alto Networks
    PaloAlto,
    /// Fortinet
    Fortinet,
    /// HPE/Hewlett Packard Enterprise
    Hpe,
    /// Dell Technologies
    Dell,
    /// Extreme Networks
    Extreme,
    /// Mikrotik
    Mikrotik,
    /// Ubiquiti
    Ubiquiti,
    /// Generic/unknown vendor
    Generic,
}

impl Display for Vendor {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Cisco => write!(f, "cisco"),
            Self::Juniper => write!(f, "juniper"),
            Self::Arista => write!(f, "arista"),
            Self::PaloAlto => write!(f, "paloalto"),
            Self::Fortinet => write!(f, "fortinet"),
            Self::Hpe => write!(f, "hpe"),
            Self::Dell => write!(f, "dell"),
            Self::Extreme => write!(f, "extreme"),
            Self::Mikrotik => write!(f, "mikrotik"),
            Self::Ubiquiti => write!(f, "ubiquiti"),
            Self::Generic => write!(f, "generic"),
        }
    }
}

impl FromStr for Vendor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "cisco" => Ok(Self::Cisco),
            "juniper" => Ok(Self::Juniper),
            "arista" => Ok(Self::Arista),
            "paloalto" => Ok(Self::PaloAlto),
            "fortinet" => Ok(Self::Fortinet),
            "hpe" => Ok(Self::Hpe),
            "dell" => Ok(Self::Dell),
            "extreme" => Ok(Self::Extreme),
            "mikrotik" => Ok(Self::Mikrotik),
            "ubiquiti" => Ok(Self::Ubiquiti),
            "generic" => Ok(Self::Generic),
            _ => Err(format!("Invalid vendor: {s}")),
        }
    }
}

impl From<String> for Vendor {
    fn from(s: String) -> Self {
        s.parse().unwrap_or(Self::Generic)
    }
}
