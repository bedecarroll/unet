//! Node model and implementation
//!
//! Contains the core `Node` struct representing network devices and their
//! management capabilities within μNet.

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

        // Validate domain
        if !self.domain.is_empty() && !crate::models::validation::is_valid_domain(&self.domain) {
            return Err("Invalid domain format".to_string());
        }

        // Validate FQDN consistency
        let expected_fqdn = if self.domain.is_empty() {
            self.name.clone()
        } else {
            format!("{}.{}", self.name, self.domain)
        };

        if self.fqdn != expected_fqdn {
            return Err("FQDN must match name.domain format".to_string());
        }

        // Validate model is not empty
        if self.model.is_empty() {
            return Err("Model cannot be empty".to_string());
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

    /// Gets a value from `custom_data` by path
    #[must_use]
    pub fn get_custom_data(&self, path: &str) -> Option<&Value> {
        // Simple dot-notation path traversal
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &self.custom_data;

        for part in parts {
            if let Value::Object(obj) = current {
                current = obj.get(part)?;
            } else {
                return None;
            }
        }

        Some(current)
    }

    /// Sets a value in `custom_data` by path
    ///
    /// # Errors
    /// Returns an error if the path is empty, cannot navigate through non-object,
    /// or cannot set value on non-object.
    pub fn set_custom_data(&mut self, path: &str, value: Value) -> Result<(), String> {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return Err("Path cannot be empty".to_string());
        }

        // Initialize custom_data as empty object if null
        if self.custom_data.is_null() {
            self.custom_data = Value::Object(serde_json::Map::new());
        }

        // Navigate to the parent and set the final key
        let mut current = &mut self.custom_data;
        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                // Last part - set the value
                if let Value::Object(obj) = current {
                    obj.insert((*part).to_string(), value);
                    return Ok(());
                }
                return Err("Cannot set value on non-object".to_string());
            }
            // Navigate deeper, creating objects as needed
            if let Value::Object(obj) = current {
                let entry = obj
                    .entry((*part).to_string())
                    .or_insert_with(|| Value::Object(serde_json::Map::new()));
                current = entry;
            } else {
                return Err("Cannot navigate through non-object".to_string());
            }
        }

        Ok(())
    }
}
