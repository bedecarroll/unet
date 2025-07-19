//! API data transfer objects and response types

use serde::{Deserialize, Serialize};
use unet_core::prelude::*;
use uuid::Uuid;

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
    pub const fn success(data: T) -> Self {
        Self {
            data,
            success: true,
            message: None,
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
    pub const fn new(error: String, code: String) -> Self {
        Self {
            error,
            code,
            success: false,
        }
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
    /// Create from node only
    pub const fn from_node(node: Node) -> Self {
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
    pub fn into_node(self) -> Result<Node> {
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
            let ip: std::net::IpAddr = management_ip.parse().map_err(|_| {
                Error::validation_with_value(
                    "management_ip",
                    "Invalid IP address format",
                    &management_ip,
                )
            })?;
            builder = builder.management_ip(ip);
        }

        if let Some(custom_data) = self.custom_data {
            builder = builder.custom_data(custom_data);
        }

        builder.build().map_err(|e| Error::Other {
            context: "Node creation".to_string(),
            message: e,
            source: None,
        })
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

impl<T> PaginatedResponse<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use uuid::Uuid;

    #[test]
    fn test_create_node_request_location_id() {
        let location_id = Uuid::new_v4();
        let request = CreateNodeRequest {
            name: "test-node".to_string(),
            domain: Some("example.com".to_string()),
            vendor: Vendor::Cisco,
            model: "ASR1000".to_string(),
            role: DeviceRole::Router,
            lifecycle: Lifecycle::Live,
            location_id: Some(location_id),
            management_ip: None,
            custom_data: None,
        };

        let result = request.into_node();
        assert!(result.is_ok());
        let node = result.unwrap();
        assert_eq!(node.location_id, Some(location_id));
    }

    #[test]
    fn test_create_node_request_invalid_ip() {
        let request = CreateNodeRequest {
            name: "test-node".to_string(),
            domain: Some("example.com".to_string()),
            vendor: Vendor::Cisco,
            model: "ASR1000".to_string(),
            role: DeviceRole::Router,
            lifecycle: Lifecycle::Live,
            location_id: None,
            management_ip: Some("invalid-ip".to_string()),
            custom_data: None,
        };

        let result = request.into_node();
        assert!(result.is_err());
    }

    #[test]
    fn test_create_node_request_valid_ip() {
        let request = CreateNodeRequest {
            name: "test-node".to_string(),
            domain: Some("example.com".to_string()),
            vendor: Vendor::Cisco,
            model: "ASR1000".to_string(),
            role: DeviceRole::Router,
            lifecycle: Lifecycle::Live,
            location_id: None,
            management_ip: Some("192.168.1.1".to_string()),
            custom_data: None,
        };

        let result = request.into_node();
        assert!(result.is_ok());
        let node = result.unwrap();
        assert_eq!(node.management_ip, Some("192.168.1.1".parse().unwrap()));
    }

    #[test]
    fn test_create_node_request_custom_data() {
        let custom_data = json!({"key": "value", "number": 42});
        let request = CreateNodeRequest {
            name: "test-node".to_string(),
            domain: Some("example.com".to_string()),
            vendor: Vendor::Cisco,
            model: "ASR1000".to_string(),
            role: DeviceRole::Router,
            lifecycle: Lifecycle::Live,
            location_id: None,
            management_ip: None,
            custom_data: Some(custom_data.clone()),
        };

        let result = request.into_node();
        assert!(result.is_ok());
        let node = result.unwrap();
        assert_eq!(node.custom_data, custom_data);
    }
}
