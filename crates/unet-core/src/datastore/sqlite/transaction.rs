//! Transaction implementation for `SQLite`

use super::super::types::{DataStoreError, DataStoreResult, Transaction};
use async_trait::async_trait;
use sea_orm::DatabaseTransaction;

/// `SeaORM` transaction wrapper
pub struct SqliteTransaction {
    /// The underlying `SeaORM` transaction
    pub txn: DatabaseTransaction,
}

#[async_trait]
impl Transaction for SqliteTransaction {
    async fn commit(self: Box<Self>) -> DataStoreResult<()> {
        self.txn
            .commit()
            .await
            .map_err(|e| DataStoreError::TransactionError {
                message: format!("Failed to commit transaction: {e}"),
            })
    }

    async fn rollback(self: Box<Self>) -> DataStoreResult<()> {
        self.txn
            .rollback()
            .await
            .map_err(|e| DataStoreError::TransactionError {
                message: format!("Failed to rollback transaction: {e}"),
            })
    }
}
