//! Latest derived-state lookups for the `SQLite` datastore.

use super::{SqliteStore, derived_state_history};
use crate::datastore::{DataStoreResult, HistoryQueryOptions};
use crate::models::derived::{InterfaceStatus, NodeStatus, PerformanceMetrics};
use uuid::Uuid;

pub async fn get_node_status(
    store: &SqliteStore,
    node_id: &Uuid,
) -> DataStoreResult<Option<NodeStatus>> {
    Ok(derived_state_history::get_node_status_history(
        store,
        node_id,
        &HistoryQueryOptions {
            limit: 1,
            since: None,
        },
    )
    .await?
    .into_iter()
    .next())
}

pub async fn get_node_interfaces(
    store: &SqliteStore,
    node_id: &Uuid,
) -> DataStoreResult<Vec<InterfaceStatus>> {
    Ok(get_node_status(store, node_id)
        .await?
        .map_or_else(Vec::new, |status| status.interfaces))
}

pub async fn get_node_metrics(
    store: &SqliteStore,
    node_id: &Uuid,
) -> DataStoreResult<Option<PerformanceMetrics>> {
    Ok(get_node_status(store, node_id)
        .await?
        .and_then(|status| status.performance))
}
