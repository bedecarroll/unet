//! Link operations for CSV datastore

use super::super::types::{
    BatchOperation, BatchResult, DataStoreError, DataStoreResult, FilterValue, PagedResult,
    QueryOptions, SortDirection,
};
use super::CsvStore;
use super::utils::apply_filters;
use crate::models::Link;
use uuid::Uuid;

/// Creates a new link
pub async fn create_link(store: &CsvStore, link: &Link) -> DataStoreResult<Link> {
    link.validate()
        .map_err(|e| DataStoreError::ValidationError { message: e })?;

    {
        let mut data = store.data.lock().await;
        if data.links.contains_key(&link.id) {
            return Err(DataStoreError::ConstraintViolation {
                message: format!("Link with ID {} already exists", link.id),
            });
        }
        data.links.insert(link.id, link.clone());
    }

    store.save_data().await?;
    Ok(link.clone())
}

/// Gets a link by ID
pub async fn get_link(store: &CsvStore, id: &Uuid) -> DataStoreResult<Option<Link>> {
    let data = store.data.lock().await;
    Ok(data.links.get(id).cloned())
}

/// Lists links with filtering, sorting, and pagination
pub async fn list_links(
    store: &CsvStore,
    options: &QueryOptions,
) -> DataStoreResult<PagedResult<Link>> {
    let mut items: Vec<Link> = {
        let data = store.data.lock().await;
        data.links.values().cloned().collect()
    };

    // Apply filters
    items = apply_filters(items, &options.filters, |link, field| match field {
        "name" => Some(FilterValue::String(link.name.clone())),
        "source_node_id" => Some(FilterValue::Uuid(link.source_node_id)),
        "node_a_interface" => Some(FilterValue::String(link.node_a_interface.clone())),
        "dest_node_id" => link.dest_node_id.map(FilterValue::Uuid),
        "is_internet_circuit" => Some(FilterValue::String(link.is_internet_circuit.to_string())),
        _ => None,
    });

    // Apply sorting
    for sort in &options.sort {
        items.sort_by(|a, b| {
            let result = match sort.field.as_str() {
                "name" => a.name.cmp(&b.name),
                "node_a_interface" => a.node_a_interface.cmp(&b.node_a_interface),
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

/// Updates an existing link
pub async fn update_link(store: &CsvStore, link: &Link) -> DataStoreResult<Link> {
    link.validate()
        .map_err(|e| DataStoreError::ValidationError { message: e })?;

    {
        let mut data = store.data.lock().await;
        if !data.links.contains_key(&link.id) {
            return Err(DataStoreError::NotFound {
                entity_type: "link".to_string(),
                id: link.id.to_string(),
            });
        }
        data.links.insert(link.id, link.clone());
    }

    store.save_data().await?;
    Ok(link.clone())
}

/// Deletes a link by ID
pub async fn delete_link(store: &CsvStore, id: &Uuid) -> DataStoreResult<()> {
    {
        let mut data = store.data.lock().await;
        if data.links.remove(id).is_none() {
            return Err(DataStoreError::NotFound {
                entity_type: "link".to_string(),
                id: id.to_string(),
            });
        }
    }

    store.save_data().await?;
    Ok(())
}

/// Gets links that involve a specific node
pub async fn get_links_for_node(store: &CsvStore, node_id: &Uuid) -> DataStoreResult<Vec<Link>> {
    let links = {
        let data = store.data.lock().await;
        data.links
            .values()
            .filter(|link| link.involves_node(*node_id))
            .cloned()
            .collect()
    };
    Ok(links)
}

/// Gets links between two specific nodes
pub async fn get_links_between_nodes(
    store: &CsvStore,
    first_node_id: &Uuid,
    second_node_id: &Uuid,
) -> DataStoreResult<Vec<Link>> {
    let links = {
        let data = store.data.lock().await;
        data.links
            .values()
            .filter(|link| link.connects_nodes(*first_node_id, *second_node_id))
            .cloned()
            .collect()
    };
    Ok(links)
}

/// Performs batch operations on links
pub async fn batch_links(
    store: &CsvStore,
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
