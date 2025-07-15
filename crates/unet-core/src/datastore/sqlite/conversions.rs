//! Entity conversion helpers for `SQLite` implementation

use super::super::types::{DataStoreError, DataStoreResult};
use crate::entities::{links, locations, nodes};
use crate::models::{DeviceRole, Lifecycle, Link, Location, Node, Vendor};
use uuid::Uuid;

/// Helper function to convert `SeaORM` link entity to our Link model
pub fn entity_to_link(entity: links::Model) -> DataStoreResult<Link> {
    let id = entity
        .id
        .parse::<Uuid>()
        .map_err(|e| DataStoreError::ValidationError {
            message: format!("Invalid UUID: {e}"),
        })?;

    let source_node_id =
        entity
            .node_a_id
            .parse::<Uuid>()
            .map_err(|e| DataStoreError::ValidationError {
                message: format!("Invalid node A UUID: {e}"),
            })?;

    let target_node_id = if let Some(node_b_id_str) = entity.node_b_id {
        Some(
            node_b_id_str
                .parse::<Uuid>()
                .map_err(|e| DataStoreError::ValidationError {
                    message: format!("Invalid node B UUID: {e}"),
                })?,
        )
    } else {
        None
    };

    let custom_data = entity
        .custom_data
        .as_ref()
        .map_or(serde_json::Value::Null, |data_str| {
            serde_json::from_str(data_str).unwrap_or_default()
        });

    Ok(Link {
        id,
        name: entity.name,
        source_node_id,
        node_a_interface: entity.interface_a,
        dest_node_id: target_node_id,
        node_z_interface: entity.interface_b,
        description: entity.description,
        bandwidth: entity.capacity.map(|c| u64::try_from(c).unwrap_or(0)),
        link_type: None, // Not stored in entity yet
        is_internet_circuit: entity.is_internet_circuit != 0,
        custom_data,
    })
}

/// Helper function to convert `SeaORM` location entity to our Location model
pub fn entity_to_location(entity: locations::Model) -> DataStoreResult<Location> {
    let id = entity
        .id
        .parse::<Uuid>()
        .map_err(|e| DataStoreError::ValidationError {
            message: format!("Invalid UUID: {e}"),
        })?;

    let parent_id = if let Some(parent_id_str) = entity.parent_id {
        Some(
            parent_id_str
                .parse::<Uuid>()
                .map_err(|e| DataStoreError::ValidationError {
                    message: format!("Invalid parent UUID: {e}"),
                })?,
        )
    } else {
        None
    };

    let custom_data = entity
        .custom_data
        .as_ref()
        .map_or(serde_json::Value::Null, |data_str| {
            serde_json::from_str(data_str).unwrap_or_default()
        });

    Ok(Location {
        id,
        name: entity.name,
        location_type: entity.location_type,
        parent_id,
        path: entity.path,
        description: entity.description,
        address: entity.address,
        custom_data,
    })
}

/// Helper function to convert `SeaORM` node entity to our Node model
pub fn entity_to_node(entity: nodes::Model) -> DataStoreResult<Node> {
    let vendor = entity
        .vendor
        .parse::<Vendor>()
        .map_err(|e| DataStoreError::ValidationError {
            message: format!("Invalid vendor: {e}"),
        })?;

    let role = entity
        .role
        .parse::<DeviceRole>()
        .map_err(|e| DataStoreError::ValidationError {
            message: format!("Invalid role: {e}"),
        })?;

    let lifecycle =
        entity
            .lifecycle
            .parse::<Lifecycle>()
            .map_err(|e| DataStoreError::ValidationError {
                message: format!("Invalid lifecycle: {e}"),
            })?;

    let id = entity
        .id
        .parse::<Uuid>()
        .map_err(|e| DataStoreError::ValidationError {
            message: format!("Invalid UUID: {e}"),
        })?;

    let location_id = if let Some(loc_id_str) = entity.location_id {
        Some(
            loc_id_str
                .parse::<Uuid>()
                .map_err(|e| DataStoreError::ValidationError {
                    message: format!("Invalid location UUID: {e}"),
                })?,
        )
    } else {
        None
    };

    let management_ip = if let Some(ip_str) = entity.management_ip {
        Some(
            ip_str
                .parse()
                .map_err(|e| DataStoreError::ValidationError {
                    message: format!("Invalid IP address: {e}"),
                })?,
        )
    } else {
        None
    };

    let custom_data = entity
        .custom_data
        .as_ref()
        .map_or(serde_json::Value::Null, |data_str| {
            serde_json::from_str(data_str).unwrap_or_default()
        });

    let domain = entity.domain.clone().unwrap_or_default();
    let name = entity.name.clone();
    let fqdn = entity.fqdn.unwrap_or_else(|| {
        if domain.is_empty() {
            name.clone()
        } else {
            format!("{name}.{domain}")
        }
    });

    Ok(Node {
        id,
        name,
        domain,
        fqdn,
        vendor,
        model: entity.model,
        role,
        lifecycle,
        management_ip,
        location_id,
        platform: None, // Not stored in entity yet
        version: None,  // Not stored in entity yet
        serial_number: entity.serial_number,
        asset_tag: entity.asset_tag,
        purchase_date: None,    // Not stored in entity yet
        warranty_expires: None, // Not stored in entity yet
        custom_data,
    })
}
