//! Advanced Logging and Tracing Infrastructure for μNet Core
//!
//! This module provides comprehensive structured logging capabilities using the `tracing`
//! ecosystem with support for:
//! - Multiple output formats (JSON, pretty, compact)
//! - File logging with rotation
//! - Log aggregation and parsing
//! - Log-based alerting with configurable rules
//! - Enhanced structured logging macros
//!
//! Future enhancements will include OpenTelemetry distributed tracing.

use crate::config::{
    AlertChannel, LogAggregationConfig, LogAlertRule, LogAlertingConfig, LoggingConfig,
};
use crate::error::{Error, Result};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

/// Global log alerting manager for processing log events
static LOG_ALERTING_MANAGER: once_cell::sync::Lazy<Arc<RwLock<Option<LogAlertingManager>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(None)));

/// Comprehensive tracing system manager
pub struct TracingSystem {
    /// Log aggregation manager
    log_aggregation: Option<LogAggregationManager>,
    /// Log alerting manager  
    log_alerting: Option<Arc<LogAlertingManager>>,
}

impl TracingSystem {
    /// Initialize the complete tracing system with all features
    pub async fn initialize(config: &LoggingConfig) -> Result<Self> {
        let mut log_aggregation = None;
        let mut log_alerting = None;

        // Initialize log aggregation if enabled
        if let Some(agg_config) = &config.aggregation {
            if agg_config.enabled {
                log_aggregation = Some(LogAggregationManager::new(agg_config)?);
            }
        }

        // Initialize log alerting if enabled
        if let Some(alert_config) = &config.alerting {
            if alert_config.enabled {
                let alerting_manager = Arc::new(LogAlertingManager::new(alert_config)?);
                log_alerting = Some(alerting_manager.clone());

                // Set global alerting manager
                *LOG_ALERTING_MANAGER.write().await = Some(alerting_manager.as_ref().clone());
            }
        }

        // Initialize tracing subscriber based on configuration
        init_subscriber(config)?;

        tracing::info!(
            level = %config.level,
            format = %config.format,
            file_output = config.file.is_some(),
            aggregation_enabled = config.aggregation.as_ref().map(|c| c.enabled).unwrap_or(false),
            alerting_enabled = config.alerting.as_ref().map(|c| c.enabled).unwrap_or(false),
            "Advanced tracing system initialized"
        );

        Ok(TracingSystem {
            log_aggregation,
            log_alerting,
        })
    }

    /// Shutdown the tracing system gracefully
    pub async fn shutdown(self) -> Result<()> {
        if let Some(alerting) = self.log_alerting {
            alerting.shutdown().await?;
        }

        if let Some(aggregation) = self.log_aggregation {
            aggregation.shutdown().await?;
        }

        // Clear global alerting manager
        *LOG_ALERTING_MANAGER.write().await = None;

        tracing::info!("Tracing system shutdown complete");
        Ok(())
    }
}

/// Initialize the tracing subscriber based on configuration
fn init_subscriber(config: &LoggingConfig) -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(&config.level))
        .map_err(|e| {
            Error::config_with_source(format!("Invalid log level '{}'", config.level), e)
        })?;

    match config.format.as_str() {
        "json" => {
            if let Some(ref file_path) = config.file {
                let file_appender = create_file_appender(file_path)?;
                tracing_subscriber::registry()
                    .with(env_filter)
                    .with(
                        fmt::layer()
                            .json()
                            .with_current_span(true)
                            .with_target(true)
                            .with_thread_ids(true)
                            .with_thread_names(true)
                            .with_file(true)
                            .with_line_number(true)
                            .with_writer(file_appender),
                    )
                    .init();
            } else {
                tracing_subscriber::registry()
                    .with(env_filter)
                    .with(
                        fmt::layer()
                            .json()
                            .with_current_span(true)
                            .with_target(true)
                            .with_thread_ids(true)
                            .with_thread_names(true)
                            .with_file(true)
                            .with_line_number(true),
                    )
                    .init();
            }
        }
        "compact" => {
            if let Some(ref file_path) = config.file {
                let file_appender = create_file_appender(file_path)?;
                tracing_subscriber::registry()
                    .with(env_filter)
                    .with(
                        fmt::layer()
                            .compact()
                            .with_target(false)
                            .with_thread_ids(false)
                            .with_thread_names(false)
                            .with_file(false)
                            .with_line_number(false)
                            .with_writer(file_appender),
                    )
                    .init();
            } else {
                tracing_subscriber::registry()
                    .with(env_filter)
                    .with(
                        fmt::layer()
                            .compact()
                            .with_target(false)
                            .with_thread_ids(false)
                            .with_thread_names(false)
                            .with_file(false)
                            .with_line_number(false),
                    )
                    .init();
            }
        }
        _ => {
            // Default to pretty format
            if let Some(ref file_path) = config.file {
                let file_appender = create_file_appender(file_path)?;
                tracing_subscriber::registry()
                    .with(env_filter)
                    .with(
                        fmt::layer()
                            .pretty()
                            .with_target(true)
                            .with_thread_ids(true)
                            .with_thread_names(true)
                            .with_file(true)
                            .with_line_number(true)
                            .with_writer(file_appender),
                    )
                    .init();
            } else {
                tracing_subscriber::registry()
                    .with(env_filter)
                    .with(
                        fmt::layer()
                            .pretty()
                            .with_target(true)
                            .with_thread_ids(true)
                            .with_thread_names(true)
                            .with_file(true)
                            .with_line_number(true),
                    )
                    .init();
            }
        }
    }

    Ok(())
}

/// Create file appender with rotation support
fn create_file_appender(file_path: &str) -> Result<tracing_appender::rolling::RollingFileAppender> {
    let path = Path::new(file_path);

    // Create parent directory if it doesn't exist
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            Error::config_with_source(
                format!("Failed to create log directory '{}'", parent.display()),
                e,
            )
        })?;
    }

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unet.log");

    let dir = path.parent().unwrap_or_else(|| Path::new("."));

    let appender = tracing_appender::rolling::daily(dir, file_name);
    Ok(appender)
}

/// Log aggregation manager for centralized log collection
pub struct LogAggregationManager {}

impl LogAggregationManager {
    fn new(config: &LogAggregationConfig) -> Result<Self> {
        if let Some(endpoint) = &config.syslog_endpoint {
            tracing::info!(endpoint = %endpoint, "Syslog aggregation configured");
        }

        tracing::info!(
            enabled = config.enabled,
            max_size_mb = config.rotation.max_size_mb,
            max_files = config.rotation.max_files,
            schedule = %config.rotation.schedule,
            parsing_enabled = config.parsing.enabled,
            enrichment = config.parsing.enrichment,
            "Log aggregation manager initialized"
        );

        Ok(LogAggregationManager {})
    }

    async fn shutdown(self) -> Result<()> {
        tracing::info!("Log aggregation manager shutdown");
        Ok(())
    }
}

/// Log alerting manager for processing alert rules
#[derive(Clone)]
pub struct LogAlertingManager {
    rules: Vec<LogAlertRule>,
    channels: Vec<AlertChannel>,
    alert_counts: Arc<RwLock<HashMap<String, AlertCounter>>>,
}

struct AlertCounter {
    count: u32,
    window_start: SystemTime,
}

impl LogAlertingManager {
    fn new(config: &LogAlertingConfig) -> Result<Self> {
        tracing::info!(
            rules_count = config.rules.len(),
            channels_count = config.channels.len(),
            "Log alerting manager initialized"
        );

        Ok(LogAlertingManager {
            rules: config.rules.clone(),
            channels: config.channels.clone(),
            alert_counts: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Process a log event for alerting
    pub async fn process_log_event(&self, level: &str, message: &str) -> Result<()> {
        for rule in &self.rules {
            if self.matches_rule(rule, level, message).await? {
                self.trigger_alert(rule, message).await?;
            }
        }
        Ok(())
    }

    async fn matches_rule(&self, rule: &LogAlertRule, level: &str, message: &str) -> Result<bool> {
        // Check log level
        if !self.level_matches(&rule.level, level) {
            return Ok(false);
        }

        // Check pattern if specified
        if let Some(pattern) = &rule.pattern {
            if !message.contains(pattern) {
                return Ok(false);
            }
        }

        // Check threshold within time window
        let mut alert_counts = self.alert_counts.write().await;
        let now = SystemTime::now();
        let window_duration = Duration::from_secs(rule.window_minutes as u64 * 60);

        let counter = alert_counts
            .entry(rule.name.clone())
            .or_insert(AlertCounter {
                count: 0,
                window_start: now,
            });

        // Reset counter if window has expired
        if now
            .duration_since(counter.window_start)
            .unwrap_or(Duration::ZERO)
            > window_duration
        {
            counter.count = 0;
            counter.window_start = now;
        }

        counter.count += 1;

        Ok(counter.count >= rule.threshold)
    }

    fn level_matches(&self, rule_level: &str, event_level: &str) -> bool {
        let levels = ["trace", "debug", "info", "warn", "error"];
        let rule_index = levels.iter().position(|&l| l == rule_level).unwrap_or(2);
        let event_index = levels.iter().position(|&l| l == event_level).unwrap_or(2);
        event_index >= rule_index
    }

    async fn trigger_alert(&self, rule: &LogAlertRule, message: &str) -> Result<()> {
        tracing::warn!(
            rule = %rule.name,
            severity = %rule.severity,
            message = %message,
            component = "alerting",
            "Alert triggered"
        );

        // Send notifications through configured channels
        for channel in &self.channels {
            self.send_notification(channel, rule, message).await?;
        }

        Ok(())
    }

    async fn send_notification(
        &self,
        channel: &AlertChannel,
        rule: &LogAlertRule,
        _message: &str,
    ) -> Result<()> {
        match channel.channel_type.as_str() {
            "email" => {
                tracing::info!(
                    channel = "email",
                    rule = %rule.name,
                    component = "alerting",
                    "Email notification sent (simulated)"
                );
            }
            "slack" => {
                tracing::info!(
                    channel = "slack",
                    rule = %rule.name,
                    component = "alerting",
                    "Slack notification sent (simulated)"
                );
            }
            "webhook" => {
                if let Some(url) = channel.config.get("url") {
                    tracing::info!(
                        channel = "webhook",
                        url = %url,
                        rule = %rule.name,
                        component = "alerting",
                        "Webhook notification sent (simulated)"
                    );
                }
            }
            "pagerduty" => {
                tracing::info!(
                    channel = "pagerduty",
                    rule = %rule.name,
                    component = "alerting",
                    "PagerDuty notification sent (simulated)"
                );
            }
            _ => {
                tracing::warn!(
                    channel_type = %channel.channel_type,
                    component = "alerting",
                    "Unknown notification channel type"
                );
            }
        }
        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        tracing::info!("Log alerting manager shutdown");
        Ok(())
    }
}

/// Initialize tracing with default configuration
pub async fn init_default_tracing() -> Result<TracingSystem> {
    let config = LoggingConfig {
        level: "info".to_string(),
        format: "pretty".to_string(),
        file: None,
        opentelemetry: None,
        aggregation: None,
        alerting: None,
    };
    TracingSystem::initialize(&config).await
}

/// Initialize tracing with basic configuration (backwards compatibility)
pub fn init_tracing(config: &LoggingConfig) -> Result<()> {
    init_subscriber(config)
}

/// Creates a structured log context for operations
#[macro_export]
macro_rules! log_context {
    ($operation:expr) => {
        tracing::info_span!("operation", op = %$operation)
    };
    ($operation:expr, $($key:tt = $value:expr),*) => {
        tracing::info_span!("operation", op = %$operation, $($key = %$value),*)
    };
}

/// Logs the start and end of an operation with timing
#[macro_export]
macro_rules! timed_operation {
    ($operation:expr, $code:block) => {{
        let _span = $crate::log_context!($operation).entered();
        let start = std::time::Instant::now();
        tracing::info!("Operation started");

        let result = $code;

        let duration = start.elapsed();
        match &result {
            Ok(_) => tracing::info!(duration_ms = %duration.as_millis(), "Operation completed successfully"),
            Err(e) => tracing::error!(duration_ms = %duration.as_millis(), error = %e, "Operation failed"),
        }

        result
    }};
}

/// Enhanced macros for common logging patterns with structured fields
#[macro_export]
macro_rules! log_database_operation {
    ($operation:expr, $table:expr) => {
        tracing::debug!(
            operation = %$operation,
            table = %$table,
            component = "database",
            "Database operation"
        );
    };
    ($operation:expr, $table:expr, $($key:tt = $value:expr),*) => {
        tracing::debug!(
            operation = %$operation,
            table = %$table,
            component = "database",
            $($key = %$value),*,
            "Database operation"
        );
    };
}

#[macro_export]
macro_rules! log_snmp_operation {
    ($operation:expr, $target:expr) => {
        tracing::debug!(
            operation = %$operation,
            target = %$target,
            component = "snmp",
            "SNMP operation"
        );
    };
    ($operation:expr, $target:expr, $($key:tt = $value:expr),*) => {
        tracing::debug!(
            operation = %$operation,
            target = %$target,
            component = "snmp",
            $($key = %$value),*,
            "SNMP operation"
        );
    };
}

#[macro_export]
macro_rules! log_network_operation {
    ($operation:expr, $endpoint:expr) => {
        tracing::debug!(
            operation = %$operation,
            endpoint = %$endpoint,
            component = "network",
            "Network operation"
        );
    };
    ($operation:expr, $endpoint:expr, $($key:tt = $value:expr),*) => {
        tracing::debug!(
            operation = %$operation,
            endpoint = %$endpoint,
            component = "network",
            $($key = %$value),*,
            "Network operation"
        );
    };
}

/// Macro for security event logging with alerting integration
#[macro_export]
macro_rules! log_security_event {
    ($event_type:expr, $severity:expr, $message:expr) => {
        tracing::warn!(
            event_type = %$event_type,
            severity = %$severity,
            component = "security",
            %$message,
            "Security event detected"
        );
    };
    ($event_type:expr, $severity:expr, $message:expr, $($key:tt = $value:expr),*) => {
        tracing::warn!(
            event_type = %$event_type,
            severity = %$severity,
            component = "security",
            $($key = %$value),*,
            %$message,
            "Security event detected"
        );
    };
}

/// Macro for API request logging with structured fields
#[macro_export]
macro_rules! log_api_request {
    ($method:expr, $path:expr, $status:expr) => {
        tracing::info!(
            method = %$method,
            path = %$path,
            status = %$status,
            component = "api",
            "API request processed"
        );
    };
    ($method:expr, $path:expr, $status:expr, $($key:tt = $value:expr),*) => {
        tracing::info!(
            method = %$method,
            path = %$path,
            status = %$status,
            component = "api",
            $($key = %$value),*,
            "API request processed"
        );
    };
}

/// Macro for Git operation logging
#[macro_export]
macro_rules! log_git_operation {
    ($operation:expr, $repository:expr) => {
        tracing::debug!(
            operation = %$operation,
            repository = %$repository,
            component = "git",
            "Git operation"
        );
    };
    ($operation:expr, $repository:expr, $($key:tt = $value:expr),*) => {
        tracing::debug!(
            operation = %$operation,
            repository = %$repository,
            component = "git",
            $($key = %$value),*,
            "Git operation"
        );
    };
}

/// Utility functions for log validation
pub fn validate_log_level(level: &str) -> Result<()> {
    match level.to_lowercase().as_str() {
        "trace" | "debug" | "info" | "warn" | "error" => Ok(()),
        _ => Err(Error::validation(
            "log_level",
            "Must be one of: trace, debug, info, warn, error",
        )),
    }
}

pub fn validate_log_format(format: &str) -> Result<()> {
    match format {
        "json" | "pretty" | "compact" => Ok(()),
        _ => Err(Error::validation(
            "log_format",
            "Must be one of: json, pretty, compact",
        )),
    }
}

/// Get global alerting manager for use in other modules
pub async fn get_global_alerting_manager() -> Option<LogAlertingManager> {
    LOG_ALERTING_MANAGER.read().await.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::FutureExt;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_init_default_tracing() {
        let result = init_default_tracing().await;
        assert!(result.is_ok());

        if let Ok(system) = result {
            let shutdown_result = system.shutdown().await;
            assert!(shutdown_result.is_ok());
        }
    }

    #[test]
    fn test_validate_log_level() {
        assert!(validate_log_level("trace").is_ok());
        assert!(validate_log_level("debug").is_ok());
        assert!(validate_log_level("info").is_ok());
        assert!(validate_log_level("warn").is_ok());
        assert!(validate_log_level("error").is_ok());
        assert!(validate_log_level("INFO").is_ok());
        assert!(validate_log_level("invalid").is_err());
    }

    #[test]
    fn test_validate_log_format() {
        assert!(validate_log_format("json").is_ok());
        assert!(validate_log_format("pretty").is_ok());
        assert!(validate_log_format("compact").is_ok());
        assert!(validate_log_format("invalid").is_err());
    }

    #[tokio::test]
    async fn test_tracing_system_with_features() {
        let config = LoggingConfig {
            level: "debug".to_string(),
            format: "json".to_string(),
            file: None,
            opentelemetry: None, // Keep disabled for now
            aggregation: Some(LogAggregationConfig {
                enabled: false, // Disabled for test
                syslog_endpoint: None,
                rotation: crate::config::LogRotationConfig {
                    max_size_mb: 10,
                    max_files: 5,
                    schedule: "hourly".to_string(),
                },
                parsing: crate::config::LogParsingConfig {
                    enabled: true,
                    indexed_fields: vec!["level".to_string(), "message".to_string()],
                    enrichment: false,
                },
            }),
            alerting: Some(LogAlertingConfig {
                enabled: false, // Disabled for test
                rules: vec![],
                channels: vec![],
            }),
        };

        let result = std::panic::AssertUnwindSafe(TracingSystem::initialize(&config))
            .catch_unwind()
            .await;
        match result {
            Ok(Ok(system)) => {
                let shutdown_result = system.shutdown().await;
                assert!(shutdown_result.is_ok());
            }
            Ok(Err(e)) => {
                if !format!("{e}").contains("global default trace dispatcher") {
                    panic!("{e}");
                }
            }
            Err(_) => {
                // tracing already initialized; ignore
            }
        }
    }

    #[test]
    fn test_tracing_macros() {
        use crate::{log_context, timed_operation};

        let _span = log_context!("test_operation");
        let result: Result<i32> = timed_operation!("test_timed", { Ok(42) });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_log_alerting_manager() {
        let config = LogAlertingConfig {
            enabled: true,
            rules: vec![LogAlertRule {
                name: "test_rule".to_string(),
                level: "error".to_string(),
                pattern: Some("TEST".to_string()),
                window_minutes: 1,
                threshold: 1,
                severity: "high".to_string(),
            }],
            channels: vec![AlertChannel {
                channel_type: "webhook".to_string(),
                config: {
                    let mut config = HashMap::new();
                    config.insert(
                        "url".to_string(),
                        "http://localhost:8080/webhook".to_string(),
                    );
                    config
                },
            }],
        };

        let manager = LogAlertingManager::new(&config).unwrap();
        let result = manager.process_log_event("error", "TEST message").await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_file_appender_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();

        let result = create_file_appender(&file_path);
        assert!(result.is_ok());
    }
}
