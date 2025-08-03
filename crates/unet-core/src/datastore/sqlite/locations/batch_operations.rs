//! Batch operations for `SQLite` location datastore
//!
//! Contains batch processing functionality for multiple location operations.

use super::super::super::types::{BatchOperation, BatchResult, DataStoreResult};
use super::super::SqliteStore;
use super::crud_operations::{create_location, delete_location, update_location};
use crate::models::Location;

/// Performs batch operations on locations
pub async fn batch_locations(
    store: &SqliteStore,
    operations: &[BatchOperation<Location>],
) -> DataStoreResult<BatchResult> {
    let mut success_count = 0;
    let mut errors = Vec::new();

    for (index, operation) in operations.iter().enumerate() {
        let result = match operation {
            BatchOperation::Insert(location) => create_location(store, location).await.map(|_| ()),
            BatchOperation::Update(location) => update_location(store, location).await.map(|_| ()),
            BatchOperation::Delete(id) => delete_location(store, id).await,
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
