//! Metadata and statistics queries for the `SQLite` datastore

use super::SqliteStore;
use crate::datastore::types::{DataStoreError, DataStoreResult};
use crate::entities::{
    interface_status, links, locations, node_status, nodes, polling_tasks, vendors,
};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use std::collections::HashMap;

pub async fn get_entity_counts(store: &SqliteStore) -> DataStoreResult<HashMap<String, usize>> {
    let mut counts = HashMap::new();

    counts.insert(
        "vendors".to_string(),
        count_query(vendors::Entity::find().count(&store.db).await, "vendors")?,
    );
    counts.insert(
        "locations".to_string(),
        count_query(
            locations::Entity::find().count(&store.db).await,
            "locations",
        )?,
    );
    counts.insert(
        "nodes".to_string(),
        count_query(nodes::Entity::find().count(&store.db).await, "nodes")?,
    );
    counts.insert(
        "links".to_string(),
        count_query(links::Entity::find().count(&store.db).await, "links")?,
    );
    counts.insert(
        "node_status".to_string(),
        count_query(
            node_status::Entity::find().count(&store.db).await,
            "node_status",
        )?,
    );
    counts.insert(
        "interface_status".to_string(),
        count_query(
            interface_status::Entity::find().count(&store.db).await,
            "interface_status",
        )?,
    );
    counts.insert(
        "polling_tasks".to_string(),
        count_query(
            polling_tasks::Entity::find().count(&store.db).await,
            "polling_tasks",
        )?,
    );

    Ok(counts)
}

pub async fn get_statistics(
    store: &SqliteStore,
) -> DataStoreResult<HashMap<String, serde_json::Value>> {
    let counts = get_entity_counts(store).await?;
    let nodes_with_status = *counts.get("node_status").unwrap_or(&0);
    let interfaces_monitored = *counts.get("interface_status").unwrap_or(&0);
    let reachable_nodes = node_status::Entity::find()
        .filter(node_status::Column::Reachable.eq(true))
        .count(&store.db)
        .await
        .map_err(|e| DataStoreError::InternalError {
            message: format!("Failed to count reachable node_status rows: {e}"),
        })?
        .try_into()
        .unwrap_or(usize::MAX);
    let latest_status_update = node_status::Entity::find()
        .order_by_desc(node_status::Column::LastUpdated)
        .one(&store.db)
        .await
        .map_err(|e| DataStoreError::InternalError {
            message: format!("Failed to query latest node_status row: {e}"),
        })?
        .map(|model| model.last_updated);

    let mut stats = HashMap::new();
    stats.insert("datastore".to_string(), serde_json::Value::from("sqlite"));
    for (key, value) in counts {
        stats.insert(key, serde_json::Value::from(value));
    }
    stats.insert(
        "nodes_with_status".to_string(),
        serde_json::Value::from(nodes_with_status),
    );
    stats.insert(
        "interfaces_monitored".to_string(),
        serde_json::Value::from(interfaces_monitored),
    );
    stats.insert(
        "reachable_nodes".to_string(),
        serde_json::Value::from(reachable_nodes),
    );
    stats.insert(
        "unreachable_nodes".to_string(),
        serde_json::Value::from(nodes_with_status.saturating_sub(reachable_nodes)),
    );
    stats.insert(
        "latest_status_update".to_string(),
        latest_status_update.map_or(serde_json::Value::Null, serde_json::Value::from),
    );

    Ok(stats)
}

fn count_query(result: Result<u64, sea_orm::DbErr>, label: &str) -> DataStoreResult<usize> {
    result
        .map_err(|e| DataStoreError::InternalError {
            message: format!("Failed to count {label}: {e}"),
        })?
        .try_into()
        .map_err(|e| DataStoreError::InternalError {
            message: format!("Failed to convert count for {label}: {e}"),
        })
}
