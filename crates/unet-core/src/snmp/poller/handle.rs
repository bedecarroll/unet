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
