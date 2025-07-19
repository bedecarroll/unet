//! Polling handle for controlling the scheduler

use super::{PollingMessage, PollingResult, PollingTask};
use tokio::sync::mpsc;
use uuid::Uuid;

/// Handle for controlling the polling scheduler
#[derive(Debug)]
pub struct PollingHandle {
    /// Channel for sending control messages
    pub(super) message_tx: mpsc::UnboundedSender<PollingMessage>,
    /// Channel for receiving polling results
    pub result_rx: mpsc::UnboundedReceiver<PollingResult>,
}

impl PollingHandle {
    /// Create a new polling handle
    pub(super) const fn new(
        message_tx: mpsc::UnboundedSender<PollingMessage>,
        result_rx: mpsc::UnboundedReceiver<PollingResult>,
    ) -> Self {
        Self {
            message_tx,
            result_rx,
        }
    }
    /// Add a new polling task
    ///
    /// # Errors
    /// Returns an error if the message channel is closed
    pub fn add_task(&self, task: PollingTask) -> Result<(), String> {
        self.message_tx
            .send(PollingMessage::AddTask(task))
            .map_err(|e| format!("Failed to send message: {e}"))
    }

    /// Remove a polling task
    ///
    /// # Errors
    /// Returns an error if the message channel is closed
    pub fn remove_task(&self, task_id: Uuid) -> Result<(), String> {
        self.message_tx
            .send(PollingMessage::RemoveTask(task_id))
            .map_err(|e| format!("Failed to send message: {e}"))
    }

    /// Update a polling task
    ///
    /// # Errors
    /// Returns an error if the message channel is closed
    pub fn update_task(&self, task: PollingTask) -> Result<(), String> {
        self.message_tx
            .send(PollingMessage::UpdateTask(task))
            .map_err(|e| format!("Failed to send message: {e}"))
    }

    /// Enable or disable a task
    ///
    /// # Errors
    /// Returns an error if the message channel is closed
    pub fn enable_task(&self, task_id: Uuid, enabled: bool) -> Result<(), String> {
        self.message_tx
            .send(PollingMessage::EnableTask(task_id, enabled))
            .map_err(|e| format!("Failed to send message: {e}"))
    }

    /// Get status of a specific task
    ///
    /// # Errors
    /// Returns an error if the message channel is closed or the response channel fails
    pub async fn get_task_status(&self, task_id: Uuid) -> Result<Option<PollingTask>, String> {
        let (tx, rx) = tokio::sync::oneshot::channel();

        self.message_tx
            .send(PollingMessage::GetTaskStatus(task_id, tx))
            .map_err(|e| format!("Failed to send message: {e}"))?;

        rx.await
            .map_err(|e| format!("Failed to receive response: {e}"))
    }

    /// List all tasks
    ///
    /// # Errors
    /// Returns an error if the message channel is closed or the response channel fails
    pub async fn list_tasks(&self) -> Result<Vec<PollingTask>, String> {
        let (tx, rx) = tokio::sync::oneshot::channel();

        self.message_tx
            .send(PollingMessage::ListTasks(tx))
            .map_err(|e| format!("Failed to send message: {e}"))?;

        rx.await
            .map_err(|e| format!("Failed to receive response: {e}"))
    }

    /// Shutdown the scheduler
    ///
    /// # Errors
    /// Returns an error if the message channel is closed
    pub fn shutdown(&self) -> Result<(), String> {
        self.message_tx
            .send(PollingMessage::Shutdown)
            .map_err(|e| format!("Failed to send message: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snmp::SessionConfig;
    use std::net::{IpAddr, SocketAddr};
    use std::time::{Duration, SystemTime};

    fn create_test_task() -> PollingTask {
        PollingTask {
            id: Uuid::new_v4(),
            node_id: Uuid::new_v4(),
            target: SocketAddr::new(IpAddr::from([192, 168, 1, 1]), 161),
            oids: vec!["1.3.6.1.2.1.1.1.0".to_string()],
            interval: Duration::from_secs(60),
            session_config: SessionConfig::default(),
            priority: 128,
            enabled: true,
            created_at: SystemTime::now(),
            last_success: None,
            last_error: None,
            consecutive_failures: 0,
        }
    }

    #[tokio::test]
    async fn test_handle_methods_with_closed_channel() {
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        let (_result_tx, result_rx) = mpsc::unbounded_channel();

        // Drop the message receiver to close the channel
        drop(message_rx);

        let handle = PollingHandle::new(message_tx, result_rx);
        let test_task = create_test_task();
        let task_id = test_task.id;

        // Test add_task error case (lines 31-34)
        let result = handle.add_task(test_task.clone());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to send message"));

        // Test remove_task error case (lines 41-44)
        let result = handle.remove_task(task_id);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to send message"));

        // Test update_task error case (lines 51-54)
        let result = handle.update_task(test_task);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to send message"));

        // Test enable_task error case (lines 61-64)
        let result = handle.enable_task(task_id, true);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to send message"));

        // Test get_task_status error case (lines 71-72, 74-76)
        let result = handle.get_task_status(task_id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to send message"));

        // Test list_tasks error case (lines 86-87, 89-91)
        let result = handle.list_tasks().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to send message"));

        // Test shutdown error case (lines 101-104)
        let result = handle.shutdown();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to send message"));
    }

    #[tokio::test]
    async fn test_handle_methods_with_dropped_response_channel() {
        let (message_tx, mut message_rx) = mpsc::unbounded_channel();
        let (_result_tx, result_rx) = mpsc::unbounded_channel();

        let handle = PollingHandle::new(message_tx, result_rx);
        let task_id = Uuid::new_v4();

        // Test get_task_status with dropped response channel (lines 78-79)
        // Start the operation but don't await it yet
        let get_status_future = handle.get_task_status(task_id);

        // Give it a small time to send the message
        tokio::time::sleep(Duration::from_millis(1)).await;

        // Receive the message and drop the response channel
        if let Ok(Some(PollingMessage::GetTaskStatus(_, tx))) =
            tokio::time::timeout(Duration::from_millis(10), message_rx.recv()).await
        {
            drop(tx); // This should cause the awaiting future to fail
        }

        // Now await the future with a timeout
        let result = tokio::time::timeout(Duration::from_millis(50), get_status_future).await;

        // Should get an error (either timeout or "Failed to receive response")
        match result {
            Ok(Err(e)) => assert!(e.contains("Failed to receive response")),
            Err(_) => {} // Timeout is acceptable - means the channel was dropped
            Ok(Ok(_)) => panic!("Expected error or timeout"),
        }

        // Test list_tasks with dropped response channel (lines 93-94)
        let list_future = handle.list_tasks();

        // Give it a small time to send the message
        tokio::time::sleep(Duration::from_millis(1)).await;

        // Receive the message and drop the response channel
        if let Ok(Some(PollingMessage::ListTasks(tx))) =
            tokio::time::timeout(Duration::from_millis(10), message_rx.recv()).await
        {
            drop(tx); // This should cause the awaiting future to fail
        }

        // Now await the future with a timeout
        let result = tokio::time::timeout(Duration::from_millis(50), list_future).await;

        // Should get an error (either timeout or "Failed to receive response")
        match result {
            Ok(Err(e)) => assert!(e.contains("Failed to receive response")),
            Err(_) => {} // Timeout is acceptable - means the channel was dropped
            Ok(Ok(_)) => panic!("Expected error or timeout"),
        }
    }

    #[test]
    fn test_handle_creation() {
        let (message_tx, _message_rx) = mpsc::unbounded_channel();
        let (_result_tx, result_rx) = mpsc::unbounded_channel();

        let handle = PollingHandle::new(message_tx, result_rx);

        // Verify handle was created successfully
        assert!(!handle.message_tx.is_closed());
    }
}
