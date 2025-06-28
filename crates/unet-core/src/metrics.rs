//! Metrics and monitoring for μNet Core
//!
//! This module provides Prometheus metrics collection and custom business metrics
//! for monitoring the health and performance of the μNet system.

use crate::config::MetricsConfig;
use crate::error::{Error, Result};
use prometheus::{
    Counter as PrometheusCounter, Encoder, Gauge as PrometheusGauge,
    Histogram as PrometheusHistogram, HistogramOpts, Opts, Registry, TextEncoder, default_registry,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use sysinfo::System;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Metrics manager for Prometheus integration and custom metrics
#[derive(Clone)]
pub struct MetricsManager {
    config: MetricsConfig,
    registry: Arc<Registry>,
    /// HTTP request metrics
    http_requests_total: Arc<PrometheusCounter>,
    http_request_duration: Arc<PrometheusHistogram>,
    /// Database metrics
    database_query_duration: Arc<PrometheusHistogram>,
    database_connections_active: Arc<PrometheusGauge>,
    database_connections_total: Arc<PrometheusGauge>,
    /// Business metrics
    nodes_total: Arc<PrometheusGauge>,
    users_total: Arc<PrometheusGauge>,
    policy_evaluations_total: Arc<PrometheusCounter>,
    template_renderings_total: Arc<PrometheusCounter>,
    auth_attempts_total: Arc<PrometheusCounter>,
    auth_failures_total: Arc<PrometheusCounter>,
    git_sync_operations_total: Arc<PrometheusCounter>,
    snmp_queries_total: Arc<PrometheusCounter>,
    config_changes_total: Arc<PrometheusCounter>,
    active_connections: Arc<PrometheusGauge>,
    /// System metrics
    memory_usage_bytes: Arc<PrometheusGauge>,
    cpu_usage_percent: Arc<PrometheusGauge>,
    disk_usage_bytes: Arc<PrometheusGauge>,
    background_tasks_active: Arc<PrometheusGauge>,
    /// Performance tracking state
    performance_data: Arc<RwLock<PerformanceTracker>>,
}

/// Metrics snapshot for API responses
#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    /// Timestamp of the snapshot
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Business metrics snapshot
    pub business: BusinessMetricsSnapshot,
    /// Performance metrics snapshot
    pub performance: PerformanceMetricsSnapshot,
    /// System metrics snapshot
    pub system: SystemMetricsSnapshot,
}

/// Business metrics snapshot
#[derive(Debug, Serialize, Deserialize)]
pub struct BusinessMetricsSnapshot {
    pub http_requests_total: f64,
    pub nodes_total: f64,
    pub users_total: f64,
    pub policy_evaluations_total: f64,
    pub template_renderings_total: f64,
    pub git_sync_operations_total: f64,
    pub auth_attempts_total: f64,
    pub auth_failures_total: f64,
    pub snmp_queries_total: f64,
    pub config_changes_total: f64,
}

/// Performance metrics snapshot
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceMetricsSnapshot {
    pub http_request_duration_avg: f64,
    pub database_query_duration_avg: f64,
    pub policy_evaluation_duration_avg: f64,
    pub template_rendering_duration_avg: f64,
    pub snmp_query_duration_avg: f64,
    pub git_sync_duration_avg: f64,
    pub background_task_duration_avg: f64,
}

/// System metrics snapshot
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemMetricsSnapshot {
    pub cpu_usage_percent: f64,
    pub memory_usage_bytes: f64,
    pub memory_usage_percent: f64,
    pub disk_usage_bytes: f64,
    pub disk_usage_percent: f64,
    pub active_connections: f64,
    pub database_pool_size: f64,
    pub database_active_connections: f64,
    pub background_tasks_active: f64,
}

/// Performance tracking data
#[derive(Debug, Default)]
struct PerformanceTracker {
    /// HTTP request durations (rolling window)
    http_durations: Vec<Duration>,
    /// Database query durations (rolling window)
    db_durations: Vec<Duration>,
    /// Policy evaluation durations (rolling window)
    policy_durations: Vec<Duration>,
    /// Template rendering durations (rolling window)
    template_durations: Vec<Duration>,
    /// SNMP query durations (rolling window)
    snmp_durations: Vec<Duration>,
    /// Git sync durations (rolling window)
    git_durations: Vec<Duration>,
    /// Background task durations (rolling window)
    background_durations: Vec<Duration>,
    /// System information handle
    system: System,
}

const PERFORMANCE_WINDOW_SIZE: usize = 1000;

impl MetricsManager {
    /// Creates a new metrics manager with the given configuration
    pub fn new(config: MetricsConfig) -> Result<Self> {
        if !config.enabled {
            info!("Metrics collection is disabled");
            // Return minimal manager for disabled metrics
            return Ok(Self::new_disabled(config));
        }

        let registry = Arc::new(Registry::new());

        // Create HTTP metrics
        let http_requests_total = Arc::new(
            PrometheusCounter::new("unet_http_requests_total", "Total number of HTTP requests")
                .map_err(|e| {
                    Error::metrics_with_source(
                        "counter_creation",
                        "Failed to create HTTP requests counter",
                        e,
                    )
                })?,
        );

        let http_request_duration = Arc::new(
            PrometheusHistogram::with_opts(
                HistogramOpts::new(
                    "unet_http_request_duration_seconds",
                    "HTTP request duration in seconds",
                )
                .buckets(vec![
                    0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
                ]),
            )
            .map_err(|e| {
                Error::metrics_with_source(
                    "histogram_creation",
                    "Failed to create HTTP duration histogram",
                    e,
                )
            })?,
        );

        // Create database metrics
        let database_query_duration = Arc::new(
            PrometheusHistogram::with_opts(
                HistogramOpts::new(
                    "unet_database_query_duration_seconds",
                    "Database query duration in seconds",
                )
                .buckets(vec![
                    0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0,
                ]),
            )
            .map_err(|e| {
                Error::metrics_with_source(
                    "histogram_creation",
                    "Failed to create database duration histogram",
                    e,
                )
            })?,
        );

        let database_connections_active = Arc::new(
            PrometheusGauge::new(
                "unet_database_connections_active",
                "Active database connections",
            )
            .map_err(|e| {
                Error::metrics_with_source(
                    "gauge_creation",
                    "Failed to create database connections gauge",
                    e,
                )
            })?,
        );

        let database_connections_total = Arc::new(
            PrometheusGauge::new(
                "unet_database_connections_total",
                "Total database connection pool size",
            )
            .map_err(|e| {
                Error::metrics_with_source(
                    "gauge_creation",
                    "Failed to create database pool gauge",
                    e,
                )
            })?,
        );

        // Create business metrics
        let nodes_total = Arc::new(
            PrometheusGauge::new("unet_nodes_total", "Total number of nodes").map_err(|e| {
                Error::metrics_with_source("gauge_creation", "Failed to create nodes gauge", e)
            })?,
        );

        let users_total = Arc::new(
            PrometheusGauge::new("unet_users_total", "Total number of users").map_err(|e| {
                Error::metrics_with_source("gauge_creation", "Failed to create users gauge", e)
            })?,
        );

        let policy_evaluations_total = Arc::new(
            PrometheusCounter::new(
                "unet_policy_evaluations_total",
                "Total number of policy evaluations",
            )
            .map_err(|e| {
                Error::metrics_with_source(
                    "counter_creation",
                    "Failed to create policy evaluations counter",
                    e,
                )
            })?,
        );

        let template_renderings_total = Arc::new(
            PrometheusCounter::new(
                "unet_template_renderings_total",
                "Total number of template renderings",
            )
            .map_err(|e| {
                Error::metrics_with_source(
                    "counter_creation",
                    "Failed to create template renderings counter",
                    e,
                )
            })?,
        );

        let auth_attempts_total = Arc::new(
            PrometheusCounter::new(
                "unet_auth_attempts_total",
                "Total number of authentication attempts",
            )
            .map_err(|e| {
                Error::metrics_with_source(
                    "counter_creation",
                    "Failed to create auth attempts counter",
                    e,
                )
            })?,
        );

        let auth_failures_total = Arc::new(
            PrometheusCounter::new(
                "unet_auth_failures_total",
                "Total number of authentication failures",
            )
            .map_err(|e| {
                Error::metrics_with_source(
                    "counter_creation",
                    "Failed to create auth failures counter",
                    e,
                )
            })?,
        );

        let git_sync_operations_total = Arc::new(
            PrometheusCounter::new(
                "unet_git_sync_operations_total",
                "Total number of Git sync operations",
            )
            .map_err(|e| {
                Error::metrics_with_source(
                    "counter_creation",
                    "Failed to create git sync operations counter",
                    e,
                )
            })?,
        );

        let snmp_queries_total = Arc::new(
            PrometheusCounter::new(
                "unet_snmp_queries_total",
                "Total number of SNMP queries executed",
            )
            .map_err(|e| {
                Error::metrics_with_source(
                    "counter_creation",
                    "Failed to create SNMP queries counter",
                    e,
                )
            })?,
        );

        let config_changes_total = Arc::new(
            PrometheusCounter::new(
                "unet_config_changes_total",
                "Total number of configuration changes",
            )
            .map_err(|e| {
                Error::metrics_with_source(
                    "counter_creation",
                    "Failed to create config changes counter",
                    e,
                )
            })?,
        );

        let active_connections = Arc::new(
            PrometheusGauge::new(
                "unet_active_connections",
                "Number of active network connections",
            )
            .map_err(|e| {
                Error::metrics_with_source(
                    "gauge_creation",
                    "Failed to create active connections gauge",
                    e,
                )
            })?,
        );

        // Create system metrics
        let memory_usage_bytes = Arc::new(
            PrometheusGauge::new("unet_memory_usage_bytes", "Memory usage in bytes").map_err(
                |e| {
                    Error::metrics_with_source("gauge_creation", "Failed to create memory gauge", e)
                },
            )?,
        );

        let cpu_usage_percent = Arc::new(
            PrometheusGauge::new("unet_cpu_usage_percent", "CPU usage percentage").map_err(
                |e| Error::metrics_with_source("gauge_creation", "Failed to create CPU gauge", e),
            )?,
        );

        let disk_usage_bytes = Arc::new(
            PrometheusGauge::new("unet_disk_usage_bytes", "Disk usage in bytes").map_err(|e| {
                Error::metrics_with_source("gauge_creation", "Failed to create disk gauge", e)
            })?,
        );

        let background_tasks_active = Arc::new(
            PrometheusGauge::new(
                "unet_background_tasks_active",
                "Number of active background tasks",
            )
            .map_err(|e| {
                Error::metrics_with_source(
                    "gauge_creation",
                    "Failed to create background tasks gauge",
                    e,
                )
            })?,
        );

        // Register all metrics
        registry
            .register(Box::new((*http_requests_total).clone()))
            .map_err(|e| {
                Error::metrics_with_source(
                    "registration",
                    "Failed to register HTTP requests counter",
                    e,
                )
            })?;
        registry
            .register(Box::new((*http_request_duration).clone()))
            .map_err(|e| {
                Error::metrics_with_source(
                    "registration",
                    "Failed to register HTTP duration histogram",
                    e,
                )
            })?;
        registry
            .register(Box::new((*database_query_duration).clone()))
            .map_err(|e| {
                Error::metrics_with_source(
                    "registration",
                    "Failed to register database duration histogram",
                    e,
                )
            })?;
        registry
            .register(Box::new((*database_connections_active).clone()))
            .map_err(|e| {
                Error::metrics_with_source(
                    "registration",
                    "Failed to register database connections gauge",
                    e,
                )
            })?;
        registry
            .register(Box::new((*database_connections_total).clone()))
            .map_err(|e| {
                Error::metrics_with_source(
                    "registration",
                    "Failed to register database pool gauge",
                    e,
                )
            })?;
        registry
            .register(Box::new((*nodes_total).clone()))
            .map_err(|e| {
                Error::metrics_with_source("registration", "Failed to register nodes gauge", e)
            })?;
        registry
            .register(Box::new((*policy_evaluations_total).clone()))
            .map_err(|e| {
                Error::metrics_with_source(
                    "registration",
                    "Failed to register policy evaluations counter",
                    e,
                )
            })?;
        registry
            .register(Box::new((*template_renderings_total).clone()))
            .map_err(|e| {
                Error::metrics_with_source(
                    "registration",
                    "Failed to register template renderings counter",
                    e,
                )
            })?;
        registry
            .register(Box::new((*auth_attempts_total).clone()))
            .map_err(|e| {
                Error::metrics_with_source(
                    "registration",
                    "Failed to register auth attempts counter",
                    e,
                )
            })?;
        registry
            .register(Box::new((*auth_failures_total).clone()))
            .map_err(|e| {
                Error::metrics_with_source(
                    "registration",
                    "Failed to register auth failures counter",
                    e,
                )
            })?;
        registry
            .register(Box::new((*users_total).clone()))
            .map_err(|e| {
                Error::metrics_with_source("registration", "Failed to register users gauge", e)
            })?;
        registry
            .register(Box::new((*git_sync_operations_total).clone()))
            .map_err(|e| {
                Error::metrics_with_source(
                    "registration",
                    "Failed to register git sync operations counter",
                    e,
                )
            })?;
        registry
            .register(Box::new((*snmp_queries_total).clone()))
            .map_err(|e| {
                Error::metrics_with_source(
                    "registration",
                    "Failed to register SNMP queries counter",
                    e,
                )
            })?;
        registry
            .register(Box::new((*config_changes_total).clone()))
            .map_err(|e| {
                Error::metrics_with_source(
                    "registration",
                    "Failed to register config changes counter",
                    e,
                )
            })?;
        registry
            .register(Box::new((*active_connections).clone()))
            .map_err(|e| {
                Error::metrics_with_source(
                    "registration",
                    "Failed to register active connections gauge",
                    e,
                )
            })?;
        registry
            .register(Box::new((*memory_usage_bytes).clone()))
            .map_err(|e| {
                Error::metrics_with_source("registration", "Failed to register memory gauge", e)
            })?;
        registry
            .register(Box::new((*cpu_usage_percent).clone()))
            .map_err(|e| {
                Error::metrics_with_source("registration", "Failed to register CPU gauge", e)
            })?;
        registry
            .register(Box::new((*disk_usage_bytes).clone()))
            .map_err(|e| {
                Error::metrics_with_source("registration", "Failed to register disk gauge", e)
            })?;
        registry
            .register(Box::new((*background_tasks_active).clone()))
            .map_err(|e| {
                Error::metrics_with_source(
                    "registration",
                    "Failed to register background tasks gauge",
                    e,
                )
            })?;

        let manager = Self {
            config: config.clone(),
            registry: registry.clone(),
            http_requests_total,
            http_request_duration,
            database_query_duration,
            database_connections_active,
            database_connections_total,
            nodes_total,
            users_total,
            policy_evaluations_total,
            template_renderings_total,
            auth_attempts_total,
            auth_failures_total,
            git_sync_operations_total,
            snmp_queries_total,
            config_changes_total,
            active_connections,
            memory_usage_bytes,
            cpu_usage_percent,
            disk_usage_bytes,
            background_tasks_active,
            performance_data: Arc::new(RwLock::new(PerformanceTracker::default())),
        };

        info!(
            "Metrics collection initialized with Prometheus registry and {} metrics",
            registry.gather().len()
        );
        Ok(manager)
    }

    /// Create a minimal disabled manager
    fn new_disabled(config: MetricsConfig) -> Self {
        let registry = Arc::new(Registry::new());
        let dummy_counter = Arc::new(PrometheusCounter::new("disabled", "disabled").unwrap());
        let dummy_gauge = Arc::new(PrometheusGauge::new("disabled", "disabled").unwrap());
        let dummy_histogram = Arc::new(
            PrometheusHistogram::with_opts(HistogramOpts::new("disabled", "disabled")).unwrap(),
        );

        Self {
            config,
            registry,
            http_requests_total: dummy_counter.clone(),
            http_request_duration: dummy_histogram.clone(),
            database_query_duration: dummy_histogram.clone(),
            database_connections_active: dummy_gauge.clone(),
            database_connections_total: dummy_gauge.clone(),
            nodes_total: dummy_gauge.clone(),
            users_total: dummy_gauge.clone(),
            policy_evaluations_total: dummy_counter.clone(),
            template_renderings_total: dummy_counter.clone(),
            auth_attempts_total: dummy_counter.clone(),
            auth_failures_total: dummy_counter.clone(),
            git_sync_operations_total: dummy_counter.clone(),
            snmp_queries_total: dummy_counter.clone(),
            config_changes_total: dummy_counter.clone(),
            active_connections: dummy_gauge.clone(),
            memory_usage_bytes: dummy_gauge.clone(),
            cpu_usage_percent: dummy_gauge.clone(),
            disk_usage_bytes: dummy_gauge.clone(),
            background_tasks_active: dummy_gauge.clone(),
            performance_data: Arc::new(RwLock::new(PerformanceTracker::default())),
        }
    }

    /// Get the Prometheus metrics output in text format
    pub fn get_prometheus_metrics(&self) -> Result<String> {
        if !self.config.enabled {
            return Ok("# Metrics collection is disabled\n".to_string());
        }

        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();

        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).map_err(|e| {
            Error::metrics_with_source("encoding", "Failed to encode Prometheus metrics", e)
        })?;

        String::from_utf8(buffer).map_err(|e| {
            Error::metrics_with_source("utf8_conversion", "Failed to convert metrics to UTF-8", e)
        })
    }

    /// Increment HTTP request counter
    pub async fn increment_http_requests(&self) {
        if self.config.enabled && self.config.enable_business_metrics {
            self.http_requests_total.inc();
            debug!("HTTP request counter incremented");
        }
    }

    /// Record HTTP request duration
    pub async fn record_http_request_duration(&self, duration: Duration) {
        if self.config.enabled && self.config.enable_performance_metrics {
            self.http_request_duration.observe(duration.as_secs_f64());

            // Update performance tracking
            let mut tracker = self.performance_data.write().await;
            tracker.http_durations.push(duration);
            if tracker.http_durations.len() > PERFORMANCE_WINDOW_SIZE {
                tracker.http_durations.remove(0);
            }

            debug!("HTTP request duration recorded: {:?}", duration);
        }
    }

    /// Record database query duration
    pub async fn record_database_query_duration(&self, duration: Duration) {
        if self.config.enabled && self.config.enable_performance_metrics {
            self.database_query_duration.observe(duration.as_secs_f64());

            // Update performance tracking
            let mut tracker = self.performance_data.write().await;
            tracker.db_durations.push(duration);
            if tracker.db_durations.len() > PERFORMANCE_WINDOW_SIZE {
                tracker.db_durations.remove(0);
            }

            debug!("Database query duration recorded: {:?}", duration);
        }
    }

    /// Record policy evaluation duration
    pub async fn record_policy_evaluation_duration(&self, duration: Duration) {
        if self.config.enabled && self.config.enable_performance_metrics {
            self.policy_evaluations_total.inc();

            // Update performance tracking
            let mut tracker = self.performance_data.write().await;
            tracker.policy_durations.push(duration);
            if tracker.policy_durations.len() > PERFORMANCE_WINDOW_SIZE {
                tracker.policy_durations.remove(0);
            }

            debug!("Policy evaluation duration recorded: {:?}", duration);
        }
    }

    /// Record template rendering duration
    pub async fn record_template_rendering_duration(&self, duration: Duration) {
        if self.config.enabled && self.config.enable_performance_metrics {
            self.template_renderings_total.inc();

            // Update performance tracking
            let mut tracker = self.performance_data.write().await;
            tracker.template_durations.push(duration);
            if tracker.template_durations.len() > PERFORMANCE_WINDOW_SIZE {
                tracker.template_durations.remove(0);
            }

            debug!("Template rendering duration recorded: {:?}", duration);
        }
    }

    /// Update database connection metrics
    pub async fn update_database_connections(&self, active: u64, total: u64) {
        if self.config.enabled && self.config.enable_system_metrics {
            self.database_connections_active.set(active as f64);
            self.database_connections_total.set(total as f64);
            debug!(
                "Database connections updated: active={}, total={}",
                active, total
            );
        }
    }

    /// Update node count metric
    pub async fn update_nodes_total(&self, count: u64) {
        if self.config.enabled && self.config.enable_business_metrics {
            self.nodes_total.set(count as f64);
            debug!("Nodes total updated: {}", count);
        }
    }

    /// Record authentication attempt
    pub async fn record_auth_attempt(&self, success: bool) {
        if self.config.enabled && self.config.enable_business_metrics {
            self.auth_attempts_total.inc();
            if !success {
                self.auth_failures_total.inc();
            }
            debug!("Auth attempt recorded: success={}", success);
        }
    }

    /// Update background tasks count
    pub async fn update_background_tasks_active(&self, count: u64) {
        if self.config.enabled && self.config.enable_system_metrics {
            self.background_tasks_active.set(count as f64);
            debug!("Background tasks active updated: {}", count);
        }
    }

    /// Update user count metric
    pub async fn update_users_total(&self, count: u64) {
        if self.config.enabled && self.config.enable_business_metrics {
            self.users_total.set(count as f64);
            debug!("Users total updated: {}", count);
        }
    }

    /// Record Git sync operation
    pub async fn record_git_sync_operation(&self) {
        if self.config.enabled && self.config.enable_business_metrics {
            self.git_sync_operations_total.inc();
            debug!("Git sync operation recorded");
        }
    }

    /// Record Git sync operation with duration tracking
    pub async fn record_git_sync_duration(&self, duration: Duration) {
        if self.config.enabled && self.config.enable_performance_metrics {
            self.record_git_sync_operation().await;

            // Update performance tracking
            let mut tracker = self.performance_data.write().await;
            tracker.git_durations.push(duration);
            if tracker.git_durations.len() > PERFORMANCE_WINDOW_SIZE {
                tracker.git_durations.remove(0);
            }

            debug!("Git sync duration recorded: {:?}", duration);
        }
    }

    /// Record SNMP query
    pub async fn record_snmp_query(&self) {
        if self.config.enabled && self.config.enable_business_metrics {
            self.snmp_queries_total.inc();
            debug!("SNMP query recorded");
        }
    }

    /// Record SNMP query with duration tracking
    pub async fn record_snmp_query_duration(&self, duration: Duration) {
        if self.config.enabled && self.config.enable_performance_metrics {
            self.record_snmp_query().await;

            // Update performance tracking
            let mut tracker = self.performance_data.write().await;
            tracker.snmp_durations.push(duration);
            if tracker.snmp_durations.len() > PERFORMANCE_WINDOW_SIZE {
                tracker.snmp_durations.remove(0);
            }

            debug!("SNMP query duration recorded: {:?}", duration);
        }
    }

    /// Record configuration change
    pub async fn record_config_change(&self) {
        if self.config.enabled && self.config.enable_business_metrics {
            self.config_changes_total.inc();
            debug!("Configuration change recorded");
        }
    }

    /// Update active connections count
    pub async fn update_active_connections(&self, count: u64) {
        if self.config.enabled && self.config.enable_system_metrics {
            self.active_connections.set(count as f64);
            debug!("Active connections updated: {}", count);
        }
    }

    /// Record background task duration
    pub async fn record_background_task_duration(&self, duration: Duration) {
        if self.config.enabled && self.config.enable_performance_metrics {
            // Update performance tracking
            let mut tracker = self.performance_data.write().await;
            tracker.background_durations.push(duration);
            if tracker.background_durations.len() > PERFORMANCE_WINDOW_SIZE {
                tracker.background_durations.remove(0);
            }

            debug!("Background task duration recorded: {:?}", duration);
        }
    }

    /// Update system metrics with real system information
    pub async fn update_system_metrics(&self) -> Result<()> {
        if !self.config.enabled || !self.config.enable_system_metrics {
            return Ok(());
        }

        let mut tracker = self.performance_data.write().await;

        // Refresh system information
        tracker.system.refresh_all();

        // Update CPU usage
        let cpu_usage = tracker.system.global_cpu_info().cpu_usage() as f64;
        self.cpu_usage_percent.set(cpu_usage);

        // Update memory usage
        let memory_used = tracker.system.used_memory();
        let memory_total = tracker.system.total_memory();
        let memory_usage_percent = if memory_total > 0 {
            (memory_used as f64 / memory_total as f64) * 100.0
        } else {
            0.0
        };

        self.memory_usage_bytes.set(memory_used as f64);

        // Update disk usage - simplified for now
        // TODO: Implement proper disk usage monitoring when sysinfo API is clarified
        self.disk_usage_bytes.set(0.0);

        debug!(
            "System metrics updated: CPU={:.1}%, Memory={:.1}%, Disk={}MB",
            cpu_usage,
            memory_usage_percent,
            memory_used / 1024 / 1024
        );

        Ok(())
    }

    /// Get a metrics snapshot for API responses
    pub async fn get_metrics_snapshot(&self) -> Result<MetricsSnapshot> {
        if !self.config.enabled {
            return Err(Error::metrics(
                "snapshot_disabled",
                "Metrics collection is disabled",
            ));
        }

        let tracker = self.performance_data.read().await;

        // Calculate averages from performance tracking
        let http_avg = Self::calculate_average(&tracker.http_durations);
        let db_avg = Self::calculate_average(&tracker.db_durations);
        let policy_avg = Self::calculate_average(&tracker.policy_durations);
        let template_avg = Self::calculate_average(&tracker.template_durations);
        let snmp_avg = Self::calculate_average(&tracker.snmp_durations);
        let git_avg = Self::calculate_average(&tracker.git_durations);
        let background_avg = Self::calculate_average(&tracker.background_durations);

        // Get current metric values
        let http_requests = self.http_requests_total.get();
        let nodes_count = self.nodes_total.get();
        let users_count = self.users_total.get();
        let policy_evaluations = self.policy_evaluations_total.get();
        let template_renderings = self.template_renderings_total.get();
        let auth_attempts = self.auth_attempts_total.get();
        let auth_failures = self.auth_failures_total.get();
        let git_sync_ops = self.git_sync_operations_total.get();
        let snmp_queries = self.snmp_queries_total.get();
        let config_changes = self.config_changes_total.get();

        let cpu_usage = self.cpu_usage_percent.get();
        let memory_usage = self.memory_usage_bytes.get();
        let disk_usage = self.disk_usage_bytes.get();
        let db_active = self.database_connections_active.get();
        let db_total = self.database_connections_total.get();
        let bg_tasks = self.background_tasks_active.get();
        let active_conns = self.active_connections.get();

        // Calculate memory usage percentage
        let memory_total = tracker.system.total_memory() as f64;
        let memory_percent = if memory_total > 0.0 {
            (memory_usage / memory_total) * 100.0
        } else {
            0.0
        };

        // Calculate disk usage percentage - simplified for now
        let disk_percent = 0.0;

        Ok(MetricsSnapshot {
            timestamp: chrono::Utc::now(),
            business: BusinessMetricsSnapshot {
                http_requests_total: http_requests,
                nodes_total: nodes_count,
                users_total: users_count,
                policy_evaluations_total: policy_evaluations,
                template_renderings_total: template_renderings,
                git_sync_operations_total: git_sync_ops,
                auth_attempts_total: auth_attempts,
                auth_failures_total: auth_failures,
                snmp_queries_total: snmp_queries,
                config_changes_total: config_changes,
            },
            performance: PerformanceMetricsSnapshot {
                http_request_duration_avg: http_avg,
                database_query_duration_avg: db_avg,
                policy_evaluation_duration_avg: policy_avg,
                template_rendering_duration_avg: template_avg,
                snmp_query_duration_avg: snmp_avg,
                git_sync_duration_avg: git_avg,
                background_task_duration_avg: background_avg,
            },
            system: SystemMetricsSnapshot {
                cpu_usage_percent: cpu_usage,
                memory_usage_bytes: memory_usage,
                memory_usage_percent: memory_percent,
                disk_usage_bytes: disk_usage,
                disk_usage_percent: disk_percent,
                active_connections: active_conns,
                database_pool_size: db_total,
                database_active_connections: db_active,
                background_tasks_active: bg_tasks,
            },
        })
    }

    /// Calculate average duration from a vector of durations
    fn calculate_average(durations: &[Duration]) -> f64 {
        if durations.is_empty() {
            return 0.0;
        }

        let total_nanos: u128 = durations.iter().map(|d| d.as_nanos()).sum();
        let avg_nanos = total_nanos / durations.len() as u128;
        avg_nanos as f64 / 1_000_000_000.0 // Convert to seconds
    }

    /// Start background metrics collection
    pub fn start_background_collection(&self) {
        if !self.config.enabled {
            return;
        }

        let manager = self.clone();
        let interval = std::time::Duration::from_secs(self.config.collection_interval);

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            info!(
                "Starting background metrics collection with {}s interval",
                manager.config.collection_interval
            );

            loop {
                interval_timer.tick().await;

                if let Err(e) = manager.update_system_metrics().await {
                    warn!("Failed to update system metrics: {}", e);
                }
            }
        });
    }

    /// Check if metrics are enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get metrics configuration
    pub fn get_config(&self) -> &MetricsConfig {
        &self.config
    }

    /// Record a custom business event (increments counter)
    /// Use this for tracking custom business logic events
    pub async fn record_custom_business_event(
        &self,
        event_name: &str,
        labels: Option<&HashMap<String, String>>,
    ) {
        if self.config.enabled && self.config.enable_business_metrics {
            debug!(
                "Custom business event recorded: {} (labels: {:?})",
                event_name, labels
            );
            // Note: For dynamic custom metrics, consider using a dynamic metric registry
            // This is a placeholder for custom business event tracking
        }
    }

    /// Set a custom business gauge value
    /// Use this for tracking custom business metrics that represent current state
    pub async fn set_custom_business_gauge(
        &self,
        metric_name: &str,
        value: f64,
        labels: Option<&HashMap<String, String>>,
    ) {
        if self.config.enabled && self.config.enable_business_metrics {
            debug!(
                "Custom business gauge set: {} = {} (labels: {:?})",
                metric_name, value, labels
            );
            // Note: For dynamic custom metrics, consider using a dynamic metric registry
            // This is a placeholder for custom business gauge tracking
        }
    }

    /// Record a workflow step completion
    pub async fn record_workflow_step(
        &self,
        workflow: &str,
        step: &str,
        success: bool,
        duration: Duration,
    ) {
        if self.config.enabled && self.config.enable_business_metrics {
            debug!(
                "Workflow step recorded: {}::{} (success: {}, duration: {:?})",
                workflow, step, success, duration
            );
            // This could be used for tracking complex business workflows
        }
    }

    /// Track user activity metrics
    pub async fn record_user_activity(&self, activity_type: &str, user_id: Option<&str>) {
        if self.config.enabled && self.config.enable_business_metrics {
            debug!(
                "User activity recorded: {} (user: {:?})",
                activity_type, user_id
            );
            // Track different types of user activities for business analytics
        }
    }

    /// Track feature usage metrics
    pub async fn record_feature_usage(
        &self,
        feature: &str,
        context: Option<&HashMap<String, String>>,
    ) {
        if self.config.enabled && self.config.enable_business_metrics {
            debug!(
                "Feature usage recorded: {} (context: {:?})",
                feature, context
            );
            // Track which features are being used for product analytics
        }
    }

    /// Check if the metrics system is operational
    pub async fn is_operational(&self) -> bool {
        if !self.config.enabled {
            // If metrics are disabled, consider it operational
            return true;
        }

        // Try to collect a simple metric to verify the system is working
        match self.registry.gather().is_empty() {
            false => {
                // Registry has metrics, system is operational
                debug!("Metrics system operational - registry contains metrics");
                true
            }
            true => {
                // Registry is empty, this might indicate an issue
                warn!("Metrics system potentially degraded - registry is empty");
                false
            }
        }
    }
}

/// Utility functions for common metric operations
pub mod utils {
    use super::*;

    /// Time a function execution and record the duration
    pub async fn time_function<F, T>(
        metrics_manager: &MetricsManager,
        histogram_name: &str,
        func: F,
    ) -> T
    where
        F: std::future::Future<Output = T>,
    {
        let start = std::time::Instant::now();
        let result = func.await;
        let duration = start.elapsed();

        // Record duration based on histogram name
        match histogram_name {
            "http_request_duration" => {
                metrics_manager.record_http_request_duration(duration).await;
            }
            _ => {
                debug!("Duration recorded for {}: {:?}", histogram_name, duration);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_manager_creation() {
        let config = MetricsConfig {
            enabled: true,
            endpoint: "/metrics".to_string(),
            bind_address: None,
            collection_interval: 15,
            enable_performance_metrics: true,
            enable_business_metrics: true,
            enable_system_metrics: true,
            labels: HashMap::new(),
            retention_days: 7,
        };

        let manager = MetricsManager::new(config).unwrap();
        assert!(manager.is_enabled());
    }

    #[tokio::test]
    async fn test_disabled_metrics() {
        let config = MetricsConfig {
            enabled: false,
            endpoint: "/metrics".to_string(),
            bind_address: None,
            collection_interval: 15,
            enable_performance_metrics: true,
            enable_business_metrics: true,
            enable_system_metrics: true,
            labels: HashMap::new(),
            retention_days: 7,
        };

        let manager = MetricsManager::new(config).unwrap();
        assert!(!manager.is_enabled());

        let metrics_output = manager.get_prometheus_metrics().unwrap();
        assert!(metrics_output.contains("Metrics collection is disabled"));
    }

    #[tokio::test]
    async fn test_prometheus_metrics_output() {
        let config = MetricsConfig {
            enabled: true,
            endpoint: "/metrics".to_string(),
            bind_address: None,
            collection_interval: 15,
            enable_performance_metrics: true,
            enable_business_metrics: true,
            enable_system_metrics: true,
            labels: HashMap::new(),
            retention_days: 7,
        };

        let manager = MetricsManager::new(config).unwrap();

        let metrics_output = manager.get_prometheus_metrics().unwrap();
        assert!(!metrics_output.is_empty());
    }

    #[tokio::test]
    async fn test_metrics_snapshot() {
        let config = MetricsConfig {
            enabled: true,
            endpoint: "/metrics".to_string(),
            bind_address: None,
            collection_interval: 15,
            enable_performance_metrics: true,
            enable_business_metrics: true,
            enable_system_metrics: true,
            labels: HashMap::new(),
            retention_days: 7,
        };

        let manager = MetricsManager::new(config).unwrap();

        let snapshot = manager.get_metrics_snapshot().await.unwrap();
        assert!(snapshot.timestamp <= chrono::Utc::now());
        assert_eq!(snapshot.business.http_requests_total, 0.0);
    }
}
