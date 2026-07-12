//! Link operations for `SQLite` datastore

use super::super::types::{
    BatchOperation, BatchResult, DataStoreError, DataStoreResult, PagedResult, QueryOptions,
};
use super::SqliteStore;
use super::conversions::entity_to_link;
use super::filters::{apply_link_filters, apply_link_sorting};
use crate::entities::links;
use crate::models::Link;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect, Set,
};
use uuid::Uuid;

/// Creates a new link
pub async fn create_link(store: &SqliteStore, link: &Link) -> DataStoreResult<Link> {
    let active_link = links::ActiveModel {
        id: Set(link.id.to_string()),
        name: Set(link.name.clone()),
        node_a_id: Set(link.source_node_id.to_string()),
        interface_a: Set(link.node_a_interface.clone()),
        node_b_id: Set(link.dest_node_id.map(|id| id.to_string())),
        interface_b: Set(link.node_z_interface.clone()),
        capacity: Set(link.bandwidth.map(|b| b.try_into().unwrap_or(i64::MAX))),
        utilization: Set(None), // Not in Link model yet
        is_internet_circuit: Set(i32::from(link.is_internet_circuit)),
        circuit_id: Set(None), // Not in Link model yet
        provider: Set(None),   // Not in Link model yet
        description: Set(link.description.clone()),
        custom_data: Set(Some(
            serde_json::to_string(&link.custom_data).unwrap_or_default(),
        )),
        created_at: Set(Utc::now().to_rfc3339()),
        updated_at: Set(Utc::now().to_rfc3339()),
    };

    active_link
        .insert(&store.db)
        .await
        .map_err(|e| DataStoreError::InternalError {
            message: format!("Failed to create link: {e}"),
        })?;

    // Convert back to Link model
    get_link(store, &link.id)
        .await?
        .ok_or_else(|| DataStoreError::NotFound {
            entity_type: "Link".to_string(),
            id: link.id.to_string(),
        })
}

/// Gets a link by ID
pub async fn get_link(store: &SqliteStore, id: &Uuid) -> DataStoreResult<Option<Link>> {
    let entity = links::Entity::find_by_id(id.to_string())
        .one(&store.db)
        .await
        .map_err(|e| DataStoreError::InternalError {
            message: format!("Failed to query link: {e}"),
        })?;

    match entity {
        Some(e) => Ok(Some(entity_to_link(e)?)),
        None => Ok(None),
    }
}

/// Lists links with filtering, sorting, and pagination
pub async fn list_links(
    store: &SqliteStore,
    options: &QueryOptions,
) -> DataStoreResult<PagedResult<Link>> {
    let mut query = links::Entity::find();

    // Apply filters and sorting using helper functions
    query = apply_link_filters(query, &options.filters)?;
    query = apply_link_sorting(query, &options.sort)?;

    // Get total count
    let total_count =
        query
            .clone()
            .count(&store.db)
            .await
            .map_err(|e| DataStoreError::InternalError {
                message: format!("Failed to count links: {e}"),
            })?;

    // Apply pagination
    if let Some(pagination) = &options.pagination {
        query = query
            .offset(pagination.offset as u64)
            .limit(pagination.limit as u64);
    }

    // Execute query
    let entities = query
        .all(&store.db)
        .await
        .map_err(|e| DataStoreError::InternalError {
            message: format!("Failed to query links: {e}"),
        })?;

    // Convert entities to Link models
    let links = entities
        .into_iter()
        .map(entity_to_link)
        .collect::<DataStoreResult<Vec<_>>>()?;

    Ok(PagedResult::new(
        links,
        total_count.try_into().unwrap_or(usize::MAX),
        options.pagination.as_ref(),
    ))
}

/// Updates an existing link
pub async fn update_link(store: &SqliteStore, link: &Link) -> DataStoreResult<Link> {
    let active_link = links::ActiveModel {
        id: Set(link.id.to_string()),
        name: Set(link.name.clone()),
        node_a_id: Set(link.source_node_id.to_string()),
        interface_a: Set(link.node_a_interface.clone()),
        node_b_id: Set(link.dest_node_id.map(|id| id.to_string())),
        interface_b: Set(link.node_z_interface.clone()),
        capacity: Set(link.bandwidth.map(|b| b.try_into().unwrap_or(i64::MAX))),
        utilization: Set(None), // Not in Link model yet
        is_internet_circuit: Set(i32::from(link.is_internet_circuit)),
        circuit_id: Set(None), // Not in Link model yet
        provider: Set(None),   // Not in Link model yet
        description: Set(link.description.clone()),
        custom_data: Set(Some(
            serde_json::to_string(&link.custom_data).unwrap_or_default(),
        )),
        created_at: Set(Utc::now().to_rfc3339()),
        updated_at: Set(Utc::now().to_rfc3339()),
    };

    active_link
        .update(&store.db)
        .await
        .map_err(|e| DataStoreError::InternalError {
            message: format!("Failed to update link: {e}"),
        })?;

    // Convert back to Link model
    get_link(store, &link.id)
        .await?
        .ok_or_else(|| DataStoreError::NotFound {
            entity_type: "Link".to_string(),
            id: link.id.to_string(),
        })
}

/// Deletes a link by ID
pub async fn delete_link(store: &SqliteStore, id: &Uuid) -> DataStoreResult<()> {
    let result = links::Entity::delete_by_id(id.to_string())
        .exec(&store.db)
        .await
        .map_err(|e| DataStoreError::InternalError {
            message: format!("Failed to delete link: {e}"),
        })?;

    if result.rows_affected == 0 {
        return Err(DataStoreError::NotFound {
            entity_type: "Link".to_string(),
            id: id.to_string(),
        });
    }

    Ok(())
}

/// Gets links that involve a specific node
pub async fn get_links_for_node(store: &SqliteStore, node_id: &Uuid) -> DataStoreResult<Vec<Link>> {
    let entities = links::Entity::find()
        .filter(
            links::Column::NodeAId
                .eq(node_id.to_string())
                .or(links::Column::NodeBId.eq(node_id.to_string())),
        )
        .all(&store.db)
        .await
        .map_err(|e| DataStoreError::InternalError {
            message: format!("Failed to query links for node: {e}"),
        })?;

    entities
        .into_iter()
        .map(entity_to_link)
        .collect::<DataStoreResult<Vec<_>>>()
}

/// Gets links between two specific nodes
pub async fn get_links_between_nodes(
    store: &SqliteStore,
    first_node_id: &Uuid,
    second_node_id: &Uuid,
) -> DataStoreResult<Vec<Link>> {
    let entities = links::Entity::find()
        .filter(
            links::Column::NodeAId
                .eq(first_node_id.to_string())
                .and(links::Column::NodeBId.eq(second_node_id.to_string()))
                .or(links::Column::NodeAId
                    .eq(second_node_id.to_string())
                    .and(links::Column::NodeBId.eq(first_node_id.to_string()))),
        )
        .all(&store.db)
        .await
        .map_err(|e| DataStoreError::InternalError {
            message: format!("Failed to query links between nodes: {e}"),
        })?;

    entities
        .into_iter()
        .map(entity_to_link)
        .collect::<DataStoreResult<Vec<_>>>()
}

/// Performs batch operations on links
pub async fn batch_links(
    store: &SqliteStore,
    operations: &[BatchOperation<Link>],
) -> DataStoreResult<BatchResult> {
    let mut success_count = 0;
    let mut errors = Vec::new();

    for (index, operation) in operations.iter().enumerate() {
        let result = match operation {
            BatchOperation::Insert(link) => create_link(store, link).await.map(|_| ()),
            BatchOperation::Update(link) => update_link(store, link).await.map(|_| ()),
            BatchOperation::Delete(id) => delete_link(store, id).await,
        };

        match result {
            Ok(()) => success_count += 1,
            Err(e) => errors.push((index, e)),
        }
    }

    Ok(BatchResult {
        success_count,
        error_count: errors.len(),
        errors,
    })
}
