//! Enhanced configuration validation for μNet
//!
//! This module provides comprehensive configuration validation including:
//! - Template validation and processing
//! - Environment-specific validation rules  
//! - Cross-field dependency validation
//! - Security compliance checks
//! - Performance recommendation analysis

use crate::config::Config;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Configuration validation context
#[derive(Debug, Clone)]
pub struct ValidationContext {
    /// Environment type (development, staging, production)
    pub environment: String,
    /// Whether this is a production environment
    pub is_production: bool,
    /// Expected deployment type (standalone, cluster, kubernetes)
    pub deployment_type: DeploymentType,
    /// Additional validation rules to apply
    pub strict_mode: bool,
}

/// Deployment type affects validation rules
#[derive(Debug, Clone, PartialEq)]
pub enum DeploymentType {
    /// Single instance deployment
    Standalone,
    /// Multi-node cluster deployment
    Cluster,
    /// Kubernetes deployment
    Kubernetes,
    /// Docker Compose deployment
    DockerCompose,
}

/// Validation result with detailed feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether validation passed
    pub valid: bool,
    /// List of validation errors (blocking issues)
    pub errors: Vec<ValidationError>,
    /// List of validation warnings (non-blocking issues)
    pub warnings: Vec<ValidationWarning>,
    /// Performance and security recommendations
    pub recommendations: Vec<ValidationRecommendation>,
    /// Configuration summary
    pub summary: ConfigurationSummary,
}

/// Validation error details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error category
    pub category: ErrorCategory,
    /// Configuration field path
    pub field: String,
    /// Error message
    pub message: String,
    /// Current value (if applicable)
    pub current_value: Option<String>,
    /// Suggested fix
    pub suggestion: Option<String>,
}

/// Validation warning details  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// Warning category
    pub category: WarningCategory,
    /// Configuration field path
    pub field: String,
    /// Warning message
    pub message: String,
    /// Current value (if applicable)
    pub current_value: Option<String>,
    /// Recommended value or action
    pub recommendation: Option<String>,
}

/// Validation recommendation for optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRecommendation {
    /// Recommendation category
    pub category: RecommendationCategory,
    /// Priority level
    pub priority: RecommendationPriority,
    /// Configuration field or area
    pub area: String,
    /// Recommendation message
    pub message: String,
    /// Detailed explanation
    pub explanation: String,
    /// Implementation steps
    pub steps: Vec<String>,
}

/// Configuration summary for overview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationSummary {
    /// Environment type
    pub environment: String,
    /// Database type (sqlite, postgresql)
    pub database_type: String,
    /// Authentication enabled
    pub authentication_enabled: bool,
    /// TLS/HTTPS enabled
    pub tls_enabled: bool,
    /// Clustering enabled
    pub clustering_enabled: bool,
    /// Monitoring enabled
    pub monitoring_enabled: bool,
    /// Security features enabled
    pub security_features: Vec<String>,
    /// Performance features enabled
    pub performance_features: Vec<String>,
    /// Estimated resource usage
    pub resource_usage: ResourceUsageEstimate,
}

/// Estimated resource usage based on configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageEstimate {
    /// Estimated memory usage in MB
    pub memory_mb: u64,
    /// Estimated CPU cores needed
    pub cpu_cores: f64,
    /// Estimated disk space in MB
    pub disk_mb: u64,
    /// Estimated network connections
    pub max_connections: u32,
}

/// Error categories for classification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// Required field missing
    MissingRequired,
    /// Invalid format or value
    InvalidFormat,
    /// Security vulnerability
    Security,
    /// Performance issue
    Performance,
    /// Compatibility issue
    Compatibility,
    /// Resource limit exceeded
    ResourceLimit,
}

/// Warning categories for classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WarningCategory {
    /// Suboptimal configuration
    Suboptimal,
    /// Deprecated setting
    Deprecated,
    /// Development setting in production
    DevelopmentSetting,
    /// Missing optional feature
    MissingOptional,
    /// Resource inefficiency
    ResourceInefficiency,
    /// Compatibility issue
    Compatibility,
    /// Security issue
    Security,
    /// Performance issue
    Performance,
}

/// Recommendation categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationCategory {
    /// Security improvement
    Security,
    /// Performance optimization
    Performance,
    /// Resource optimization
    ResourceOptimization,
    /// Monitoring improvement
    Monitoring,
    /// High availability
    HighAvailability,
    /// Maintenance
    Maintenance,
}

/// Recommendation priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    /// Low priority - nice to have
    Low,
    /// Medium priority - should implement
    Medium,
    /// High priority - strongly recommended
    High,
    /// Critical priority - implement immediately
    Critical,
}

impl ValidationContext {
    /// Create a new validation context
    pub fn new(environment: &str, deployment_type: DeploymentType) -> Self {
        let is_production = environment.to_lowercase() == "production";
        Self {
            environment: environment.to_string(),
            is_production,
            deployment_type,
            strict_mode: is_production,
        }
    }

    /// Create validation context for production environment
    pub fn production(deployment_type: DeploymentType) -> Self {
        Self::new("production", deployment_type)
    }

    /// Create validation context for staging environment
    pub fn staging(deployment_type: DeploymentType) -> Self {
        Self::new("staging", deployment_type)
    }

    /// Create validation context for development environment
    pub fn development() -> Self {
        Self::new("development", DeploymentType::Standalone)
    }
}

impl Config {
    /// Comprehensive configuration validation with context
    pub fn validate_with_context(&self, context: &ValidationContext) -> ValidationResult {
        let mut result = ValidationResult {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            recommendations: Vec::new(),
            summary: self.generate_summary(context),
        };

        // Core validation (always required)
        self.validate_core(&mut result, context);

        // Environment-specific validation
        self.validate_environment_specific(&mut result, context);

        // Security validation
        self.validate_security(&mut result, context);

        // Performance validation
        self.validate_performance(&mut result, context);

        // Deployment-specific validation
        self.validate_deployment_specific(&mut result, context);

        // Generate recommendations
        self.generate_recommendations(&mut result, context);

        // Set overall validation status
        result.valid = result.errors.is_empty();

        result
    }

    /// Validate core configuration requirements
    fn validate_core(&self, result: &mut ValidationResult, context: &ValidationContext) {
        // Database validation
        self.validate_database_core(&mut result.errors, context);

        // Server validation
        self.validate_server_core(&mut result.errors, context);

        // Logging validation
        self.validate_logging_core(&mut result.errors, context);
    }

    /// Validate environment-specific requirements
    fn validate_environment_specific(
        &self,
        result: &mut ValidationResult,
        context: &ValidationContext,
    ) {
        match context.environment.as_str() {
            "production" => self.validate_production_requirements(result, context),
            "staging" => self.validate_staging_requirements(result, context),
            "development" => self.validate_development_requirements(result, context),
            _ => {
                result.warnings.push(ValidationWarning {
                    category: WarningCategory::Suboptimal,
                    field: "environment".to_string(),
                    message: format!("Unknown environment type: {}", context.environment),
                    current_value: Some(context.environment.clone()),
                    recommendation: Some(
                        "Use one of: production, staging, development".to_string(),
                    ),
                });
            }
        }
    }

    /// Validate production environment requirements
    fn validate_production_requirements(
        &self,
        result: &mut ValidationResult,
        _context: &ValidationContext, // Reserved for future validation context features // Reserved for future validation context features
    ) {
        // TLS must be enabled in production
        if self.server.tls.is_none() {
            result.errors.push(ValidationError {
                category: ErrorCategory::Security,
                field: "server.tls".to_string(),
                message: "TLS configuration is required in production environment".to_string(),
                current_value: None,
                suggestion: Some("Configure TLS with valid certificates".to_string()),
            });
        }

        // Authentication must be enabled in production
        if !self.auth.enabled {
            result.errors.push(ValidationError {
                category: ErrorCategory::Security,
                field: "auth.enabled".to_string(),
                message: "Authentication must be enabled in production environment".to_string(),
                current_value: Some("false".to_string()),
                suggestion: Some("Set auth.enabled = true and configure JWT secret".to_string()),
            });
        }

        // Production should use PostgreSQL
        if self.database.url.starts_with("sqlite:") {
            result.warnings.push(ValidationWarning {
                category: WarningCategory::DevelopmentSetting,
                field: "database.url".to_string(),
                message: "SQLite is not recommended for production environments".to_string(),
                current_value: Some(self.database.url.clone()),
                recommendation: Some("Use PostgreSQL for production deployments".to_string()),
            });
        }

        // JWT secret should not be default
        if self.auth.jwt_secret == "your-secret-key-change-in-production" {
            result.errors.push(ValidationError {
                category: ErrorCategory::Security,
                field: "auth.jwt_secret".to_string(),
                message: "Default JWT secret must be changed in production".to_string(),
                current_value: None,
                suggestion: Some(
                    "Set a cryptographically secure JWT secret (256-bit minimum)".to_string(),
                ),
            });
        }

        // Log level should not be debug in production
        if self.logging.level == "debug" || self.logging.level == "trace" {
            result.warnings.push(ValidationWarning {
                category: WarningCategory::Performance,
                field: "logging.level".to_string(),
                message: "Debug/trace logging may impact performance in production".to_string(),
                current_value: Some(self.logging.level.clone()),
                recommendation: Some("Use 'info' or 'warn' level for production".to_string()),
            });
        }
    }

    /// Validate staging environment requirements
    fn validate_staging_requirements(
        &self,
        result: &mut ValidationResult,
        _context: &ValidationContext, // Reserved for future validation context features // Reserved for future validation context features
    ) {
        // Staging should mirror production but can be more permissive
        if !self.auth.enabled {
            result.warnings.push(ValidationWarning {
                category: WarningCategory::Suboptimal,
                field: "auth.enabled".to_string(),
                message: "Authentication should be enabled in staging to mirror production"
                    .to_string(),
                current_value: Some("false".to_string()),
                recommendation: Some(
                    "Enable authentication for better production testing".to_string(),
                ),
            });
        }
    }

    /// Validate development environment requirements
    fn validate_development_requirements(
        &self,
        result: &mut ValidationResult,
        _context: &ValidationContext, // Reserved for future validation context features
    ) {
        // Development can be permissive but warn about production settings
        if self.server.tls.is_some() && self.server.tls.as_ref().unwrap().force_https {
            result.warnings.push(ValidationWarning {
                category: WarningCategory::DevelopmentSetting,
                field: "server.tls.force_https".to_string(),
                message: "Forced HTTPS may complicate local development".to_string(),
                current_value: Some("true".to_string()),
                recommendation: Some("Consider disabling force_https for development".to_string()),
            });
        }
    }

    /// Validate security configuration
    fn validate_security(&self, result: &mut ValidationResult, context: &ValidationContext) {
        // Network security validation
        if self.network.enabled {
            if self.network.allowed_ips.is_empty() && self.network.allowed_ranges.is_empty() {
                result.warnings.push(ValidationWarning {
                    category: WarningCategory::Security,
                    field: "network".to_string(),
                    message: "Network access controls enabled but no restrictions configured"
                        .to_string(),
                    current_value: None,
                    recommendation: Some(
                        "Configure allowed IP ranges or disable network controls".to_string(),
                    ),
                });
            }
        } else if context.is_production {
            result.warnings.push(ValidationWarning {
                category: WarningCategory::Security,
                field: "network.enabled".to_string(),
                message: "Network access controls are disabled in production environment"
                    .to_string(),
                current_value: Some("false".to_string()),
                recommendation: Some("Enable network controls for production security".to_string()),
            });
        }

        // Secrets management validation (checking for auto_init instead of enabled)
        if context.is_production && !self.secrets.auto_init {
            result.warnings.push(ValidationWarning {
                category: WarningCategory::Security,
                field: "secrets.auto_init".to_string(),
                message: "Secrets auto-initialization is disabled in production environment"
                    .to_string(),
                current_value: Some("false".to_string()),
                recommendation: Some(
                    "Enable secrets auto-initialization for production security".to_string(),
                ),
            });
        }
    }

    /// Validate performance configuration
    fn validate_performance(&self, result: &mut ValidationResult, context: &ValidationContext) {
        // Database connection pool validation
        if let Some(ref pool) = self.database.pool {
            if let Some(max_conn) = pool.max_connections {
                if max_conn < 5 && context.deployment_type != DeploymentType::Standalone {
                    result.warnings.push(ValidationWarning {
                        category: WarningCategory::Performance,
                        field: "database.pool.max_connections".to_string(),
                        message: "Low maximum connection count may limit performance".to_string(),
                        current_value: Some(max_conn.to_string()),
                        recommendation: Some(
                            "Consider increasing max_connections for cluster deployments"
                                .to_string(),
                        ),
                    });
                }

                if max_conn > 100 {
                    result.warnings.push(ValidationWarning {
                        category: WarningCategory::ResourceInefficiency,
                        field: "database.pool.max_connections".to_string(),
                        message: "Very high connection count may waste resources".to_string(),
                        current_value: Some(max_conn.to_string()),
                        recommendation: Some(
                            "Consider reducing max_connections if not needed".to_string(),
                        ),
                    });
                }
            }
        }

        // Metrics collection interval validation
        if self.metrics.enabled && self.metrics.collection_interval < 5 {
            result.warnings.push(ValidationWarning {
                category: WarningCategory::Performance,
                field: "metrics.collection_interval".to_string(),
                message: "Very frequent metrics collection may impact performance".to_string(),
                current_value: Some(self.metrics.collection_interval.to_string()),
                recommendation: Some(
                    "Consider using intervals of 15-30 seconds for production".to_string(),
                ),
            });
        }
    }

    /// Validate deployment-specific requirements
    fn validate_deployment_specific(
        &self,
        result: &mut ValidationResult,
        context: &ValidationContext,
    ) {
        match context.deployment_type {
            DeploymentType::Kubernetes => self.validate_kubernetes_requirements(result),
            DeploymentType::Cluster => self.validate_cluster_requirements(result),
            DeploymentType::DockerCompose => self.validate_docker_requirements(result),
            DeploymentType::Standalone => self.validate_standalone_requirements(result),
        }
    }

    /// Validate Kubernetes deployment requirements
    fn validate_kubernetes_requirements(&self, result: &mut ValidationResult) {
        if !self.cluster.enabled {
            result.warnings.push(ValidationWarning {
                category: WarningCategory::Suboptimal,
                field: "cluster.enabled".to_string(),
                message: "Cluster coordination should be enabled for Kubernetes deployments"
                    .to_string(),
                current_value: Some("false".to_string()),
                recommendation: Some(
                    "Enable cluster coordination for better pod management".to_string(),
                ),
            });
        }

        if !self.load_balancer.enabled {
            result.warnings.push(ValidationWarning {
                category: WarningCategory::Suboptimal,
                field: "load_balancer.enabled".to_string(),
                message: "Load balancer health checks should be enabled for Kubernetes".to_string(),
                current_value: Some("false".to_string()),
                recommendation: Some(
                    "Enable load balancer for proper Kubernetes health checks".to_string(),
                ),
            });
        }
    }

    /// Validate cluster deployment requirements
    fn validate_cluster_requirements(&self, result: &mut ValidationResult) {
        if !self.cluster.enabled {
            result.errors.push(ValidationError {
                category: ErrorCategory::Compatibility,
                field: "cluster.enabled".to_string(),
                message: "Cluster coordination must be enabled for cluster deployments".to_string(),
                current_value: Some("false".to_string()),
                suggestion: Some(
                    "Enable cluster coordination and configure cluster settings".to_string(),
                ),
            });
        }

        if !self.shared_state.backend.eq("redis") && !self.shared_state.backend.eq("etcd") {
            result.warnings.push(ValidationWarning {
                category: WarningCategory::Compatibility,
                field: "shared_state.backend".to_string(),
                message: "Memory backend is not suitable for cluster deployments".to_string(),
                current_value: Some(self.shared_state.backend.clone()),
                recommendation: Some("Use Redis or etcd for cluster shared state".to_string()),
            });
        }
    }

    /// Validate Docker Compose deployment requirements
    fn validate_docker_requirements(&self, result: &mut ValidationResult) {
        if self.server.host == "127.0.0.1" {
            result.warnings.push(ValidationWarning {
                category: WarningCategory::Compatibility,
                field: "server.host".to_string(),
                message: "Localhost binding may not work in Docker containers".to_string(),
                current_value: Some(self.server.host.clone()),
                recommendation: Some("Use '0.0.0.0' for Docker deployments".to_string()),
            });
        }
    }

    /// Validate standalone deployment requirements
    fn validate_standalone_requirements(&self, result: &mut ValidationResult) {
        if self.cluster.enabled {
            result.warnings.push(ValidationWarning {
                category: WarningCategory::ResourceInefficiency,
                field: "cluster.enabled".to_string(),
                message: "Cluster coordination is unnecessary for standalone deployments"
                    .to_string(),
                current_value: Some("true".to_string()),
                recommendation: Some(
                    "Disable cluster coordination for single-instance deployments".to_string(),
                ),
            });
        }
    }

    /// Core database validation
    fn validate_database_core(
        &self,
        errors: &mut Vec<ValidationError>,
        _context: &ValidationContext, // Reserved for future validation context features
    ) {
        // URL format validation
        if !self.database.url.starts_with("sqlite:")
            && !self.database.url.starts_with("postgresql:")
            && !self.database.url.starts_with("postgres:")
        {
            errors.push(ValidationError {
                category: ErrorCategory::InvalidFormat,
                field: "database.url".to_string(),
                message: "Invalid database URL format".to_string(),
                current_value: Some(self.database.url.clone()),
                suggestion: Some("Use sqlite:// or postgresql:// URL format".to_string()),
            });
        }

        // Connection count validation
        if let Some(max_conn) = self.database.max_connections {
            if max_conn == 0 {
                errors.push(ValidationError {
                    category: ErrorCategory::InvalidFormat,
                    field: "database.max_connections".to_string(),
                    message: "Maximum connections must be greater than 0".to_string(),
                    current_value: Some(max_conn.to_string()),
                    suggestion: Some("Set a positive integer value".to_string()),
                });
            }
        }
    }

    /// Core server validation
    fn validate_server_core(
        &self,
        errors: &mut Vec<ValidationError>,
        _context: &ValidationContext, // Reserved for future validation context features
    ) {
        // Port validation
        if self.server.port == 0 {
            errors.push(ValidationError {
                category: ErrorCategory::InvalidFormat,
                field: "server.port".to_string(),
                message: "Server port must be greater than 0".to_string(),
                current_value: Some(self.server.port.to_string()),
                suggestion: Some("Use a valid port number (1-65535)".to_string()),
            });
        }

        // Host validation
        if self.server.host.is_empty() {
            errors.push(ValidationError {
                category: ErrorCategory::MissingRequired,
                field: "server.host".to_string(),
                message: "Server host cannot be empty".to_string(),
                current_value: None,
                suggestion: Some("Set a valid IP address or hostname".to_string()),
            });
        }

        // TLS validation if enabled
        if let Some(ref tls) = self.server.tls {
            if !Path::new(&tls.cert_file).exists() {
                errors.push(ValidationError {
                    category: ErrorCategory::InvalidFormat,
                    field: "server.tls.cert_file".to_string(),
                    message: "TLS certificate file does not exist".to_string(),
                    current_value: Some(tls.cert_file.clone()),
                    suggestion: Some("Provide a valid path to the certificate file".to_string()),
                });
            }

            if !Path::new(&tls.key_file).exists() {
                errors.push(ValidationError {
                    category: ErrorCategory::InvalidFormat,
                    field: "server.tls.key_file".to_string(),
                    message: "TLS private key file does not exist".to_string(),
                    current_value: Some(tls.key_file.clone()),
                    suggestion: Some("Provide a valid path to the private key file".to_string()),
                });
            }
        }
    }

    /// Core logging validation
    fn validate_logging_core(
        &self,
        errors: &mut Vec<ValidationError>,
        _context: &ValidationContext, // Reserved for future validation context features
    ) {
        // Log level validation
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.logging.level.as_str()) {
            errors.push(ValidationError {
                category: ErrorCategory::InvalidFormat,
                field: "logging.level".to_string(),
                message: "Invalid log level".to_string(),
                current_value: Some(self.logging.level.clone()),
                suggestion: Some("Use one of: trace, debug, info, warn, error".to_string()),
            });
        }

        // Log format validation
        let valid_formats = ["json", "pretty", "compact"];
        if !valid_formats.contains(&self.logging.format.as_str()) {
            errors.push(ValidationError {
                category: ErrorCategory::InvalidFormat,
                field: "logging.format".to_string(),
                message: "Invalid log format".to_string(),
                current_value: Some(self.logging.format.clone()),
                suggestion: Some("Use one of: json, pretty, compact".to_string()),
            });
        }
    }

    /// Generate configuration recommendations
    fn generate_recommendations(&self, result: &mut ValidationResult, context: &ValidationContext) {
        // Security recommendations
        self.generate_security_recommendations(&mut result.recommendations, context);

        // Performance recommendations
        self.generate_performance_recommendations(&mut result.recommendations, context);

        // High availability recommendations
        self.generate_ha_recommendations(&mut result.recommendations, context);

        // Monitoring recommendations
        self.generate_monitoring_recommendations(&mut result.recommendations, context);
    }

    /// Generate security recommendations
    fn generate_security_recommendations(
        &self,
        recommendations: &mut Vec<ValidationRecommendation>,
        context: &ValidationContext,
    ) {
        if context.is_production {
            // Rate limiting recommendation
            if !self.network.enable_network_rate_limits {
                recommendations.push(ValidationRecommendation {
                    category: RecommendationCategory::Security,
                    priority: RecommendationPriority::High,
                    area: "network.enable_network_rate_limits".to_string(),
                    message: "Enable network-based rate limiting for DDoS protection".to_string(),
                    explanation: "Rate limiting helps protect against abuse and DoS attacks"
                        .to_string(),
                    steps: vec![
                        "Set network.enable_network_rate_limits = true".to_string(),
                        "Configure appropriate rate limits for your use case".to_string(),
                        "Monitor rate limit metrics".to_string(),
                    ],
                });
            }

            // Geolocation blocking recommendation
            if !self.network.enable_geolocation {
                recommendations.push(ValidationRecommendation {
                    category: RecommendationCategory::Security,
                    priority: RecommendationPriority::Medium,
                    area: "network.enable_geolocation".to_string(),
                    message: "Consider enabling geolocation-based access controls".to_string(),
                    explanation:
                        "Geolocation blocking can help prevent attacks from specific regions"
                            .to_string(),
                    steps: vec![
                        "Set network.enable_geolocation = true".to_string(),
                        "Configure blocked_countries list".to_string(),
                        "Test with your legitimate user base".to_string(),
                    ],
                });
            }
        }
    }

    /// Generate performance recommendations
    fn generate_performance_recommendations(
        &self,
        recommendations: &mut Vec<ValidationRecommendation>,
        _context: &ValidationContext, // Reserved for future validation context features
    ) {
        // Resource management recommendation
        if !self.resource_management.memory.enabled {
            recommendations.push(ValidationRecommendation {
                category: RecommendationCategory::Performance,
                priority: RecommendationPriority::High,
                area: "resource_management.memory.enabled".to_string(),
                message: "Enable resource management for better performance and stability".to_string(),
                explanation: "Resource management provides memory optimization, throttling, and graceful degradation".to_string(),
                steps: vec![
                    "Set resource_management.memory.enabled = true".to_string(),
                    "Configure appropriate memory and CPU limits".to_string(),
                    "Set up monitoring for resource usage".to_string(),
                ],
            });
        }

        // Memory pooling recommendation
        if !self.resource_management.memory.pool.enabled {
            recommendations.push(ValidationRecommendation {
                category: RecommendationCategory::Performance,
                priority: RecommendationPriority::Medium,
                area: "resource_management.memory.pool.enabled".to_string(),
                message: "Enable memory pooling for better allocation performance".to_string(),
                explanation: "Memory pooling reduces allocation overhead and improves performance"
                    .to_string(),
                steps: vec![
                    "Set resource_management.memory.pool.enabled = true".to_string(),
                    "Configure appropriate pool sizes".to_string(),
                    "Monitor memory pool utilization".to_string(),
                ],
            });
        }
    }

    /// Generate high availability recommendations
    fn generate_ha_recommendations(
        &self,
        recommendations: &mut Vec<ValidationRecommendation>,
        context: &ValidationContext,
    ) {
        if context.deployment_type == DeploymentType::Cluster || context.is_production {
            // Cluster coordination recommendation
            if !self.cluster.enabled {
                recommendations.push(ValidationRecommendation {
                    category: RecommendationCategory::HighAvailability,
                    priority: RecommendationPriority::High,
                    area: "cluster.enabled".to_string(),
                    message: "Enable cluster coordination for high availability".to_string(),
                    explanation: "Cluster coordination provides failover, load distribution, and service discovery".to_string(),
                    steps: vec![
                        "Set cluster.enabled = true".to_string(),
                        "Configure cluster.node settings".to_string(),
                        "Set up service discovery".to_string(),
                        "Configure health monitoring".to_string(),
                    ],
                });
            }

            // Load balancer recommendation
            if !self.load_balancer.enabled {
                recommendations.push(ValidationRecommendation {
                    category: RecommendationCategory::HighAvailability,
                    priority: RecommendationPriority::High,
                    area: "load_balancer.enabled".to_string(),
                    message: "Enable load balancer integration for high availability".to_string(),
                    explanation:
                        "Load balancer integration provides health checks and traffic distribution"
                            .to_string(),
                    steps: vec![
                        "Set load_balancer.enabled = true".to_string(),
                        "Configure health check endpoints".to_string(),
                        "Set up trusted proxy ranges".to_string(),
                    ],
                });
            }
        }
    }

    /// Generate monitoring recommendations
    fn generate_monitoring_recommendations(
        &self,
        recommendations: &mut Vec<ValidationRecommendation>,
        context: &ValidationContext,
    ) {
        // Metrics recommendation
        if !self.metrics.enabled && context.is_production {
            recommendations.push(ValidationRecommendation {
                category: RecommendationCategory::Monitoring,
                priority: RecommendationPriority::Critical,
                area: "metrics.enabled".to_string(),
                message: "Enable metrics collection for production monitoring".to_string(),
                explanation: "Metrics are essential for monitoring system health and performance"
                    .to_string(),
                steps: vec![
                    "Set metrics.enabled = true".to_string(),
                    "Configure Prometheus scraping".to_string(),
                    "Set up alerting rules".to_string(),
                    "Create monitoring dashboards".to_string(),
                ],
            });
        }

        // OpenTelemetry recommendation
        if let Some(ref otel) = self.logging.opentelemetry {
            if !otel.enabled && context.is_production {
                recommendations.push(ValidationRecommendation {
                    category: RecommendationCategory::Monitoring,
                    priority: RecommendationPriority::Medium,
                    area: "logging.opentelemetry.enabled".to_string(),
                    message: "Enable OpenTelemetry for distributed tracing".to_string(),
                    explanation:
                        "Distributed tracing helps debug performance issues in complex systems"
                            .to_string(),
                    steps: vec![
                        "Set logging.opentelemetry.enabled = true".to_string(),
                        "Configure Jaeger or similar tracing backend".to_string(),
                        "Set appropriate sample rate".to_string(),
                    ],
                });
            }
        }
    }

    /// Generate configuration summary
    fn generate_summary(&self, context: &ValidationContext) -> ConfigurationSummary {
        let database_type = if self.database.url.starts_with("sqlite:") {
            "sqlite".to_string()
        } else if self.database.url.starts_with("postgresql:")
            || self.database.url.starts_with("postgres:")
        {
            "postgresql".to_string()
        } else {
            "unknown".to_string()
        };

        let mut security_features = Vec::new();
        if self.auth.enabled {
            security_features.push("JWT Authentication".to_string());
        }
        if self.server.tls.is_some() {
            security_features.push("TLS/HTTPS".to_string());
        }
        if self.network.enabled {
            security_features.push("Network Access Controls".to_string());
        }
        if self.secrets.auto_init {
            security_features.push("Secrets Management".to_string());
        }

        let mut performance_features = Vec::new();
        if self.resource_management.memory.enabled {
            performance_features.push("Resource Management".to_string());
        }
        if self.resource_management.memory.pool.enabled {
            performance_features.push("Memory Pooling".to_string());
        }
        if self.database.pool.is_some() {
            performance_features.push("Connection Pooling".to_string());
        }

        // Estimate resource usage
        let resource_usage = self.estimate_resource_usage(context);

        ConfigurationSummary {
            environment: context.environment.clone(),
            database_type,
            authentication_enabled: self.auth.enabled,
            tls_enabled: self.server.tls.is_some(),
            clustering_enabled: self.cluster.enabled,
            monitoring_enabled: self.metrics.enabled,
            security_features,
            performance_features,
            resource_usage,
        }
    }

    /// Estimate resource usage based on configuration
    fn estimate_resource_usage(&self, _context: &ValidationContext) -> ResourceUsageEstimate {
        let mut memory_mb = 128; // Base memory usage
        let mut cpu_cores = 0.5; // Base CPU usage
        let mut disk_mb = 100; // Base disk usage

        // Database connection pool impact
        if let Some(ref pool) = self.database.pool {
            if let Some(max_conn) = pool.max_connections {
                memory_mb += max_conn * 2; // ~2MB per connection
            }
        }

        // Clustering impact
        if self.cluster.enabled {
            memory_mb += 64; // Cluster coordination overhead
            cpu_cores += 0.2;
        }

        // Metrics collection impact
        if self.metrics.enabled {
            memory_mb += 32; // Metrics storage
            cpu_cores += 0.1;
            disk_mb += 500; // Metrics retention
        }

        // Logging impact
        if self.logging.level == "debug" || self.logging.level == "trace" {
            cpu_cores += 0.1; // Extra logging overhead
            disk_mb += 1000; // More log storage
        }

        // Resource management cache impact
        if self.resource_management.memory.enabled {
            memory_mb += self.resource_management.memory.cache.max_size_mb as u32;
        }

        let max_connections = self.database.max_connections.unwrap_or(10)
            + if self.cluster.enabled { 100 } else { 0 };

        ResourceUsageEstimate {
            memory_mb: memory_mb as u64,
            cpu_cores,
            disk_mb: disk_mb as u64,
            max_connections,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn test_production_validation() {
        let mut config = Config::default();
        config.auth.enabled = false; // This should trigger an error

        let context = ValidationContext::production(DeploymentType::Kubernetes);
        let result = config.validate_with_context(&context);

        assert!(!result.valid);
        assert!(!result.errors.is_empty());

        // Should have authentication error
        let auth_error = result
            .errors
            .iter()
            .find(|e| e.field == "auth.enabled")
            .expect("Should have authentication error");

        assert_eq!(auth_error.category, ErrorCategory::Security);
    }

    #[test]
    fn test_development_validation() {
        let config = Config::default();
        let context = ValidationContext::development();
        let result = config.validate_with_context(&context);

        // Development should be more permissive
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_kubernetes_specific_validation() {
        let mut config = Config::default();
        config.cluster.enabled = false;

        let context = ValidationContext::new("production", DeploymentType::Kubernetes);
        let result = config.validate_with_context(&context);

        // Should have warning about cluster coordination
        let cluster_warning = result
            .warnings
            .iter()
            .find(|w| w.field == "cluster.enabled");

        assert!(cluster_warning.is_some());
    }

    #[test]
    fn test_resource_estimation() {
        let mut config = Config::default();
        config.resource_management.memory.cache.max_size_mb = 1024;
        if let Some(ref mut pool) = config.database.pool {
            pool.max_connections = Some(50);
        }
        config.database.max_connections = Some(50);

        let context = ValidationContext::production(DeploymentType::Cluster);
        let result = config.validate_with_context(&context);

        // Should estimate higher resource usage
        assert!(result.summary.resource_usage.memory_mb > 1000);
        assert!(result.summary.resource_usage.max_connections >= 50);
    }
}
