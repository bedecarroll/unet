//! Recording transaction fakes for tests.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::datastore::{DataStoreResult, Transaction};

/// Snapshot of transaction activity for assertions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TransactionSnapshot {
    /// Whether `commit` was called.
    pub committed: bool,
    /// Whether `rollback` was called.
    pub rolled_back: bool,
}

/// Shared tracker used by `RecordingTransaction`.
#[derive(Clone, Debug, Default)]
pub struct TransactionTracker {
    state: Arc<Mutex<TransactionSnapshot>>,
}

impl TransactionTracker {
    /// Returns the current transaction activity snapshot.
    #[must_use]
    pub fn snapshot(&self) -> TransactionSnapshot {
        *self.state()
    }

    fn mark_committed(&self) {
        self.state().committed = true;
    }

    fn mark_rolled_back(&self) {
        self.state().rolled_back = true;
    }

    fn state(&self) -> std::sync::MutexGuard<'_, TransactionSnapshot> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// Transaction fake that records commit and rollback attempts.
pub struct RecordingTransaction {
    tracker: TransactionTracker,
    commit_result: DataStoreResult<()>,
    rollback_result: DataStoreResult<()>,
}

impl RecordingTransaction {
    /// Creates a transaction that succeeds for both commit and rollback.
    #[must_use]
    pub const fn successful(tracker: TransactionTracker) -> Self {
        Self {
            tracker,
            commit_result: Ok(()),
            rollback_result: Ok(()),
        }
    }

    /// Creates a transaction with explicit commit and rollback results.
    #[must_use]
    pub const fn with_results(
        tracker: TransactionTracker,
        commit_result: DataStoreResult<()>,
        rollback_result: DataStoreResult<()>,
    ) -> Self {
        Self {
            tracker,
            commit_result,
            rollback_result,
        }
    }
}

#[async_trait]
impl Transaction for RecordingTransaction {
    async fn commit(self: Box<Self>) -> DataStoreResult<()> {
        self.tracker.mark_committed();
        self.commit_result.clone()
    }

    async fn rollback(self: Box<Self>) -> DataStoreResult<()> {
        self.tracker.mark_rolled_back();
        self.rollback_result.clone()
    }
}
