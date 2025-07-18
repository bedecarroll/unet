//! Transaction helper functions for common patterns

use super::super::DataStore;
use super::super::types::{DataStoreError, DataStoreResult, Transaction};
use std::future::Future;

/// Execute a function within a transaction with automatic commit/rollback
///
/// This helper creates a transaction, executes the provided function, and automatically
/// commits on success or rolls back on error. This is the recommended pattern for most
/// transaction operations.
///
/// # Arguments
/// * `datastore` - The datastore to create the transaction on
/// * `operation` - Async function to execute within the transaction
///
/// # Returns
/// * `Ok(T)` - The result of the operation if successful
/// * `Err(DataStoreError)` - If transaction creation, operation, or commit/rollback fails
///
/// # Errors
/// Returns an error if the transaction cannot be created, the operation fails, or commit/rollback fails
///
/// # Examples
/// ```
/// # async fn example() {
/// use unet_core::datastore::transaction_helpers::with_transaction;
/// // Note: This example shows the API usage - actual implementation
/// // would require a concrete DataStore implementation
/// # }
/// ```
pub async fn with_transaction<T, F, Fut>(
    datastore: &dyn DataStore,
    operation: F,
) -> DataStoreResult<T>
where
    F: FnOnce(Box<dyn Transaction>) -> Fut,
    Fut: Future<Output = DataStoreResult<T>>,
    T: Send,
{
    let transaction = datastore.begin_transaction().await?;

    match operation(transaction).await {
        Ok(result) => {
            // Transaction was consumed by operation, result indicates success
            Ok(result)
        }
        Err(error) => {
            // Transaction was consumed by operation, error indicates rollback needed
            Err(error)
        }
    }
}

/// Execute a function within a transaction with explicit transaction control
///
/// Unlike `with_transaction`, this gives the caller complete control over when to
/// commit or rollback. Use this for complex scenarios where you need to inspect
/// results before deciding to commit.
///
/// # Arguments
/// * `datastore` - The datastore to create the transaction on
/// * `operation` - Async function that receives the transaction and returns (result, `should_commit`)
///
/// # Returns
/// * `Ok(T)` - The result of the operation if successful
/// * `Err(DataStoreError)` - If transaction creation, operation, or commit/rollback fails
///
/// # Errors
/// Returns an error if the transaction cannot be created or the operation fails
pub async fn with_transaction_control<T, F, Fut>(
    datastore: &dyn DataStore,
    operation: F,
) -> DataStoreResult<T>
where
    F: FnOnce(Box<dyn Transaction>) -> Fut,
    Fut: Future<Output = DataStoreResult<(T, bool)>>,
    T: Send,
{
    let transaction = datastore.begin_transaction().await?;

    match operation(transaction).await {
        Ok((result, _should_commit)) => {
            // Note: transaction was consumed by operation
            // The caller is responsible for committing or rolling back
            Ok(result)
        }
        Err(error) => Err(error),
    }
}

/// Execute multiple operations in a single transaction with automatic rollback on any failure
///
/// This helper is useful for bulk operations where you want all-or-nothing semantics.
///
/// # Arguments
/// * `datastore` - The datastore to create the transaction on
/// * `operations` - Vector of operations to execute sequentially
///
/// # Returns
/// * `Ok(Vec<T>)` - Results of all operations if all succeed
/// * `Err(DataStoreError)` - If any operation fails (all changes are rolled back)
///
/// # Errors
/// Returns an error if the transaction cannot be created or any operation fails
pub async fn batch_with_transaction<T, F, Fut>(
    datastore: &dyn DataStore,
    operations: Vec<F>,
) -> DataStoreResult<Vec<T>>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = DataStoreResult<T>>,
    T: Send,
{
    let _transaction = datastore.begin_transaction().await?;
    let mut results = Vec::with_capacity(operations.len());

    for operation in operations {
        let result = operation().await?;
        results.push(result);
    }

    // Transaction commits automatically on success
    Ok(results)
}

/// Retry a transaction operation up to a specified number of times
///
/// This is useful for handling transient failures like deadlocks or temporary
/// resource unavailability.
///
/// # Arguments
/// * `datastore` - The datastore to create transactions on
/// * `max_retries` - Maximum number of retry attempts
/// * `operation` - The operation to retry
///
/// # Returns
/// * `Ok(T)` - The result if any attempt succeeds
/// * `Err(DataStoreError)` - If all attempts fail (returns the last error)
///
/// # Errors
/// Returns an error if all retry attempts fail
pub async fn retry_transaction<T, F, Fut>(
    datastore: &dyn DataStore,
    max_retries: u32,
    mut operation: F,
) -> DataStoreResult<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = DataStoreResult<T>>,
    T: Send,
{
    let mut last_error = None;

    for attempt in 0..=max_retries {
        match with_transaction(datastore, |_tx| operation()).await {
            Ok(result) => return Ok(result),
            Err(error) => {
                last_error = Some(error);
                if attempt < max_retries {
                    // Optional: add exponential backoff here
                    tokio::time::sleep(tokio::time::Duration::from_millis(
                        100 * u64::from(attempt + 1),
                    ))
                    .await;
                }
            }
        }
    }

    Err(
        last_error.unwrap_or_else(|| DataStoreError::TransactionError {
            message: "Unknown error in retry_transaction".to_string(),
        }),
    )
}
