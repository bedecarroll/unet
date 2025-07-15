//! Node operations for CSV datastore

use super::super::types::{
    BatchOperation, BatchResult, DataStoreError, DataStoreResult, FilterValue, PagedResult,
    QueryOptions, SortDirection,
};
use super::CsvStore;
use super::utils::apply_filters;
use crate::models::Node;
use uuid::Uuid;

/// Creates a new node
pub async fn create_node(store: &CsvStore, node: &Node) -> DataStoreResult<Node> {
    node.validate()
        .map_err(|e| DataStoreError::ValidationError { message: e })?;

    {
        let mut data = store.data.lock().await;
        if data.nodes.contains_key(&node.id) {
            return Err(DataStoreError::ConstraintViolation {
                message: format!("Node with ID {} already exists", node.id),
            });
        }
        data.nodes.insert(node.id, node.clone());
    }

    store.save_data().await?;
    Ok(node.clone())
}

/// Gets a node by ID
pub async fn get_node(store: &CsvStore, id: &Uuid) -> DataStoreResult<Option<Node>> {
    let data = store.data.lock().await;
    Ok(data.nodes.get(id).cloned())
}

/// Lists nodes with filtering, sorting, and pagination
pub async fn list_nodes(
    store: &CsvStore,
    options: &QueryOptions,
) -> DataStoreResult<PagedResult<Node>> {
    let mut items: Vec<Node> = {
        let data = store.data.lock().await;
        data.nodes.values().cloned().collect()
    };

    // Apply filters
    items = apply_filters(items, &options.filters, |node, field| match field {
        "name" => Some(FilterValue::String(node.name.clone())),
        "domain" => Some(FilterValue::String(node.domain.clone())),
        "fqdn" => Some(FilterValue::String(node.fqdn.clone())),
        "vendor" => Some(FilterValue::String(node.vendor.to_string())),
        "model" => Some(FilterValue::String(node.model.clone())),
        "role" => Some(FilterValue::String(node.role.to_string())),
        "lifecycle" => Some(FilterValue::String(node.lifecycle.to_string())),
        "location_id" => node.location_id.map(FilterValue::Uuid),
        _ => None,
    });

    // Apply sorting
    for sort in &options.sort {
        items.sort_by(|a, b| {
            let result = match sort.field.as_str() {
                "name" => a.name.cmp(&b.name),
                "domain" => a.domain.cmp(&b.domain),
                "fqdn" => a.fqdn.cmp(&b.fqdn),
                "model" => a.model.cmp(&b.model),
                _ => std::cmp::Ordering::Equal,
            };
            if matches!(sort.direction, SortDirection::Descending) {
                result.reverse()
            } else {
                result
            }
        });
    }

    // Apply pagination
    let total = items.len();
    let (start, page_limit) = options
        .pagination
        .as_ref()
        .map_or((0, total), |pagination| {
            (pagination.offset, pagination.limit)
        });
    let paginated_items = items.into_iter().skip(start).take(page_limit).collect();

    Ok(PagedResult::new(
        paginated_items,
        total,
        options.pagination.as_ref(),
    ))
}

/// Updates an existing node
pub async fn update_node(store: &CsvStore, node: &Node) -> DataStoreResult<Node> {
    node.validate()
        .map_err(|e| DataStoreError::ValidationError { message: e })?;

    {
        let mut data = store.data.lock().await;
        if !data.nodes.contains_key(&node.id) {
            return Err(DataStoreError::NotFound {
                entity_type: "node".to_string(),
                id: node.id.to_string(),
            });
        }
        data.nodes.insert(node.id, node.clone());
    }

    store.save_data().await?;
    Ok(node.clone())
}

/// Deletes a node by ID
pub async fn delete_node(store: &CsvStore, id: &Uuid) -> DataStoreResult<()> {
    {
        let mut data = store.data.lock().await;
        if data.nodes.remove(id).is_none() {
            return Err(DataStoreError::NotFound {
                entity_type: "node".to_string(),
                id: id.to_string(),
            });
        }
    }

    store.save_data().await?;
    Ok(())
}

/// Gets nodes by location ID
pub async fn get_nodes_by_location(
    store: &CsvStore,
    location_id: &Uuid,
) -> DataStoreResult<Vec<Node>> {
    let nodes = {
        let data = store.data.lock().await;
        data.nodes
            .values()
            .filter(|node| node.location_id.as_ref() == Some(location_id))
            .cloned()
            .collect()
    };
    Ok(nodes)
}

/// Searches nodes by name (case-insensitive substring match)
pub async fn search_nodes_by_name(store: &CsvStore, name: &str) -> DataStoreResult<Vec<Node>> {
    let name_lower = name.to_lowercase();
    let nodes = {
        let data = store.data.lock().await;
        data.nodes
            .values()
            .filter(|node| node.name.to_lowercase().contains(&name_lower))
            .cloned()
            .collect()
    };
    Ok(nodes)
}

/// Performs batch operations on nodes
pub async fn batch_nodes(
    store: &CsvStore,
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
