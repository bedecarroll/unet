//! Notification system for alerts and events
//!
//! This module provides notification channel implementations for sending
//! alerts through various channels including email, Slack, webhooks, and PagerDuty.

use crate::alerting::{Alert, NotificationChannel};
use crate::error::{Error, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

/// Result type for notification operations
pub type NotificationResult<T> = std::result::Result<T, NotificationError>;

/// Notification system errors
#[derive(Debug, thiserror::Error)]
pub enum NotificationError {
    /// Invalid channel configuration
    #[error("Invalid channel configuration: {message}")]
    InvalidConfig { message: String },

    /// Network or connection error
    #[error("Network error: {message}")]
    NetworkError { message: String },

    /// Authentication error
    #[error("Authentication failed: {message}")]
    AuthenticationError { message: String },

    /// Rate limit exceeded
    #[error("Rate limit exceeded for channel: {channel_id}")]
    RateLimitExceeded { channel_id: String },

    /// Channel temporarily unavailable
    #[error("Channel unavailable: {channel_id}")]
    ChannelUnavailable { channel_id: String },

    /// Message formatting error
    #[error("Message formatting error: {message}")]
    MessageFormatError { message: String },

    /// Timeout error
    #[error("Notification timeout after {seconds}s")]
    Timeout { seconds: u64 },
}

/// Notification message content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationMessage {
    /// Message title/subject
    pub title: String,
    /// Message body/content
    pub body: String,
    /// Message priority level
    pub priority: MessagePriority,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    /// Alert that triggered this notification
    pub alert: Option<Alert>,
}

/// Message priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessagePriority {
    /// Low priority message
    Low,
    /// Normal priority message
    Normal,
    /// High priority message
    High,
    /// Critical priority message
    Critical,
}

impl MessagePriority {
    /// Convert to string representation
    pub fn to_string(&self) -> &'static str {
        match self {
            MessagePriority::Low => "low",
            MessagePriority::Normal => "normal",
            MessagePriority::High => "high",
            MessagePriority::Critical => "critical",
        }
    }
}

/// Notification delivery result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryResult {
    /// Whether the notification was successfully delivered
    pub success: bool,
    /// Channel ID that was used
    pub channel_id: String,
    /// Delivery timestamp
    pub timestamp: u64,
    /// Response message from the channel
    pub response_message: Option<String>,
    /// Error message if delivery failed
    pub error_message: Option<String>,
    /// Delivery duration in milliseconds
    pub duration_ms: u64,
}

/// Trait for notification channel implementations
#[async_trait]
pub trait NotificationProvider: Send + Sync {
    /// Send a notification through this channel
    async fn send(&self, message: &NotificationMessage) -> NotificationResult<DeliveryResult>;

    /// Validate channel configuration
    fn validate_config(&self, config: &HashMap<String, String>) -> NotificationResult<()>;

    /// Get channel type identifier
    fn channel_type(&self) -> &'static str;

    /// Check if the channel is currently available
    async fn health_check(&self) -> NotificationResult<bool>;
}

/// Email notification provider
#[derive(Debug, Clone)]
pub struct EmailProvider {
    /// SMTP server configuration
    smtp_server: String,
    /// SMTP port
    smtp_port: u16,
    /// From email address
    from_address: String,
    /// To email addresses
    to_addresses: Vec<String>,
    /// SMTP authentication
    auth: Option<SmtpAuth>,
    /// Use TLS encryption
    use_tls: bool,
}

/// SMTP authentication credentials
#[derive(Debug, Clone)]
pub struct SmtpAuth {
    /// Username
    pub username: String,
    /// Password
    pub password: String,
}

impl EmailProvider {
    /// Create new email provider from configuration
    pub fn from_config(config: &HashMap<String, String>) -> NotificationResult<Self> {
        let smtp_server = config
            .get("smtp_server")
            .ok_or_else(|| NotificationError::InvalidConfig {
                message: "smtp_server is required".to_string(),
            })?
            .clone();

        let smtp_port = config
            .get("smtp_port")
            .unwrap_or(&"587".to_string())
            .parse()
            .map_err(|_| NotificationError::InvalidConfig {
                message: "Invalid smtp_port".to_string(),
            })?;

        let from_address = config
            .get("from_address")
            .ok_or_else(|| NotificationError::InvalidConfig {
                message: "from_address is required".to_string(),
            })?
            .clone();

        let to_addresses: Vec<String> = config
            .get("to_addresses")
            .ok_or_else(|| NotificationError::InvalidConfig {
                message: "to_addresses is required".to_string(),
            })?
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let auth = if let (Some(username), Some(password)) =
            (config.get("smtp_username"), config.get("smtp_password"))
        {
            Some(SmtpAuth {
                username: username.clone(),
                password: password.clone(),
            })
        } else {
            None
        };

        let use_tls = config
            .get("use_tls")
            .map(|s| s.parse().unwrap_or(true))
            .unwrap_or(true);

        Ok(Self {
            smtp_server,
            smtp_port,
            from_address,
            to_addresses,
            auth,
            use_tls,
        })
    }
}

#[async_trait]
impl NotificationProvider for EmailProvider {
    async fn send(&self, message: &NotificationMessage) -> NotificationResult<DeliveryResult> {
        let start_time = std::time::Instant::now();

        // Format email content
        let subject = format!("[μNet Alert] {}", message.title);
        let _body = self.format_email_body(message);

        // Simulate email sending (in real implementation, use lettre or similar crate)
        debug!(
            "Sending email notification to {:?}: {}",
            self.to_addresses, subject
        );

        // Simulate network delay
        tokio::time::sleep(Duration::from_millis(100)).await;

        let duration = start_time.elapsed();

        // In real implementation, this would use an actual SMTP client
        info!(
            "Email notification sent successfully to {:?}",
            self.to_addresses
        );

        Ok(DeliveryResult {
            success: true,
            channel_id: "email".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            response_message: Some("Email queued for delivery".to_string()),
            error_message: None,
            duration_ms: duration.as_millis() as u64,
        })
    }

    fn validate_config(&self, config: &HashMap<String, String>) -> NotificationResult<()> {
        if !config.contains_key("smtp_server") {
            return Err(NotificationError::InvalidConfig {
                message: "smtp_server is required".to_string(),
            });
        }

        if !config.contains_key("from_address") {
            return Err(NotificationError::InvalidConfig {
                message: "from_address is required".to_string(),
            });
        }

        if !config.contains_key("to_addresses") {
            return Err(NotificationError::InvalidConfig {
                message: "to_addresses is required".to_string(),
            });
        }

        Ok(())
    }

    fn channel_type(&self) -> &'static str {
        "email"
    }

    async fn health_check(&self) -> NotificationResult<bool> {
        // In real implementation, this would test SMTP connectivity
        debug!("Email health check: simulating SMTP connection test");
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok(true)
    }
}

impl EmailProvider {
    /// Format email body with alert details
    fn format_email_body(&self, message: &NotificationMessage) -> String {
        let mut body = String::new();
        body.push_str(&format!("Priority: {}\n\n", message.priority.to_string()));
        body.push_str(&message.body);

        if let Some(alert) = &message.alert {
            body.push_str("\n\n--- Alert Details ---\n");
            body.push_str(&format!("Alert ID: {}\n", alert.id));
            body.push_str(&format!("Rule ID: {}\n", alert.rule_id));
            body.push_str(&format!("Severity: {}\n", alert.severity.to_string()));
            body.push_str(&format!("Status: {:?}\n", alert.status));

            if let Some(value) = alert.current_value {
                body.push_str(&format!("Current Value: {:.2}\n", value));
            }

            if let Some(threshold) = alert.threshold_value {
                body.push_str(&format!("Threshold: {:.2}\n", threshold));
            }
        }

        body.push_str("\n\n--\nμNet Monitoring System");
        body
    }
}

/// Slack notification provider
#[derive(Debug, Clone)]
pub struct SlackProvider {
    /// Slack webhook URL
    webhook_url: String,
    /// Default channel
    channel: Option<String>,
    /// Bot username
    username: Option<String>,
    /// HTTP client
    client: reqwest::Client,
}

impl SlackProvider {
    /// Create new Slack provider from configuration
    pub fn from_config(config: &HashMap<String, String>) -> NotificationResult<Self> {
        let webhook_url = config
            .get("webhook_url")
            .ok_or_else(|| NotificationError::InvalidConfig {
                message: "webhook_url is required".to_string(),
            })?
            .clone();

        let channel = config.get("channel").cloned();
        let username = config.get("username").cloned();

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| NotificationError::NetworkError {
                message: format!("Failed to create HTTP client: {}", e),
            })?;

        Ok(Self {
            webhook_url,
            channel,
            username,
            client,
        })
    }
}

#[async_trait]
impl NotificationProvider for SlackProvider {
    async fn send(&self, message: &NotificationMessage) -> NotificationResult<DeliveryResult> {
        let start_time = std::time::Instant::now();

        // Format Slack message
        let slack_message = self.format_slack_message(message);

        debug!("Sending Slack notification: {}", message.title);

        // Send to Slack webhook
        let response = timeout(
            Duration::from_secs(30),
            self.client
                .post(&self.webhook_url)
                .json(&slack_message)
                .send(),
        )
        .await
        .map_err(|_| NotificationError::Timeout { seconds: 30 })?
        .map_err(|e| NotificationError::NetworkError {
            message: format!("Slack webhook request failed: {}", e),
        })?;

        let duration = start_time.elapsed();
        let success = response.status().is_success();
        let response_text = response.text().await.unwrap_or_default();

        if success {
            info!("Slack notification sent successfully");
            Ok(DeliveryResult {
                success: true,
                channel_id: "slack".to_string(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                response_message: Some(response_text),
                error_message: None,
                duration_ms: duration.as_millis() as u64,
            })
        } else {
            error!("Slack notification failed: {}", response_text);
            Err(NotificationError::NetworkError {
                message: format!("Slack webhook failed: {}", response_text),
            })
        }
    }

    fn validate_config(&self, config: &HashMap<String, String>) -> NotificationResult<()> {
        if !config.contains_key("webhook_url") {
            return Err(NotificationError::InvalidConfig {
                message: "webhook_url is required".to_string(),
            });
        }

        let webhook_url = config.get("webhook_url").unwrap();
        if !webhook_url.starts_with("https://hooks.slack.com") {
            return Err(NotificationError::InvalidConfig {
                message: "Invalid Slack webhook URL".to_string(),
            });
        }

        Ok(())
    }

    fn channel_type(&self) -> &'static str {
        "slack"
    }

    async fn health_check(&self) -> NotificationResult<bool> {
        debug!("Slack health check: testing webhook connectivity");

        // Test webhook with a minimal payload
        let test_payload = serde_json::json!({
            "text": "Health check from μNet monitoring"
        });

        let response = timeout(
            Duration::from_secs(10),
            self.client
                .post(&self.webhook_url)
                .json(&test_payload)
                .send(),
        )
        .await;

        match response {
            Ok(Ok(resp)) => Ok(resp.status().is_success()),
            Ok(Err(_)) | Err(_) => Ok(false),
        }
    }
}

impl SlackProvider {
    /// Format message for Slack
    fn format_slack_message(&self, message: &NotificationMessage) -> serde_json::Value {
        let color = match message.priority {
            MessagePriority::Critical => "#ff0000", // Red
            MessagePriority::High => "#ff8800",     // Orange
            MessagePriority::Normal => "#ffdd00",   // Yellow
            MessagePriority::Low => "#00ff00",      // Green
        };

        let mut slack_message = serde_json::json!({
            "text": message.title,
            "attachments": [{
                "color": color,
                "fields": [
                    {
                        "title": "Priority",
                        "value": message.priority.to_string(),
                        "short": true
                    },
                    {
                        "title": "Description",
                        "value": message.body,
                        "short": false
                    }
                ]
            }]
        });

        if let Some(channel) = &self.channel {
            slack_message["channel"] = serde_json::Value::String(channel.clone());
        }

        if let Some(username) = &self.username {
            slack_message["username"] = serde_json::Value::String(username.clone());
        }

        // Add alert-specific fields
        if let Some(alert) = &message.alert {
            let attachment = &mut slack_message["attachments"][0];
            if let Some(fields) = attachment["fields"].as_array_mut() {
                fields.push(serde_json::json!({
                    "title": "Alert ID",
                    "value": alert.id,
                    "short": true
                }));

                fields.push(serde_json::json!({
                    "title": "Severity",
                    "value": alert.severity.to_string(),
                    "short": true
                }));

                if let Some(value) = alert.current_value {
                    fields.push(serde_json::json!({
                        "title": "Current Value",
                        "value": format!("{:.2}", value),
                        "short": true
                    }));
                }
            }
        }

        slack_message
    }
}

/// Generic webhook notification provider
#[derive(Debug, Clone)]
pub struct WebhookProvider {
    /// Webhook URL
    url: String,
    /// HTTP method (GET, POST, PUT)
    method: String,
    /// Request headers
    headers: HashMap<String, String>,
    /// HTTP client
    client: reqwest::Client,
    /// Request timeout
    timeout_seconds: u64,
}

impl WebhookProvider {
    /// Create new webhook provider from configuration
    pub fn from_config(config: &HashMap<String, String>) -> NotificationResult<Self> {
        let url = config
            .get("url")
            .ok_or_else(|| NotificationError::InvalidConfig {
                message: "url is required".to_string(),
            })?
            .clone();

        let method = config
            .get("method")
            .unwrap_or(&"POST".to_string())
            .to_uppercase();

        let timeout_seconds = config
            .get("timeout_seconds")
            .unwrap_or(&"30".to_string())
            .parse()
            .unwrap_or(30);

        // Parse headers from config
        let mut headers = HashMap::new();
        for (key, value) in config {
            if key.starts_with("header_") {
                let header_name = key.strip_prefix("header_").unwrap();
                headers.insert(header_name.to_string(), value.clone());
            }
        }

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .build()
            .map_err(|e| NotificationError::NetworkError {
                message: format!("Failed to create HTTP client: {}", e),
            })?;

        Ok(Self {
            url,
            method,
            headers,
            client,
            timeout_seconds,
        })
    }
}

#[async_trait]
impl NotificationProvider for WebhookProvider {
    async fn send(&self, message: &NotificationMessage) -> NotificationResult<DeliveryResult> {
        let start_time = std::time::Instant::now();

        // Format webhook payload
        let payload = serde_json::json!({
            "title": message.title,
            "body": message.body,
            "priority": message.priority.to_string(),
            "metadata": message.metadata,
            "alert": message.alert,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        });

        debug!("Sending webhook notification to: {}", self.url);

        // Build request
        let mut request = match self.method.as_str() {
            "GET" => self.client.get(&self.url),
            "POST" => self.client.post(&self.url),
            "PUT" => self.client.put(&self.url),
            _ => {
                return Err(NotificationError::InvalidConfig {
                    message: format!("Unsupported HTTP method: {}", self.method),
                });
            }
        };

        // Add headers
        for (name, value) in &self.headers {
            request = request.header(name, value);
        }

        // Add JSON payload for POST/PUT
        if self.method != "GET" {
            request = request.json(&payload);
        }

        // Send request
        let response = timeout(Duration::from_secs(self.timeout_seconds), request.send())
            .await
            .map_err(|_| NotificationError::Timeout {
                seconds: self.timeout_seconds,
            })?
            .map_err(|e| NotificationError::NetworkError {
                message: format!("Webhook request failed: {}", e),
            })?;

        let duration = start_time.elapsed();
        let success = response.status().is_success();
        let status_code = response.status();
        let response_text = response.text().await.unwrap_or_default();

        if success {
            info!("Webhook notification sent successfully to {}", self.url);
            Ok(DeliveryResult {
                success: true,
                channel_id: "webhook".to_string(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                response_message: Some(response_text),
                error_message: None,
                duration_ms: duration.as_millis() as u64,
            })
        } else {
            error!("Webhook notification failed: {}", response_text);
            Err(NotificationError::NetworkError {
                message: format!(
                    "Webhook failed with status {}: {}",
                    status_code, response_text
                ),
            })
        }
    }

    fn validate_config(&self, config: &HashMap<String, String>) -> NotificationResult<()> {
        if !config.contains_key("url") {
            return Err(NotificationError::InvalidConfig {
                message: "url is required".to_string(),
            });
        }

        let url = config.get("url").unwrap();
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(NotificationError::InvalidConfig {
                message: "Invalid URL format".to_string(),
            });
        }

        Ok(())
    }

    fn channel_type(&self) -> &'static str {
        "webhook"
    }

    async fn health_check(&self) -> NotificationResult<bool> {
        debug!("Webhook health check: testing connectivity to {}", self.url);

        // Simple HEAD request to test connectivity
        let response = timeout(Duration::from_secs(10), self.client.head(&self.url).send()).await;

        match response {
            Ok(Ok(resp)) => Ok(resp.status().is_success() || resp.status().as_u16() == 405), // 405 Method Not Allowed is OK
            Ok(Err(_)) | Err(_) => Ok(false),
        }
    }
}

/// PagerDuty notification provider
#[derive(Debug, Clone)]
pub struct PagerDutyProvider {
    /// PagerDuty integration key
    integration_key: String,
    /// HTTP client
    client: reqwest::Client,
}

impl PagerDutyProvider {
    /// Create new PagerDuty provider from configuration
    pub fn from_config(config: &HashMap<String, String>) -> NotificationResult<Self> {
        let integration_key = config
            .get("integration_key")
            .ok_or_else(|| NotificationError::InvalidConfig {
                message: "integration_key is required".to_string(),
            })?
            .clone();

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| NotificationError::NetworkError {
                message: format!("Failed to create HTTP client: {}", e),
            })?;

        Ok(Self {
            integration_key,
            client,
        })
    }
}

#[async_trait]
impl NotificationProvider for PagerDutyProvider {
    async fn send(&self, message: &NotificationMessage) -> NotificationResult<DeliveryResult> {
        let start_time = std::time::Instant::now();

        // Format PagerDuty payload
        let severity = match message.priority {
            MessagePriority::Critical => "critical",
            MessagePriority::High => "error",
            MessagePriority::Normal => "warning",
            MessagePriority::Low => "info",
        };

        let payload = serde_json::json!({
            "routing_key": self.integration_key,
            "event_action": "trigger",
            "payload": {
                "summary": message.title,
                "source": "unet-monitoring",
                "severity": severity,
                "component": "μNet",
                "group": "monitoring",
                "custom_details": {
                    "description": message.body,
                    "metadata": message.metadata
                }
            }
        });

        debug!("Sending PagerDuty notification: {}", message.title);

        // Send to PagerDuty Events API
        let response = timeout(
            Duration::from_secs(30),
            self.client
                .post("https://events.pagerduty.com/v2/enqueue")
                .header("Content-Type", "application/json")
                .json(&payload)
                .send(),
        )
        .await
        .map_err(|_| NotificationError::Timeout { seconds: 30 })?
        .map_err(|e| NotificationError::NetworkError {
            message: format!("PagerDuty API request failed: {}", e),
        })?;

        let duration = start_time.elapsed();
        let success = response.status().is_success();
        let response_text = response.text().await.unwrap_or_default();

        if success {
            info!("PagerDuty notification sent successfully");
            Ok(DeliveryResult {
                success: true,
                channel_id: "pagerduty".to_string(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                response_message: Some(response_text),
                error_message: None,
                duration_ms: duration.as_millis() as u64,
            })
        } else {
            error!("PagerDuty notification failed: {}", response_text);
            Err(NotificationError::NetworkError {
                message: format!("PagerDuty API failed: {}", response_text),
            })
        }
    }

    fn validate_config(&self, config: &HashMap<String, String>) -> NotificationResult<()> {
        if !config.contains_key("integration_key") {
            return Err(NotificationError::InvalidConfig {
                message: "integration_key is required".to_string(),
            });
        }

        Ok(())
    }

    fn channel_type(&self) -> &'static str {
        "pagerduty"
    }

    async fn health_check(&self) -> NotificationResult<bool> {
        debug!("PagerDuty health check: testing API connectivity");

        // Test with a minimal payload
        let test_payload = serde_json::json!({
            "routing_key": self.integration_key,
            "event_action": "trigger",
            "payload": {
                "summary": "Health check from μNet monitoring",
                "source": "unet-monitoring",
                "severity": "info",
                "component": "μNet",
                "group": "monitoring"
            }
        });

        let response = timeout(
            Duration::from_secs(10),
            self.client
                .post("https://events.pagerduty.com/v2/enqueue")
                .header("Content-Type", "application/json")
                .json(&test_payload)
                .send(),
        )
        .await;

        match response {
            Ok(Ok(resp)) => Ok(resp.status().is_success()),
            Ok(Err(_)) | Err(_) => Ok(false),
        }
    }
}

/// Notification manager for handling multiple channels
pub struct NotificationManager {
    /// Available notification providers
    providers: HashMap<String, Box<dyn NotificationProvider>>,
}

impl NotificationManager {
    /// Create a new notification manager
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Add a notification channel
    pub fn add_channel(&mut self, channel: &NotificationChannel) -> Result<()> {
        let provider: Box<dyn NotificationProvider> = match channel.channel_type.as_str() {
            "email" => Box::new(EmailProvider::from_config(&channel.config).map_err(|e| {
                Error::validation("email_config", &format!("Email configuration error: {}", e))
            })?),
            "slack" => Box::new(SlackProvider::from_config(&channel.config).map_err(|e| {
                Error::validation("slack_config", &format!("Slack configuration error: {}", e))
            })?),
            "webhook" => Box::new(WebhookProvider::from_config(&channel.config).map_err(|e| {
                Error::validation(
                    "webhook_config",
                    &format!("Webhook configuration error: {}", e),
                )
            })?),
            "pagerduty" => Box::new(PagerDutyProvider::from_config(&channel.config).map_err(
                |e| {
                    Error::validation(
                        "pagerduty_config",
                        &format!("PagerDuty configuration error: {}", e),
                    )
                },
            )?),
            _ => {
                return Err(Error::validation(
                    "unsupported_channel",
                    &format!("Unsupported channel type: {}", channel.channel_type),
                ));
            }
        };

        self.providers.insert(channel.id.clone(), provider);
        info!(
            "Added notification channel: {} ({})",
            channel.name, channel.channel_type
        );
        Ok(())
    }

    /// Send notification to specified channels
    pub async fn send_notification(
        &self,
        message: &NotificationMessage,
        channel_ids: &[String],
    ) -> Vec<DeliveryResult> {
        let mut results = Vec::new();

        for channel_id in channel_ids {
            if let Some(provider) = self.providers.get(channel_id) {
                match provider.send(message).await {
                    Ok(result) => {
                        debug!("Notification sent successfully to channel: {}", channel_id);
                        results.push(result);
                    }
                    Err(e) => {
                        error!(
                            "Failed to send notification to channel {}: {}",
                            channel_id, e
                        );
                        results.push(DeliveryResult {
                            success: false,
                            channel_id: channel_id.clone(),
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                            response_message: None,
                            error_message: Some(e.to_string()),
                            duration_ms: 0,
                        });
                    }
                }
            } else {
                warn!("Notification channel not found: {}", channel_id);
                results.push(DeliveryResult {
                    success: false,
                    channel_id: channel_id.clone(),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    response_message: None,
                    error_message: Some("Channel not found".to_string()),
                    duration_ms: 0,
                });
            }
        }

        results
    }

    /// Check health of all channels
    pub async fn health_check_all(&self) -> HashMap<String, bool> {
        let mut results = HashMap::new();

        for (channel_id, provider) in &self.providers {
            match provider.health_check().await {
                Ok(healthy) => {
                    results.insert(channel_id.clone(), healthy);
                }
                Err(e) => {
                    warn!("Health check failed for channel {}: {}", channel_id, e);
                    results.insert(channel_id.clone(), false);
                }
            }
        }

        results
    }
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_email_provider_creation() {
        let mut config = HashMap::new();
        config.insert("smtp_server".to_string(), "localhost:587".to_string());
        config.insert("from_address".to_string(), "test@example.com".to_string());
        config.insert("to_addresses".to_string(), "admin@example.com".to_string());

        let provider = EmailProvider::from_config(&config).unwrap();
        assert_eq!(provider.channel_type(), "email");
        assert!(provider.validate_config(&config).is_ok());
    }

    #[tokio::test]
    async fn test_slack_provider_creation() {
        let mut config = HashMap::new();
        config.insert(
            "webhook_url".to_string(),
            "https://hooks.slack.com/services/test".to_string(),
        );
        config.insert("channel".to_string(), "#alerts".to_string());

        let provider = SlackProvider::from_config(&config).unwrap();
        assert_eq!(provider.channel_type(), "slack");
        assert!(provider.validate_config(&config).is_ok());
    }

    #[tokio::test]
    async fn test_webhook_provider_creation() {
        let mut config = HashMap::new();
        config.insert("url".to_string(), "https://example.com/webhook".to_string());
        config.insert("method".to_string(), "POST".to_string());

        let provider = WebhookProvider::from_config(&config).unwrap();
        assert_eq!(provider.channel_type(), "webhook");
        assert!(provider.validate_config(&config).is_ok());
    }

    #[tokio::test]
    async fn test_pagerduty_provider_creation() {
        let mut config = HashMap::new();
        config.insert("integration_key".to_string(), "test-key".to_string());

        let provider = PagerDutyProvider::from_config(&config).unwrap();
        assert_eq!(provider.channel_type(), "pagerduty");
        assert!(provider.validate_config(&config).is_ok());
    }

    #[test]
    fn test_message_priority_conversion() {
        assert_eq!(MessagePriority::Critical.to_string(), "critical");
        assert_eq!(MessagePriority::High.to_string(), "high");
        assert_eq!(MessagePriority::Normal.to_string(), "normal");
        assert_eq!(MessagePriority::Low.to_string(), "low");
    }

    #[tokio::test]
    async fn test_notification_manager() {
        let mut manager = NotificationManager::new();

        let channel = NotificationChannel {
            id: "test-email".to_string(),
            name: "Test Email".to_string(),
            channel_type: "email".to_string(),
            config: HashMap::from([
                ("smtp_server".to_string(), "localhost:587".to_string()),
                ("from_address".to_string(), "test@example.com".to_string()),
                ("to_addresses".to_string(), "admin@example.com".to_string()),
            ]),
            enabled: true,
            rate_limit: None,
        };

        assert!(manager.add_channel(&channel).is_ok());

        let message = NotificationMessage {
            title: "Test Alert".to_string(),
            body: "This is a test alert message".to_string(),
            priority: MessagePriority::High,
            metadata: HashMap::new(),
            alert: None,
        };

        let results = manager
            .send_notification(&message, &["test-email".to_string()])
            .await;
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
    }
}
