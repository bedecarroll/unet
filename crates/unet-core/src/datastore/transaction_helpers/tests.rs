//! Tests for transaction helper functions

#[cfg(test)]
mod transaction_helper_tests {
    use super::super::super::testing::{
        RecordingTransaction, TransactionTracker, ready_err, ready_ok,
    };
    use super::super::super::types::{DataStoreError, Transaction};
    use super::super::super::{MockDataStore, transaction_helpers::*};

    fn mock_datastore(should_fail: bool) -> MockDataStore {
        let mut datastore = MockDataStore::new();

        if should_fail {
            datastore.expect_begin_transaction().returning(|| {
                ready_err(DataStoreError::ConnectionError {
                    message: "Failed to begin transaction".to_string(),
                })
            });
        } else {
            datastore.expect_begin_transaction().returning(|| {
                ready_ok(Box::new(RecordingTransaction::successful(
                    TransactionTracker::default(),
                )) as Box<dyn Transaction>)
            });
        }

        datastore
    }

    #[tokio::test]
    async fn test_with_transaction_success() {
        let datastore = mock_datastore(false);

        let result = with_transaction(
            &datastore,
            |_tx| async move { Ok::<i32, DataStoreError>(42) },
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_with_transaction_operation_failure() {
        let datastore = mock_datastore(false);

        let result = with_transaction(&datastore, |_tx| async move {
            Err::<i32, DataStoreError>(DataStoreError::ValidationError {
                message: "test error".to_string(),
            })
        })
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_with_transaction_begin_failure() {
        let datastore = mock_datastore(true);

        let result = with_transaction(
            &datastore,
            |_tx| async move { Ok::<i32, DataStoreError>(42) },
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_with_transaction_control_success() {
        let datastore = mock_datastore(false);

        let result = with_transaction_control(&datastore, |_tx| async move {
            Ok::<(i32, bool), DataStoreError>((42, true))
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_with_transaction_control_failure() {
        let datastore = mock_datastore(false);

        let result = with_transaction_control(&datastore, |_tx| async move {
            Err::<(i32, bool), DataStoreError>(DataStoreError::ValidationError {
                message: "test error".to_string(),
            })
        })
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_batch_with_transaction_success() {
        let datastore = mock_datastore(false);

        let empty_operations: Vec<fn() -> std::future::Ready<Result<i32, DataStoreError>>> = vec![];
        let result = batch_with_transaction(&datastore, empty_operations).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_batch_with_transaction_failure() {
        let datastore = mock_datastore(true);

        let empty_operations: Vec<fn() -> std::future::Ready<Result<i32, DataStoreError>>> = vec![];
        let result = batch_with_transaction(&datastore, empty_operations).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_batch_with_transaction_empty() {
        let datastore = mock_datastore(false);
        let operations: Vec<fn() -> std::future::Ready<Result<i32, DataStoreError>>> = vec![];

        let result = batch_with_transaction(&datastore, operations).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_retry_transaction_success_first_try() {
        let datastore = mock_datastore(false);

        let result =
            retry_transaction(&datastore, 3, || async { Ok::<i32, DataStoreError>(42) }).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_transaction_all_failures() {
        let datastore = mock_datastore(false);

        let result = retry_transaction(&datastore, 2, || async {
            Err::<i32, DataStoreError>(DataStoreError::ValidationError {
                message: "persistent error".to_string(),
            })
        })
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_retry_transaction_success_after_failures() {
        let datastore = mock_datastore(false);
        let attempt_count = std::sync::Arc::new(std::sync::Mutex::new(0));

        let attempt_counter = attempt_count.clone();
        let result = retry_transaction(&datastore, 3, move || {
            let counter = attempt_counter.clone();
            async move {
                let mut count = counter.lock().expect("lock retry attempt count");
                *count += 1;
                if *count < 3 {
                    Err::<i32, DataStoreError>(DataStoreError::ValidationError {
                        message: "temporary error".to_string(),
                    })
                } else {
                    Ok(42)
                }
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(*attempt_count.lock().expect("lock final retry count"), 3);
    }
}
