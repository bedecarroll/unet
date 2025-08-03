//! Core Node struct definition and basic methods
//!
//! Contains the primary Node structure representing network devices
//! and fundamental operations like construction and validation.

use crate::models::{DeviceRole, Lifecycle, Vendor};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::net::IpAddr;
use uuid::Uuid;

/// Network node/device representation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    /// Unique identifier for the node
    pub id: Uuid,
    /// Human-readable name for the node
    pub name: String,
    /// Network domain (e.g., "example.com")
    pub domain: String,
    /// Fully qualified domain name (name.domain)
    pub fqdn: String,
    /// Device vendor
    pub vendor: Vendor,
    /// Device model/type
    pub model: String,
    /// Device role in the network
    pub role: DeviceRole,
    /// Lifecycle state
    pub lifecycle: Lifecycle,
    /// Primary management IP address
    pub management_ip: Option<IpAddr>,
    /// Location identifier (references Location.id)
    pub location_id: Option<Uuid>,
    /// Platform/OS information
    pub platform: Option<String>,
    /// Software version
    pub version: Option<String>,
    /// Serial number
    pub serial_number: Option<String>,
    /// Asset tag or inventory number
    pub asset_tag: Option<String>,
    /// Purchase date (ISO 8601 string)
    pub purchase_date: Option<String>,
    /// Warranty expiration date (ISO 8601 string)
    pub warranty_expires: Option<String>,
    /// Extended/custom data as JSON
    pub custom_data: Value,
}

impl Node {
    /// Creates a new node with minimal required fields
    #[must_use]
    pub fn new(name: String, domain: String, vendor: Vendor, role: DeviceRole) -> Self {
        let fqdn = if domain.is_empty() {
            name.clone()
        } else {
            format!("{name}.{domain}")
        };

        Self {
            id: Uuid::new_v4(),
            name,
            domain,
            fqdn,
            vendor,
            model: String::new(),
            role,
            lifecycle: Lifecycle::Planned,
            management_ip: None,
            location_id: None,
            platform: None,
            version: None,
            serial_number: None,
            asset_tag: None,
            purchase_date: None,
            warranty_expires: None,
            custom_data: Value::Null,
        }
    }

    /// Validates the node configuration
    ///
    /// # Errors
    /// Returns an error if the node name is empty, contains invalid characters,
    /// or if the domain format is invalid.
    pub fn validate(&self) -> Result<(), String> {
        // Validate name
        if self.name.is_empty() {
            return Err("Node name cannot be empty".to_string());
        }

        if !self
            .name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(
                "Node name must contain only alphanumeric characters, hyphens, and underscores"
                    .to_string(),
            );
        }

        // Validate domain format if present
        if !self.domain.is_empty()
            && !self
                .domain
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
        {
            return Err(
                "Domain must contain only alphanumeric characters, dots, and hyphens".to_string(),
            );
        }

        // Validate FQDN consistency
        let expected_fqdn = if self.domain.is_empty() {
            self.name.clone()
        } else {
            format!("{}.{}", self.name, self.domain)
        };

        if self.fqdn != expected_fqdn {
            return Err(format!(
                "FQDN '{}' does not match expected '{}' based on name and domain",
                self.fqdn, expected_fqdn
            ));
        }

        // Validate model is not empty
        if self.model.is_empty() {
            return Err("Node model cannot be empty".to_string());
        }

        Ok(())
    }

    /// Updates the FQDN based on current name and domain
    pub fn update_fqdn(&mut self) {
        self.fqdn = if self.domain.is_empty() {
            self.name.clone()
        } else {
            format!("{}.{}", self.name, self.domain)
        };
    }
}
