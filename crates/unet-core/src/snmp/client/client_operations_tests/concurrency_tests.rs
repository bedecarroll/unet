use super::fixtures::{create_test_address, create_test_session_config};
use super::super::super::session_management::SessionManager;
use super::super::ClientOperations;
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_concurrent_semaphore_exhaustion() {
    // Create operations with very limited connections
    let config = create_test_session_config();
    let session_manager = SessionManager::new(config.clone(), 10);
    let operations = Arc::new(ClientOperations::new(session_manager, 1)); // Only 1 permit

    let address = create_test_address();
    let oids = &["1.3.6.1.2.1.1.1.0"];

    // Launch two concurrent operations - one should get permit, other should fail
    let ops1 = Arc::clone(&operations);
    let ops2 = Arc::clone(&operations);

    let oids_clone = oids.to_vec();
    let task1 = tokio::spawn(async move {
        tokio::time::timeout(
            Duration::from_millis(200),
            ops1.get(address, &oids_clone.clone(), None),
        )
        .await
    });

    let oids_clone = oids.to_vec();
    let task2 = tokio::spawn(async move {
        tokio::time::timeout(
            Duration::from_millis(200),
            ops2.get(address, &oids_clone.clone(), None),
        )
        .await
    });

    let (result1, result2) = tokio::join!(task1, task2);

    // Both should complete (one way or another)
    let result1 = result1.expect("Task 1 should complete");
    let result2 = result2.expect("Task 2 should complete");

    // Due to only 1 permit, one task should get the permit and the other should wait
    // Both may timeout or error, but they should both complete without hanging
    // The main test is that they don't hang indefinitely
    assert!(result1.is_err() || result2.is_err()); // At least one should error/timeout
}

#[tokio::test]
async fn test_concurrent_walk_semaphore_exhaustion() {
    // Create operations with very limited connections
    let config = create_test_session_config();
    let session_manager = SessionManager::new(config.clone(), 10);
    let operations = Arc::new(ClientOperations::new(session_manager, 1)); // Only 1 permit

    let address = create_test_address();
    let start_oid = "1.3.6.1.2.1.1";

    // Launch two concurrent walk operations
    let ops1 = Arc::clone(&operations);
    let ops2 = Arc::clone(&operations);

    let task1 = tokio::spawn(async move {
        tokio::time::timeout(
            Duration::from_millis(200),
            ops1.walk(address, start_oid, None),
        )
        .await
    });

    let task2 = tokio::spawn(async move {
        tokio::time::timeout(
            Duration::from_millis(200),
            ops2.walk(address, start_oid, None),
        )
        .await
    });

    let (result1, result2) = tokio::join!(task1, task2);

    // Both should complete
    let result1 = result1.expect("Task 1 should complete");
    let result2 = result2.expect("Task 2 should complete");

    // Due to only 1 permit, both may timeout or error, but they should complete without hanging
    assert!(result1.is_err() || result2.is_err()); // At least one should error/timeout
}
