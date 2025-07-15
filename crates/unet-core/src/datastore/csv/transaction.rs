//! CSV transaction implementation
//!
//! Contains the `CsvTransaction` structure for managing transactional
//! operations in the CSV-based datastore.

use super::super::types::{DataStoreError, DataStoreResult, Transaction};
use super::store::{CsvData, CsvStore};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Simple transaction implementation for CSV store
pub struct CsvTransaction {
    /// Reference to the store
    pub(crate) store: Arc<CsvStore>,
    /// Transaction changes
    pub(crate) changes: Mutex<CsvData>,
    /// Whether transaction has been committed or rolled back
    pub(crate) committed: Mutex<bool>,
}

#[async_trait]
impl Transaction for CsvTransaction {
    async fn commit(self: Box<Self>) -> DataStoreResult<()> {
        {
            let mut committed = self.committed.lock().await;
            if *committed {
                return Err(DataStoreError::TransactionError {
                    message: "Transaction already committed or rolled back".to_string(),
                });
            }

            // Apply changes to the store
            let changes = self.changes.lock().await;
            let mut store_data = self.store.data.lock().await;

            // Merge changes (simplified - in real implementation you'd need proper conflict resolution)
            for (id, node) in &changes.nodes {
                store_data.nodes.insert(*id, node.clone());
            }
            for (id, link) in &changes.links {
                store_data.links.insert(*id, link.clone());
            }
            for (id, location) in &changes.locations {
                store_data.locations.insert(*id, location.clone());
            }

            drop(store_data);
            drop(changes);
            self.store.save_data().await?;
            *committed = true;
        }
        Ok(())
    }

    async fn rollback(self: Box<Self>) -> DataStoreResult<()> {
        {
            let mut committed = self.committed.lock().await;
            if *committed {
                return Err(DataStoreError::TransactionError {
                    message: "Transaction already committed or rolled back".to_string(),
                });
            }
            *committed = true;
        }
        Ok(())
    }
}
