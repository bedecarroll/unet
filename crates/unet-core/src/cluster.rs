//! Cluster coordination for μNet Core
//!
//! This module provides cluster management capabilities including service discovery,
//! membership management, health monitoring, configuration synchronization, failover,
//! and cluster-wide coordination for horizontal scaling.

use crate::error::Result;
use crate::shared_state::SharedStateManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Cluster coordination configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    /// Enable cluster coordination
    pub enabled: bool,
    /// Unique cluster ID
    pub cluster_id: String,
    /// Node configuration
    pub node: NodeConfig,
    /// Service discovery configuration
    pub service_discovery: ServiceDiscoveryConfig,
    /// Health monitoring configuration
    pub health_monitoring: HealthMonitoringConfig,
    /// Configuration synchronization settings
    pub config_sync: ConfigSyncConfig,
    /// Failover and load redistribution settings
    pub failover: FailoverConfig,
    /// Cluster scaling configuration
    pub scaling: ScalingConfig,
}

/// Node configuration within the cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Unique node ID (auto-generated if not specified)
    pub node_id: Option<String>,
    /// Node name for identification
    pub name: String,
    /// Node roles (e.g., "primary", "worker", "standby")
    pub roles: Vec<String>,
    /// Node priority for leader election (higher = more preferred)
    pub priority: u32,
    /// Node advertise address for cluster communication
    pub advertise_addr: String,
    /// Node metadata tags
    pub metadata: HashMap<String, String>,
    /// Node capacity configuration
    pub capacity: NodeCapacity,
}

/// Node capacity configuration for resource management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCapacity {
    /// Maximum number of connections this node can handle
    pub max_connections: Option<u32>,
    /// Maximum CPU percentage allocation (0.0-1.0)
    pub max_cpu: Option<f64>,
    /// Maximum memory usage in MB
    pub max_memory_mb: Option<u64>,
    /// Custom capacity metrics
    pub custom_metrics: HashMap<String, f64>,
}

/// Service discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDiscoveryConfig {
    /// Service discovery method ("static", "dns", "consul", "etcd", "kubernetes")
    pub method: String,
    /// Service discovery interval in seconds
    pub discovery_interval: u64,
    /// Service TTL in seconds
    pub service_ttl: u64,
    /// Consul-specific configuration
    pub consul: Option<ConsulConfig>,
    /// DNS-based discovery configuration
    pub dns: Option<DnsDiscoveryConfig>,
    /// Static discovery configuration
    pub static_nodes: Option<Vec<StaticNodeConfig>>,
    /// Kubernetes service discovery configuration
    pub kubernetes: Option<KubernetesConfig>,
}

/// Consul service discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsulConfig {
    /// Consul HTTP API address
    pub address: String,
    /// Consul datacenter
    pub datacenter: Option<String>,
    /// Service name in Consul
    pub service_name: String,
    /// Service tags
    pub tags: Vec<String>,
    /// Health check configuration
    pub health_check: ConsulHealthCheck,
}

/// Consul health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsulHealthCheck {
    /// Health check HTTP endpoint
    pub http: String,
    /// Check interval in seconds
    pub interval: u64,
    /// Check timeout in seconds
    pub timeout: u64,
    /// Deregister critical services after this duration
    pub deregister_critical_after: Option<u64>,
}

/// DNS-based service discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsDiscoveryConfig {
    /// DNS domain for service discovery
    pub domain: String,
    /// DNS record type ("A", "SRV")
    pub record_type: String,
    /// DNS server address
    pub dns_server: Option<String>,
}

/// Static node configuration for static discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticNodeConfig {
    /// Node ID
    pub node_id: String,
    /// Node address
    pub address: String,
    /// Node roles
    pub roles: Vec<String>,
    /// Node metadata
    pub metadata: HashMap<String, String>,
}

/// Kubernetes service discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesConfig {
    /// Kubernetes namespace
    pub namespace: String,
    /// Service selector labels
    pub selector: HashMap<String, String>,
    /// Pod port for service discovery
    pub port: u16,
}

/// Health monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMonitoringConfig {
    /// Health check interval in seconds
    pub check_interval: u64,
    /// Health check timeout in seconds
    pub check_timeout: u64,
    /// Number of consecutive failures before marking unhealthy
    pub failure_threshold: u32,
    /// Number of consecutive successes before marking healthy
    pub success_threshold: u32,
    /// Enable deep health checks
    pub enable_deep_checks: bool,
    /// Custom health check endpoints
    pub custom_checks: Vec<CustomHealthCheck>,
}

/// Custom health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomHealthCheck {
    /// Check name
    pub name: String,
    /// Check type ("http", "tcp", "script")
    pub check_type: String,
    /// Check target (URL, address, or script path)
    pub target: String,
    /// Check interval in seconds
    pub interval: u64,
    /// Check timeout in seconds
    pub timeout: u64,
    /// Check criticality ("critical", "warning", "info")
    pub criticality: String,
}

/// Configuration synchronization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSyncConfig {
    /// Enable configuration synchronization
    pub enabled: bool,
    /// Configuration sync interval in seconds
    pub sync_interval: u64,
    /// Configuration sync timeout in seconds
    pub sync_timeout: u64,
    /// Configuration change propagation method ("immediate", "scheduled", "manual")
    pub propagation_method: String,
    /// Configuration validation before sync
    pub validate_before_sync: bool,
    /// Configuration rollback on sync failure
    pub rollback_on_failure: bool,
}

/// Failover and load redistribution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailoverConfig {
    /// Enable automatic failover
    pub enabled: bool,
    /// Failover detection timeout in seconds
    pub detection_timeout: u64,
    /// Failover execution timeout in seconds
    pub execution_timeout: u64,
    /// Leader election configuration
    pub leader_election: LeaderElectionConfig,
    /// Load redistribution settings
    pub load_redistribution: LoadRedistributionConfig,
}

/// Leader election configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderElectionConfig {
    /// Leader election key prefix
    pub key_prefix: String,
    /// Leader lease duration in seconds
    pub lease_duration: u64,
    /// Leader renew deadline in seconds
    pub renew_deadline: u64,
    /// Leader retry period in seconds
    pub retry_period: u64,
}

/// Load redistribution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadRedistributionConfig {
    /// Load redistribution strategy ("round_robin", "least_connections", "weighted")
    pub strategy: String,
    /// Redistribution threshold (percentage of load difference)
    pub threshold: f64,
    /// Redistribution cooldown period in seconds
    pub cooldown_period: u64,
    /// Maximum redistribution percentage per operation
    pub max_redistribution_percent: f64,
}

/// Cluster scaling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingConfig {
    /// Enable automatic scaling
    pub enabled: bool,
    /// Scaling metric thresholds
    pub thresholds: ScalingThresholds,
    /// Scaling evaluation interval in seconds
    pub evaluation_interval: u64,
    /// Scaling cooldown period in seconds
    pub cooldown_period: u64,
    /// Notification configuration for scaling events
    pub notifications: ScalingNotifications,
}

/// Scaling metric thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingThresholds {
    /// CPU utilization threshold for scale-up (0.0-1.0)
    pub cpu_scale_up: f64,
    /// CPU utilization threshold for scale-down (0.0-1.0)
    pub cpu_scale_down: f64,
    /// Memory utilization threshold for scale-up (0.0-1.0)
    pub memory_scale_up: f64,
    /// Memory utilization threshold for scale-down (0.0-1.0)
    pub memory_scale_down: f64,
    /// Connection count threshold for scale-up
    pub connections_scale_up: u32,
    /// Connection count threshold for scale-down
    pub connections_scale_down: u32,
    /// Custom metric thresholds
    pub custom_metrics: HashMap<String, (f64, f64)>, // (scale_up, scale_down)
}

/// Scaling notifications configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingNotifications {
    /// Enable scaling event notifications
    pub enabled: bool,
    /// Notification channels
    pub channels: Vec<String>,
    /// Notification severity levels to include
    pub severity_levels: Vec<String>,
}

/// Cluster node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterNode {
    /// Node ID
    pub node_id: String,
    /// Node name
    pub name: String,
    /// Node address
    pub address: String,
    /// Node roles
    pub roles: Vec<String>,
    /// Node priority
    pub priority: u32,
    /// Node metadata
    pub metadata: HashMap<String, String>,
    /// Node capacity
    pub capacity: NodeCapacity,
    /// Node health status
    pub health: NodeHealth,
    /// Last seen timestamp
    pub last_seen: u64,
    /// Node version
    pub version: String,
}

/// Node health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeHealth {
    /// Overall health status
    pub status: HealthStatus,
    /// Last health check timestamp
    pub last_check: u64,
    /// Health check results
    pub checks: HashMap<String, HealthCheckResult>,
    /// Node metrics
    pub metrics: NodeMetrics,
}

/// Health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Check status
    pub status: HealthStatus,
    /// Check message
    pub message: String,
    /// Check duration in milliseconds
    pub duration_ms: u64,
    /// Check timestamp
    pub timestamp: u64,
}

/// Node metrics for monitoring and scaling decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    /// CPU utilization (0.0-1.0)
    pub cpu_utilization: f64,
    /// Memory utilization (0.0-1.0)
    pub memory_utilization: f64,
    /// Active connections count
    pub connections: u32,
    /// Request rate (requests per second)
    pub request_rate: f64,
    /// Error rate (errors per second)
    pub error_rate: f64,
    /// Response time percentiles
    pub response_times: ResponseTimeMetrics,
    /// Custom metrics
    pub custom_metrics: HashMap<String, f64>,
}

/// Response time metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTimeMetrics {
    /// P50 response time in milliseconds
    pub p50: f64,
    /// P95 response time in milliseconds
    pub p95: f64,
    /// P99 response time in milliseconds
    pub p99: f64,
    /// Average response time in milliseconds
    pub avg: f64,
}

/// Cluster membership information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterMembership {
    /// Cluster ID
    pub cluster_id: String,
    /// All known nodes
    pub nodes: HashMap<String, ClusterNode>,
    /// Current leader node ID
    pub leader: Option<String>,
    /// Cluster version/generation
    pub version: u64,
    /// Last updated timestamp
    pub last_updated: u64,
}

/// Cluster coordination manager
pub struct ClusterManager {
    config: ClusterConfig,
    node_id: String,
    shared_state: SharedStateManager,
    membership: ClusterMembership,
    local_metrics: NodeMetrics,
}

impl ClusterManager {
    /// Creates a new cluster manager
    pub fn new(config: ClusterConfig, shared_state: SharedStateManager) -> Result<Self> {
        let node_id = config
            .node
            .node_id
            .clone()
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let membership = ClusterMembership {
            cluster_id: config.cluster_id.clone(),
            nodes: HashMap::new(),
            leader: None,
            version: 0,
            last_updated: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        let local_metrics = NodeMetrics {
            cpu_utilization: 0.0,
            memory_utilization: 0.0,
            connections: 0,
            request_rate: 0.0,
            error_rate: 0.0,
            response_times: ResponseTimeMetrics {
                p50: 0.0,
                p95: 0.0,
                p99: 0.0,
                avg: 0.0,
            },
            custom_metrics: HashMap::new(),
        };

        Ok(Self {
            config,
            node_id,
            shared_state,
            membership,
            local_metrics,
        })
    }

    /// Starts the cluster manager with all coordination tasks
    pub async fn start(&mut self) -> Result<()> {
        tracing::info!("Starting cluster manager for node {}", self.node_id);

        // Register this node with the cluster
        self.register_node().await?;

        // Clone necessary data for background tasks
        let node_id = self.node_id.clone();
        let cluster_id = self.config.cluster_id.clone();

        // Start background tasks
        // Note: These would need Arc<Mutex<Self>> or similar for proper implementation
        tracing::info!(
            "Background tasks would be started here for node {} in cluster {}",
            node_id,
            cluster_id
        );

        Ok(())
    }

    /// Registers this node with the cluster
    async fn register_node(&mut self) -> Result<()> {
        let node = ClusterNode {
            node_id: self.node_id.clone(),
            name: self.config.node.name.clone(),
            address: self.config.node.advertise_addr.clone(),
            roles: self.config.node.roles.clone(),
            priority: self.config.node.priority,
            metadata: self.config.node.metadata.clone(),
            capacity: self.config.node.capacity.clone(),
            health: NodeHealth {
                status: HealthStatus::Healthy,
                last_check: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                checks: HashMap::new(),
                metrics: self.local_metrics.clone(),
            },
            last_seen: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        };

        // Store node information in shared state
        let key = format!("cluster:nodes:{}", self.node_id);
        let ttl = Duration::from_secs(self.config.service_discovery.service_ttl);
        self.shared_state.store_json(&key, &node, Some(ttl)).await?;

        // Update local membership
        self.membership.nodes.insert(self.node_id.clone(), node);

        tracing::info!("Node {} registered with cluster", self.node_id);
        Ok(())
    }

    /// Updates local node metrics
    pub fn update_metrics(&mut self, metrics: NodeMetrics) {
        self.local_metrics = metrics;
    }

    /// Gets current cluster membership
    pub fn get_membership(&self) -> &ClusterMembership {
        &self.membership
    }

    /// Gets cluster statistics
    pub async fn get_cluster_stats(&self) -> Result<ClusterStats> {
        let mut total_nodes = 0;
        let mut healthy_nodes = 0;
        let mut total_connections = 0;
        let mut total_cpu = 0.0;
        let mut total_memory = 0.0;

        // Check all known nodes
        for node_id in self.membership.nodes.keys() {
            let key = format!("cluster:nodes:{}", node_id);
            if let Some(node) = self.shared_state.retrieve_json::<ClusterNode>(&key).await? {
                total_nodes += 1;
                if node.health.status == HealthStatus::Healthy {
                    healthy_nodes += 1;
                }
                total_connections += node.health.metrics.connections;
                total_cpu += node.health.metrics.cpu_utilization;
                total_memory += node.health.metrics.memory_utilization;
            }
        }

        Ok(ClusterStats {
            total_nodes,
            healthy_nodes,
            total_connections,
            avg_cpu_utilization: if total_nodes > 0 {
                total_cpu / total_nodes as f64
            } else {
                0.0
            },
            avg_memory_utilization: if total_nodes > 0 {
                total_memory / total_nodes as f64
            } else {
                0.0
            },
            cluster_version: self.membership.version,
            leader_node: self.membership.leader.clone(),
        })
    }
}

/// Cluster statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStats {
    /// Total number of nodes
    pub total_nodes: u32,
    /// Number of healthy nodes
    pub healthy_nodes: u32,
    /// Total connections across cluster
    pub total_connections: u32,
    /// Average CPU utilization
    pub avg_cpu_utilization: f64,
    /// Average memory utilization
    pub avg_memory_utilization: f64,
    /// Cluster version
    pub cluster_version: u64,
    /// Current leader node ID
    pub leader_node: Option<String>,
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cluster_id: "unet-cluster".to_string(),
            node: NodeConfig {
                node_id: None,
                name: "unet-node".to_string(),
                roles: vec!["worker".to_string()],
                priority: 100,
                advertise_addr: "127.0.0.1:8080".to_string(),
                metadata: HashMap::new(),
                capacity: NodeCapacity {
                    max_connections: Some(1000),
                    max_cpu: Some(0.8),
                    max_memory_mb: Some(1024),
                    custom_metrics: HashMap::new(),
                },
            },
            service_discovery: ServiceDiscoveryConfig {
                method: "static".to_string(),
                discovery_interval: 30,
                service_ttl: 60,
                consul: None,
                dns: None,
                static_nodes: Some(vec![]),
                kubernetes: None,
            },
            health_monitoring: HealthMonitoringConfig {
                check_interval: 10,
                check_timeout: 5,
                failure_threshold: 3,
                success_threshold: 2,
                enable_deep_checks: false,
                custom_checks: vec![],
            },
            config_sync: ConfigSyncConfig {
                enabled: false,
                sync_interval: 300,
                sync_timeout: 30,
                propagation_method: "immediate".to_string(),
                validate_before_sync: true,
                rollback_on_failure: true,
            },
            failover: FailoverConfig {
                enabled: false,
                detection_timeout: 30,
                execution_timeout: 60,
                leader_election: LeaderElectionConfig {
                    key_prefix: "cluster:leader".to_string(),
                    lease_duration: 30,
                    renew_deadline: 20,
                    retry_period: 5,
                },
                load_redistribution: LoadRedistributionConfig {
                    strategy: "round_robin".to_string(),
                    threshold: 0.2,
                    cooldown_period: 300,
                    max_redistribution_percent: 0.5,
                },
            },
            scaling: ScalingConfig {
                enabled: false,
                thresholds: ScalingThresholds {
                    cpu_scale_up: 0.7,
                    cpu_scale_down: 0.3,
                    memory_scale_up: 0.8,
                    memory_scale_down: 0.4,
                    connections_scale_up: 800,
                    connections_scale_down: 200,
                    custom_metrics: HashMap::new(),
                },
                evaluation_interval: 60,
                cooldown_period: 300,
                notifications: ScalingNotifications {
                    enabled: true,
                    channels: vec!["log".to_string()],
                    severity_levels: vec!["info".to_string(), "warning".to_string()],
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared_state::{InMemoryStateProvider, SharedStateConfig, SharedStateManager};

    #[tokio::test]
    async fn test_cluster_manager_creation() {
        let config = ClusterConfig::default();
        let shared_state = SharedStateManager::in_memory();

        let result = ClusterManager::new(config, shared_state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cluster_config_default() {
        let config = ClusterConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.cluster_id, "unet-cluster");
        assert_eq!(config.node.name, "unet-node");
        assert_eq!(config.service_discovery.method, "static");
    }
}
