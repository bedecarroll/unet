//! API data transfer objects and response types

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use unet_core::prelude::*;

/// Standard API response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    /// The response data
    pub data: T,
    /// Success indicator
    pub success: bool,
    /// Optional message
    pub message: Option<String>,
}

impl<T> ApiResponse<T> {
    /// Create a successful response
    pub fn success(data: T) -> Self {
        Self {
            data,
            success: true,
            message: None,
        }
    }
    
    /// Create a successful response with message
    pub fn success_with_message(data: T, message: String) -> Self {
        Self {
            data,
            success: true,
            message: Some(message),
        }
    }
}

/// Error response for API failures
#[derive(Debug, Serialize)]
pub struct ApiError {
    /// Error message
    pub error: String,
    /// Error code
    pub code: String,
    /// Success indicator (always false)
    pub success: bool,
}

impl ApiError {
    /// Create a new API error
    pub fn new(error: String, code: String) -> Self {
        Self {
            error,
            code,
            success: false,
        }
    }
    
    /// Create a validation error
    pub fn validation(message: String) -> Self {
        Self::new(message, "VALIDATION_ERROR".to_string())
    }
    
    /// Create a not found error
    pub fn not_found(resource: String) -> Self {
        Self::new(format!("{} not found", resource), "NOT_FOUND".to_string())
    }
    
    /// Create an internal server error
    pub fn internal(message: String) -> Self {
        Self::new(message, "INTERNAL_ERROR".to_string())
    }
}

/// Extended node response including derived state
#[derive(Debug, Serialize)]
pub struct NodeResponse {
    /// Core node data
    #[serde(flatten)]
    pub node: Node,
    /// Current status and derived state (if available)
    pub status: Option<NodeStatus>,
}

impl NodeResponse {
    /// Create from node and optional status
    pub fn new(node: Node, status: Option<NodeStatus>) -> Self {
        Self { node, status }
    }
    
    /// Create from node only
    pub fn from_node(node: Node) -> Self {
        Self { node, status: None }
    }
}

/// Request to create a new node
#[derive(Debug, Deserialize)]
pub struct CreateNodeRequest {
    /// Node name
    pub name: String,
    /// Domain (optional)
    pub domain: Option<String>,
    /// Device vendor
    pub vendor: Vendor,
    /// Device model
    pub model: String,
    /// Device role
    pub role: DeviceRole,
    /// Lifecycle state
    pub lifecycle: Lifecycle,
    /// Location ID (optional)
    pub location_id: Option<Uuid>,
    /// Management IP (optional)
    pub management_ip: Option<String>,
    /// Custom data (optional)
    pub custom_data: Option<serde_json::Value>,
}

impl CreateNodeRequest {
    /// Convert to Node using builder
    pub fn to_node(self) -> Result<Node> {
        let mut builder = NodeBuilder::new()
            .name(self.name)
            .vendor(self.vendor)
            .model(self.model)
            .role(self.role)
            .lifecycle(self.lifecycle);
        
        if let Some(domain) = self.domain {
            builder = builder.domain(domain);
        }
        
        if let Some(location_id) = self.location_id {
            builder = builder.location_id(location_id);
        }
        
        if let Some(management_ip) = self.management_ip {
            // Parse IP address string to IpAddr
            let ip: std::net::IpAddr = management_ip.parse()
                .map_err(|e| Error::Other(format!("Invalid IP address: {}", e)))?;
            builder = builder.management_ip(ip);
        }
        
        if let Some(custom_data) = self.custom_data {
            builder = builder.custom_data(custom_data);
        }
        
        builder.build().map_err(|e| Error::Other(e))
    }
}

/// Request to update an existing node
#[derive(Debug, Deserialize)]
pub struct UpdateNodeRequest {
    /// Node name (optional)
    pub name: Option<String>,
    /// Domain (optional)
    pub domain: Option<String>,
    /// Device vendor (optional)
    pub vendor: Option<Vendor>,
    /// Device model (optional)
    pub model: Option<String>,
    /// Device role (optional)
    pub role: Option<DeviceRole>,
    /// Lifecycle state (optional)
    pub lifecycle: Option<Lifecycle>,
    /// Location ID (optional)
    pub location_id: Option<Uuid>,
    /// Management IP (optional)
    pub management_ip: Option<String>,
    /// Custom data (optional)
    pub custom_data: Option<serde_json::Value>,
}

/// Request to create a new location
#[derive(Debug, Deserialize)]
pub struct CreateLocationRequest {
    /// Location name
    pub name: String,
    /// Location type
    pub location_type: String,
    /// Parent location ID (optional)
    pub parent_id: Option<Uuid>,
    /// Custom data (optional)
    pub custom_data: Option<serde_json::Value>,
}

impl CreateLocationRequest {
    /// Convert to Location using builder
    pub fn to_location(self) -> Result<Location> {
        let mut builder = LocationBuilder::new()
            .name(self.name)
            .location_type(self.location_type);
        
        if let Some(parent_id) = self.parent_id {
            builder = builder.parent_id(parent_id);
        }
        
        if let Some(custom_data) = self.custom_data {
            builder = builder.custom_data(custom_data);
        }
        
        builder.build().map_err(|e| Error::Other(e))
    }
}

/// Request to update an existing location
#[derive(Debug, Deserialize)]
pub struct UpdateLocationRequest {
    /// Location name (optional)
    pub name: Option<String>,
    /// Location type (optional)
    pub location_type: Option<String>,
    /// Parent location ID (optional)
    pub parent_id: Option<Uuid>,
    /// Custom data (optional)
    pub custom_data: Option<serde_json::Value>,
}

/// Request to create a new link
#[derive(Debug, Deserialize)]
pub struct CreateLinkRequest {
    /// Link name
    pub name: String,
    /// Source node ID
    pub node_a_id: Uuid,
    /// Source interface
    pub interface_a: String,
    /// Destination node ID (optional for internet circuits)
    pub node_z_id: Option<Uuid>,
    /// Destination interface (optional for internet circuits)
    pub interface_z: Option<String>,
    /// Link bandwidth in bits per second (optional)
    pub bandwidth_bps: Option<u64>,
    /// Custom data (optional)
    pub custom_data: Option<serde_json::Value>,
}

impl CreateLinkRequest {
    /// Convert to Link using builder
    pub fn to_link(self) -> Result<Link> {
        let mut builder = LinkBuilder::new()
            .name(self.name)
            .node_a_id(self.node_a_id)
            .node_a_interface(self.interface_a);
        
        if let Some(node_z_id) = self.node_z_id {
            builder = builder.node_z_id(node_z_id);
        }
        
        if let Some(interface_z) = self.interface_z {
            builder = builder.node_z_interface(interface_z);
        }
        
        if let Some(bandwidth_bps) = self.bandwidth_bps {
            builder = builder.bandwidth(bandwidth_bps);
        }
        
        if let Some(custom_data) = self.custom_data {
            builder = builder.custom_data(custom_data);
        }
        
        builder.build().map_err(|e| Error::Other(e))
    }
}

/// Request to update an existing link
#[derive(Debug, Deserialize)]
pub struct UpdateLinkRequest {
    /// Link name (optional)
    pub name: Option<String>,
    /// Source node ID (optional)
    pub node_a_id: Option<Uuid>,
    /// Source interface (optional)
    pub interface_a: Option<String>,
    /// Destination node ID (optional)
    pub node_z_id: Option<Uuid>,
    /// Destination interface (optional)
    pub interface_z: Option<String>,
    /// Link bandwidth in bits per second (optional)
    pub bandwidth_bps: Option<u64>,
    /// Custom data (optional)
    pub custom_data: Option<serde_json::Value>,
}

/// Paginated response wrapper
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    /// The page of data
    pub data: Vec<T>,
    /// Total number of items
    pub total: u64,
    /// Current page number (1-based)
    pub page: u64,
    /// Number of items per page
    pub per_page: u64,
    /// Total number of pages
    pub total_pages: u64,
    /// Whether there is a next page
    pub has_next: bool,
    /// Whether there is a previous page
    pub has_prev: bool,
}

impl<T> PaginatedResponse<T> {
    /// Create from PagedResult
    pub fn from_paged_result(result: PagedResult<T>) -> Self {
        Self {
            data: result.items,
            total: result.total_count as u64,
            page: result.page as u64,
            per_page: result.page_size as u64,
            total_pages: result.total_pages as u64,
            has_next: result.has_next,
            has_prev: result.has_previous,
        }
    }
}