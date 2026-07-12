//! Entity conversion helpers for `SQLite` implementation

use super::super::types::{DataStoreError, DataStoreResult};
use crate::entities::{interface_status, links, locations, node_status, nodes};
use crate::models::derived::{
    InterfaceAdminStatus, InterfaceOperStatus, InterfaceStatus, NodeStatus,
};
use crate::models::{DeviceRole, Lifecycle, Link, Location, Node, Vendor};
use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use std::time::SystemTime;
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

pub fn entity_to_node_status(
    entity: node_status::Model,
    interfaces: Vec<interface_status::Model>,
) -> DataStoreResult<NodeStatus> {
    let node_id = entity
        .node_id
        .parse::<Uuid>()
        .map_err(|e| DataStoreError::ValidationError {
            message: format!("Invalid node status node UUID: {e}"),
        })?;

    let last_updated = parse_timestamp(&entity.last_updated, "node_status.last_updated")?;
    let last_snmp_success = entity
        .last_snmp_success
        .as_deref()
        .map(|value| parse_timestamp(value, "node_status.last_snmp_success"))
        .transpose()?;
    let system_info = parse_optional_json(entity.system_info, "node_status.system_info")?;
    let performance = parse_optional_json(entity.performance, "node_status.performance")?;
    let environmental = parse_optional_json(entity.environmental, "node_status.environmental")?;
    let vendor_metrics =
        parse_json_with_default(entity.vendor_metrics, "node_status.vendor_metrics")?;
    let raw_snmp_data = parse_json_with_default(entity.raw_snmp_data, "node_status.raw_snmp_data")?;
    let interfaces = interfaces
        .into_iter()
        .map(entity_to_interface_status)
        .collect::<DataStoreResult<Vec<_>>>()?;

    Ok(NodeStatus {
        node_id,
        last_updated,
        reachable: entity.reachable,
        system_info,
        interfaces,
        performance,
        environmental,
        vendor_metrics,
        raw_snmp_data,
        last_snmp_success,
        last_error: entity.last_error,
        consecutive_failures: u32::try_from(entity.consecutive_failures).map_err(|e| {
            DataStoreError::ValidationError {
                message: format!("Invalid consecutive failure count: {e}"),
            }
        })?,
    })
}

pub fn entity_to_interface_status(
    entity: interface_status::Model,
) -> DataStoreResult<InterfaceStatus> {
    Ok(InterfaceStatus {
        index: u32::try_from(entity.index).map_err(|e| DataStoreError::ValidationError {
            message: format!("Invalid interface index: {e}"),
        })?,
        name: entity.name,
        interface_type: u32::try_from(entity.interface_type).map_err(|e| {
            DataStoreError::ValidationError {
                message: format!("Invalid interface type: {e}"),
            }
        })?,
        mtu: entity
            .mtu
            .map(|value| {
                u32::try_from(value).map_err(|e| DataStoreError::ValidationError {
                    message: format!("Invalid interface MTU: {e}"),
                })
            })
            .transpose()?,
        speed: entity
            .speed
            .map(|value| {
                u64::try_from(value).map_err(|e| DataStoreError::ValidationError {
                    message: format!("Invalid interface speed: {e}"),
                })
            })
            .transpose()?,
        physical_address: entity.physical_address,
        admin_status: parse_admin_status(&entity.admin_status)?,
        oper_status: parse_oper_status(&entity.oper_status)?,
        last_change: entity
            .last_change
            .map(|value| {
                u32::try_from(value).map_err(|e| DataStoreError::ValidationError {
                    message: format!("Invalid interface last_change: {e}"),
                })
            })
            .transpose()?,
        input_stats: parse_required_json(&entity.input_stats, "interface_status.input_stats")?,
        output_stats: parse_required_json(&entity.output_stats, "interface_status.output_stats")?,
    })
}

fn parse_timestamp(value: &str, field: &str) -> DataStoreResult<SystemTime> {
    DateTime::parse_from_rfc3339(value)
        .map(|timestamp| timestamp.with_timezone(&Utc).into())
        .map_err(|e| DataStoreError::ValidationError {
            message: format!("Invalid timestamp in {field}: {e}"),
        })
}

pub fn parse_optional_json<T>(value: Option<String>, field: &str) -> DataStoreResult<Option<T>>
where
    T: DeserializeOwned,
{
    value
        .map(|json| parse_required_json::<T>(&json, field))
        .transpose()
}

fn parse_json_with_default<T>(value: Option<String>, field: &str) -> DataStoreResult<T>
where
    T: DeserializeOwned + Default,
{
    value.map_or_else(
        || Ok(T::default()),
        |json| parse_required_json(&json, field),
    )
}

fn parse_required_json<T>(value: &str, field: &str) -> DataStoreResult<T>
where
    T: DeserializeOwned,
{
    serde_json::from_str(value).map_err(|e| DataStoreError::ValidationError {
        message: format!("Invalid JSON in {field}: {e}"),
    })
}

fn parse_admin_status(value: &str) -> DataStoreResult<InterfaceAdminStatus> {
    match value {
        "up" => Ok(InterfaceAdminStatus::Up),
        "down" => Ok(InterfaceAdminStatus::Down),
        "testing" => Ok(InterfaceAdminStatus::Testing),
        "unknown" => Ok(InterfaceAdminStatus::Unknown),
        _ => Err(DataStoreError::ValidationError {
            message: format!("Invalid interface admin status: {value}"),
        }),
    }
}

fn parse_oper_status(value: &str) -> DataStoreResult<InterfaceOperStatus> {
    match value {
        "up" => Ok(InterfaceOperStatus::Up),
        "down" => Ok(InterfaceOperStatus::Down),
        "testing" => Ok(InterfaceOperStatus::Testing),
        "unknown" => Ok(InterfaceOperStatus::Unknown),
        "dormant" => Ok(InterfaceOperStatus::Dormant),
        "notPresent" => Ok(InterfaceOperStatus::NotPresent),
        "lowerLayerDown" => Ok(InterfaceOperStatus::LowerLayerDown),
        _ => Err(DataStoreError::ValidationError {
            message: format!("Invalid interface oper status: {value}"),
        }),
    }
}

#[cfg(test)]
#[path = "conversions_tests.rs"]
mod tests;
