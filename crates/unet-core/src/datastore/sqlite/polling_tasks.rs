//! Polling-task persistence for the `SQLite` datastore

use super::super::types::{DataStoreError, DataStoreResult};
use super::SqliteStore;
use crate::config::network;
use crate::entities::polling_tasks;
use crate::snmp::PollingTask;
use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Serialize, de::DeserializeOwned};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

pub async fn get_node_polling_task(
    store: &SqliteStore,
    node_id: &Uuid,
) -> DataStoreResult<Option<PollingTask>> {
    polling_tasks::Entity::find()
        .filter(polling_tasks::Column::NodeId.eq(node_id.to_string()))
        .one(&store.db)
        .await
        .map_err(|e| DataStoreError::InternalError {
            message: format!("Failed to query polling task for node {node_id}: {e}"),
        })?
        .map(entity_to_polling_task)
        .transpose()
}

pub async fn upsert_polling_task(
    store: &SqliteStore,
    task: &PollingTask,
) -> DataStoreResult<PollingTask> {
    let active_model = polling_tasks::ActiveModel {
        id: Set(task.id.to_string()),
        node_id: Set(task.node_id.to_string()),
        target: Set(task.target.to_string()),
        oids: Set(serialize_json(&task.oids, "polling_tasks.oids")?),
        interval_seconds: Set(i64::try_from(task.interval.as_secs()).map_err(|e| {
            DataStoreError::ValidationError {
                message: format!("Invalid polling interval: {e}"),
            }
        })?),
        session_config: Set(serialize_json(
            &task.session_config,
            "polling_tasks.session_config",
        )?),
        priority: Set(i16::from(task.priority)),
        enabled: Set(task.enabled),
        created_at: Set(format_timestamp(task.created_at)),
        last_success: Set(task.last_success.map(format_timestamp)),
        last_error: Set(task.last_error.clone()),
        consecutive_failures: Set(i32::try_from(task.consecutive_failures).map_err(|e| {
            DataStoreError::ValidationError {
                message: format!("Invalid consecutive failures count: {e}"),
            }
        })?),
    };

    let existing = polling_tasks::Entity::find()
        .filter(polling_tasks::Column::NodeId.eq(task.node_id.to_string()))
        .one(&store.db)
        .await
        .map_err(|e| DataStoreError::InternalError {
            message: format!(
                "Failed to query existing polling task for {}: {e}",
                task.node_id
            ),
        })?;

    if existing.is_some() {
        active_model
            .update(&store.db)
            .await
            .map_err(|e| DataStoreError::InternalError {
                message: format!("Failed to update polling task for {}: {e}", task.node_id),
            })?;
    } else {
        active_model
            .insert(&store.db)
            .await
            .map_err(|e| DataStoreError::InternalError {
                message: format!("Failed to create polling task for {}: {e}", task.node_id),
            })?;
    }

    get_node_polling_task(store, &task.node_id)
        .await?
        .ok_or_else(|| DataStoreError::NotFound {
            entity_type: "PollingTask".to_string(),
            id: task.node_id.to_string(),
        })
}

fn entity_to_polling_task(entity: polling_tasks::Model) -> DataStoreResult<PollingTask> {
    let id = entity
        .id
        .parse::<Uuid>()
        .map_err(|e| DataStoreError::ValidationError {
            message: format!("Invalid polling task UUID: {e}"),
        })?;
    let node_id = entity
        .node_id
        .parse::<Uuid>()
        .map_err(|e| DataStoreError::ValidationError {
            message: format!("Invalid polling task node UUID: {e}"),
        })?;
    let target = network::parse_socket_addr(&entity.target).map_err(|e| {
        DataStoreError::ValidationError {
            message: format!("Invalid polling task target '{}': {e}", entity.target),
        }
    })?;

    Ok(PollingTask {
        id,
        target,
        node_id,
        oids: deserialize_json(&entity.oids, "polling_tasks.oids")?,
        interval: Duration::from_secs(u64::try_from(entity.interval_seconds).map_err(|e| {
            DataStoreError::ValidationError {
                message: format!("Invalid polling interval: {e}"),
            }
        })?),
        session_config: deserialize_json(&entity.session_config, "polling_tasks.session_config")?,
        priority: u8::try_from(entity.priority).map_err(|e| DataStoreError::ValidationError {
            message: format!("Invalid polling task priority: {e}"),
        })?,
        enabled: entity.enabled,
        created_at: parse_timestamp(&entity.created_at, "polling_tasks.created_at")?,
        last_success: entity
            .last_success
            .as_deref()
            .map(|value| parse_timestamp(value, "polling_tasks.last_success"))
            .transpose()?,
        last_error: entity.last_error,
        consecutive_failures: u32::try_from(entity.consecutive_failures).map_err(|e| {
            DataStoreError::ValidationError {
                message: format!("Invalid consecutive failure count: {e}"),
            }
        })?,
    })
}

fn format_timestamp(value: SystemTime) -> String {
    DateTime::<Utc>::from(value).to_rfc3339()
}

fn parse_timestamp(value: &str, field: &str) -> DataStoreResult<SystemTime> {
    DateTime::parse_from_rfc3339(value)
        .map(|timestamp| timestamp.with_timezone(&Utc).into())
        .map_err(|e| DataStoreError::ValidationError {
            message: format!("Invalid timestamp in {field}: {e}"),
        })
}

fn serialize_json<T>(value: &T, field: &str) -> DataStoreResult<String>
where
    T: Serialize,
{
    serde_json::to_string(value).map_err(|e| DataStoreError::ValidationError {
        message: format!("Invalid JSON in {field}: {e}"),
    })
}

fn deserialize_json<T>(value: &str, field: &str) -> DataStoreResult<T>
where
    T: DeserializeOwned,
{
    serde_json::from_str(value).map_err(|e| DataStoreError::ValidationError {
        message: format!("Invalid JSON in {field}: {e}"),
    })
}
