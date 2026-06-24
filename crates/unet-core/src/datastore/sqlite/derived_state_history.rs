//! Derived-state snapshot storage and history queries for the `SQLite` datastore.

use super::SqliteStore;
use super::conversions::entity_to_node_status;
use crate::datastore::{DataStoreError, DataStoreResult, HistoryQueryOptions};
use crate::entities::{interface_status, node_status};
use crate::models::derived::{InterfaceAdminStatus, InterfaceOperStatus, NodeStatus};
use chrono::{SecondsFormat, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set,
    TransactionTrait,
};
use std::collections::HashMap;
use std::time::SystemTime;
use uuid::Uuid;

pub async fn store_node_status_snapshot(
    store: &SqliteStore,
    snapshot: &NodeStatus,
) -> DataStoreResult<()> {
    let txn = store
        .db
        .begin()
        .await
        .map_err(|e| DataStoreError::TransactionError {
            message: format!("Failed to begin derived-state snapshot transaction: {e}"),
        })?;
    let snapshot_id = Uuid::new_v4().to_string();

    node_status::ActiveModel {
        id: Set(snapshot_id.clone()),
        node_id: Set(snapshot.node_id.to_string()),
        last_updated: Set(format_timestamp(snapshot.last_updated)),
        reachable: Set(snapshot.reachable),
        system_info: Set(serialize_optional_json(snapshot.system_info.as_ref())?),
        performance: Set(serialize_optional_json(snapshot.performance.as_ref())?),
        environmental: Set(serialize_optional_json(snapshot.environmental.as_ref())?),
        vendor_metrics: Set(serialize_map_json(&snapshot.vendor_metrics)?),
        raw_snmp_data: Set(serialize_map_json(&snapshot.raw_snmp_data)?),
        last_snmp_success: Set(snapshot.last_snmp_success.map(format_timestamp)),
        last_error: Set(snapshot.last_error.clone()),
        consecutive_failures: Set(i32::try_from(snapshot.consecutive_failures).map_err(|e| {
            DataStoreError::ValidationError {
                message: format!("Invalid consecutive failure count: {e}"),
            }
        })?),
    }
    .insert(&txn)
    .await
    .map_err(|e| DataStoreError::InternalError {
        message: format!(
            "Failed to insert node_status snapshot for {}: {e}",
            snapshot.node_id
        ),
    })?;

    for interface in &snapshot.interfaces {
        interface_status::ActiveModel {
            id: Set(Uuid::new_v4().to_string()),
            node_status_id: Set(snapshot_id.clone()),
            index: Set(i32::try_from(interface.index).map_err(|e| {
                DataStoreError::ValidationError {
                    message: format!("Invalid interface index: {e}"),
                }
            })?),
            name: Set(interface.name.clone()),
            interface_type: Set(i32::try_from(interface.interface_type).map_err(|e| {
                DataStoreError::ValidationError {
                    message: format!("Invalid interface type: {e}"),
                }
            })?),
            mtu: Set(interface.mtu.map(i32::try_from).transpose().map_err(|e| {
                DataStoreError::ValidationError {
                    message: format!("Invalid interface MTU: {e}"),
                }
            })?),
            speed: Set(interface
                .speed
                .map(i64::try_from)
                .transpose()
                .map_err(|e| DataStoreError::ValidationError {
                    message: format!("Invalid interface speed: {e}"),
                })?),
            physical_address: Set(interface.physical_address.clone()),
            admin_status: Set(admin_status_value(interface.admin_status).to_string()),
            oper_status: Set(oper_status_value(interface.oper_status).to_string()),
            last_change: Set(interface
                .last_change
                .map(i32::try_from)
                .transpose()
                .map_err(|e| DataStoreError::ValidationError {
                    message: format!("Invalid interface last_change: {e}"),
                })?),
            input_stats: Set(serialize_required_json(&interface.input_stats)?),
            output_stats: Set(serialize_required_json(&interface.output_stats)?),
        }
        .insert(&txn)
        .await
        .map_err(|e| DataStoreError::InternalError {
            message: format!(
                "Failed to insert interface snapshot {} for {}: {e}",
                interface.name, snapshot.node_id
            ),
        })?;
    }

    txn.commit()
        .await
        .map_err(|e| DataStoreError::TransactionError {
            message: format!("Failed to commit derived-state snapshot transaction: {e}"),
        })
}

pub async fn get_node_status_history(
    store: &SqliteStore,
    node_id: &Uuid,
    options: &HistoryQueryOptions,
) -> DataStoreResult<Vec<NodeStatus>> {
    let mut query = node_status::Entity::find()
        .filter(node_status::Column::NodeId.eq(node_id.to_string()))
        .order_by_desc(node_status::Column::LastUpdated);

    if let Some(since) = options.since {
        query = query.filter(node_status::Column::LastUpdated.gte(format_timestamp(since)));
    }
    if options.limit > 0 {
        query = query.limit(options.limit as u64);
    }

    let status_entities =
        query
            .all(&store.db)
            .await
            .map_err(|e| DataStoreError::InternalError {
                message: format!("Failed to query node_status history for node {node_id}: {e}"),
            })?;
    if status_entities.is_empty() {
        return Ok(Vec::new());
    }

    let interface_entities = interface_status::Entity::find()
        .filter(
            interface_status::Column::NodeStatusId
                .is_in(status_entities.iter().map(|status| status.id.clone())),
        )
        .order_by_asc(interface_status::Column::Index)
        .all(&store.db)
        .await
        .map_err(|e| DataStoreError::InternalError {
            message: format!("Failed to query interface history for node {node_id}: {e}"),
        })?;

    let mut interfaces_by_status: HashMap<String, Vec<interface_status::Model>> = HashMap::new();
    for interface in interface_entities {
        interfaces_by_status
            .entry(interface.node_status_id.clone())
            .or_default()
            .push(interface);
    }

    status_entities
        .into_iter()
        .map(|status| {
            let interfaces = interfaces_by_status.remove(&status.id).unwrap_or_default();
            entity_to_node_status(status, interfaces)
        })
        .collect()
}

fn serialize_required_json<T>(value: &T) -> DataStoreResult<String>
where
    T: serde::Serialize,
{
    serde_json::to_string(value).map_err(|e| DataStoreError::InternalError {
        message: format!("Failed to serialize derived-state JSON: {e}"),
    })
}

fn serialize_optional_json<T>(value: Option<&T>) -> DataStoreResult<Option<String>>
where
    T: serde::Serialize,
{
    value.map(serialize_required_json).transpose()
}

fn serialize_map_json<T>(value: &HashMap<String, T>) -> DataStoreResult<Option<String>>
where
    T: serde::Serialize,
{
    if value.is_empty() {
        Ok(None)
    } else {
        serialize_required_json(value).map(Some)
    }
}

fn format_timestamp(value: SystemTime) -> String {
    chrono::DateTime::<Utc>::from(value).to_rfc3339_opts(SecondsFormat::Secs, true)
}

const fn admin_status_value(value: InterfaceAdminStatus) -> &'static str {
    match value {
        InterfaceAdminStatus::Up => "up",
        InterfaceAdminStatus::Down => "down",
        InterfaceAdminStatus::Testing => "testing",
        InterfaceAdminStatus::Unknown => "unknown",
    }
}

const fn oper_status_value(value: InterfaceOperStatus) -> &'static str {
    match value {
        InterfaceOperStatus::Up => "up",
        InterfaceOperStatus::Down => "down",
        InterfaceOperStatus::Testing => "testing",
        InterfaceOperStatus::Unknown => "unknown",
        InterfaceOperStatus::Dormant => "dormant",
        InterfaceOperStatus::NotPresent => "notPresent",
        InterfaceOperStatus::LowerLayerDown => "lowerLayerDown",
    }
}
