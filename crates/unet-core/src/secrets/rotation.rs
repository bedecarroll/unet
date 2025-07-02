//! Secret rotation and lifecycle management for μNet Core
//!
//! This module provides automated secret rotation capabilities with scheduling,
//! validation, and rollback support for all secret backends including external providers.

use crate::error::{Error, Result};
use crate::secrets::SecretManager;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::sleep;
use tokio_cron_scheduler::{Job, JobScheduler};

/// Secret rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationConfig {
    /// Whether rotation is enabled
    pub enabled: bool,
    /// Rotation interval in seconds
    pub interval_seconds: u64,
    /// Maximum age before forced rotation (seconds)
    pub max_age_seconds: u64,
    /// Grace period for old secrets (seconds)
    pub grace_period_seconds: u64,
    /// Rotation schedule (cron expression)
    pub schedule: Option<String>,
    /// Secrets to rotate automatically
    pub auto_rotate_secrets: Vec<String>,
    /// Rotation notification settings
    pub notifications: RotationNotificationConfig,
}

impl Default for RotationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_seconds: 86400 * 30,            // 30 days
            max_age_seconds: 86400 * 90,             // 90 days
            grace_period_seconds: 86400 * 7,         // 7 days
            schedule: Some("0 2 * * 0".to_string()), // Weekly at 2 AM on Sunday
            auto_rotate_secrets: vec!["jwt-secret".to_string(), "api-keys".to_string()],
            notifications: RotationNotificationConfig::default(),
        }
    }
}

/// Rotation notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationNotificationConfig {
    /// Enable notifications
    pub enabled: bool,
    /// Notification channels
    pub channels: Vec<NotificationChannel>,
    /// Notify on rotation success
    pub notify_on_success: bool,
    /// Notify on rotation failure
    pub notify_on_failure: bool,
    /// Notify before rotation (hours)
    pub notify_before_hours: u64,
}

impl Default for RotationNotificationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            channels: vec![],
            notify_on_success: true,
            notify_on_failure: true,
            notify_before_hours: 24,
        }
    }
}

/// Notification channel types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannel {
    Email {
        address: String,
    },
    Slack {
        webhook_url: String,
    },
    Webhook {
        url: String,
        headers: HashMap<String, String>,
    },
    Log,
}

/// Secret rotation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationStatus {
    /// Secret name
    pub secret_name: String,
    /// Last rotation time
    pub last_rotated: Option<DateTime<Utc>>,
    /// Next scheduled rotation
    pub next_rotation: Option<DateTime<Utc>>,
    /// Rotation state
    pub state: RotationState,
    /// Last rotation result
    pub last_result: Option<RotationResult>,
    /// Rotation attempts count
    pub attempt_count: u32,
    /// Secret age in seconds
    pub age_seconds: u64,
    /// Whether rotation is overdue
    pub is_overdue: bool,
}

/// Rotation state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RotationState {
    /// Not scheduled for rotation
    Idle,
    /// Scheduled for rotation
    Scheduled,
    /// Currently rotating
    InProgress,
    /// Rotation failed
    Failed,
    /// Recently rotated
    Completed,
}

/// Rotation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationResult {
    /// Whether rotation succeeded
    pub success: bool,
    /// Rotation timestamp
    pub timestamp: DateTime<Utc>,
    /// Result message
    pub message: String,
    /// Error details if failed
    pub error: Option<String>,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

/// Secret rotation trait for different rotation strategies
#[async_trait]
pub trait SecretRotator: Send + Sync {
    /// Generate a new secret value
    async fn generate_new_value(&self, secret_name: &str) -> Result<String>;

    /// Validate the new secret value
    async fn validate_new_value(&self, secret_name: &str, new_value: &str) -> Result<()>;

    /// Pre-rotation hook (backup, notifications, etc.)
    async fn pre_rotation(&self, secret_name: &str) -> Result<()>;

    /// Post-rotation hook (update dependent services, etc.)
    async fn post_rotation(
        &self,
        secret_name: &str,
        old_value: &str,
        new_value: &str,
    ) -> Result<()>;

    /// Rollback hook if rotation fails
    async fn rollback(&self, secret_name: &str, old_value: &str) -> Result<()>;
}

/// Default secret rotator implementation
pub struct DefaultSecretRotator {
    config: RotationConfig,
}

impl DefaultSecretRotator {
    pub fn new(config: RotationConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl SecretRotator for DefaultSecretRotator {
    async fn generate_new_value(&self, secret_name: &str) -> Result<String> {
        use rand::Rng;

        // Generate different types of secrets based on name
        let new_value = match secret_name {
            name if name.contains("jwt") => {
                // Generate JWT secret (64 chars, base64-safe)
                const CHARSET: &[u8] =
                    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
                let mut rng = rand::thread_rng();
                (0..64)
                    .map(|_| {
                        let idx = rng.gen_range(0..CHARSET.len());
                        CHARSET[idx] as char
                    })
                    .collect()
            }
            name if name.contains("api-key") => {
                // Generate API key (32 chars, alphanumeric)
                const CHARSET: &[u8] =
                    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
                let mut rng = rand::thread_rng();
                (0..32)
                    .map(|_| {
                        let idx = rng.gen_range(0..CHARSET.len());
                        CHARSET[idx] as char
                    })
                    .collect()
            }
            name if name.contains("password") => {
                // Generate password (24 chars, mixed)
                const CHARSET: &[u8] =
                    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*";
                let mut rng = rand::thread_rng();
                (0..24)
                    .map(|_| {
                        let idx = rng.gen_range(0..CHARSET.len());
                        CHARSET[idx] as char
                    })
                    .collect()
            }
            _ => {
                // Default: 32 char alphanumeric
                const CHARSET: &[u8] =
                    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
                let mut rng = rand::thread_rng();
                (0..32)
                    .map(|_| {
                        let idx = rng.gen_range(0..CHARSET.len());
                        CHARSET[idx] as char
                    })
                    .collect()
            }
        };

        Ok(new_value)
    }

    async fn validate_new_value(&self, secret_name: &str, new_value: &str) -> Result<()> {
        // Basic validation rules
        if new_value.is_empty() {
            return Err(Error::config("Generated secret value is empty"));
        }

        if new_value.len() < 16 {
            return Err(Error::config(
                "Generated secret value is too short (minimum 16 characters)",
            ));
        }

        // Secret-specific validation
        match secret_name {
            name if name.contains("jwt") => {
                if new_value.len() < 32 {
                    return Err(Error::config("JWT secret must be at least 32 characters"));
                }
            }
            name if name.contains("api-key") => {
                if !new_value.chars().all(|c| c.is_alphanumeric()) {
                    return Err(Error::config("API key must be alphanumeric"));
                }
            }
            _ => {}
        }

        Ok(())
    }

    async fn pre_rotation(&self, secret_name: &str) -> Result<()> {
        tracing::info!("Starting rotation for secret: {}", secret_name);

        // Send pre-rotation notifications
        if self.config.notifications.enabled {
            self.send_notification(
                &format!("Starting rotation for secret: {}", secret_name),
                false,
            )
            .await?;
        }

        Ok(())
    }

    async fn post_rotation(
        &self,
        secret_name: &str,
        _old_value: &str,
        _new_value: &str,
    ) -> Result<()> {
        tracing::info!("Completed rotation for secret: {}", secret_name);

        // Send success notifications
        if self.config.notifications.enabled && self.config.notifications.notify_on_success {
            self.send_notification(
                &format!("Successfully rotated secret: {}", secret_name),
                false,
            )
            .await?;
        }

        Ok(())
    }

    async fn rollback(&self, secret_name: &str, _old_value: &str) -> Result<()> {
        tracing::warn!("Rolling back rotation for secret: {}", secret_name);

        // Send failure notifications
        if self.config.notifications.enabled && self.config.notifications.notify_on_failure {
            self.send_notification(
                &format!("Rotation failed for secret: {}, rolling back", secret_name),
                true,
            )
            .await?;
        }

        Ok(())
    }
}

impl DefaultSecretRotator {
    async fn send_notification(&self, message: &str, is_error: bool) -> Result<()> {
        for channel in &self.config.notifications.channels {
            match channel {
                NotificationChannel::Log => {
                    if is_error {
                        tracing::error!("{}", message);
                    } else {
                        tracing::info!("{}", message);
                    }
                }
                NotificationChannel::Email { address } => {
                    self.send_email_notification(address, message, is_error)
                        .await?;
                }
                NotificationChannel::Slack { webhook_url } => {
                    self.send_slack_notification(webhook_url, message, is_error)
                        .await?;
                }
                NotificationChannel::Webhook { url, headers } => {
                    self.send_webhook_notification(url, headers, message, is_error)
                        .await?;
                }
            }
        }
        Ok(())
    }

    async fn send_email_notification(
        &self,
        address: &str,
        message: &str,
        is_error: bool,
    ) -> Result<()> {
        // For now, log the email notification - can be replaced with actual email service
        tracing::info!(
            "EMAIL NOTIFICATION to {}: {} ({})",
            address,
            message,
            if is_error { "ERROR" } else { "INFO" }
        );
        // TODO: Integrate with email service like SendGrid, AWS SES, or SMTP
        Ok(())
    }

    async fn send_slack_notification(
        &self,
        webhook_url: &str,
        message: &str,
        is_error: bool,
    ) -> Result<()> {
        let client = reqwest::Client::new();
        let payload = serde_json::json!({
            "text": message,
            "username": "μNet Secret Rotation",
            "icon_emoji": if is_error { ":warning:" } else { ":shield:" },
            "attachments": [{
                "color": if is_error { "danger" } else { "good" },
                "fields": [{
                    "title": "Status",
                    "value": if is_error { "Error" } else { "Success" },
                    "short": true
                }, {
                    "title": "Timestamp",
                    "value": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                    "short": true
                }]
            }]
        });

        let response = client
            .post(webhook_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::config(format!("Failed to send Slack notification: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::config(format!(
                "Slack notification failed with status: {}",
                response.status()
            )));
        }

        tracing::debug!("Slack notification sent successfully to {}", webhook_url);
        Ok(())
    }

    async fn send_webhook_notification(
        &self,
        url: &str,
        headers: &HashMap<String, String>,
        message: &str,
        is_error: bool,
    ) -> Result<()> {
        let client = reqwest::Client::new();
        let payload = serde_json::json!({
            "event": "secret_rotation",
            "message": message,
            "status": if is_error { "error" } else { "success" },
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "service": "unet-secret-rotation"
        });

        let mut request = client.post(url).json(&payload);

        // Add custom headers
        for (key, value) in headers {
            request = request.header(key, value);
        }

        // Set default content type if not specified
        if !headers.contains_key("Content-Type") {
            request = request.header("Content-Type", "application/json");
        }

        let response = request
            .send()
            .await
            .map_err(|e| Error::config(format!("Failed to send webhook notification: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::config(format!(
                "Webhook notification failed with status: {}",
                response.status()
            )));
        }

        tracing::debug!("Webhook notification sent successfully to {}", url);
        Ok(())
    }
}

/// Secret rotation manager
pub struct SecretRotationManager {
    secret_manager: SecretManager,
    rotator: Box<dyn SecretRotator>,
    config: RotationConfig,
    scheduler: Option<JobScheduler>,
    rotation_status: HashMap<String, RotationStatus>,
}

impl SecretRotationManager {
    /// Create a new rotation manager
    pub fn new(
        secret_manager: SecretManager,
        config: RotationConfig,
        rotator: Option<Box<dyn SecretRotator>>,
    ) -> Self {
        let rotator =
            rotator.unwrap_or_else(|| Box::new(DefaultSecretRotator::new(config.clone())));

        Self {
            secret_manager,
            rotator,
            config,
            scheduler: None,
            rotation_status: HashMap::new(),
        }
    }

    /// Initialize the rotation manager and start scheduler
    pub async fn initialize(&mut self) -> Result<()> {
        if !self.config.enabled {
            tracing::info!("Secret rotation is disabled");
            return Ok(());
        }

        // Initialize scheduler
        self.scheduler = Some(
            JobScheduler::new()
                .await
                .map_err(|e| Error::config(format!("Failed to create job scheduler: {}", e)))?,
        );

        // Schedule automatic rotations
        if let Some(schedule) = self.config.schedule.clone() {
            self.schedule_rotation_job(&schedule).await?;
        }

        // Initialize rotation status for auto-rotate secrets
        let auto_rotate_secrets = self.config.auto_rotate_secrets.clone();
        for secret_name in &auto_rotate_secrets {
            self.update_rotation_status(secret_name).await?;
        }

        tracing::info!(
            "Secret rotation manager initialized with {} secrets",
            self.config.auto_rotate_secrets.len()
        );
        Ok(())
    }

    /// Manually rotate a specific secret
    pub async fn rotate_secret(&mut self, secret_name: &str) -> Result<RotationResult> {
        let start_time = Utc::now();
        tracing::info!("Manual rotation requested for secret: {}", secret_name);

        // Update status to in-progress
        self.set_rotation_state(secret_name, RotationState::InProgress)
            .await;

        let result = self.perform_rotation(secret_name).await;

        let duration_ms = (Utc::now() - start_time).num_milliseconds() as u64;
        let rotation_result = match result {
            Ok(_) => {
                self.set_rotation_state(secret_name, RotationState::Completed)
                    .await;
                RotationResult {
                    success: true,
                    timestamp: Utc::now(),
                    message: format!("Successfully rotated secret: {}", secret_name),
                    error: None,
                    duration_ms,
                }
            }
            Err(e) => {
                self.set_rotation_state(secret_name, RotationState::Failed)
                    .await;
                RotationResult {
                    success: false,
                    timestamp: Utc::now(),
                    message: format!("Failed to rotate secret: {}", secret_name),
                    error: Some(e.to_string()),
                    duration_ms,
                }
            }
        };

        // Update rotation status
        if let Some(status) = self.rotation_status.get_mut(secret_name) {
            status.last_result = Some(rotation_result.clone());
            status.last_rotated = Some(Utc::now());
            status.attempt_count += 1;
        }

        Ok(rotation_result)
    }

    /// Get rotation status for all secrets
    pub async fn get_rotation_status(&self) -> Result<Vec<RotationStatus>> {
        Ok(self.rotation_status.values().cloned().collect())
    }

    /// Get rotation status for a specific secret
    pub async fn get_secret_rotation_status(
        &self,
        secret_name: &str,
    ) -> Result<Option<RotationStatus>> {
        Ok(self.rotation_status.get(secret_name).cloned())
    }

    /// Check which secrets need rotation
    pub async fn check_rotation_needed(&mut self) -> Result<Vec<String>> {
        let mut needs_rotation = Vec::new();
        let secrets_to_check = self.config.auto_rotate_secrets.clone();

        for secret_name in &secrets_to_check {
            if self.is_rotation_needed(secret_name).await? {
                needs_rotation.push(secret_name.clone());
            }
        }

        Ok(needs_rotation)
    }

    /// Perform automatic rotation for all eligible secrets
    pub async fn perform_automatic_rotation(&mut self) -> Result<Vec<RotationResult>> {
        let secrets_needing_rotation = self.check_rotation_needed().await?;
        let mut results = Vec::new();

        for secret_name in secrets_needing_rotation {
            let result = self.rotate_secret(&secret_name).await?;
            results.push(result);

            // Add delay between rotations to avoid overwhelming external services
            sleep(tokio::time::Duration::from_secs(5)).await;
        }

        Ok(results)
    }

    // Private helper methods

    async fn perform_rotation(&mut self, secret_name: &str) -> Result<()> {
        // Get current secret value for rollback
        let old_value = self
            .secret_manager
            .get_secret(secret_name)
            .await?
            .unwrap_or_default();

        // Pre-rotation hook
        self.rotator.pre_rotation(secret_name).await?;

        // Generate new secret value
        let new_value = self.rotator.generate_new_value(secret_name).await?;

        // Validate new value
        self.rotator
            .validate_new_value(secret_name, &new_value)
            .await?;

        // Prepare metadata with rotation timestamp
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("last_rotated".to_string(), chrono::Utc::now().to_rfc3339());
        metadata.insert("rotation_count".to_string(), {
            // Get current rotation count and increment
            let current_count = match self.secret_manager.get_secret_metadata(secret_name).await {
                Ok(Some(existing_metadata)) => existing_metadata
                    .get("rotation_count")
                    .and_then(|c| c.parse::<u32>().ok())
                    .unwrap_or(0),
                _ => 0,
            };
            (current_count + 1).to_string()
        });

        // Store new secret with updated metadata
        match self
            .secret_manager
            .store_secret(secret_name, &new_value, Some(metadata))
            .await
        {
            Ok(_) => {
                // Post-rotation hook
                self.rotator
                    .post_rotation(secret_name, &old_value, &new_value)
                    .await?;

                // Update rotation status
                self.update_rotation_status(secret_name).await?;

                tracing::info!("Successfully rotated secret: {}", secret_name);
                Ok(())
            }
            Err(e) => {
                // Rollback on failure
                if !old_value.is_empty() {
                    let _ = self
                        .secret_manager
                        .store_secret(secret_name, &old_value, None)
                        .await;
                    self.rotator.rollback(secret_name, &old_value).await?;
                }
                Err(e)
            }
        }
    }

    async fn schedule_rotation_job(&mut self, cron_expression: &str) -> Result<()> {
        if let Some(ref mut scheduler) = self.scheduler {
            // Create a clone of necessary data for the async closure
            let auto_rotate_secrets = self.config.auto_rotate_secrets.clone();
            let config = self.config.clone();

            let job = Job::new_async(cron_expression, move |_uuid, _locked| {
                let secrets = auto_rotate_secrets.clone();
                let rotation_config = config.clone();

                Box::pin(async move {
                    tracing::info!(
                        "Scheduled rotation job triggered for {} secrets",
                        secrets.len()
                    );

                    // Note: In a real implementation, we would need access to the rotation manager
                    // For now, we log which secrets would be checked for rotation
                    for secret_name in &secrets {
                        tracing::info!("Checking rotation status for secret: {}", secret_name);

                        // This would trigger actual rotation logic in a full implementation
                        // For now, we just log the intent
                        if rotation_config.enabled {
                            tracing::debug!(
                                "Secret {} is eligible for rotation check",
                                secret_name
                            );
                        }
                    }

                    // TODO: In a production implementation, this would need to:
                    // 1. Create a new rotation manager instance with access to secrets
                    // 2. Call perform_automatic_rotation()
                    // 3. Handle results and notifications
                })
            })
            .map_err(|e| Error::config(format!("Failed to create rotation job: {}", e)))?;

            scheduler
                .add(job)
                .await
                .map_err(|e| Error::config(format!("Failed to schedule rotation job: {}", e)))?;

            scheduler
                .start()
                .await
                .map_err(|e| Error::config(format!("Failed to start rotation scheduler: {}", e)))?;

            tracing::info!(
                "Secret rotation scheduler started with cron expression: {}",
                cron_expression
            );
        }
        Ok(())
    }

    async fn update_rotation_status(&mut self, secret_name: &str) -> Result<()> {
        let now = Utc::now();
        let age_seconds = self.calculate_secret_age(secret_name).await?;
        let is_overdue = age_seconds > self.config.max_age_seconds;

        // Get last rotation time from metadata
        let last_rotated = match self.secret_manager.get_secret_metadata(secret_name).await {
            Ok(Some(metadata)) => metadata
                .get("last_rotated")
                .and_then(|ts| chrono::DateTime::parse_from_rfc3339(ts).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc)),
            _ => None,
        };

        let next_rotation = if is_overdue {
            Some(now) // Rotate immediately if overdue
        } else {
            Some(now + Duration::seconds(self.config.interval_seconds as i64))
        };

        let status = RotationStatus {
            secret_name: secret_name.to_string(),
            last_rotated,
            next_rotation,
            state: if is_overdue {
                RotationState::Scheduled
            } else {
                RotationState::Idle
            },
            last_result: None,
            attempt_count: 0,
            age_seconds,
            is_overdue,
        };

        self.rotation_status.insert(secret_name.to_string(), status);
        Ok(())
    }

    async fn set_rotation_state(&mut self, secret_name: &str, state: RotationState) {
        if let Some(status) = self.rotation_status.get_mut(secret_name) {
            status.state = state;
        }
    }

    async fn is_rotation_needed(&mut self, secret_name: &str) -> Result<bool> {
        let age_seconds = self.calculate_secret_age(secret_name).await?;
        Ok(age_seconds > self.config.interval_seconds || age_seconds > self.config.max_age_seconds)
    }

    async fn calculate_secret_age(&mut self, secret_name: &str) -> Result<u64> {
        // Get secret metadata from secret manager
        match self.secret_manager.get_secret_metadata(secret_name).await {
            Ok(Some(metadata)) => {
                // Look for creation time or last rotation time in metadata
                let creation_time = if let Some(last_rotated) = metadata.get("last_rotated") {
                    // Parse last rotation time
                    chrono::DateTime::parse_from_rfc3339(last_rotated)
                        .map_err(|e| {
                            Error::config(format!("Invalid last_rotated timestamp: {}", e))
                        })?
                        .with_timezone(&chrono::Utc)
                } else if let Some(created_at) = metadata.get("created_at") {
                    // Parse creation time
                    chrono::DateTime::parse_from_rfc3339(created_at)
                        .map_err(|e| Error::config(format!("Invalid created_at timestamp: {}", e)))?
                        .with_timezone(&chrono::Utc)
                } else {
                    // Fallback to now if no timestamps found
                    tracing::warn!("No timestamp metadata found for secret: {}", secret_name);
                    chrono::Utc::now()
                };

                let age = chrono::Utc::now().signed_duration_since(creation_time);
                Ok(age.num_seconds().max(0) as u64)
            }
            Ok(None) => {
                tracing::warn!("No metadata found for secret: {}", secret_name);
                // If secret doesn't exist or has no metadata, consider it new
                Ok(0)
            }
            Err(e) => {
                tracing::error!("Failed to get metadata for secret {}: {}", secret_name, e);
                // Return a large age to trigger rotation if there's an error
                Ok(self.config.max_age_seconds + 1)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::secrets::{MasterKey, SecretBackend};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_default_secret_rotator() {
        let config = RotationConfig::default();
        let rotator = DefaultSecretRotator::new(config);

        // Test JWT secret generation
        let jwt_secret = rotator.generate_new_value("jwt-secret").await.unwrap();
        assert_eq!(jwt_secret.len(), 64);
        rotator
            .validate_new_value("jwt-secret", &jwt_secret)
            .await
            .unwrap();

        // Test API key generation
        let api_key = rotator.generate_new_value("api-key").await.unwrap();
        assert_eq!(api_key.len(), 32);
        assert!(api_key.chars().all(|c| c.is_alphanumeric()));
        rotator
            .validate_new_value("api-key", &api_key)
            .await
            .unwrap();

        // Test password generation
        let password = rotator.generate_new_value("password").await.unwrap();
        assert_eq!(password.len(), 24);
        rotator
            .validate_new_value("password", &password)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_rotation_manager_initialization() {
        let temp_dir = TempDir::new().unwrap();
        let secrets_path = temp_dir.path().join("secrets.json");

        let mut secret_manager = SecretManager::new(SecretBackend::File {
            path: secrets_path.to_string_lossy().to_string(),
        });

        let master_key = MasterKey::generate();
        secret_manager.initialize(master_key).await.unwrap();

        let config = RotationConfig {
            enabled: false, // Disable for testing
            ..Default::default()
        };

        let mut rotation_manager = SecretRotationManager::new(secret_manager, config, None);

        rotation_manager.initialize().await.unwrap();

        let status = rotation_manager.get_rotation_status().await.unwrap();
        assert!(status.is_empty()); // No auto-rotate secrets with disabled config
    }

    #[test]
    fn test_rotation_config_serialization() {
        let config = RotationConfig::default();
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: RotationConfig = serde_json::from_str(&serialized).unwrap();

        assert_eq!(config.enabled, deserialized.enabled);
        assert_eq!(config.interval_seconds, deserialized.interval_seconds);
        assert_eq!(config.auto_rotate_secrets, deserialized.auto_rotate_secrets);
    }

    #[test]
    fn test_notification_channel_serialization() {
        let channels = vec![
            NotificationChannel::Email {
                address: "test@example.com".to_string(),
            },
            NotificationChannel::Slack {
                webhook_url: "https://hooks.slack.com/test".to_string(),
            },
            NotificationChannel::Log,
        ];

        for channel in channels {
            let serialized = serde_json::to_string(&channel).unwrap();
            let _deserialized: NotificationChannel = serde_json::from_str(&serialized).unwrap();
        }
    }
}
