//! Data models for μNet Core

pub mod derived;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::net::IpAddr;
use std::str::FromStr;
use uuid::Uuid;

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

/// Network node/device representation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    pub fn new(name: String, domain: String, vendor: Vendor, role: DeviceRole) -> Self {
        let fqdn = if domain.is_empty() {
            name.clone()
        } else {
            format!("{}.{}", name, domain)
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
    pub fn validate(&self) -> Result<(), String> {
        // Validate name
        if self.name.is_empty() {
            return Err("Node name cannot be empty".to_string());
        }
        
        if !self.name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
            return Err("Node name must contain only alphanumeric characters, hyphens, and underscores".to_string());
        }

        // Validate domain
        if !self.domain.is_empty() && !is_valid_domain(&self.domain) {
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

    /// Gets a value from custom_data by path
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

    /// Sets a value in custom_data by path
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
                    obj.insert(part.to_string(), value);
                    return Ok(());
                } else {
                    return Err("Cannot set value on non-object".to_string());
                }
            } else {
                // Navigate deeper, creating objects as needed
                if let Value::Object(obj) = current {
                    let entry = obj.entry(part.to_string()).or_insert_with(|| Value::Object(serde_json::Map::new()));
                    current = entry;
                } else {
                    return Err("Cannot navigate through non-object".to_string());
                }
            }
        }
        
        Ok(())
    }
}

/// Builder pattern for Node creation with validation
#[derive(Debug, Default)]
pub struct NodeBuilder {
    id: Option<Uuid>,
    name: Option<String>,
    domain: Option<String>,
    vendor: Option<Vendor>,
    model: Option<String>,
    role: Option<DeviceRole>,
    lifecycle: Option<Lifecycle>,
    management_ip: Option<IpAddr>,
    location_id: Option<Uuid>,
    platform: Option<String>,
    version: Option<String>,
    serial_number: Option<String>,
    asset_tag: Option<String>,
    purchase_date: Option<String>,
    warranty_expires: Option<String>,
    custom_data: Option<Value>,
}

impl NodeBuilder {
    /// Creates a new node builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the node ID (optional, will generate UUID if not provided)
    pub fn id(mut self, id: Uuid) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the node name (required)
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the domain (required)
    pub fn domain<S: Into<String>>(mut self, domain: S) -> Self {
        self.domain = Some(domain.into());
        self
    }

    /// Sets the vendor (required)
    pub fn vendor(mut self, vendor: Vendor) -> Self {
        self.vendor = Some(vendor);
        self
    }

    /// Sets the model (required)
    pub fn model<S: Into<String>>(mut self, model: S) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Sets the device role (required)
    pub fn role(mut self, role: DeviceRole) -> Self {
        self.role = Some(role);
        self
    }

    /// Sets the lifecycle state (optional, defaults to Planned)
    pub fn lifecycle(mut self, lifecycle: Lifecycle) -> Self {
        self.lifecycle = Some(lifecycle);
        self
    }

    /// Sets the management IP address (optional)
    pub fn management_ip(mut self, ip: IpAddr) -> Self {
        self.management_ip = Some(ip);
        self
    }

    /// Sets the location ID (optional)
    pub fn location_id(mut self, location_id: Uuid) -> Self {
        self.location_id = Some(location_id);
        self
    }

    /// Sets the platform (optional)
    pub fn platform<S: Into<String>>(mut self, platform: S) -> Self {
        self.platform = Some(platform.into());
        self
    }

    /// Sets the version (optional)
    pub fn version<S: Into<String>>(mut self, version: S) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Sets the serial number (optional)
    pub fn serial_number<S: Into<String>>(mut self, serial_number: S) -> Self {
        self.serial_number = Some(serial_number.into());
        self
    }

    /// Sets the asset tag (optional)
    pub fn asset_tag<S: Into<String>>(mut self, asset_tag: S) -> Self {
        self.asset_tag = Some(asset_tag.into());
        self
    }

    /// Sets the purchase date (optional)
    pub fn purchase_date<S: Into<String>>(mut self, purchase_date: S) -> Self {
        self.purchase_date = Some(purchase_date.into());
        self
    }

    /// Sets the warranty expiration date (optional)
    pub fn warranty_expires<S: Into<String>>(mut self, warranty_expires: S) -> Self {
        self.warranty_expires = Some(warranty_expires.into());
        self
    }

    /// Sets custom data (optional)
    pub fn custom_data(mut self, custom_data: Value) -> Self {
        self.custom_data = Some(custom_data);
        self
    }

    /// Builds the node with validation
    pub fn build(self) -> Result<Node, String> {
        let name = self.name.ok_or("Name is required")?;
        let domain = self.domain.unwrap_or_default();
        let vendor = self.vendor.ok_or("Vendor is required")?;
        let model = self.model.ok_or("Model is required")?;
        let role = self.role.ok_or("Role is required")?;

        let fqdn = if domain.is_empty() {
            name.clone()
        } else {
            format!("{}.{}", name, domain)
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

/// Network link/connection between nodes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Link {
    /// Unique identifier for the link
    pub id: Uuid,
    /// Human-readable name/description for the link
    pub name: String,
    /// Node A identifier (required)
    pub node_a_id: Uuid,
    /// Node A interface name
    pub node_a_interface: String,
    /// Node Z identifier (optional for internet circuits)
    pub node_z_id: Option<Uuid>,
    /// Node Z interface name (optional for internet circuits)
    pub node_z_interface: Option<String>,
    /// Link description
    pub description: Option<String>,
    /// Link bandwidth in bits per second
    pub bandwidth: Option<u64>,
    /// Link type (e.g., "ethernet", "fiber", "wireless")
    pub link_type: Option<String>,
    /// Whether this is an internet circuit
    pub is_internet_circuit: bool,
    /// Extended/custom data as JSON
    pub custom_data: Value,
}

impl Link {
    /// Creates a new link between two nodes
    pub fn new(
        name: String,
        node_a_id: Uuid,
        node_a_interface: String,
        node_z_id: Uuid,
        node_z_interface: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            node_a_id,
            node_a_interface,
            node_z_id: Some(node_z_id),
            node_z_interface: Some(node_z_interface),
            description: None,
            bandwidth: None,
            link_type: None,
            is_internet_circuit: false,
            custom_data: Value::Null,
        }
    }

    /// Creates a new internet circuit (single-ended link)
    pub fn new_internet_circuit(
        name: String,
        node_a_id: Uuid,
        node_a_interface: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            node_a_id,
            node_a_interface,
            node_z_id: None,
            node_z_interface: None,
            description: None,
            bandwidth: None,
            link_type: None,
            is_internet_circuit: true,
            custom_data: Value::Null,
        }
    }

    /// Validates the link configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate name
        if self.name.is_empty() {
            return Err("Link name cannot be empty".to_string());
        }

        // Validate node A interface
        if self.node_a_interface.is_empty() {
            return Err("Node A interface cannot be empty".to_string());
        }

        // Validate interface name format
        if !is_valid_interface_name(&self.node_a_interface) {
            return Err("Invalid node A interface name format".to_string());
        }

        // Check consistency for internet circuits
        if self.is_internet_circuit {
            if self.node_z_id.is_some() {
                return Err("Internet circuits cannot have node Z".to_string());
            }
            if self.node_z_interface.is_some() {
                return Err("Internet circuits cannot have node Z interface".to_string());
            }
        } else {
            // For regular links, both ends must be specified
            if self.node_z_id.is_none() {
                return Err("Regular links must have node Z".to_string());
            }
            if self.node_z_interface.is_none() {
                return Err("Regular links must have node Z interface".to_string());
            }

            // Validate node Z interface name if present
            if let Some(ref interface) = self.node_z_interface {
                if interface.is_empty() {
                    return Err("Node Z interface cannot be empty".to_string());
                }
                if !is_valid_interface_name(interface) {
                    return Err("Invalid node Z interface name format".to_string());
                }
            }

            // Prevent self-links
            if Some(self.node_a_id) == self.node_z_id {
                return Err("Links cannot connect a node to itself".to_string());
            }
        }

        Ok(())
    }

    /// Returns the other node ID if this is a bidirectional link
    pub fn get_other_node_id(&self, node_id: Uuid) -> Option<Uuid> {
        if self.node_a_id == node_id {
            self.node_z_id
        } else if Some(node_id) == self.node_z_id {
            Some(self.node_a_id)
        } else {
            None
        }
    }

    /// Returns the interface name for the given node
    pub fn get_interface_for_node(&self, node_id: Uuid) -> Option<&str> {
        if self.node_a_id == node_id {
            Some(&self.node_a_interface)
        } else if Some(node_id) == self.node_z_id {
            self.node_z_interface.as_deref()
        } else {
            None
        }
    }

    /// Checks if this link connects the two specified nodes
    pub fn connects_nodes(&self, node1_id: Uuid, node2_id: Uuid) -> bool {
        (self.node_a_id == node1_id && Some(node2_id) == self.node_z_id) ||
        (self.node_a_id == node2_id && Some(node1_id) == self.node_z_id)
    }

    /// Checks if this link involves the specified node
    pub fn involves_node(&self, node_id: Uuid) -> bool {
        self.node_a_id == node_id || Some(node_id) == self.node_z_id
    }

    /// Gets a value from custom_data by path
    pub fn get_custom_data(&self, path: &str) -> Option<&Value> {
        // Simple dot-notation path traversal (same as Node)
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

    /// Sets a value in custom_data by path
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
                    obj.insert(part.to_string(), value);
                    return Ok(());
                } else {
                    return Err("Cannot set value on non-object".to_string());
                }
            } else {
                // Navigate deeper, creating objects as needed
                if let Value::Object(obj) = current {
                    let entry = obj.entry(part.to_string()).or_insert_with(|| Value::Object(serde_json::Map::new()));
                    current = entry;
                } else {
                    return Err("Cannot navigate through non-object".to_string());
                }
            }
        }
        
        Ok(())
    }
}

/// Builder pattern for Link creation with validation
#[derive(Debug, Default)]
pub struct LinkBuilder {
    id: Option<Uuid>,
    name: Option<String>,
    node_a_id: Option<Uuid>,
    node_a_interface: Option<String>,
    node_z_id: Option<Uuid>,
    node_z_interface: Option<String>,
    description: Option<String>,
    bandwidth: Option<u64>,
    link_type: Option<String>,
    is_internet_circuit: Option<bool>,
    custom_data: Option<Value>,
}

impl LinkBuilder {
    /// Creates a new link builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the link ID (optional, will generate UUID if not provided)
    pub fn id(mut self, id: Uuid) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the link name (required)
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets node A ID (required)
    pub fn node_a_id(mut self, node_a_id: Uuid) -> Self {
        self.node_a_id = Some(node_a_id);
        self
    }

    /// Sets node A interface (required)
    pub fn node_a_interface<S: Into<String>>(mut self, interface: S) -> Self {
        self.node_a_interface = Some(interface.into());
        self
    }

    /// Sets node Z ID (optional for internet circuits)
    pub fn node_z_id(mut self, node_z_id: Uuid) -> Self {
        self.node_z_id = Some(node_z_id);
        self
    }

    /// Sets node Z interface (optional for internet circuits)
    pub fn node_z_interface<S: Into<String>>(mut self, interface: S) -> Self {
        self.node_z_interface = Some(interface.into());
        self
    }

    /// Sets the description (optional)
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the bandwidth (optional)
    pub fn bandwidth(mut self, bandwidth: u64) -> Self {
        self.bandwidth = Some(bandwidth);
        self
    }

    /// Sets the link type (optional)
    pub fn link_type<S: Into<String>>(mut self, link_type: S) -> Self {
        self.link_type = Some(link_type.into());
        self
    }

    /// Sets whether this is an internet circuit (optional, defaults to false)
    pub fn is_internet_circuit(mut self, is_internet_circuit: bool) -> Self {
        self.is_internet_circuit = Some(is_internet_circuit);
        self
    }

    /// Sets custom data (optional)
    pub fn custom_data(mut self, custom_data: Value) -> Self {
        self.custom_data = Some(custom_data);
        self
    }

    /// Builds the link with validation
    pub fn build(self) -> Result<Link, String> {
        let name = self.name.ok_or("Name is required")?;
        let node_a_id = self.node_a_id.ok_or("Node A ID is required")?;
        let node_a_interface = self.node_a_interface.ok_or("Node A interface is required")?;
        let is_internet_circuit = self.is_internet_circuit.unwrap_or(false);

        let link = Link {
            id: self.id.unwrap_or_else(Uuid::new_v4),
            name,
            node_a_id,
            node_a_interface,
            node_z_id: self.node_z_id,
            node_z_interface: self.node_z_interface,
            description: self.description,
            bandwidth: self.bandwidth,
            link_type: self.link_type,
            is_internet_circuit,
            custom_data: self.custom_data.unwrap_or(Value::Null),
        };

        link.validate()?;
        Ok(link)
    }
}

/// Physical or logical location in a hierarchy
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Location {
    /// Unique identifier for the location
    pub id: Uuid,
    /// Human-readable name for the location
    pub name: String,
    /// Location type (e.g., "country", "city", "building", "floor", "rack")
    pub location_type: String,
    /// Parent location identifier (None for root locations)
    pub parent_id: Option<Uuid>,
    /// Full path from root (e.g., "USA/California/San Francisco/Building A/Floor 3")
    pub path: String,
    /// Description of the location
    pub description: Option<String>,
    /// Address or coordinates
    pub address: Option<String>,
    /// Extended/custom data as JSON
    pub custom_data: Value,
}

impl Location {
    /// Creates a new root location (no parent)
    pub fn new_root(name: String, location_type: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.clone(),
            location_type,
            parent_id: None,
            path: name,
            description: None,
            address: None,
            custom_data: Value::Null,
        }
    }

    /// Creates a new child location with the given parent
    pub fn new_child(
        name: String,
        location_type: String,
        parent_path: String,
    ) -> Self {
        let path = if parent_path.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", parent_path, name)
        };

        Self {
            id: Uuid::new_v4(),
            name,
            location_type,
            parent_id: None, // Will be set by caller
            path,
            description: None,
            address: None,
            custom_data: Value::Null,
        }
    }

    /// Validates the location configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate name
        if self.name.is_empty() {
            return Err("Location name cannot be empty".to_string());
        }

        // Validate location type
        if self.location_type.is_empty() {
            return Err("Location type cannot be empty".to_string());
        }

        // Validate path consistency
        if self.path.is_empty() {
            return Err("Location path cannot be empty".to_string());
        }

        // For root locations, path should equal name
        if self.parent_id.is_none() && self.path != self.name {
            return Err("Root location path must equal name".to_string());
        }

        // For child locations, path should end with name
        if self.parent_id.is_some() && !self.path.ends_with(&self.name) {
            return Err("Location path must end with location name".to_string());
        }

        Ok(())
    }

    /// Updates the path based on parent path and name
    pub fn update_path(&mut self, parent_path: Option<&str>) {
        match parent_path {
            Some(parent) if !parent.is_empty() => {
                self.path = format!("{}/{}", parent, self.name);
            }
            _ => {
                self.path = self.name.clone();
            }
        }
    }

    /// Gets the depth level in the hierarchy (0 for root)
    pub fn get_depth(&self) -> usize {
        if self.path.is_empty() {
            0
        } else {
            self.path.matches('/').count()
        }
    }

    /// Gets all path components as a vector
    pub fn get_path_components(&self) -> Vec<&str> {
        if self.path.is_empty() {
            Vec::new()
        } else {
            self.path.split('/').collect()
        }
    }

    /// Checks if this location is an ancestor of another location
    pub fn is_ancestor_of(&self, other: &Location) -> bool {
        other.path.starts_with(&format!("{}/", self.path)) || 
        (self.parent_id.is_none() && other.parent_id.is_some() && other.path.starts_with(&self.path))
    }

    /// Checks if this location is a descendant of another location
    pub fn is_descendant_of(&self, other: &Location) -> bool {
        other.is_ancestor_of(self)
    }

    /// Checks if this location is a direct child of another location
    pub fn is_child_of(&self, other: &Location) -> bool {
        self.parent_id == Some(other.id)
    }

    /// Checks if this location is the direct parent of another location
    pub fn is_parent_of(&self, other: &Location) -> bool {
        other.is_child_of(self)
    }

    /// Gets a value from custom_data by path
    pub fn get_custom_data(&self, path: &str) -> Option<&Value> {
        // Simple dot-notation path traversal (same as Node and Link)
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

    /// Sets a value in custom_data by path
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
                    obj.insert(part.to_string(), value);
                    return Ok(());
                } else {
                    return Err("Cannot set value on non-object".to_string());
                }
            } else {
                // Navigate deeper, creating objects as needed
                if let Value::Object(obj) = current {
                    let entry = obj.entry(part.to_string()).or_insert_with(|| Value::Object(serde_json::Map::new()));
                    current = entry;
                } else {
                    return Err("Cannot navigate through non-object".to_string());
                }
            }
        }
        
        Ok(())
    }
}

/// Builder pattern for Location creation with validation
#[derive(Debug, Default)]
pub struct LocationBuilder {
    id: Option<Uuid>,
    name: Option<String>,
    location_type: Option<String>,
    parent_id: Option<Uuid>,
    parent_path: Option<String>,
    description: Option<String>,
    address: Option<String>,
    custom_data: Option<Value>,
}

impl LocationBuilder {
    /// Creates a new location builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the location ID (optional, will generate UUID if not provided)
    pub fn id(mut self, id: Uuid) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the location name (required)
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the location type (required)
    pub fn location_type<S: Into<String>>(mut self, location_type: S) -> Self {
        self.location_type = Some(location_type.into());
        self
    }

    /// Sets the parent location ID (optional for root locations)
    pub fn parent_id(mut self, parent_id: Uuid) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    /// Sets the parent path for building the full path (optional for root locations)
    pub fn parent_path<S: Into<String>>(mut self, parent_path: S) -> Self {
        self.parent_path = Some(parent_path.into());
        self
    }

    /// Sets the description (optional)
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the address (optional)
    pub fn address<S: Into<String>>(mut self, address: S) -> Self {
        self.address = Some(address.into());
        self
    }

    /// Sets custom data (optional)
    pub fn custom_data(mut self, custom_data: Value) -> Self {
        self.custom_data = Some(custom_data);
        self
    }

    /// Builds the location with validation
    pub fn build(self) -> Result<Location, String> {
        let name = self.name.ok_or("Name is required")?;
        let location_type = self.location_type.ok_or("Location type is required")?;

        let path = match self.parent_path {
            Some(parent_path) if !parent_path.is_empty() => {
                format!("{}/{}", parent_path, name)
            }
            _ => name.clone(),
        };

        let location = Location {
            id: self.id.unwrap_or_else(Uuid::new_v4),
            name,
            location_type,
            parent_id: self.parent_id,
            path,
            description: self.description,
            address: self.address,
            custom_data: self.custom_data.unwrap_or(Value::Null),
        };

        location.validate()?;
        Ok(location)
    }
}

/// Location hierarchy operations
impl Location {
    /// Detects circular references in a location hierarchy
    pub fn detect_circular_reference(
        locations: &[Location],
        potential_parent_id: Uuid,
        child_id: Uuid,
    ) -> bool {
        if potential_parent_id == child_id {
            return true;
        }

        // Find the potential parent
        let potential_parent = locations.iter().find(|l| l.id == potential_parent_id);
        if let Some(parent) = potential_parent {
            // Check if the child is an ancestor of the potential parent
            let child = locations.iter().find(|l| l.id == child_id);
            if let Some(child_loc) = child {
                return child_loc.is_ancestor_of(parent);
            }
        }

        false
    }

    /// Gets all ancestors of a location
    pub fn get_ancestors<'a>(
        &self,
        all_locations: &'a [Location],
    ) -> Vec<&'a Location> {
        let mut ancestors = Vec::new();
        let mut current = self;

        while let Some(parent_id) = current.parent_id {
            if let Some(parent) = all_locations.iter().find(|l| l.id == parent_id) {
                ancestors.push(parent);
                current = parent;
            } else {
                break; // Parent not found, stop traversal
            }
        }

        ancestors
    }

    /// Gets all descendants of a location
    pub fn get_descendants<'a>(
        &self,
        all_locations: &'a [Location],
    ) -> Vec<&'a Location> {
        let mut descendants = Vec::new();
        let mut to_check = vec![self.id];

        while let Some(current_id) = to_check.pop() {
            for location in all_locations {
                if location.parent_id == Some(current_id) {
                    descendants.push(location);
                    to_check.push(location.id);
                }
            }
        }

        descendants
    }

    /// Gets direct children of a location
    pub fn get_children<'a>(
        &self,
        all_locations: &'a [Location],
    ) -> Vec<&'a Location> {
        all_locations
            .iter()
            .filter(|l| l.parent_id == Some(self.id))
            .collect()
    }
}

/// Helper function to validate interface names
fn is_valid_interface_name(interface: &str) -> bool {
    if interface.is_empty() || interface.len() > 64 {
        return false;
    }

    // Allow alphanumeric characters, slashes, dashes, dots, and colons
    // Common patterns: eth0, GigabitEthernet0/0/1, xe-0/0/0, etc.
    interface.chars().all(|c| {
        c.is_ascii_alphanumeric() || c == '/' || c == '-' || c == '.' || c == ':'
    })
}

/// Helper function to validate domain names
fn is_valid_domain(domain: &str) -> bool {
    if domain.is_empty() || domain.len() > 253 {
        return false;
    }

    // Check for valid domain format (simplified validation)
    domain.split('.').all(|label| {
        !label.is_empty() 
            && label.len() <= 63 
            && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
            && !label.starts_with('-')
            && !label.ends_with('-')
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_lifecycle_display() {
        assert_eq!(Lifecycle::Planned.to_string(), "planned");
        assert_eq!(Lifecycle::Implementing.to_string(), "implementing");
        assert_eq!(Lifecycle::Live.to_string(), "live");
        assert_eq!(Lifecycle::Decommissioned.to_string(), "decommissioned");
    }

    #[test]
    fn test_lifecycle_from_str() {
        assert_eq!("planned".parse::<Lifecycle>().unwrap(), Lifecycle::Planned);
        assert_eq!("IMPLEMENTING".parse::<Lifecycle>().unwrap(), Lifecycle::Implementing);
        assert_eq!("Live".parse::<Lifecycle>().unwrap(), Lifecycle::Live);
        assert_eq!("DECOMMISSIONED".parse::<Lifecycle>().unwrap(), Lifecycle::Decommissioned);
        
        assert!("invalid".parse::<Lifecycle>().is_err());
    }

    #[test]
    fn test_lifecycle_from_string() {
        assert_eq!(Lifecycle::from("planned".to_string()), Lifecycle::Planned);
        assert_eq!(Lifecycle::from("invalid".to_string()), Lifecycle::Planned); // fallback
    }

    #[test]
    fn test_lifecycle_serde() {
        let planned = Lifecycle::Planned;
        let json = serde_json::to_string(&planned).unwrap();
        assert_eq!(json, "\"planned\"");
        
        let deserialized: Lifecycle = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, planned);
    }

    #[test]
    fn test_device_role_display() {
        assert_eq!(DeviceRole::Router.to_string(), "router");
        assert_eq!(DeviceRole::Switch.to_string(), "switch");
        assert_eq!(DeviceRole::Firewall.to_string(), "firewall");
        assert_eq!(DeviceRole::LoadBalancer.to_string(), "loadbalancer");
        assert_eq!(DeviceRole::AccessPoint.to_string(), "accesspoint");
        assert_eq!(DeviceRole::SecurityAppliance.to_string(), "securityappliance");
        assert_eq!(DeviceRole::Monitor.to_string(), "monitor");
        assert_eq!(DeviceRole::Server.to_string(), "server");
        assert_eq!(DeviceRole::Storage.to_string(), "storage");
        assert_eq!(DeviceRole::Other.to_string(), "other");
    }

    #[test]
    fn test_device_role_from_str() {
        assert_eq!("router".parse::<DeviceRole>().unwrap(), DeviceRole::Router);
        assert_eq!("SWITCH".parse::<DeviceRole>().unwrap(), DeviceRole::Switch);
        assert_eq!("Firewall".parse::<DeviceRole>().unwrap(), DeviceRole::Firewall);
        
        assert!("invalid".parse::<DeviceRole>().is_err());
    }

    #[test]
    fn test_device_role_from_string() {
        assert_eq!(DeviceRole::from("router".to_string()), DeviceRole::Router);
        assert_eq!(DeviceRole::from("invalid".to_string()), DeviceRole::Other); // fallback
    }

    #[test]
    fn test_device_role_serde() {
        let router = DeviceRole::Router;
        let json = serde_json::to_string(&router).unwrap();
        assert_eq!(json, "\"router\"");
        
        let deserialized: DeviceRole = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, router);
    }

    #[test]
    fn test_vendor_display() {
        assert_eq!(Vendor::Cisco.to_string(), "cisco");
        assert_eq!(Vendor::Juniper.to_string(), "juniper");
        assert_eq!(Vendor::Arista.to_string(), "arista");
        assert_eq!(Vendor::PaloAlto.to_string(), "paloalto");
        assert_eq!(Vendor::Fortinet.to_string(), "fortinet");
        assert_eq!(Vendor::Hpe.to_string(), "hpe");
        assert_eq!(Vendor::Dell.to_string(), "dell");
        assert_eq!(Vendor::Extreme.to_string(), "extreme");
        assert_eq!(Vendor::Mikrotik.to_string(), "mikrotik");
        assert_eq!(Vendor::Ubiquiti.to_string(), "ubiquiti");
        assert_eq!(Vendor::Generic.to_string(), "generic");
    }

    #[test]
    fn test_vendor_from_str() {
        assert_eq!("cisco".parse::<Vendor>().unwrap(), Vendor::Cisco);
        assert_eq!("JUNIPER".parse::<Vendor>().unwrap(), Vendor::Juniper);
        assert_eq!("Arista".parse::<Vendor>().unwrap(), Vendor::Arista);
        
        assert!("invalid".parse::<Vendor>().is_err());
    }

    #[test]
    fn test_vendor_from_string() {
        assert_eq!(Vendor::from("cisco".to_string()), Vendor::Cisco);
        assert_eq!(Vendor::from("invalid".to_string()), Vendor::Generic); // fallback
    }

    #[test]
    fn test_vendor_serde() {
        let cisco = Vendor::Cisco;
        let json = serde_json::to_string(&cisco).unwrap();
        assert_eq!(json, "\"cisco\"");
        
        let deserialized: Vendor = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, cisco);
    }

    #[test]
    fn test_all_enum_variants_coverage() {
        // Test that all variants are handled in serialization/deserialization
        let lifecycles = [
            Lifecycle::Planned,
            Lifecycle::Implementing,
            Lifecycle::Live,
            Lifecycle::Decommissioned,
        ];
        
        for lifecycle in lifecycles {
            let json = serde_json::to_string(&lifecycle).unwrap();
            let deserialized: Lifecycle = serde_json::from_str(&json).unwrap();
            assert_eq!(lifecycle, deserialized);
        }

        let roles = [
            DeviceRole::Router,
            DeviceRole::Switch,
            DeviceRole::Firewall,
            DeviceRole::LoadBalancer,
            DeviceRole::AccessPoint,
            DeviceRole::SecurityAppliance,
            DeviceRole::Monitor,
            DeviceRole::Server,
            DeviceRole::Storage,
            DeviceRole::Other,
        ];
        
        for role in roles {
            let json = serde_json::to_string(&role).unwrap();
            let deserialized: DeviceRole = serde_json::from_str(&json).unwrap();
            assert_eq!(role, deserialized);
        }

        let vendors = [
            Vendor::Cisco,
            Vendor::Juniper,
            Vendor::Arista,
            Vendor::PaloAlto,
            Vendor::Fortinet,
            Vendor::Hpe,
            Vendor::Dell,
            Vendor::Extreme,
            Vendor::Mikrotik,
            Vendor::Ubiquiti,
            Vendor::Generic,
        ];
        
        for vendor in vendors {
            let json = serde_json::to_string(&vendor).unwrap();
            let deserialized: Vendor = serde_json::from_str(&json).unwrap();
            assert_eq!(vendor, deserialized);
        }
    }

    // Node tests
    #[test]
    fn test_node_new() {
        let node = Node::new(
            "router1".to_string(),
            "example.com".to_string(),
            Vendor::Cisco,
            DeviceRole::Router,
        );

        assert_eq!(node.name, "router1");
        assert_eq!(node.domain, "example.com");
        assert_eq!(node.fqdn, "router1.example.com");
        assert_eq!(node.vendor, Vendor::Cisco);
        assert_eq!(node.role, DeviceRole::Router);
        assert_eq!(node.lifecycle, Lifecycle::Planned);
        assert!(node.model.is_empty());
        assert!(node.custom_data.is_null());
    }

    #[test]
    fn test_node_new_no_domain() {
        let node = Node::new(
            "router1".to_string(),
            "".to_string(),
            Vendor::Cisco,
            DeviceRole::Router,
        );

        assert_eq!(node.name, "router1");
        assert_eq!(node.domain, "");
        assert_eq!(node.fqdn, "router1");
    }

    #[test]
    fn test_node_validation_success() {
        let mut node = Node::new(
            "router1".to_string(),
            "example.com".to_string(),
            Vendor::Cisco,
            DeviceRole::Router,
        );
        node.model = "ISR4331".to_string();

        assert!(node.validate().is_ok());
    }

    #[test]
    fn test_node_validation_empty_name() {
        let mut node = Node::new(
            "".to_string(),
            "example.com".to_string(),
            Vendor::Cisco,
            DeviceRole::Router,
        );
        node.model = "ISR4331".to_string();

        assert!(node.validate().is_err());
        assert!(node.validate().unwrap_err().contains("name cannot be empty"));
    }

    #[test]
    fn test_node_validation_invalid_name() {
        let mut node = Node::new(
            "router@1".to_string(),
            "example.com".to_string(),
            Vendor::Cisco,
            DeviceRole::Router,
        );
        node.model = "ISR4331".to_string();
        node.update_fqdn();

        assert!(node.validate().is_err());
        assert!(node.validate().unwrap_err().contains("alphanumeric"));
    }

    #[test]
    fn test_node_validation_invalid_domain() {
        let mut node = Node::new(
            "router1".to_string(),
            "invalid..domain".to_string(),
            Vendor::Cisco,
            DeviceRole::Router,
        );
        node.model = "ISR4331".to_string();
        node.update_fqdn();

        assert!(node.validate().is_err());
        assert!(node.validate().unwrap_err().contains("Invalid domain format"));
    }

    #[test]
    fn test_node_validation_empty_model() {
        let node = Node::new(
            "router1".to_string(),
            "example.com".to_string(),
            Vendor::Cisco,
            DeviceRole::Router,
        );

        assert!(node.validate().is_err());
        assert!(node.validate().unwrap_err().contains("Model cannot be empty"));
    }

    #[test]
    fn test_node_validation_fqdn_mismatch() {
        let mut node = Node::new(
            "router1".to_string(),
            "example.com".to_string(),
            Vendor::Cisco,
            DeviceRole::Router,
        );
        node.model = "ISR4331".to_string();
        node.fqdn = "wrong.fqdn.com".to_string();

        assert!(node.validate().is_err());
        assert!(node.validate().unwrap_err().contains("FQDN must match"));
    }

    #[test]
    fn test_node_update_fqdn() {
        let mut node = Node::new(
            "router1".to_string(),
            "example.com".to_string(),
            Vendor::Cisco,
            DeviceRole::Router,
        );

        node.name = "router2".to_string();
        node.domain = "newdomain.com".to_string();
        node.update_fqdn();

        assert_eq!(node.fqdn, "router2.newdomain.com");
    }

    #[test]
    fn test_node_custom_data() {
        let mut node = Node::new(
            "router1".to_string(),
            "example.com".to_string(),
            Vendor::Cisco,
            DeviceRole::Router,
        );

        // Set custom data
        let value = serde_json::json!("test_value");
        assert!(node.set_custom_data("config.vlan", value.clone()).is_ok());

        // Get custom data
        let retrieved = node.get_custom_data("config.vlan");
        assert_eq!(retrieved, Some(&value));

        // Test nested path
        let nested_value = serde_json::json!(42);
        assert!(node.set_custom_data("config.ports.count", nested_value.clone()).is_ok());
        
        let retrieved_nested = node.get_custom_data("config.ports.count");
        assert_eq!(retrieved_nested, Some(&nested_value));

        // Test non-existent path
        let missing = node.get_custom_data("nonexistent.path");
        assert_eq!(missing, None);
    }

    #[test]
    fn test_node_builder_success() {
        use std::net::Ipv4Addr;

        let node = NodeBuilder::new()
            .name("router1")
            .domain("example.com")
            .vendor(Vendor::Cisco)
            .model("ISR4331")
            .role(DeviceRole::Router)
            .lifecycle(Lifecycle::Live)
            .management_ip(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)))
            .platform("IOS XE")
            .version("16.12.04")
            .build()
            .unwrap();

        assert_eq!(node.name, "router1");
        assert_eq!(node.domain, "example.com");
        assert_eq!(node.fqdn, "router1.example.com");
        assert_eq!(node.vendor, Vendor::Cisco);
        assert_eq!(node.model, "ISR4331");
        assert_eq!(node.role, DeviceRole::Router);
        assert_eq!(node.lifecycle, Lifecycle::Live);
        assert_eq!(node.management_ip, Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))));
        assert_eq!(node.platform, Some("IOS XE".to_string()));
        assert_eq!(node.version, Some("16.12.04".to_string()));
    }

    #[test]
    fn test_node_builder_missing_required_fields() {
        let result = NodeBuilder::new()
            .name("router1")
            // Missing domain, vendor, model, role
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Vendor is required"));
    }

    #[test]
    fn test_node_builder_validation_failure() {
        let result = NodeBuilder::new()
            .name("") // Invalid empty name
            .domain("example.com")
            .vendor(Vendor::Cisco)
            .model("ISR4331")
            .role(DeviceRole::Router)
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("name cannot be empty"));
    }

    #[test]
    fn test_node_builder_custom_data() {
        let custom_data = serde_json::json!({
            "config": {
                "vlans": [10, 20, 30]
            }
        });

        let node = NodeBuilder::new()
            .name("switch1")
            .domain("example.com")
            .vendor(Vendor::Cisco)
            .model("Catalyst 9300")
            .role(DeviceRole::Switch)
            .custom_data(custom_data.clone())
            .build()
            .unwrap();

        assert_eq!(node.custom_data, custom_data);
        assert_eq!(
            node.get_custom_data("config.vlans"), 
            Some(&serde_json::json!([10, 20, 30]))
        );
    }

    #[test]
    fn test_node_serde() {
        let node = NodeBuilder::new()
            .name("router1")
            .domain("example.com")
            .vendor(Vendor::Cisco)
            .model("ISR4331")
            .role(DeviceRole::Router)
            .build()
            .unwrap();

        let json = serde_json::to_string(&node).unwrap();
        let deserialized: Node = serde_json::from_str(&json).unwrap();

        assert_eq!(node.name, deserialized.name);
        assert_eq!(node.domain, deserialized.domain);
        assert_eq!(node.fqdn, deserialized.fqdn);
        assert_eq!(node.vendor, deserialized.vendor);
        assert_eq!(node.model, deserialized.model);
        assert_eq!(node.role, deserialized.role);
    }

    #[test]
    fn test_is_valid_domain() {
        assert!(is_valid_domain("example.com"));
        assert!(is_valid_domain("sub.example.com"));
        assert!(is_valid_domain("a.b.c.d.example.com"));
        assert!(is_valid_domain("test-domain.com"));
        assert!(is_valid_domain("123.example.com"));

        assert!(!is_valid_domain(""));
        assert!(!is_valid_domain("invalid..domain"));
        assert!(!is_valid_domain("-invalid.com"));
        assert!(!is_valid_domain("invalid-.com"));
        assert!(!is_valid_domain("too-long-label-that-exceeds-sixty-three-characters-which-is-invalid.com"));
    }

    // Link tests
    #[test]
    fn test_link_new() {
        let node_a_id = Uuid::new_v4();
        let node_z_id = Uuid::new_v4();
        
        let link = Link::new(
            "link1".to_string(),
            node_a_id,
            "eth0".to_string(),
            node_z_id,
            "eth1".to_string(),
        );

        assert_eq!(link.name, "link1");
        assert_eq!(link.node_a_id, node_a_id);
        assert_eq!(link.node_a_interface, "eth0");
        assert_eq!(link.node_z_id, Some(node_z_id));
        assert_eq!(link.node_z_interface, Some("eth1".to_string()));
        assert!(!link.is_internet_circuit);
        assert!(link.custom_data.is_null());
    }

    #[test]
    fn test_link_new_internet_circuit() {
        let node_a_id = Uuid::new_v4();
        
        let link = Link::new_internet_circuit(
            "internet-link".to_string(),
            node_a_id,
            "eth0".to_string(),
        );

        assert_eq!(link.name, "internet-link");
        assert_eq!(link.node_a_id, node_a_id);
        assert_eq!(link.node_a_interface, "eth0");
        assert_eq!(link.node_z_id, None);
        assert_eq!(link.node_z_interface, None);
        assert!(link.is_internet_circuit);
    }

    #[test]
    fn test_link_validation_success() {
        let node_a_id = Uuid::new_v4();
        let node_z_id = Uuid::new_v4();
        
        let link = Link::new(
            "link1".to_string(),
            node_a_id,
            "eth0".to_string(),
            node_z_id,
            "eth1".to_string(),
        );

        assert!(link.validate().is_ok());
    }

    #[test]
    fn test_link_validation_internet_circuit_success() {
        let node_a_id = Uuid::new_v4();
        
        let link = Link::new_internet_circuit(
            "internet-link".to_string(),
            node_a_id,
            "eth0".to_string(),
        );

        assert!(link.validate().is_ok());
    }

    #[test]
    fn test_link_validation_empty_name() {
        let node_a_id = Uuid::new_v4();
        let node_z_id = Uuid::new_v4();
        
        let link = Link::new(
            "".to_string(),
            node_a_id,
            "eth0".to_string(),
            node_z_id,
            "eth1".to_string(),
        );

        assert!(link.validate().is_err());
        assert!(link.validate().unwrap_err().contains("name cannot be empty"));
    }

    #[test]
    fn test_link_validation_empty_interface() {
        let node_a_id = Uuid::new_v4();
        let node_z_id = Uuid::new_v4();
        
        let link = Link::new(
            "link1".to_string(),
            node_a_id,
            "".to_string(),
            node_z_id,
            "eth1".to_string(),
        );

        assert!(link.validate().is_err());
        assert!(link.validate().unwrap_err().contains("Node A interface cannot be empty"));
    }

    #[test]
    fn test_link_validation_invalid_interface() {
        let node_a_id = Uuid::new_v4();
        let node_z_id = Uuid::new_v4();
        
        let link = Link::new(
            "link1".to_string(),
            node_a_id,
            "invalid@interface".to_string(),
            node_z_id,
            "eth1".to_string(),
        );

        assert!(link.validate().is_err());
        assert!(link.validate().unwrap_err().contains("Invalid node A interface name format"));
    }

    #[test]
    fn test_link_validation_self_link() {
        let node_id = Uuid::new_v4();
        
        let link = Link::new(
            "self-link".to_string(),
            node_id,
            "eth0".to_string(),
            node_id, // Same node!
            "eth1".to_string(),
        );

        assert!(link.validate().is_err());
        assert!(link.validate().unwrap_err().contains("cannot connect a node to itself"));
    }

    #[test]
    fn test_link_validation_internet_circuit_with_node_z() {
        let node_a_id = Uuid::new_v4();
        let node_z_id = Uuid::new_v4();
        
        let mut link = Link::new_internet_circuit(
            "internet-link".to_string(),
            node_a_id,
            "eth0".to_string(),
        );
        
        // Manually set node_z_id (invalid for internet circuit)
        link.node_z_id = Some(node_z_id);

        assert!(link.validate().is_err());
        assert!(link.validate().unwrap_err().contains("Internet circuits cannot have node Z"));
    }

    #[test]
    fn test_link_validation_regular_link_missing_node_z() {
        let node_a_id = Uuid::new_v4();
        
        let mut link = Link::new_internet_circuit(
            "regular-link".to_string(),
            node_a_id,
            "eth0".to_string(),
        );
        
        // Make it not an internet circuit but leave node_z_id as None
        link.is_internet_circuit = false;

        assert!(link.validate().is_err());
        assert!(link.validate().unwrap_err().contains("Regular links must have node Z"));
    }

    #[test]
    fn test_link_get_other_node_id() {
        let node_a_id = Uuid::new_v4();
        let node_z_id = Uuid::new_v4();
        let other_node_id = Uuid::new_v4();
        
        let link = Link::new(
            "link1".to_string(),
            node_a_id,
            "eth0".to_string(),
            node_z_id,
            "eth1".to_string(),
        );

        assert_eq!(link.get_other_node_id(node_a_id), Some(node_z_id));
        assert_eq!(link.get_other_node_id(node_z_id), Some(node_a_id));
        assert_eq!(link.get_other_node_id(other_node_id), None);
    }

    #[test]
    fn test_link_get_interface_for_node() {
        let node_a_id = Uuid::new_v4();
        let node_z_id = Uuid::new_v4();
        let other_node_id = Uuid::new_v4();
        
        let link = Link::new(
            "link1".to_string(),
            node_a_id,
            "eth0".to_string(),
            node_z_id,
            "eth1".to_string(),
        );

        assert_eq!(link.get_interface_for_node(node_a_id), Some("eth0"));
        assert_eq!(link.get_interface_for_node(node_z_id), Some("eth1"));
        assert_eq!(link.get_interface_for_node(other_node_id), None);
    }

    #[test]
    fn test_link_connects_nodes() {
        let node_a_id = Uuid::new_v4();
        let node_z_id = Uuid::new_v4();
        let other_node_id = Uuid::new_v4();
        
        let link = Link::new(
            "link1".to_string(),
            node_a_id,
            "eth0".to_string(),
            node_z_id,
            "eth1".to_string(),
        );

        assert!(link.connects_nodes(node_a_id, node_z_id));
        assert!(link.connects_nodes(node_z_id, node_a_id));
        assert!(!link.connects_nodes(node_a_id, other_node_id));
        assert!(!link.connects_nodes(other_node_id, node_z_id));
    }

    #[test]
    fn test_link_involves_node() {
        let node_a_id = Uuid::new_v4();
        let node_z_id = Uuid::new_v4();
        let other_node_id = Uuid::new_v4();
        
        let link = Link::new(
            "link1".to_string(),
            node_a_id,
            "eth0".to_string(),
            node_z_id,
            "eth1".to_string(),
        );

        assert!(link.involves_node(node_a_id));
        assert!(link.involves_node(node_z_id));
        assert!(!link.involves_node(other_node_id));
    }

    #[test]
    fn test_link_custom_data() {
        let node_a_id = Uuid::new_v4();
        let node_z_id = Uuid::new_v4();
        
        let mut link = Link::new(
            "link1".to_string(),
            node_a_id,
            "eth0".to_string(),
            node_z_id,
            "eth1".to_string(),
        );

        // Set custom data
        let value = serde_json::json!("1Gbps");
        assert!(link.set_custom_data("qos.bandwidth", value.clone()).is_ok());

        // Get custom data
        let retrieved = link.get_custom_data("qos.bandwidth");
        assert_eq!(retrieved, Some(&value));

        // Test non-existent path
        let missing = link.get_custom_data("nonexistent.path");
        assert_eq!(missing, None);
    }

    #[test]
    fn test_link_builder_success() {
        let node_a_id = Uuid::new_v4();
        let node_z_id = Uuid::new_v4();

        let link = LinkBuilder::new()
            .name("core-link")
            .node_a_id(node_a_id)
            .node_a_interface("GigabitEthernet0/0/1")
            .node_z_id(node_z_id)
            .node_z_interface("GigabitEthernet0/0/2")
            .description("Core network link")
            .bandwidth(1_000_000_000) // 1 Gbps
            .link_type("fiber")
            .build()
            .unwrap();

        assert_eq!(link.name, "core-link");
        assert_eq!(link.node_a_id, node_a_id);
        assert_eq!(link.node_a_interface, "GigabitEthernet0/0/1");
        assert_eq!(link.node_z_id, Some(node_z_id));
        assert_eq!(link.node_z_interface, Some("GigabitEthernet0/0/2".to_string()));
        assert_eq!(link.description, Some("Core network link".to_string()));
        assert_eq!(link.bandwidth, Some(1_000_000_000));
        assert_eq!(link.link_type, Some("fiber".to_string()));
        assert!(!link.is_internet_circuit);
    }

    #[test]
    fn test_link_builder_internet_circuit() {
        let node_a_id = Uuid::new_v4();

        let link = LinkBuilder::new()
            .name("internet-circuit")
            .node_a_id(node_a_id)
            .node_a_interface("eth0")
            .is_internet_circuit(true)
            .bandwidth(100_000_000) // 100 Mbps
            .build()
            .unwrap();

        assert_eq!(link.name, "internet-circuit");
        assert_eq!(link.node_a_id, node_a_id);
        assert_eq!(link.node_a_interface, "eth0");
        assert_eq!(link.node_z_id, None);
        assert_eq!(link.node_z_interface, None);
        assert!(link.is_internet_circuit);
        assert_eq!(link.bandwidth, Some(100_000_000));
    }

    #[test]
    fn test_link_builder_missing_required_fields() {
        let result = LinkBuilder::new()
            .name("incomplete-link")
            // Missing node_a_id and node_a_interface
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Node A ID is required"));
    }

    #[test]
    fn test_link_builder_validation_failure() {
        let node_a_id = Uuid::new_v4();

        let result = LinkBuilder::new()
            .name("") // Invalid empty name
            .node_a_id(node_a_id)
            .node_a_interface("eth0")
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("name cannot be empty"));
    }

    #[test]
    fn test_link_serde() {
        let node_a_id = Uuid::new_v4();
        let node_z_id = Uuid::new_v4();

        let link = LinkBuilder::new()
            .name("test-link")
            .node_a_id(node_a_id)
            .node_a_interface("eth0")
            .node_z_id(node_z_id)
            .node_z_interface("eth1")
            .build()
            .unwrap();

        let json = serde_json::to_string(&link).unwrap();
        let deserialized: Link = serde_json::from_str(&json).unwrap();

        assert_eq!(link.name, deserialized.name);
        assert_eq!(link.node_a_id, deserialized.node_a_id);
        assert_eq!(link.node_a_interface, deserialized.node_a_interface);
        assert_eq!(link.node_z_id, deserialized.node_z_id);
        assert_eq!(link.node_z_interface, deserialized.node_z_interface);
    }

    #[test]
    fn test_is_valid_interface_name() {
        // Valid interface names
        assert!(is_valid_interface_name("eth0"));
        assert!(is_valid_interface_name("GigabitEthernet0/0/1"));
        assert!(is_valid_interface_name("xe-0/0/0"));
        assert!(is_valid_interface_name("FastEthernet0/1"));
        assert!(is_valid_interface_name("Port-channel1"));
        assert!(is_valid_interface_name("Vlan100"));
        assert!(is_valid_interface_name("mgmt0"));
        assert!(is_valid_interface_name("lo0"));

        // Invalid interface names
        assert!(!is_valid_interface_name(""));
        assert!(!is_valid_interface_name("invalid@interface"));
        assert!(!is_valid_interface_name("interface with spaces"));
        assert!(!is_valid_interface_name("interface#with#hash"));
        
        // Test length limit
        let long_interface = "a".repeat(65);
        assert!(!is_valid_interface_name(&long_interface));
    }

    // Location tests
    #[test]
    fn test_location_new_root() {
        let location = Location::new_root("USA".to_string(), "country".to_string());

        assert_eq!(location.name, "USA");
        assert_eq!(location.location_type, "country");
        assert_eq!(location.parent_id, None);
        assert_eq!(location.path, "USA");
        assert!(location.custom_data.is_null());
    }

    #[test]
    fn test_location_new_child() {
        let location = Location::new_child(
            "California".to_string(),
            "state".to_string(),
            "USA".to_string(),
        );

        assert_eq!(location.name, "California");
        assert_eq!(location.location_type, "state");
        assert_eq!(location.parent_id, None); // Will be set by caller
        assert_eq!(location.path, "USA/California");
    }

    #[test]
    fn test_location_new_child_empty_parent() {
        let location = Location::new_child(
            "RootLocation".to_string(),
            "building".to_string(),
            "".to_string(),
        );

        assert_eq!(location.path, "RootLocation");
    }

    #[test]
    fn test_location_validation_success() {
        let location = Location::new_root("USA".to_string(), "country".to_string());
        assert!(location.validate().is_ok());

        let mut child = Location::new_child(
            "California".to_string(),
            "state".to_string(),
            "USA".to_string(),
        );
        child.parent_id = Some(Uuid::new_v4());
        assert!(child.validate().is_ok());
    }

    #[test]
    fn test_location_validation_empty_name() {
        let location = Location::new_root("".to_string(), "country".to_string());
        assert!(location.validate().is_err());
        assert!(location.validate().unwrap_err().contains("name cannot be empty"));
    }

    #[test]
    fn test_location_validation_empty_type() {
        let location = Location::new_root("USA".to_string(), "".to_string());
        assert!(location.validate().is_err());
        assert!(location.validate().unwrap_err().contains("type cannot be empty"));
    }

    #[test]
    fn test_location_validation_root_path_mismatch() {
        let mut location = Location::new_root("USA".to_string(), "country".to_string());
        location.path = "Wrong".to_string();
        
        assert!(location.validate().is_err());
        assert!(location.validate().unwrap_err().contains("Root location path must equal name"));
    }

    #[test]
    fn test_location_validation_child_path_mismatch() {
        let mut location = Location::new_child(
            "California".to_string(),
            "state".to_string(),
            "USA".to_string(),
        );
        location.parent_id = Some(Uuid::new_v4());
        location.path = "USA/WrongState".to_string();
        
        assert!(location.validate().is_err());
        assert!(location.validate().unwrap_err().contains("path must end with location name"));
    }

    #[test]
    fn test_location_update_path() {
        let mut location = Location::new_root("California".to_string(), "state".to_string());
        
        // Update with parent path
        location.update_path(Some("USA"));
        assert_eq!(location.path, "USA/California");

        // Update without parent path
        location.update_path(None);
        assert_eq!(location.path, "California");

        // Update with empty parent path
        location.update_path(Some(""));
        assert_eq!(location.path, "California");
    }

    #[test]
    fn test_location_get_depth() {
        let root = Location::new_root("USA".to_string(), "country".to_string());
        assert_eq!(root.get_depth(), 0);

        let child = Location::new_child(
            "California".to_string(),
            "state".to_string(),
            "USA".to_string(),
        );
        assert_eq!(child.get_depth(), 1);

        let grandchild = Location::new_child(
            "San Francisco".to_string(),
            "city".to_string(),
            "USA/California".to_string(),
        );
        assert_eq!(grandchild.get_depth(), 2);
    }

    #[test]
    fn test_location_get_path_components() {
        let root = Location::new_root("USA".to_string(), "country".to_string());
        assert_eq!(root.get_path_components(), vec!["USA"]);

        let child = Location::new_child(
            "California".to_string(),
            "state".to_string(),
            "USA".to_string(),
        );
        assert_eq!(child.get_path_components(), vec!["USA", "California"]);

        let grandchild = Location::new_child(
            "San Francisco".to_string(),
            "city".to_string(),
            "USA/California".to_string(),
        );
        assert_eq!(grandchild.get_path_components(), vec!["USA", "California", "San Francisco"]);
    }

    #[test]
    fn test_location_hierarchy_relationships() {
        let root = Location::new_root("USA".to_string(), "country".to_string());
        
        let mut child = Location::new_child(
            "California".to_string(),
            "state".to_string(),
            "USA".to_string(),
        );
        child.parent_id = Some(root.id);

        let mut grandchild = Location::new_child(
            "San Francisco".to_string(),
            "city".to_string(),
            "USA/California".to_string(),
        );
        grandchild.parent_id = Some(child.id);

        // Test ancestor relationships
        assert!(root.is_ancestor_of(&child));
        assert!(root.is_ancestor_of(&grandchild));
        assert!(child.is_ancestor_of(&grandchild));
        assert!(!child.is_ancestor_of(&root));
        assert!(!grandchild.is_ancestor_of(&root));

        // Test descendant relationships
        assert!(child.is_descendant_of(&root));
        assert!(grandchild.is_descendant_of(&root));
        assert!(grandchild.is_descendant_of(&child));
        assert!(!root.is_descendant_of(&child));

        // Test parent-child relationships
        assert!(child.is_child_of(&root));
        assert!(grandchild.is_child_of(&child));
        assert!(root.is_parent_of(&child));
        assert!(child.is_parent_of(&grandchild));
        assert!(!root.is_child_of(&child));
        assert!(!child.is_parent_of(&root));
    }

    #[test]
    fn test_location_detect_circular_reference() {
        let root = Location::new_root("USA".to_string(), "country".to_string());
        
        let mut child = Location::new_child(
            "California".to_string(),
            "state".to_string(),
            "USA".to_string(),
        );
        child.parent_id = Some(root.id);

        let locations = vec![root.clone(), child.clone()];

        // Test self-reference
        assert!(Location::detect_circular_reference(&locations, root.id, root.id));

        // Test valid parent-child
        assert!(!Location::detect_circular_reference(&locations, root.id, child.id));

        // Test circular reference (child becoming parent of its ancestor)
        assert!(Location::detect_circular_reference(&locations, child.id, root.id));
    }

    #[test]
    fn test_location_get_ancestors() {
        let root = Location::new_root("USA".to_string(), "country".to_string());
        
        let mut child = Location::new_child(
            "California".to_string(),
            "state".to_string(),
            "USA".to_string(),
        );
        child.parent_id = Some(root.id);

        let mut grandchild = Location::new_child(
            "San Francisco".to_string(),
            "city".to_string(),
            "USA/California".to_string(),
        );
        grandchild.parent_id = Some(child.id);

        let locations = vec![root.clone(), child.clone(), grandchild.clone()];

        // Root has no ancestors
        let root_ancestors = root.get_ancestors(&locations);
        assert!(root_ancestors.is_empty());

        // Child has root as ancestor
        let child_ancestors = child.get_ancestors(&locations);
        assert_eq!(child_ancestors.len(), 1);
        assert_eq!(child_ancestors[0].id, root.id);

        // Grandchild has child and root as ancestors
        let grandchild_ancestors = grandchild.get_ancestors(&locations);
        assert_eq!(grandchild_ancestors.len(), 2);
        assert_eq!(grandchild_ancestors[0].id, child.id);
        assert_eq!(grandchild_ancestors[1].id, root.id);
    }

    #[test]
    fn test_location_get_descendants() {
        let root = Location::new_root("USA".to_string(), "country".to_string());
        
        let mut child1 = Location::new_child(
            "California".to_string(),
            "state".to_string(),
            "USA".to_string(),
        );
        child1.parent_id = Some(root.id);

        let mut child2 = Location::new_child(
            "Texas".to_string(),
            "state".to_string(),
            "USA".to_string(),
        );
        child2.parent_id = Some(root.id);

        let mut grandchild = Location::new_child(
            "San Francisco".to_string(),
            "city".to_string(),
            "USA/California".to_string(),
        );
        grandchild.parent_id = Some(child1.id);

        let locations = vec![root.clone(), child1.clone(), child2.clone(), grandchild.clone()];

        // Root has all others as descendants
        let root_descendants = root.get_descendants(&locations);
        assert_eq!(root_descendants.len(), 3);

        // Child1 has only grandchild as descendant
        let child1_descendants = child1.get_descendants(&locations);
        assert_eq!(child1_descendants.len(), 1);
        assert_eq!(child1_descendants[0].id, grandchild.id);

        // Child2 has no descendants
        let child2_descendants = child2.get_descendants(&locations);
        assert!(child2_descendants.is_empty());

        // Grandchild has no descendants
        let grandchild_descendants = grandchild.get_descendants(&locations);
        assert!(grandchild_descendants.is_empty());
    }

    #[test]
    fn test_location_get_children() {
        let root = Location::new_root("USA".to_string(), "country".to_string());
        
        let mut child1 = Location::new_child(
            "California".to_string(),
            "state".to_string(),
            "USA".to_string(),
        );
        child1.parent_id = Some(root.id);

        let mut child2 = Location::new_child(
            "Texas".to_string(),
            "state".to_string(),
            "USA".to_string(),
        );
        child2.parent_id = Some(root.id);

        let mut grandchild = Location::new_child(
            "San Francisco".to_string(),
            "city".to_string(),
            "USA/California".to_string(),
        );
        grandchild.parent_id = Some(child1.id);

        let locations = vec![root.clone(), child1.clone(), child2.clone(), grandchild.clone()];

        // Root has two direct children
        let root_children = root.get_children(&locations);
        assert_eq!(root_children.len(), 2);
        let child_ids: Vec<Uuid> = root_children.iter().map(|l| l.id).collect();
        assert!(child_ids.contains(&child1.id));
        assert!(child_ids.contains(&child2.id));

        // Child1 has one direct child
        let child1_children = child1.get_children(&locations);
        assert_eq!(child1_children.len(), 1);
        assert_eq!(child1_children[0].id, grandchild.id);

        // Child2 has no direct children
        let child2_children = child2.get_children(&locations);
        assert!(child2_children.is_empty());
    }

    #[test]
    fn test_location_custom_data() {
        let mut location = Location::new_root("USA".to_string(), "country".to_string());

        // Set custom data
        let value = serde_json::json!("UTC-8");
        assert!(location.set_custom_data("timezone", value.clone()).is_ok());

        // Get custom data
        let retrieved = location.get_custom_data("timezone");
        assert_eq!(retrieved, Some(&value));

        // Test nested path
        let coords = serde_json::json!({"lat": 37.7749, "lng": -122.4194});
        assert!(location.set_custom_data("coordinates.center", coords.clone()).is_ok());
        
        let retrieved_coords = location.get_custom_data("coordinates.center");
        assert_eq!(retrieved_coords, Some(&coords));
    }

    #[test]
    fn test_location_builder_success() {
        let parent_id = Uuid::new_v4();

        let location = LocationBuilder::new()
            .name("Building A")
            .location_type("building")
            .parent_id(parent_id)
            .parent_path("USA/California/San Francisco")
            .description("Main office building")
            .address("123 Main St, San Francisco, CA")
            .build()
            .unwrap();

        assert_eq!(location.name, "Building A");
        assert_eq!(location.location_type, "building");
        assert_eq!(location.parent_id, Some(parent_id));
        assert_eq!(location.path, "USA/California/San Francisco/Building A");
        assert_eq!(location.description, Some("Main office building".to_string()));
        assert_eq!(location.address, Some("123 Main St, San Francisco, CA".to_string()));
    }

    #[test]
    fn test_location_builder_root_location() {
        let location = LocationBuilder::new()
            .name("USA")
            .location_type("country")
            .description("United States of America")
            .build()
            .unwrap();

        assert_eq!(location.name, "USA");
        assert_eq!(location.location_type, "country");
        assert_eq!(location.parent_id, None);
        assert_eq!(location.path, "USA");
        assert_eq!(location.description, Some("United States of America".to_string()));
    }

    #[test]
    fn test_location_builder_missing_required_fields() {
        let result = LocationBuilder::new()
            .name("Incomplete")
            // Missing location_type
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Location type is required"));
    }

    #[test]
    fn test_location_builder_validation_failure() {
        let result = LocationBuilder::new()
            .name("") // Invalid empty name
            .location_type("building")
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("name cannot be empty"));
    }

    #[test]
    fn test_location_serde() {
        let location = LocationBuilder::new()
            .name("Test Location")
            .location_type("test")
            .description("Test description")
            .build()
            .unwrap();

        let json = serde_json::to_string(&location).unwrap();
        let deserialized: Location = serde_json::from_str(&json).unwrap();

        assert_eq!(location.name, deserialized.name);
        assert_eq!(location.location_type, deserialized.location_type);
        assert_eq!(location.parent_id, deserialized.parent_id);
        assert_eq!(location.path, deserialized.path);
        assert_eq!(location.description, deserialized.description);
    }
}
