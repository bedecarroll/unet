//! Load balancer compatibility and health check management
//!
//! This module provides load balancer compatibility features including
//! enhanced health checks, graceful shutdown, and load balancer-specific headers.

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Load balancer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerConfig {
    /// Enable load balancer mode
    pub enabled: bool,
    /// Health check configuration
    pub health_check: HealthCheckConfig,
    /// Graceful shutdown timeout in seconds
    pub shutdown_timeout: u64,
    /// Enable proxy protocol support
    pub proxy_protocol: bool,
    /// Trusted proxy IPs/CIDRs
    pub trusted_proxies: Vec<String>,
    /// Custom health check endpoints
    pub custom_endpoints: Vec<HealthEndpoint>,
    /// Load balancer type hints
    pub balancer_type: LoadBalancerType,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Basic health check path (default: /health)
    pub basic_path: String,
    /// Detailed health check path (default: /health/detailed)  
    pub detailed_path: String,
    /// Readiness check path for Kubernetes (default: /ready)
    pub readiness_path: String,
    /// Liveness check path for Kubernetes (default: /live)
    pub liveness_path: String,
    /// Health check timeout in seconds
    pub timeout: u64,
    /// Include dependency checks in health
    pub include_dependencies: bool,
    /// Health check interval for self-monitoring
    pub check_interval: u64,
    /// Custom health checks
    pub custom_checks: Vec<CustomHealthCheck>,
}

/// Custom health endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthEndpoint {
    /// Endpoint path
    pub path: String,
    /// HTTP method (GET, POST, etc.)
    pub method: String,
    /// Required response status code
    pub expected_status: u16,
    /// Custom response body content
    pub response_body: Option<String>,
    /// Check specific dependencies
    pub dependencies: Vec<String>,
}

/// Load balancer type configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancerType {
    /// Generic HTTP load balancer
    Generic,
    /// AWS Application Load Balancer
    Alb,
    /// AWS Network Load Balancer  
    Nlb,
    /// NGINX load balancer
    Nginx,
    /// HAProxy load balancer
    HaProxy,
    /// Kubernetes Ingress
    KubernetesIngress,
    /// Cloudflare Load Balancing
    Cloudflare,
    /// Custom load balancer configuration
    Custom(HashMap<String, String>),
}

/// Custom health check definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomHealthCheck {
    /// Check name/identifier
    pub name: String,
    /// Check type (database, external_service, cache, etc.)
    pub check_type: String,
    /// Check parameters
    pub parameters: HashMap<String, String>,
    /// Check timeout in seconds
    pub timeout: u64,
    /// Check criticality (critical, warning, info)
    pub criticality: HealthCriticality,
}

/// Health check criticality levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthCriticality {
    /// Critical - failure causes service to be marked unhealthy
    Critical,
    /// Warning - failure causes warning but service remains healthy
    Warning,
    /// Info - failure logged but doesn't affect health status
    Info,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Overall health status
    pub status: HealthStatus,
    /// Individual component results
    pub components: HashMap<String, ComponentHealth>,
    /// Check timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Check duration in milliseconds
    pub duration_ms: u64,
    /// Service metadata
    pub metadata: ServiceMetadata,
}

/// Health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    /// Service is healthy and ready to serve traffic
    Healthy,
    /// Service is starting up and not ready for traffic
    Starting,
    /// Service is degraded but still functional
    Degraded,
    /// Service is unhealthy and should not receive traffic
    Unhealthy,
    /// Service is shutting down gracefully
    ShuttingDown,
}

/// Component health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component status
    pub status: HealthStatus,
    /// Status message
    pub message: Option<String>,
    /// Last check time
    pub last_check: chrono::DateTime<chrono::Utc>,
    /// Check duration
    pub duration_ms: u64,
    /// Component metadata
    pub metadata: HashMap<String, String>,
}

/// Service metadata for health checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetadata {
    /// Service name
    pub service: String,
    /// Service version
    pub version: String,
    /// Build info
    pub build: BuildInfo,
    /// Runtime information
    pub runtime: RuntimeInfo,
    /// Load balancer specific info
    pub load_balancer: LoadBalancerInfo,
}

/// Build information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildInfo {
    /// Git commit hash
    pub commit: String,
    /// Build timestamp
    pub timestamp: String,
    /// Rust version used for build
    pub rust_version: String,
}

/// Runtime information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeInfo {
    /// Server uptime in seconds
    pub uptime_seconds: u64,
    /// Process ID
    pub pid: u32,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Number of active connections
    pub active_connections: u64,
}

/// Load balancer specific information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerInfo {
    /// Load balancer type
    pub balancer_type: String,
    /// Instance ID for tracking
    pub instance_id: String,
    /// Availability zone or region
    pub availability_zone: Option<String>,
    /// Load balancer specific metadata
    pub metadata: HashMap<String, String>,
}

/// Load balancer health manager
pub struct LoadBalancerHealthManager {
    config: LoadBalancerConfig,
    start_time: Instant,
    status: Arc<RwLock<HealthStatus>>,
    component_health: Arc<RwLock<HashMap<String, ComponentHealth>>>,
    custom_checks: Vec<CustomHealthCheck>,
}

impl LoadBalancerHealthManager {
    /// Create new load balancer health manager
    pub fn new(config: LoadBalancerConfig) -> Self {
        let custom_checks = config.health_check.custom_checks.clone();

        Self {
            config,
            start_time: Instant::now(),
            status: Arc::new(RwLock::new(HealthStatus::Starting)),
            component_health: Arc::new(RwLock::new(HashMap::new())),
            custom_checks,
        }
    }

    /// Start health monitoring
    pub async fn start(&self) -> Result<()> {
        info!("Starting load balancer health manager");

        // Initialize component health
        self.initialize_components().await;

        // Set initial status to healthy
        *self.status.write().await = HealthStatus::Healthy;

        // Start background health monitoring if enabled
        if self.config.health_check.check_interval > 0 {
            self.start_background_monitoring().await;
        }

        Ok(())
    }

    /// Initialize component health tracking
    async fn initialize_components(&self) {
        let mut components = self.component_health.write().await;

        // Initialize core components
        components.insert(
            "datastore".to_string(),
            ComponentHealth {
                status: HealthStatus::Healthy,
                message: Some("Database connection ready".to_string()),
                last_check: chrono::Utc::now(),
                duration_ms: 0,
                metadata: HashMap::new(),
            },
        );

        components.insert(
            "metrics".to_string(),
            ComponentHealth {
                status: HealthStatus::Healthy,
                message: Some("Metrics system operational".to_string()),
                last_check: chrono::Utc::now(),
                duration_ms: 0,
                metadata: HashMap::new(),
            },
        );

        // Initialize custom check components
        for check in &self.custom_checks {
            components.insert(
                check.name.clone(),
                ComponentHealth {
                    status: HealthStatus::Starting,
                    message: Some("Initializing custom check".to_string()),
                    last_check: chrono::Utc::now(),
                    duration_ms: 0,
                    metadata: check.parameters.clone(),
                },
            );
        }

        debug!("Initialized {} health components", components.len());
    }

    /// Start background health monitoring
    async fn start_background_monitoring(&self) {
        let status = Arc::clone(&self.status);
        let component_health = Arc::clone(&self.component_health);
        let custom_checks = self.custom_checks.clone();
        let interval = self.config.health_check.check_interval;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(interval));

            loop {
                interval.tick().await;

                debug!("Running background health checks");

                // Run custom health checks
                for check in &custom_checks {
                    if let Err(e) =
                        Self::run_custom_health_check(check, Arc::clone(&component_health)).await
                    {
                        warn!("Custom health check '{}' failed: {}", check.name, e);
                    }
                }

                // Update overall status based on component health
                Self::update_overall_status(Arc::clone(&status), Arc::clone(&component_health))
                    .await;
            }
        });
    }

    /// Run a custom health check
    async fn run_custom_health_check(
        check: &CustomHealthCheck,
        component_health: Arc<RwLock<HashMap<String, ComponentHealth>>>,
    ) -> Result<()> {
        let start = Instant::now();

        // Simulate custom health check logic based on type
        let result = match check.check_type.as_str() {
            "database" => Self::check_database_health(check).await,
            "external_service" => Self::check_external_service_health(check).await,
            "cache" => Self::check_cache_health(check).await,
            "disk_space" => Self::check_disk_space_health(check).await,
            _ => {
                warn!("Unknown health check type: {}", check.check_type);
                Ok(HealthStatus::Degraded)
            }
        };

        let duration = start.elapsed();
        let status = result.unwrap_or(HealthStatus::Unhealthy);

        // Update component health
        let mut components = component_health.write().await;
        components.insert(
            check.name.clone(),
            ComponentHealth {
                status: status.clone(),
                message: Some(format!("Custom check completed with status: {:?}", status)),
                last_check: chrono::Utc::now(),
                duration_ms: duration.as_millis() as u64,
                metadata: check.parameters.clone(),
            },
        );

        Ok(())
    }

    /// Check database health
    async fn check_database_health(check: &CustomHealthCheck) -> Result<HealthStatus> {
        // Simulate database health check
        if check
            .parameters
            .get("simulate_failure")
            .unwrap_or(&"false".to_string())
            == "true"
        {
            return Ok(HealthStatus::Unhealthy);
        }
        Ok(HealthStatus::Healthy)
    }

    /// Check external service health
    async fn check_external_service_health(check: &CustomHealthCheck) -> Result<HealthStatus> {
        // Simulate external service check
        if let Some(url) = check.parameters.get("url") {
            debug!("Checking external service health: {}", url);
            // In a real implementation, this would make an HTTP request
            Ok(HealthStatus::Healthy)
        } else {
            Ok(HealthStatus::Degraded)
        }
    }

    /// Check cache health
    async fn check_cache_health(check: &CustomHealthCheck) -> Result<HealthStatus> {
        // Simulate cache health check
        if check
            .parameters
            .get("cache_type")
            .unwrap_or(&"redis".to_string())
            == "redis"
        {
            debug!("Checking Redis cache health");
            Ok(HealthStatus::Healthy)
        } else {
            Ok(HealthStatus::Degraded)
        }
    }

    /// Check disk space health
    async fn check_disk_space_health(check: &CustomHealthCheck) -> Result<HealthStatus> {
        // Simulate disk space check
        let threshold: f64 = check
            .parameters
            .get("threshold_percent")
            .unwrap_or(&"80".to_string())
            .parse()
            .unwrap_or(80.0);

        // In a real implementation, this would check actual disk usage
        let usage_percent = 65.0; // Simulated usage

        if usage_percent > threshold {
            Ok(HealthStatus::Degraded)
        } else {
            Ok(HealthStatus::Healthy)
        }
    }

    /// Update overall status based on component health
    async fn update_overall_status(
        status: Arc<RwLock<HealthStatus>>,
        component_health: Arc<RwLock<HashMap<String, ComponentHealth>>>,
    ) {
        let components = component_health.read().await;
        let current_status = status.read().await.clone();

        // Don't change status if shutting down
        if current_status == HealthStatus::ShuttingDown {
            return;
        }

        let has_critical_failures = components
            .values()
            .any(|c| c.status == HealthStatus::Unhealthy);

        let has_warnings = components
            .values()
            .any(|c| c.status == HealthStatus::Degraded);

        let new_status = if has_critical_failures {
            HealthStatus::Unhealthy
        } else if has_warnings {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        if new_status != current_status {
            drop(components); // Release read lock
            *status.write().await = new_status.clone();
            info!("Health status changed to: {:?}", new_status);
        }
    }

    /// Get current health check result
    pub async fn get_health_result(&self, detailed: bool) -> HealthCheckResult {
        let start = Instant::now();
        let status = self.status.read().await.clone();
        let components = if detailed {
            self.component_health.read().await.clone()
        } else {
            HashMap::new()
        };

        HealthCheckResult {
            status,
            components,
            timestamp: chrono::Utc::now(),
            duration_ms: start.elapsed().as_millis() as u64,
            metadata: self.get_service_metadata().await,
        }
    }

    /// Get service metadata
    async fn get_service_metadata(&self) -> ServiceMetadata {
        ServiceMetadata {
            service: "μNet".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            build: BuildInfo {
                commit: option_env!("VERGEN_GIT_SHA")
                    .unwrap_or("unknown")
                    .to_string(),
                timestamp: option_env!("VERGEN_BUILD_TIMESTAMP")
                    .unwrap_or("unknown")
                    .to_string(),
                rust_version: option_env!("VERGEN_RUSTC_SEMVER")
                    .unwrap_or("unknown")
                    .to_string(),
            },
            runtime: RuntimeInfo {
                uptime_seconds: self.start_time.elapsed().as_secs(),
                pid: std::process::id(),
                memory_usage_bytes: Self::get_memory_usage(),
                cpu_usage_percent: Self::get_cpu_usage(),
                active_connections: Self::get_active_connections(),
            },
            load_balancer: LoadBalancerInfo {
                balancer_type: format!("{:?}", self.config.balancer_type),
                instance_id: Self::get_instance_id(),
                availability_zone: Self::get_availability_zone(),
                metadata: Self::get_load_balancer_metadata(&self.config.balancer_type),
            },
        }
    }

    /// Get memory usage (placeholder implementation)
    fn get_memory_usage() -> u64 {
        // In a real implementation, this would get actual memory usage
        1024 * 1024 * 64 // 64MB placeholder
    }

    /// Get CPU usage (placeholder implementation)
    fn get_cpu_usage() -> f64 {
        // In a real implementation, this would get actual CPU usage
        15.5 // 15.5% placeholder
    }

    /// Get active connections (placeholder implementation)
    fn get_active_connections() -> u64 {
        // In a real implementation, this would get actual connection count
        25 // 25 connections placeholder
    }

    /// Get instance ID
    fn get_instance_id() -> String {
        std::env::var("INSTANCE_ID")
            .or_else(|_| std::env::var("HOSTNAME"))
            .unwrap_or_else(|_| format!("unet-{}", std::process::id()))
    }

    /// Get availability zone
    fn get_availability_zone() -> Option<String> {
        std::env::var("AVAILABILITY_ZONE")
            .or_else(|_| std::env::var("AWS_AVAILABILITY_ZONE"))
            .ok()
    }

    /// Get load balancer specific metadata
    fn get_load_balancer_metadata(balancer_type: &LoadBalancerType) -> HashMap<String, String> {
        let mut metadata = HashMap::new();

        match balancer_type {
            LoadBalancerType::Alb => {
                metadata.insert("provider".to_string(), "aws".to_string());
                metadata.insert("type".to_string(), "application".to_string());
            }
            LoadBalancerType::Nlb => {
                metadata.insert("provider".to_string(), "aws".to_string());
                metadata.insert("type".to_string(), "network".to_string());
            }
            LoadBalancerType::Nginx => {
                metadata.insert("provider".to_string(), "nginx".to_string());
                metadata.insert("type".to_string(), "reverse_proxy".to_string());
            }
            LoadBalancerType::HaProxy => {
                metadata.insert("provider".to_string(), "haproxy".to_string());
                metadata.insert("type".to_string(), "tcp_http".to_string());
            }
            LoadBalancerType::KubernetesIngress => {
                metadata.insert("provider".to_string(), "kubernetes".to_string());
                metadata.insert("type".to_string(), "ingress".to_string());
            }
            LoadBalancerType::Cloudflare => {
                metadata.insert("provider".to_string(), "cloudflare".to_string());
                metadata.insert("type".to_string(), "edge".to_string());
            }
            LoadBalancerType::Custom(custom_metadata) => {
                metadata.extend(custom_metadata.clone());
            }
            LoadBalancerType::Generic => {
                metadata.insert("provider".to_string(), "generic".to_string());
                metadata.insert("type".to_string(), "http".to_string());
            }
        }

        metadata
    }

    /// Update component health status
    pub async fn update_component_health(
        &self,
        component: &str,
        status: HealthStatus,
        message: Option<String>,
    ) {
        let mut components = self.component_health.write().await;

        let health = ComponentHealth {
            status: status.clone(),
            message,
            last_check: chrono::Utc::now(),
            duration_ms: 0,
            metadata: HashMap::new(),
        };

        components.insert(component.to_string(), health);
        debug!("Updated health for component '{}': {:?}", component, status);
    }

    /// Mark service as shutting down
    pub async fn shutdown(&self) {
        info!("Marking service as shutting down");
        *self.status.write().await = HealthStatus::ShuttingDown;
    }

    /// Check if service is ready for traffic
    pub async fn is_ready(&self) -> bool {
        let status = self.status.read().await;
        matches!(*status, HealthStatus::Healthy | HealthStatus::Degraded)
    }

    /// Check if service is alive
    pub async fn is_alive(&self) -> bool {
        let status = self.status.read().await;
        !matches!(*status, HealthStatus::Unhealthy)
    }
}

impl Default for LoadBalancerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            health_check: HealthCheckConfig::default(),
            shutdown_timeout: 30,
            proxy_protocol: false,
            trusted_proxies: vec!["127.0.0.1".to_string(), "::1".to_string()],
            custom_endpoints: vec![],
            balancer_type: LoadBalancerType::Generic,
        }
    }
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            basic_path: "/health".to_string(),
            detailed_path: "/health/detailed".to_string(),
            readiness_path: "/ready".to_string(),
            liveness_path: "/live".to_string(),
            timeout: 10,
            include_dependencies: true,
            check_interval: 30,
            custom_checks: vec![],
        }
    }
}

impl Default for HealthStatus {
    fn default() -> Self {
        HealthStatus::Starting
    }
}
