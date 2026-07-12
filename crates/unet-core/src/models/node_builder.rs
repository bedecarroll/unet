//! Node builder for creating nodes with validation
//!
//! Provides a builder pattern for constructing `Node` instances with proper
//! validation and error handling.

use crate::models::{DeviceRole, Lifecycle, Node, Vendor};
use serde_json::Value;
use std::net::IpAddr;
use uuid::Uuid;

/// Builder pattern for Node creation with validation
#[derive(Debug, Default)]
pub struct NodeBuilder {
    /// Node ID (optional, will generate UUID if not provided)
    id: Option<Uuid>,
    /// Node name (required)
    name: Option<String>,
    /// Network domain (required)
    domain: Option<String>,
    /// Device vendor (required)
    vendor: Option<Vendor>,
    /// Device model (required)
    model: Option<String>,
    /// Device role (required)
    role: Option<DeviceRole>,
    /// Lifecycle state (optional, defaults to Planned)
    lifecycle: Option<Lifecycle>,
    /// Management IP address (optional)
    management_ip: Option<IpAddr>,
    /// Location ID (optional)
    location_id: Option<Uuid>,
    /// Platform/OS information (optional)
    platform: Option<String>,
    /// Software version (optional)
    version: Option<String>,
    /// Serial number (optional)
    serial_number: Option<String>,
    /// Asset tag (optional)
    asset_tag: Option<String>,
    /// Purchase date (optional)
    purchase_date: Option<String>,
    /// Warranty expiration date (optional)
    warranty_expires: Option<String>,
    /// Custom data (optional)
    custom_data: Option<Value>,
}

impl NodeBuilder {
    /// Creates a new node builder
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the node ID (optional, will generate UUID if not provided)
    #[must_use]
    pub const fn id(mut self, id: Uuid) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the node name (required)
    #[must_use]
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the domain (required)
    #[must_use]
    pub fn domain<S: Into<String>>(mut self, domain: S) -> Self {
        self.domain = Some(domain.into());
        self
    }

    /// Sets the vendor (required)
    #[must_use]
    pub const fn vendor(mut self, vendor: Vendor) -> Self {
        self.vendor = Some(vendor);
        self
    }

    /// Sets the model (required)
    #[must_use]
    pub fn model<S: Into<String>>(mut self, model: S) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Sets the device role (required)
    #[must_use]
    pub const fn role(mut self, role: DeviceRole) -> Self {
        self.role = Some(role);
        self
    }

    /// Sets the lifecycle state (optional, defaults to Planned)
    #[must_use]
    pub const fn lifecycle(mut self, lifecycle: Lifecycle) -> Self {
        self.lifecycle = Some(lifecycle);
        self
    }

    /// Sets the management IP address (optional)
    #[must_use]
    pub const fn management_ip(mut self, ip: IpAddr) -> Self {
        self.management_ip = Some(ip);
        self
    }

    /// Sets the location ID (optional)
    #[must_use]
    pub const fn location_id(mut self, location_id: Uuid) -> Self {
        self.location_id = Some(location_id);
        self
    }

    /// Sets the platform (optional)
    #[must_use]
    pub fn platform<S: Into<String>>(mut self, platform: S) -> Self {
        self.platform = Some(platform.into());
        self
    }

    /// Sets the version (optional)
    #[must_use]
    pub fn version<S: Into<String>>(mut self, version: S) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Sets the serial number (optional)
    #[must_use]
    pub fn serial_number<S: Into<String>>(mut self, serial_number: S) -> Self {
        self.serial_number = Some(serial_number.into());
        self
    }

    /// Sets the asset tag (optional)
    #[must_use]
    pub fn asset_tag<S: Into<String>>(mut self, asset_tag: S) -> Self {
        self.asset_tag = Some(asset_tag.into());
        self
    }

    /// Sets the purchase date (optional)
    #[must_use]
    pub fn purchase_date<S: Into<String>>(mut self, purchase_date: S) -> Self {
        self.purchase_date = Some(purchase_date.into());
        self
    }

    /// Sets the warranty expiration date (optional)
    #[must_use]
    pub fn warranty_expires<S: Into<String>>(mut self, warranty_expires: S) -> Self {
        self.warranty_expires = Some(warranty_expires.into());
        self
    }

    /// Sets custom data (optional)
    #[must_use]
    pub fn custom_data(mut self, custom_data: Value) -> Self {
        self.custom_data = Some(custom_data);
        self
    }

    /// Builds the node with validation
    ///
    /// # Errors
    /// Returns an error if required fields (name, vendor, model, role) are missing,
    /// or if the created node fails validation.
    pub fn build(self) -> Result<Node, String> {
        let name = self.name.ok_or("Name is required")?;
        let domain = self.domain.unwrap_or_default();
        let vendor = self.vendor.ok_or("Vendor is required")?;
        let model = self.model.ok_or("Model is required")?;
        let role = self.role.ok_or("Role is required")?;

        let fqdn = if domain.is_empty() {
            name.clone()
        } else {
            format!("{name}.{domain}")
        };

        let node = Node {
            id: self.id.unwrap_or_else(Uuid::new_v4),
            name,
            domain,
            fqdn,
            vendor,
            model,
            role,
            lifecycle: self.lifecycle.unwrap_or(Lifecycle::Planned),
            management_ip: self.management_ip,
            location_id: self.location_id,
            platform: self.platform,
            version: self.version,
            serial_number: self.serial_number,
            asset_tag: self.asset_tag,
            purchase_date: self.purchase_date,
            warranty_expires: self.warranty_expires,
            custom_data: self.custom_data.unwrap_or(Value::Null),
        };

        node.validate()?;
        Ok(node)
    }
}
