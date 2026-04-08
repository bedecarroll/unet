//! Small async helpers for `DataStore` tests.

use std::future::Future;
use std::pin::Pin;

use crate::datastore::{DataStoreError, DataStoreResult};

/// Boxed async `DataStore` result used by `MockDataStore` expectations.
pub type BoxedDataStoreFuture<T> = Pin<Box<dyn Future<Output = DataStoreResult<T>> + Send>>;

/// Returns a ready successful `DataStore` future.
pub fn ready_ok<T>(value: T) -> BoxedDataStoreFuture<T>
where
    T: Send + 'static,
{
    Box::pin(async move { Ok(value) })
}

/// Returns a ready failed `DataStore` future.
#[must_use]
pub fn ready_err<T>(error: DataStoreError) -> BoxedDataStoreFuture<T>
where
    T: Send + 'static,
{
    Box::pin(async move { Err(error) })
}

/// Builds a descriptive error for an unexpected `DataStore` call.
#[must_use]
pub fn unexpected_error(store_name: &str, method_name: &str) -> DataStoreError {
    DataStoreError::UnsupportedOperation {
        operation: format!("unexpected DataStore::{method_name} call on {store_name}"),
    }
}

/// Returns a descriptive failure for an unexpected `DataStore` call.
///
/// # Errors
///
/// Always returns `Err` with `DataStoreError::UnsupportedOperation`.
pub fn unexpected_call<T>(store_name: &str, method_name: &str) -> DataStoreResult<T> {
    Err(unexpected_error(store_name, method_name))
}
