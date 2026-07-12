//! Node operations for `SQLite` datastore

use super::super::types::{
    BatchOperation, BatchResult, DataStoreError, DataStoreResult, PagedResult, QueryOptions,
};
use super::SqliteStore;
use super::conversions::entity_to_node;
use super::filters::{apply_node_filters, apply_node_sorting};
use crate::entities::nodes;
use crate::models::Node;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect, Set,
};
use uuid::Uuid;

/// Creates a new node
pub async fn create_node(store: &SqliteStore, node: &Node) -> DataStoreResult<Node> {
    let active_node = nodes::ActiveModel {
        id: Set(node.id.to_string()),
        name: Set(node.name.clone()),
        fqdn: Set(Some(node.fqdn.clone())),
        domain: Set(Some(node.domain.clone())),
        vendor: Set(node.vendor.to_string()),
        model: Set(node.model.clone()),
        role: Set(node.role.to_string()),
        lifecycle: Set(node.lifecycle.to_string()),
        serial_number: Set(node.serial_number.clone()),
        asset_tag: Set(node.asset_tag.clone()),
        location_id: Set(node.location_id.map(|id| id.to_string())),
        management_ip: Set(node.management_ip.map(|ip| ip.to_string())),
        description: Set(None), // Not used in Node model yet
        custom_data: Set(Some(
            serde_json::to_string(&node.custom_data).unwrap_or_default(),
        )),
        created_at: Set(Utc::now().to_rfc3339()),
        updated_at: Set(Utc::now().to_rfc3339()),
    };

    active_node
        .insert(&store.db)
        .await
        .map_err(|e| DataStoreError::InternalError {
            message: format!("Failed to create node: {e}"),
        })?;

    // Convert back to Node model
    get_node(store, &node.id)
        .await?
        .ok_or_else(|| DataStoreError::NotFound {
            entity_type: "Node".to_string(),
            id: node.id.to_string(),
        })
}

/// Gets a node by ID
pub async fn get_node(store: &SqliteStore, id: &Uuid) -> DataStoreResult<Option<Node>> {
    let entity = nodes::Entity::find_by_id(id.to_string())
        .one(&store.db)
        .await
        .map_err(|e| DataStoreError::InternalError {
            message: format!("Failed to query node: {e}"),
        })?;

    match entity {
        Some(e) => Ok(Some(entity_to_node(e)?)),
        None => Ok(None),
    }
}

/// Lists nodes with filtering, sorting, and pagination
pub async fn list_nodes(
    store: &SqliteStore,
    options: &QueryOptions,
) -> DataStoreResult<PagedResult<Node>> {
    let mut query = nodes::Entity::find();

    // Apply filters and sorting using helper functions
    query = apply_node_filters(query, &options.filters)?;
    query = apply_node_sorting(query, &options.sort)?;

    // Get total count
    let total_count =
        query
            .clone()
            .count(&store.db)
            .await
            .map_err(|e| DataStoreError::InternalError {
                message: format!("Failed to count nodes: {e}"),
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
            message: format!("Failed to query nodes: {e}"),
        })?;

    // Convert entities to Node models
    let nodes = entities
        .into_iter()
        .map(entity_to_node)
        .collect::<DataStoreResult<Vec<_>>>()?;

    Ok(PagedResult::new(
        nodes,
        total_count.try_into().unwrap_or(usize::MAX),
        options.pagination.as_ref(),
    ))
}

/// Updates an existing node
pub async fn update_node(store: &SqliteStore, node: &Node) -> DataStoreResult<Node> {
    let active_node = nodes::ActiveModel {
        id: Set(node.id.to_string()),
        name: Set(node.name.clone()),
        fqdn: Set(Some(node.fqdn.clone())),
        domain: Set(Some(node.domain.clone())),
        vendor: Set(node.vendor.to_string()),
        model: Set(node.model.clone()),
        role: Set(node.role.to_string()),
        lifecycle: Set(node.lifecycle.to_string()),
        serial_number: Set(node.serial_number.clone()),
        asset_tag: Set(node.asset_tag.clone()),
        location_id: Set(node.location_id.map(|id| id.to_string())),
        management_ip: Set(node.management_ip.map(|ip| ip.to_string())),
        description: Set(None), // Not used in Node model yet
        custom_data: Set(Some(
            serde_json::to_string(&node.custom_data).unwrap_or_default(),
        )),
        created_at: Set(Utc::now().to_rfc3339()),
        updated_at: Set(Utc::now().to_rfc3339()),
    };

    let update_result = active_node.update(&store.db).await;
    match update_result {
        Ok(_) => {}
        Err(e) => {
            // Check if the error indicates no records were updated
            let error_msg = e.to_string();
            if error_msg.contains("None of the records are updated") {
                return Err(DataStoreError::NotFound {
                    entity_type: "Node".to_string(),
                    id: node.id.to_string(),
                });
            }
            return Err(DataStoreError::InternalError {
                message: format!("Failed to update node: {e}"),
            });
        }
    }

    // Convert back to Node model
    get_node(store, &node.id)
        .await?
        .ok_or_else(|| DataStoreError::NotFound {
            entity_type: "Node".to_string(),
            id: node.id.to_string(),
        })
}

/// Deletes a node by ID
pub async fn delete_node(store: &SqliteStore, id: &Uuid) -> DataStoreResult<()> {
    let result = nodes::Entity::delete_by_id(id.to_string())
        .exec(&store.db)
        .await
        .map_err(|e| DataStoreError::InternalError {
            message: format!("Failed to delete node: {e}"),
        })?;

    if result.rows_affected == 0 {
        return Err(DataStoreError::NotFound {
            entity_type: "Node".to_string(),
            id: id.to_string(),
        });
    }

    Ok(())
}

/// Gets nodes by location ID
pub async fn get_nodes_by_location(
    store: &SqliteStore,
    location_id: &Uuid,
) -> DataStoreResult<Vec<Node>> {
    let entities = nodes::Entity::find()
        .filter(nodes::Column::LocationId.eq(location_id.to_string()))
        .all(&store.db)
        .await
        .map_err(|e| DataStoreError::InternalError {
            message: format!("Failed to query nodes by location: {e}"),
        })?;

    entities
        .into_iter()
        .map(entity_to_node)
        .collect::<DataStoreResult<Vec<_>>>()
}

/// Searches nodes by name (case-insensitive partial match)
pub async fn search_nodes_by_name(store: &SqliteStore, name: &str) -> DataStoreResult<Vec<Node>> {
    // Escape SQL wildcard characters to treat them as literal characters
    let escaped_name = name
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");

    let entities = nodes::Entity::find()
        .filter(nodes::Column::Name.contains(&escaped_name))
        .all(&store.db)
        .await
        .map_err(|e| DataStoreError::InternalError {
            message: format!("Failed to search nodes: {e}"),
        })?;

    entities
        .into_iter()
        .map(entity_to_node)
        .collect::<DataStoreResult<Vec<_>>>()
}

/// Performs batch operations on nodes
pub async fn batch_nodes(
    store: &SqliteStore,
    operations: &[BatchOperation<Node>],
) -> DataStoreResult<BatchResult> {
    let mut success_count = 0;
    let mut errors = Vec::new();

    for (index, operation) in operations.iter().enumerate() {
        let result = match operation {
            BatchOperation::Insert(node) => create_node(store, node).await.map(|_| ()),
            BatchOperation::Update(node) => update_node(store, node).await.map(|_| ()),
            BatchOperation::Delete(id) => delete_node(store, id).await,
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
