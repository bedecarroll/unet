//! Change notification system for Git file tracking
//!
//! This module provides a flexible notification system that can dispatch
//! change events to various handlers (logging, webhooks, database, etc.).

use crate::git::change_tracker::{ChangeNotificationEvent, TrackedFileChange};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Notification delivery method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationMethod {
    /// Log notifications to the system log
    Logging { level: LogLevel },
    /// Send webhook notifications
    Webhook {
        url: String,
        headers: HashMap<String, String>,
        retry_count: usize,
    },
    /// Store notifications in database
    Database {
        table_name: String,
        retention_days: usize,
    },
    /// Send notifications via email
    Email {
        smtp_config: SmtpConfig,
        recipients: Vec<String>,
        subject_template: String,
    },
    /// Custom notification handler
    Custom {
        handler_name: String,
        config: HashMap<String, String>,
    },
}

/// Log level for logging notifications
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "trace"),
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Error => write!(f, "error"),
        }
    }
}

/// SMTP configuration for email notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
}

/// Notification delivery status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeliveryStatus {
    /// Notification was successfully delivered
    Success,
    /// Notification delivery failed
    Failed { error: String, retry_count: usize },
    /// Notification is pending delivery
    Pending,
    /// Notification delivery was skipped
    Skipped { reason: String },
}

/// Notification delivery result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationDelivery {
    /// Unique delivery ID
    pub id: String,
    /// Notification method used
    pub method: NotificationMethod,
    /// Delivery status
    pub status: DeliveryStatus,
    /// Timestamp when delivery was attempted
    pub attempted_at: DateTime<Utc>,
    /// Time taken for delivery (in milliseconds)
    pub duration_ms: Option<u64>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Trait for implementing custom notification handlers
#[async_trait]
pub trait NotificationHandler: Send + Sync {
    /// Handle a change notification event
    async fn handle_notification(&self, event: &ChangeNotificationEvent) -> Result<(), String>;

    /// Get the handler name for identification
    fn handler_name(&self) -> &str;

    /// Check if this handler supports a specific notification method
    fn supports_method(&self, method: &NotificationMethod) -> bool;

    /// Get handler configuration requirements
    fn config_requirements(&self) -> Vec<String> {
        Vec::new()
    }
}

/// Logging notification handler
#[derive(Debug)]
pub struct LoggingHandler {
    default_level: LogLevel,
}

impl LoggingHandler {
    pub fn new(default_level: LogLevel) -> Self {
        Self { default_level }
    }
}

#[async_trait]
impl NotificationHandler for LoggingHandler {
    async fn handle_notification(&self, event: &ChangeNotificationEvent) -> Result<(), String> {
        let message = format_notification_message(event);

        match self.default_level {
            LogLevel::Trace => tracing::trace!("{}", message),
            LogLevel::Debug => debug!("{}", message),
            LogLevel::Info => info!("{}", message),
            LogLevel::Warn => warn!("{}", message),
            LogLevel::Error => error!("{}", message),
        }

        Ok(())
    }

    fn handler_name(&self) -> &str {
        "logging"
    }

    fn supports_method(&self, method: &NotificationMethod) -> bool {
        matches!(method, NotificationMethod::Logging { .. })
    }
}

/// Webhook notification handler
#[derive(Debug)]
pub struct WebhookHandler {
    client: reqwest::Client,
}

impl WebhookHandler {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl NotificationHandler for WebhookHandler {
    async fn handle_notification(&self, event: &ChangeNotificationEvent) -> Result<(), String> {
        // This is a placeholder - actual webhook implementation would need the URL and config
        debug!("Webhook notification: {:?}", event);
        Ok(())
    }

    fn handler_name(&self) -> &str {
        "webhook"
    }

    fn supports_method(&self, method: &NotificationMethod) -> bool {
        matches!(method, NotificationMethod::Webhook { .. })
    }

    fn config_requirements(&self) -> Vec<String> {
        vec!["url".to_string()]
    }
}

/// Notification filter for controlling which events get delivered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationFilter {
    /// File types to include (empty means all)
    pub include_file_types: Vec<String>,
    /// File types to exclude
    pub exclude_file_types: Vec<String>,
    /// Repository URLs to include (empty means all)
    pub include_repositories: Vec<String>,
    /// Repository URLs to exclude
    pub exclude_repositories: Vec<String>,
    /// Event types to include (empty means all)
    pub include_event_types: Vec<String>,
    /// Event types to exclude
    pub exclude_event_types: Vec<String>,
    /// Minimum time between notifications for the same file (in seconds)
    pub debounce_seconds: Option<u64>,
}

impl Default for NotificationFilter {
    fn default() -> Self {
        Self {
            include_file_types: Vec::new(),
            exclude_file_types: Vec::new(),
            include_repositories: Vec::new(),
            exclude_repositories: Vec::new(),
            include_event_types: Vec::new(),
            exclude_event_types: Vec::new(),
            debounce_seconds: Some(60), // 1 minute default debounce
        }
    }
}

impl NotificationFilter {
    /// Check if an event should be delivered based on the filter criteria
    pub fn should_deliver(&self, event: &ChangeNotificationEvent) -> bool {
        // Check repository filters
        let repository_url = match event {
            ChangeNotificationEvent::PolicyFileChanged { repository_url, .. }
            | ChangeNotificationEvent::TemplateFileChanged { repository_url, .. }
            | ChangeNotificationEvent::ConfigFileChanged { repository_url, .. }
            | ChangeNotificationEvent::IntegrityValidationFailed { repository_url, .. }
            | ChangeNotificationEvent::BatchChangesDetected { repository_url, .. } => {
                repository_url
            }
        };

        if !self.include_repositories.is_empty()
            && !self.include_repositories.contains(repository_url)
        {
            return false;
        }

        if self.exclude_repositories.contains(repository_url) {
            return false;
        }

        // Check event type filters
        let event_type = match event {
            ChangeNotificationEvent::PolicyFileChanged { .. } => "policy_file_changed",
            ChangeNotificationEvent::TemplateFileChanged { .. } => "template_file_changed",
            ChangeNotificationEvent::ConfigFileChanged { .. } => "config_file_changed",
            ChangeNotificationEvent::IntegrityValidationFailed { .. } => {
                "integrity_validation_failed"
            }
            ChangeNotificationEvent::BatchChangesDetected { .. } => "batch_changes_detected",
        };

        if !self.include_event_types.is_empty()
            && !self.include_event_types.contains(&event_type.to_string())
        {
            return false;
        }

        if self.exclude_event_types.contains(&event_type.to_string()) {
            return false;
        }

        true
    }
}

/// Notification delivery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Notification methods to use
    pub methods: Vec<NotificationMethod>,
    /// Filters to apply
    pub filters: Vec<NotificationFilter>,
    /// Maximum number of delivery retries
    pub max_retries: usize,
    /// Retry delay in seconds
    pub retry_delay_seconds: u64,
    /// Whether to enable async delivery
    pub async_delivery: bool,
    /// Maximum number of concurrent deliveries
    pub max_concurrent_deliveries: usize,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            methods: vec![NotificationMethod::Logging {
                level: LogLevel::Info,
            }],
            filters: vec![NotificationFilter::default()],
            max_retries: 3,
            retry_delay_seconds: 60,
            async_delivery: true,
            max_concurrent_deliveries: 10,
        }
    }
}

/// Change notification system
pub struct ChangeNotificationSystem {
    /// Configuration
    config: NotificationConfig,
    /// Registered notification handlers
    handlers: Arc<RwLock<HashMap<String, Arc<dyn NotificationHandler>>>>,
    /// Recent deliveries for tracking and deduplication
    recent_deliveries: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
    /// Delivery history
    delivery_history: Arc<RwLock<Vec<NotificationDelivery>>>,
    /// Maximum delivery history to keep
    max_history: usize,
}

impl ChangeNotificationSystem {
    /// Create a new change notification system
    pub fn new() -> Self {
        Self::with_config(NotificationConfig::default())
    }

    /// Create a new change notification system with custom configuration
    pub fn with_config(config: NotificationConfig) -> Self {
        let system = Self {
            config,
            handlers: Arc::new(RwLock::new(HashMap::new())),
            recent_deliveries: Arc::new(RwLock::new(HashMap::new())),
            delivery_history: Arc::new(RwLock::new(Vec::new())),
            max_history: 1000,
        };

        // Note: Default handlers should be registered after creation using register_handler()
        system
    }

    /// Register a notification handler
    pub async fn register_handler(&self, handler: Arc<dyn NotificationHandler>) {
        let mut handlers = self.handlers.write().await;
        let name = handler.handler_name().to_string();
        handlers.insert(name.clone(), handler);
        info!("Registered notification handler: {}", name);
    }

    /// Unregister a notification handler
    pub async fn unregister_handler(&self, handler_name: &str) -> bool {
        let mut handlers = self.handlers.write().await;
        let removed = handlers.remove(handler_name).is_some();
        if removed {
            info!("Unregistered notification handler: {}", handler_name);
        }
        removed
    }

    /// Send a notification for a change event
    pub async fn notify(&self, event: &ChangeNotificationEvent) -> Vec<NotificationDelivery> {
        let mut deliveries = Vec::new();

        // Apply filters
        let should_deliver = self
            .config
            .filters
            .iter()
            .any(|filter| filter.should_deliver(event));

        if !should_deliver {
            debug!("Notification filtered out: {:?}", event);
            return deliveries;
        }

        // Check debouncing
        if let Some(debounce_key) = self.get_debounce_key(event) {
            let recent_deliveries = self.recent_deliveries.read().await;
            if let Some(last_delivery) = recent_deliveries.get(&debounce_key) {
                let elapsed = Utc::now().signed_duration_since(*last_delivery);
                if let Some(debounce_seconds) =
                    self.config.filters.first().and_then(|f| f.debounce_seconds)
                {
                    if elapsed.num_seconds() < debounce_seconds as i64 {
                        debug!("Notification debounced: {:?}", event);
                        return deliveries;
                    }
                }
            }
        }

        let handlers = self.handlers.read().await;

        // Process each notification method
        for method in &self.config.methods {
            // Find compatible handler
            if let Some(handler) = handlers.values().find(|h| h.supports_method(method)) {
                let delivery_id = uuid::Uuid::new_v4().to_string();
                let start_time = Utc::now();

                let status = match handler.handle_notification(event).await {
                    Ok(()) => DeliveryStatus::Success,
                    Err(error) => DeliveryStatus::Failed {
                        error,
                        retry_count: 0,
                    },
                };

                let duration_ms = Utc::now()
                    .signed_duration_since(start_time)
                    .num_milliseconds() as u64;

                let delivery = NotificationDelivery {
                    id: delivery_id,
                    method: method.clone(),
                    status,
                    attempted_at: start_time,
                    duration_ms: Some(duration_ms),
                    metadata: HashMap::new(),
                };

                deliveries.push(delivery);
            } else {
                warn!("No handler found for notification method: {:?}", method);

                deliveries.push(NotificationDelivery {
                    id: uuid::Uuid::new_v4().to_string(),
                    method: method.clone(),
                    status: DeliveryStatus::Skipped {
                        reason: "No compatible handler found".to_string(),
                    },
                    attempted_at: Utc::now(),
                    duration_ms: None,
                    metadata: HashMap::new(),
                });
            }
        }

        // Update recent deliveries for debouncing
        if let Some(debounce_key) = self.get_debounce_key(event) {
            let mut recent_deliveries = self.recent_deliveries.write().await;
            recent_deliveries.insert(debounce_key, Utc::now());
        }

        // Store delivery history
        self.store_delivery_history(deliveries.clone()).await;

        deliveries
    }

    /// Get recent delivery history
    pub async fn get_delivery_history(&self, limit: Option<usize>) -> Vec<NotificationDelivery> {
        let history = self.delivery_history.read().await;
        let limit = limit.unwrap_or(self.max_history);

        if history.len() <= limit {
            history.clone()
        } else {
            history[history.len() - limit..].to_vec()
        }
    }

    /// Clear delivery history
    pub async fn clear_delivery_history(&self) {
        let mut history = self.delivery_history.write().await;
        let mut recent_deliveries = self.recent_deliveries.write().await;

        history.clear();
        recent_deliveries.clear();

        info!("Cleared notification delivery history");
    }

    /// Get registered handlers
    pub async fn get_handlers(&self) -> Vec<String> {
        let handlers = self.handlers.read().await;
        handlers.keys().cloned().collect()
    }

    // Private helper methods

    fn get_debounce_key(&self, event: &ChangeNotificationEvent) -> Option<String> {
        match event {
            ChangeNotificationEvent::PolicyFileChanged {
                repository_url,
                file_path,
                ..
            }
            | ChangeNotificationEvent::TemplateFileChanged {
                repository_url,
                file_path,
                ..
            }
            | ChangeNotificationEvent::ConfigFileChanged {
                repository_url,
                file_path,
                ..
            } => Some(format!("{}:{}", repository_url, file_path.display())),
            _ => None,
        }
    }

    async fn store_delivery_history(&self, deliveries: Vec<NotificationDelivery>) {
        let mut history = self.delivery_history.write().await;

        for delivery in deliveries {
            history.push(delivery);
        }

        // Trim history if it exceeds maximum
        if history.len() > self.max_history {
            let history_len = history.len();
            history.drain(0..history_len - self.max_history);
        }
    }
}

impl Default for ChangeNotificationSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for ChangeNotificationSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChangeNotificationSystem")
            .field("config", &self.config)
            .field("max_history", &self.max_history)
            .field("handlers", &"<registered handlers>")
            .finish()
    }
}

// Helper functions

fn format_notification_message(event: &ChangeNotificationEvent) -> String {
    match event {
        ChangeNotificationEvent::PolicyFileChanged {
            repository_url,
            file_path,
            change_type,
            timestamp,
            ..
        } => {
            format!(
                "Policy file changed: {} in {} ({:?}) at {}",
                file_path.display(),
                repository_url,
                change_type,
                timestamp.format("%Y-%m-%d %H:%M:%S UTC")
            )
        }
        ChangeNotificationEvent::TemplateFileChanged {
            repository_url,
            file_path,
            change_type,
            timestamp,
            ..
        } => {
            format!(
                "Template file changed: {} in {} ({:?}) at {}",
                file_path.display(),
                repository_url,
                change_type,
                timestamp.format("%Y-%m-%d %H:%M:%S UTC")
            )
        }
        ChangeNotificationEvent::ConfigFileChanged {
            repository_url,
            file_path,
            change_type,
            timestamp,
            ..
        } => {
            format!(
                "Config file changed: {} in {} ({:?}) at {}",
                file_path.display(),
                repository_url,
                change_type,
                timestamp.format("%Y-%m-%d %H:%M:%S UTC")
            )
        }
        ChangeNotificationEvent::IntegrityValidationFailed {
            repository_url,
            file_path,
            timestamp,
            ..
        } => {
            format!(
                "File integrity validation failed: {} in {} at {}",
                file_path.display(),
                repository_url,
                timestamp.format("%Y-%m-%d %H:%M:%S UTC")
            )
        }
        ChangeNotificationEvent::BatchChangesDetected {
            repository_url,
            change_count,
            timestamp,
            ..
        } => {
            format!(
                "Batch changes detected: {} changes in {} at {}",
                change_count,
                repository_url,
                timestamp.format("%Y-%m-%d %H:%M:%S UTC")
            )
        }
    }
}
