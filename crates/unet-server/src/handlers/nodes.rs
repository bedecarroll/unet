//! Node API handlers

use axum::{
    extract::{Path, Query},
    response::Json,
};
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

use crate::api::{
    ApiResponse, CreateNodeRequest, NodeResponse, PaginatedResponse, UpdateNodeRequest,
};
use crate::handlers::{ServerError, ServerResult};
use unet_core::prelude::*;

/// Query parameters for listing nodes
#[derive(Debug, Deserialize)]
pub struct ListNodesQuery {
    /// Page number (1-based)
    pub page: Option<u64>,
    /// Items per page
    pub per_page: Option<u64>,
    /// Filter by lifecycle
    pub lifecycle: Option<String>,
    /// Filter by role
    pub role: Option<String>,
    /// Filter by vendor
    pub vendor: Option<String>,
    /// Include derived state in response
    pub include_status: Option<bool>,
}

/// List all nodes with optional filtering and pagination
pub async fn list_nodes(
    Query(query): Query<ListNodesQuery>,
) -> ServerResult<Json<ApiResponse<PaginatedResponse<NodeResponse>>>> {
    // TODO: Implement actual datastore integration
    // For now, return mock data to demonstrate API structure

    let mock_nodes = vec![
        NodeBuilder::new()
            .name("router-01".to_string())
            .domain("example.com".to_string())
            .vendor(Vendor::Cisco)
            .model("ISR4431".to_string())
            .role(DeviceRole::Router)
            .lifecycle(Lifecycle::Live)
            .build()
            .map_err(|e| ServerError::Internal(e))?,
        NodeBuilder::new()
            .name("switch-01".to_string())
            .domain("example.com".to_string())
            .vendor(Vendor::Cisco)
            .model("C9300-48U".to_string())
            .role(DeviceRole::Switch)
            .lifecycle(Lifecycle::Live)
            .build()
            .map_err(|e| ServerError::Internal(e))?,
    ];

    // Convert to NodeResponse with optional status
    let include_status = query.include_status.unwrap_or(false);
    let node_responses: Vec<NodeResponse> = mock_nodes
        .into_iter()
        .map(|node| {
            if include_status {
                // TODO: Fetch actual status from datastore
                let mock_status = NodeStatus::new(node.id);
                NodeResponse::new(node, Some(mock_status))
            } else {
                NodeResponse::from_node(node)
            }
        })
        .collect();

    // Create paginated response
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);
    let total = node_responses.len() as u64;

    let paginated = PaginatedResponse {
        data: node_responses,
        total,
        page,
        per_page,
        total_pages: (total + per_page - 1) / per_page,
        has_next: page * per_page < total,
        has_prev: page > 1,
    };

    Ok(Json(ApiResponse::success(paginated)))
}

/// Get a specific node by ID
pub async fn get_node(Path(id): Path<Uuid>) -> ServerResult<Json<ApiResponse<NodeResponse>>> {
    // TODO: Implement actual datastore lookup

    // For now, create a mock node
    let node = NodeBuilder::new()
        .name("router-01".to_string())
        .domain("example.com".to_string())
        .vendor(Vendor::Cisco)
        .model("ISR4431".to_string())
        .role(DeviceRole::Router)
        .lifecycle(Lifecycle::Live)
        .build()
        .map_err(|e| ServerError::Internal(e))?;

    // Mock status data
    let mut status = NodeStatus::new(id);
    let mut snmp_data = HashMap::new();
    snmp_data.insert(
        "1.3.6.1.2.1.1.1.0".to_string(),
        SnmpValue::String("Cisco IOS Software".to_string()),
    );
    snmp_data.insert(
        "1.3.6.1.2.1.1.5.0".to_string(),
        SnmpValue::String("router-01.example.com".to_string()),
    );
    status.update_from_snmp(snmp_data);

    let response = NodeResponse::new(node, Some(status));
    Ok(Json(ApiResponse::success(response)))
}

/// Get node status (derived state only)
pub async fn get_node_status(Path(id): Path<Uuid>) -> ServerResult<Json<ApiResponse<NodeStatus>>> {
    // TODO: Implement actual datastore lookup

    // Mock status data with realistic SNMP values
    let mut status = NodeStatus::new(id);
    let mut snmp_data = HashMap::new();

    // System information
    snmp_data.insert(
        "1.3.6.1.2.1.1.1.0".to_string(),
        SnmpValue::String("Cisco IOS Software, ISR4400 Software".to_string()),
    );
    snmp_data.insert(
        "1.3.6.1.2.1.1.3.0".to_string(),
        SnmpValue::TimeTicks(12345678),
    );
    snmp_data.insert(
        "1.3.6.1.2.1.1.5.0".to_string(),
        SnmpValue::String("router-01.example.com".to_string()),
    );
    snmp_data.insert(
        "1.3.6.1.2.1.1.6.0".to_string(),
        SnmpValue::String("Data Center - Rack 42".to_string()),
    );

    // Interface data (example for interface 1)
    snmp_data.insert("1.3.6.1.2.1.2.2.1.1.1".to_string(), SnmpValue::Integer(1));
    snmp_data.insert(
        "1.3.6.1.2.1.2.2.1.2.1".to_string(),
        SnmpValue::String("GigabitEthernet0/0/0".to_string()),
    );
    snmp_data.insert("1.3.6.1.2.1.2.2.1.3.1".to_string(), SnmpValue::Integer(6)); // ethernetCsmacd
    snmp_data.insert(
        "1.3.6.1.2.1.2.2.1.5.1".to_string(),
        SnmpValue::Gauge32(1000000000),
    ); // 1 Gbps
    snmp_data.insert("1.3.6.1.2.1.2.2.1.7.1".to_string(), SnmpValue::Integer(1)); // admin up
    snmp_data.insert("1.3.6.1.2.1.2.2.1.8.1".to_string(), SnmpValue::Integer(1)); // oper up

    // Traffic statistics
    snmp_data.insert(
        "1.3.6.1.2.1.2.2.1.10.1".to_string(),
        SnmpValue::Counter32(1234567890),
    ); // input octets
    snmp_data.insert(
        "1.3.6.1.2.1.2.2.1.11.1".to_string(),
        SnmpValue::Counter32(987654321),
    ); // input packets
    snmp_data.insert(
        "1.3.6.1.2.1.2.2.1.16.1".to_string(),
        SnmpValue::Counter32(2345678901),
    ); // output octets
    snmp_data.insert(
        "1.3.6.1.2.1.2.2.1.17.1".to_string(),
        SnmpValue::Counter32(1876543210),
    ); // output packets

    status.update_from_snmp(snmp_data);

    Ok(Json(ApiResponse::success(status)))
}

/// Create a new node
pub async fn create_node(
    Json(request): Json<CreateNodeRequest>,
) -> ServerResult<Json<ApiResponse<NodeResponse>>> {
    // Validate and convert request to Node
    let node = request
        .to_node()
        .map_err(|e| ServerError::Validation(e.to_string()))?;

    // TODO: Save to datastore

    let response = NodeResponse::from_node(node);
    Ok(Json(ApiResponse::success_with_message(
        response,
        "Node created successfully".to_string(),
    )))
}

/// Update an existing node
pub async fn update_node(
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateNodeRequest>,
) -> ServerResult<Json<ApiResponse<NodeResponse>>> {
    // TODO: Fetch existing node from datastore and apply updates

    // For now, create a mock updated node
    let node = NodeBuilder::new()
        .name(request.name.unwrap_or_else(|| "updated-node".to_string()))
        .domain(request.domain.unwrap_or_else(|| "example.com".to_string()))
        .vendor(request.vendor.unwrap_or(Vendor::Cisco))
        .model(request.model.unwrap_or_else(|| "Updated Model".to_string()))
        .role(request.role.unwrap_or(DeviceRole::Router))
        .lifecycle(request.lifecycle.unwrap_or(Lifecycle::Live))
        .build()
        .map_err(|e| ServerError::Internal(e))?;

    let response = NodeResponse::from_node(node);
    Ok(Json(ApiResponse::success_with_message(
        response,
        "Node updated successfully".to_string(),
    )))
}

/// Delete a node
pub async fn delete_node(Path(id): Path<Uuid>) -> ServerResult<Json<ApiResponse<()>>> {
    // TODO: Delete from datastore

    Ok(Json(ApiResponse::success_with_message(
        (),
        format!("Node {} deleted successfully", id),
    )))
}
