//! Link model and implementation
//!
//! Contains the core `Link` struct representing network connections between devices
//! and the `LinkBuilder` for creating links with validation.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Network link/connection between nodes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Link {
    /// Unique identifier for the link
    pub id: Uuid,
    /// Human-readable name/description for the link
    pub name: String,
    /// Node A identifier (required)
    pub source_node_id: Uuid,
    /// Node A interface name
    pub node_a_interface: String,
    /// Node Z identifier (optional for internet circuits)
    pub dest_node_id: Option<Uuid>,
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
    #[must_use]
    pub fn new(
        name: String,
        source_node_id: Uuid,
        source_interface: String,
        target_node_id: Uuid,
        target_interface: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            source_node_id,
            node_a_interface: source_interface,
            dest_node_id: Some(target_node_id),
            node_z_interface: Some(target_interface),
            description: None,
            bandwidth: None,
            link_type: None,
            is_internet_circuit: false,
            custom_data: Value::Null,
        }
    }

    /// Creates a new internet circuit (single-ended link)
    #[must_use]
    pub fn new_internet_circuit(
        name: String,
        source_node_id: Uuid,
        node_a_interface: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            source_node_id,
            node_a_interface,
            dest_node_id: None,
            node_z_interface: None,
            description: None,
            bandwidth: None,
            link_type: None,
            is_internet_circuit: true,
            custom_data: Value::Null,
        }
    }

    /// Validates the link configuration
    ///
    /// # Errors
    /// Returns an error if the link name is empty, node interfaces are empty,
    /// or if interface names have invalid format.
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
        if !crate::models::validation::is_valid_interface_name(&self.node_a_interface) {
            return Err("Invalid node A interface name format".to_string());
        }

        // Check consistency for internet circuits
        if self.is_internet_circuit {
            if self.dest_node_id.is_some() {
                return Err("Internet circuits cannot have node Z".to_string());
            }
            if self.node_z_interface.is_some() {
                return Err("Internet circuits cannot have node Z interface".to_string());
            }
        } else {
            // For regular links, both ends must be specified
            if self.dest_node_id.is_none() {
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
                if !crate::models::validation::is_valid_interface_name(interface) {
                    return Err("Invalid node Z interface name format".to_string());
                }
            }

            // Prevent self-links
            if Some(self.source_node_id) == self.dest_node_id {
                return Err("Links cannot connect a node to itself".to_string());
            }
        }

        Ok(())
    }

    /// Returns the other node ID if this is a bidirectional link
    #[must_use]
    pub fn get_other_node_id(&self, node_id: Uuid) -> Option<Uuid> {
        if self.source_node_id == node_id {
            self.dest_node_id
        } else if Some(node_id) == self.dest_node_id {
            Some(self.source_node_id)
        } else {
            None
        }
    }

    /// Returns the interface name for the given node
    #[must_use]
    pub fn get_interface_for_node(&self, node_id: Uuid) -> Option<&str> {
        if self.source_node_id == node_id {
            Some(&self.node_a_interface)
        } else if Some(node_id) == self.dest_node_id {
            self.node_z_interface.as_deref()
        } else {
            None
        }
    }

    /// Checks if this link connects the two specified nodes
    #[must_use]
    pub fn connects_nodes(&self, node1_id: Uuid, node2_id: Uuid) -> bool {
        (self.source_node_id == node1_id && Some(node2_id) == self.dest_node_id)
            || (self.source_node_id == node2_id && Some(node1_id) == self.dest_node_id)
    }

    /// Checks if this link involves the specified node
    #[must_use]
    pub fn involves_node(&self, node_id: Uuid) -> bool {
        self.source_node_id == node_id || Some(node_id) == self.dest_node_id
    }

    /// Gets a value from `custom_data` by path
    #[must_use]
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

/// Builder pattern for Link creation with validation
#[derive(Debug, Default)]
pub struct LinkBuilder {
    /// Link ID (optional, will generate UUID if not provided)
    id: Option<Uuid>,
    /// Link name (required)
    name: Option<String>,
    /// Source node ID (required)
    source_node_id: Option<Uuid>,
    /// Node A interface (required)
    node_a_interface: Option<String>,
    /// Destination node ID (optional for internet circuits)
    dest_node_id: Option<Uuid>,
    /// Node Z interface (optional for internet circuits)
    node_z_interface: Option<String>,
    /// Link description (optional)
    description: Option<String>,
    /// Link bandwidth (optional)
    bandwidth: Option<u64>,
    /// Link type (optional)
    link_type: Option<String>,
    /// Whether this is an internet circuit (optional)
    is_internet_circuit: Option<bool>,
    /// Custom data (optional)
    custom_data: Option<Value>,
}

impl LinkBuilder {
    /// Creates a new link builder
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the link ID (optional, will generate UUID if not provided)
    #[must_use]
    pub const fn id(mut self, id: Uuid) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the link name (required)
    #[must_use]
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets node A ID (required)
    #[must_use]
    pub const fn source_node_id(mut self, source_node_id: Uuid) -> Self {
        self.source_node_id = Some(source_node_id);
        self
    }

    /// Sets node A interface (required)
    #[must_use]
    pub fn node_a_interface<S: Into<String>>(mut self, interface: S) -> Self {
        self.node_a_interface = Some(interface.into());
        self
    }

    /// Sets node Z ID (optional for internet circuits)
    #[must_use]
    pub const fn dest_node_id(mut self, dest_node_id: Uuid) -> Self {
        self.dest_node_id = Some(dest_node_id);
        self
    }

    /// Sets node Z interface (optional for internet circuits)
    #[must_use]
    pub fn node_z_interface<S: Into<String>>(mut self, interface: S) -> Self {
        self.node_z_interface = Some(interface.into());
        self
    }

    /// Sets the description (optional)
    #[must_use]
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the bandwidth (optional)
    #[must_use]
    pub const fn bandwidth(mut self, bandwidth: u64) -> Self {
        self.bandwidth = Some(bandwidth);
        self
    }

    /// Sets the link type (optional)
    #[must_use]
    pub fn link_type<S: Into<String>>(mut self, link_type: S) -> Self {
        self.link_type = Some(link_type.into());
        self
    }

    /// Sets whether this is an internet circuit (optional, defaults to false)
    #[must_use]
    pub const fn is_internet_circuit(mut self, is_internet_circuit: bool) -> Self {
        self.is_internet_circuit = Some(is_internet_circuit);
        self
    }

    /// Sets custom data (optional)
    #[must_use]
    pub fn custom_data(mut self, custom_data: Value) -> Self {
        self.custom_data = Some(custom_data);
        self
    }

    /// Builds the link with validation
    ///
    /// # Errors
    /// Returns an error if required fields (name, `source_node_id`, `node_a_interface`) are missing,
    /// or if the created link fails validation.
    pub fn build(self) -> Result<Link, String> {
        let name = self.name.ok_or("Name is required")?;
        let source_node_id = self.source_node_id.ok_or("Node A ID is required")?;
        let node_a_interface = self
            .node_a_interface
            .ok_or("Node A interface is required")?;
        let is_internet_circuit = self.is_internet_circuit.unwrap_or(false);

        let link = Link {
            id: self.id.unwrap_or_else(Uuid::new_v4),
            name,
            source_node_id,
            node_a_interface,
            dest_node_id: self.dest_node_id,
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
