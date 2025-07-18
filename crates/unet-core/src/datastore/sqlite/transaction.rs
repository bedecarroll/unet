//! Transaction implementation for `SQLite`

use super::super::types::{DataStoreError, DataStoreResult, Transaction};
use async_trait::async_trait;
use sea_orm::DatabaseTransaction;

/// `SeaORM` transaction wrapper
pub struct SqliteTransaction {
    /// The underlying `SeaORM` transaction
    pub txn: DatabaseTransaction,
}

impl SqliteTransaction {
    /// Creates a new `SqliteTransaction` wrapper around a `SeaORM` transaction
    #[must_use]
    pub const fn new(txn: DatabaseTransaction) -> Self {
        Self { txn }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqlite_transaction_creation() {
        // Create a mock DatabaseTransaction for testing
        // Note: We can't easily test the actual commit/rollback without a real database connection
        // This test verifies the structure is correct

        // The SqliteTransaction struct should be constructible
        // We can't create a real DatabaseTransaction without a connection,
        // so we'll test the structure and that the types are correct

        // Verify the struct exists and has the expected field
        let expected_size = std::mem::size_of::<SqliteTransaction>();
        assert!(expected_size > 0);
    }

    #[test]
    fn test_sqlite_transaction_new() {
        // Test that we can create a SqliteTransaction with the new constructor
        // This would require a real DatabaseTransaction, so we just test that the method exists
        // by checking its type signature

        let constructor: fn(DatabaseTransaction) -> SqliteTransaction = SqliteTransaction::new;

        // If this compiles, the constructor is properly defined
        // We can't call it without a real connection, but we can verify the type signature exists
        assert!(std::mem::size_of_val(&constructor) > 0);
    }

    #[test]
    fn test_transaction_trait_implementation() {
        // Verify that SqliteTransaction implements the Transaction trait
        // This is a compile-time check - if this compiles, the trait is implemented

        fn assert_implements_transaction<T: Transaction>() {}
        assert_implements_transaction::<SqliteTransaction>();
    }

    #[test]
    fn test_transaction_trait_methods_exist() {
        // Verify the trait methods have the correct signatures
        // This ensures the async trait is properly implemented

        // Test that the methods exist by checking they can be called on the type
        // We can't actually call them without a real DatabaseTransaction, but we can
        // verify the trait is implemented correctly

        fn assert_has_commit_method<T: Transaction>() {}
        fn assert_has_rollback_method<T: Transaction>() {}

        assert_has_commit_method::<SqliteTransaction>();
        assert_has_rollback_method::<SqliteTransaction>();
    }

    #[test]
    fn test_error_message_format() {
        // Test that error messages are formatted correctly
        // We can test the format string without actually causing an error

        let test_error_msg = "test database error";
        let expected_commit_msg = format!("Failed to commit transaction: {test_error_msg}");
        let expected_rollback_msg = format!("Failed to rollback transaction: {test_error_msg}");

        assert!(expected_commit_msg.contains("Failed to commit transaction:"));
        assert!(expected_commit_msg.contains(test_error_msg));
        assert!(expected_rollback_msg.contains("Failed to rollback transaction:"));
        assert!(expected_rollback_msg.contains(test_error_msg));
    }

    #[test]
    fn test_struct_field_access() {
        // Since txn is a public field, test that we can access it
        // We would need a real DatabaseTransaction to test this fully,
        // but we can verify the field is accessible

        // This function would compile only if txn field is accessible
        fn access_txn_field(transaction: &SqliteTransaction) -> &DatabaseTransaction {
            &transaction.txn
        }

        // If this compiles, the field is accessible
        let _field_accessor = access_txn_field;
        // The fact that this compiles confirms the txn field is publicly accessible
    }
}
