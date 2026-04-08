//! Derived-state queries for the `SQLite` datastore

use super::SqliteStore;
use super::conversions::{entity_to_interface_status, entity_to_node_status, parse_optional_json};
use crate::datastore::types::{DataStoreError, DataStoreResult};
use crate::entities::{interface_status, node_status};
use crate::models::derived::{InterfaceStatus, NodeStatus, PerformanceMetrics};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

pub async fn get_node_status(
    store: &SqliteStore,
    node_id: &Uuid,
) -> DataStoreResult<Option<NodeStatus>> {
    let Some(status_entity) = get_node_status_entity(store, node_id).await? else {
        return Ok(None);
    };
    let interfaces = interface_status::Entity::find()
        .filter(interface_status::Column::NodeStatusId.eq(status_entity.id.clone()))
        .order_by_asc(interface_status::Column::Index)
        .all(&store.db)
        .await
        .map_err(|e| DataStoreError::InternalError {
            message: format!("Failed to query interface status for node {node_id}: {e}"),
        })?;

    entity_to_node_status(status_entity, interfaces).map(Some)
}

pub async fn get_node_interfaces(
    store: &SqliteStore,
    node_id: &Uuid,
) -> DataStoreResult<Vec<InterfaceStatus>> {
    let Some(status_entity) = get_node_status_entity(store, node_id).await? else {
        return Ok(Vec::new());
    };

    interface_status::Entity::find()
        .filter(interface_status::Column::NodeStatusId.eq(status_entity.id))
        .order_by_asc(interface_status::Column::Index)
        .all(&store.db)
        .await
        .map_err(|e| DataStoreError::InternalError {
            message: format!("Failed to query interface status for node {node_id}: {e}"),
        })?
        .into_iter()
        .map(entity_to_interface_status)
        .collect()
}

pub async fn get_node_metrics(
    store: &SqliteStore,
    node_id: &Uuid,
) -> DataStoreResult<Option<PerformanceMetrics>> {
    let Some(status_entity) = get_node_status_entity(store, node_id).await? else {
        return Ok(None);
    };

    parse_optional_json(status_entity.performance, "node_status.performance")
}

async fn get_node_status_entity(
    store: &SqliteStore,
    node_id: &Uuid,
) -> DataStoreResult<Option<node_status::Model>> {
    node_status::Entity::find()
        .filter(node_status::Column::NodeId.eq(node_id.to_string()))
        .one(&store.db)
        .await
        .map_err(|e| DataStoreError::InternalError {
            message: format!("Failed to query node_status for node {node_id}: {e}"),
        })
}
