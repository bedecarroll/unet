//! Live configuration management for network devices
//!
//! This module provides functionality to fetch live configurations from network devices
//! using various protocols (SSH, SNMP, API), with secure credential management,
//! connection pooling, and intelligent caching.

use crate::error::{Error, Result};
use crate::models::{Node, Vendor};
use crate::snmp::{SnmpClient, SnmpClientConfig};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::process::Command;
use tokio::sync::{RwLock, Semaphore};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Live configuration fetcher with connection management and caching
#[derive(Debug)]
pub struct LiveConfigManager {
    /// Connection pool manager
    connection_manager: Arc<ConnectionManager>,
    /// SNMP client for SNMP-based configuration retrieval
    snmp_client: Arc<SnmpClient>,
    /// Configuration cache
    config_cache: Arc<RwLock<ConfigCache>>,
    /// Manager configuration
    config: LiveConfigManagerConfig,
}

/// Configuration for live config manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveConfigManagerConfig {
    /// Maximum number of concurrent connections per device
    pub max_connections_per_device: usize,
    /// Default connection timeout
    pub connection_timeout: Duration,
    /// Configuration cache TTL
    pub cache_ttl: Duration,
    /// Maximum cache size (number of configurations)
    pub max_cache_size: usize,
    /// SSH connection configuration
    pub ssh: SshConfig,
    /// SNMP configuration
    pub snmp: SnmpConfig,
    /// API configuration
    pub api: ApiConfig,
}

/// SSH connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConfig {
    /// Default SSH port
    pub port: u16,
    /// Connection timeout
    pub timeout: Duration,
    /// Number of connection retries
    pub retries: u32,
    /// Keep-alive interval
    pub keepalive_interval: Duration,
    /// Enable strict host key checking
    pub strict_host_key_checking: bool,
}

/// SNMP configuration for live config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnmpConfig {
    /// Default SNMP port
    pub port: u16,
    /// Default community string
    pub community: String,
    /// Request timeout
    pub timeout: Duration,
    /// Number of retries
    pub retries: u32,
    /// SNMP version (1, 2, or 3)
    pub version: u8,
}

/// API configuration for devices with REST APIs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Default HTTPS port
    pub port: u16,
    /// Request timeout
    pub timeout: Duration,
    /// Number of retries
    pub retries: u32,
    /// TLS verification enabled
    pub verify_tls: bool,
}

impl Default for LiveConfigManagerConfig {
    fn default() -> Self {
        Self {
            max_connections_per_device: 5,
            connection_timeout: Duration::from_secs(30),
            cache_ttl: Duration::from_secs(300), // 5 minutes
            max_cache_size: 1000,
            ssh: SshConfig {
                port: 22,
                timeout: Duration::from_secs(30),
                retries: 3,
                keepalive_interval: Duration::from_secs(30),
                strict_host_key_checking: false, // For automation use cases
            },
            snmp: SnmpConfig {
                port: 161,
                community: "public".to_string(),
                timeout: Duration::from_secs(5),
                retries: 3,
                version: 2,
            },
            api: ApiConfig {
                port: 443,
                timeout: Duration::from_secs(30),
                retries: 3,
                verify_tls: true,
            },
        }
    }
}

/// Device credentials for secure access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceCredentials {
    /// SSH username/password authentication
    SshPassword { username: String, password: String },
    /// SSH key-based authentication
    SshKey {
        username: String,
        private_key_path: String,
        passphrase: Option<String>,
    },
    /// SNMP community string
    SnmpCommunity { community: String },
    /// SNMP v3 user-based security
    SnmpV3 {
        username: String,
        auth_protocol: Option<String>,
        auth_password: Option<String>,
        privacy_protocol: Option<String>,
        privacy_password: Option<String>,
    },
    /// API token authentication
    ApiToken {
        token: String,
        token_type: String, // Bearer, Basic, etc.
    },
    /// API username/password
    ApiBasic { username: String, password: String },
}

/// Configuration retrieval protocol
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConfigProtocol {
    /// SSH-based configuration retrieval
    Ssh,
    /// SNMP-based configuration retrieval
    Snmp,
    /// REST API-based configuration retrieval
    RestApi,
    /// NETCONF protocol
    Netconf,
}

/// Configuration retrieval request
#[derive(Debug, Clone)]
pub struct ConfigRequest {
    /// Target node
    pub node: Node,
    /// Retrieval protocol to use
    pub protocol: ConfigProtocol,
    /// Device credentials
    pub credentials: DeviceCredentials,
    /// Force refresh (bypass cache)
    pub force_refresh: bool,
    /// Custom timeout override
    pub timeout: Option<Duration>,
}

/// Configuration retrieval result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigResult {
    /// Node ID
    pub node_id: Uuid,
    /// Retrieved configuration content
    pub config_content: String,
    /// Retrieval timestamp
    pub retrieved_at: SystemTime,
    /// Protocol used for retrieval
    pub protocol: ConfigProtocol,
    /// Configuration format (detected)
    pub format: ConfigFormat,
    /// Size in bytes
    pub size: usize,
    /// Retrieval duration
    pub duration: Duration,
    /// Whether result came from cache
    pub from_cache: bool,
}

/// Detected configuration format
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConfigFormat {
    /// Cisco IOS/IOS-XE format
    CiscoIos,
    /// Juniper JunOS format
    JuniperJunos,
    /// Arista EOS format
    AristaEos,
    /// Generic text format
    Generic,
    /// JSON format
    Json,
    /// XML format
    Xml,
}

/// Connection manager for device connections
#[derive(Debug)]
pub struct ConnectionManager {
    /// Connection pools per device
    pools: RwLock<HashMap<Uuid, DeviceConnectionPool>>,
    /// Global connection semaphore
    global_semaphore: Semaphore,
    /// Manager configuration
    config: LiveConfigManagerConfig,
}

/// Per-device connection pool
#[derive(Debug)]
pub struct DeviceConnectionPool {
    /// Device-specific semaphore
    semaphore: Semaphore,
    /// Active connections count
    active_connections: RwLock<usize>,
    /// Last activity timestamp
    last_activity: RwLock<SystemTime>,
}

/// Configuration cache
#[derive(Debug)]
pub struct ConfigCache {
    /// Cached configurations
    entries: HashMap<Uuid, CachedConfig>,
    /// Access order for LRU eviction
    access_order: Vec<Uuid>,
    /// Current cache size
    current_size: usize,
    /// Maximum cache size
    max_size: usize,
    /// Cache TTL
    ttl: Duration,
}

/// Cached configuration entry
#[derive(Debug, Clone)]
pub struct CachedConfig {
    /// Configuration result
    pub result: ConfigResult,
    /// Cache timestamp
    pub cached_at: SystemTime,
    /// Access count
    pub access_count: usize,
    /// Last access timestamp
    pub last_accessed: SystemTime,
}

impl LiveConfigManager {
    /// Create a new live config manager
    pub fn new(config: LiveConfigManagerConfig) -> Result<Self> {
        let snmp_config = SnmpClientConfig {
            max_connections: config.max_connections_per_device * 10, // Global limit
            default_session: crate::snmp::SessionConfig {
                address: "0.0.0.0:161".parse().unwrap(), // Will be overridden
                version: config.snmp.version,
                credentials: crate::snmp::SnmpCredentials::Community {
                    community: config.snmp.community.clone(),
                },
                timeout: config.snmp.timeout,
                retries: config.snmp.retries,
                max_vars_per_request: 10,
            },
            health_check_interval: Duration::from_secs(60),
            session_timeout: Duration::from_secs(300),
        };

        let snmp_client = Arc::new(SnmpClient::new(snmp_config));

        let connection_manager = Arc::new(ConnectionManager {
            pools: RwLock::new(HashMap::new()),
            global_semaphore: Semaphore::new(config.max_connections_per_device * 100),
            config: config.clone(),
        });

        let config_cache = Arc::new(RwLock::new(ConfigCache {
            entries: HashMap::new(),
            access_order: Vec::new(),
            current_size: 0,
            max_size: config.max_cache_size,
            ttl: config.cache_ttl,
        }));

        Ok(Self {
            connection_manager,
            snmp_client,
            config_cache,
            config,
        })
    }

    /// Retrieve configuration from a device
    pub async fn get_config(&self, request: ConfigRequest) -> Result<ConfigResult> {
        let node_id = request.node.id;

        // Check cache first unless force refresh is requested
        if !request.force_refresh {
            if let Some(cached_result) = self.get_cached_config(node_id).await {
                info!(
                    node_id = %node_id,
                    node_name = %request.node.name,
                    "Returning cached configuration"
                );
                return Ok(cached_result);
            }
        }

        // Acquire connection permit
        let _permit = self.connection_manager.acquire_permit(node_id).await?;

        // Retrieve configuration based on protocol
        let result = match request.protocol {
            ConfigProtocol::Ssh => self.get_config_via_ssh(&request).await,
            ConfigProtocol::Snmp => self.get_config_via_snmp(&request).await,
            ConfigProtocol::RestApi => self.get_config_via_api(&request).await,
            ConfigProtocol::Netconf => self.get_config_via_netconf(&request).await,
        }?;

        // Cache the result
        self.cache_config(result.clone()).await;

        info!(
            node_id = %node_id,
            node_name = %request.node.name,
            protocol = ?request.protocol,
            size = result.size,
            duration_ms = result.duration.as_millis(),
            "Successfully retrieved configuration"
        );

        Ok(result)
    }

    /// Get multiple configurations concurrently
    pub async fn get_configs_batch(
        &self,
        requests: Vec<ConfigRequest>,
    ) -> Vec<Result<ConfigResult>> {
        let tasks = requests.into_iter().map(|request| {
            let manager = self.clone();
            tokio::spawn(async move { manager.get_config(request).await })
        });

        let results = futures::future::join_all(tasks).await;
        results
            .into_iter()
            .map(|result| {
                result.unwrap_or_else(|e| Err(Error::network("async_task", e.to_string())))
            })
            .collect()
    }

    /// Get cached configuration if available and not expired
    async fn get_cached_config(&self, node_id: Uuid) -> Option<ConfigResult> {
        let mut cache = self.config_cache.write().await;

        // First check if entry exists and if it's expired
        let should_remove = if let Some(cached) = cache.entries.get(&node_id) {
            cached.cached_at.elapsed().unwrap_or(Duration::MAX) > cache.ttl
        } else {
            return None;
        };

        if should_remove {
            cache.entries.remove(&node_id);
            cache.access_order.retain(|&id| id != node_id);
            return None;
        }

        // Now update access statistics
        if let Some(cached) = cache.entries.get_mut(&node_id) {
            cached.access_count += 1;
            cached.last_accessed = SystemTime::now();

            let mut result = cached.result.clone();
            result.from_cache = true;

            // Update LRU order separately to avoid borrow conflict
            drop(cached); // Release the mutable borrow
            cache.access_order.retain(|&id| id != node_id);
            cache.access_order.push(node_id);

            return Some(result);
        }

        None
    }

    /// Cache a configuration result
    async fn cache_config(&self, mut result: ConfigResult) {
        let mut cache = self.config_cache.write().await;

        // Ensure cache doesn't exceed maximum size
        while cache.entries.len() >= cache.max_size && !cache.access_order.is_empty() {
            if let Some(oldest_id) = cache.access_order.first().cloned() {
                cache.entries.remove(&oldest_id);
                cache.access_order.remove(0);
            }
        }

        let node_id = result.node_id;
        result.from_cache = false;
        let cached = CachedConfig {
            result,
            cached_at: SystemTime::now(),
            access_count: 1,
            last_accessed: SystemTime::now(),
        };

        cache.entries.insert(node_id, cached);
        cache.access_order.push(node_id);
    }

    /// Retrieve configuration via SSH
    async fn get_config_via_ssh(&self, request: &ConfigRequest) -> Result<ConfigResult> {
        let start_time = SystemTime::now();

        debug!(
            node_id = %request.node.id,
            node_name = %request.node.name,
            "Retrieving configuration via SSH"
        );

        let timeout_duration = request.timeout.unwrap_or(self.config.ssh.timeout);

        let (username, auth_method) = match &request.credentials {
            DeviceCredentials::SshPassword { username, password } => {
                (username.clone(), SshAuthMethod::Password(password.clone()))
            }
            DeviceCredentials::SshKey {
                username,
                private_key_path,
                passphrase,
            } => (
                username.clone(),
                SshAuthMethod::Key {
                    path: private_key_path.clone(),
                    passphrase: passphrase.clone(),
                },
            ),
            _ => {
                return Err(Error::validation(
                    "credentials",
                    "SSH protocol requires SSH credentials",
                ));
            }
        };

        // Get management IP from node
        let management_ip = self.get_management_ip(&request.node)?;

        // Execute SSH command to get configuration
        let config_content = self
            .execute_ssh_command(
                management_ip,
                self.config.ssh.port,
                &username,
                &auth_method,
                &self.get_config_command(&request.node.vendor),
                timeout_duration,
            )
            .await?;

        let duration = start_time.elapsed().unwrap_or(Duration::ZERO);
        let format = self.detect_config_format(&config_content, &request.node.vendor);
        let size = config_content.len();

        Ok(ConfigResult {
            node_id: request.node.id,
            config_content,
            retrieved_at: start_time,
            protocol: ConfigProtocol::Ssh,
            format,
            size,
            duration,
            from_cache: false,
        })
    }

    /// Retrieve configuration via SNMP
    async fn get_config_via_snmp(&self, request: &ConfigRequest) -> Result<ConfigResult> {
        let start_time = SystemTime::now();

        debug!(
            node_id = %request.node.id,
            node_name = %request.node.name,
            "Retrieving configuration via SNMP"
        );

        // Get management IP from node
        let management_ip = self.get_management_ip(&request.node)?;
        let snmp_address = SocketAddr::new(management_ip, self.config.snmp.port);

        // Build SNMP session config from credentials
        let session_config = self.build_snmp_session_config(snmp_address, &request.credentials)?;

        // Get configuration via SNMP (vendor-specific OIDs)
        let config_oids = self.get_config_oids(&request.node.vendor);
        let snmp_result = self
            .snmp_client
            .get(snmp_address, &config_oids, Some(session_config))
            .await
            .map_err(|e| {
                Error::snmp(
                    snmp_address.to_string(),
                    format!("SNMP configuration retrieval failed: {}", e),
                )
            })?;

        // Convert SNMP response to configuration text
        let config_content = self.snmp_response_to_config(snmp_result, &request.node.vendor)?;

        let duration = start_time.elapsed().unwrap_or(Duration::ZERO);
        let format = self.detect_config_format(&config_content, &request.node.vendor);

        let size = config_content.len();
        Ok(ConfigResult {
            node_id: request.node.id,
            config_content,
            retrieved_at: start_time,
            protocol: ConfigProtocol::Snmp,
            format,
            size,
            duration,
            from_cache: false,
        })
    }

    /// Retrieve configuration via REST API
    async fn get_config_via_api(&self, request: &ConfigRequest) -> Result<ConfigResult> {
        let start_time = SystemTime::now();

        debug!(
            node_id = %request.node.id,
            node_name = %request.node.name,
            "Retrieving configuration via REST API"
        );

        // Get management IP from node
        let management_ip = self.get_management_ip(&request.node)?;

        // Build API client with appropriate authentication
        let client = self.build_api_client(&request.credentials)?;

        // Get vendor-specific API endpoint
        let api_url =
            self.get_config_api_url(management_ip, self.config.api.port, &request.node.vendor)?;

        let timeout_duration = request.timeout.unwrap_or(self.config.api.timeout);

        // Make API request
        let response = timeout(timeout_duration, client.get(&api_url).send())
            .await
            .map_err(|_| Error::network(&api_url, "API request timeout"))?
            .map_err(|e| Error::network(&api_url, format!("API request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::network(
                &api_url,
                format!("API request failed with status: {}", response.status()),
            ));
        }

        let config_content = response
            .text()
            .await
            .map_err(|e| Error::network(&api_url, format!("Failed to read API response: {}", e)))?;

        let duration = start_time.elapsed().unwrap_or(Duration::ZERO);
        let format = self.detect_config_format(&config_content, &request.node.vendor);
        let size = config_content.len();

        Ok(ConfigResult {
            node_id: request.node.id,
            config_content,
            retrieved_at: start_time,
            protocol: ConfigProtocol::RestApi,
            format,
            size,
            duration,
            from_cache: false,
        })
    }

    /// Retrieve configuration via NETCONF
    async fn get_config_via_netconf(&self, request: &ConfigRequest) -> Result<ConfigResult> {
        let start_time = SystemTime::now();

        debug!(
            node_id = %request.node.id,
            node_name = %request.node.name,
            "Retrieving configuration via NETCONF"
        );

        // TODO: Implement NETCONF client
        // For now, return a placeholder error
        Err(Error::config(
            "NETCONF protocol not yet implemented. Use SSH, SNMP, or REST API instead.",
        ))
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> CacheStats {
        let cache = self.config_cache.read().await;
        CacheStats {
            total_entries: cache.entries.len(),
            max_size: cache.max_size,
            hit_ratio: self.calculate_hit_ratio(&cache).await,
            oldest_entry_age: self.get_oldest_entry_age(&cache).await,
            total_size_bytes: cache.entries.values().map(|entry| entry.result.size).sum(),
        }
    }

    /// Clear cache for specific node or all nodes
    pub async fn clear_cache(&self, node_id: Option<Uuid>) {
        let mut cache = self.config_cache.write().await;

        if let Some(id) = node_id {
            cache.entries.remove(&id);
            cache.access_order.retain(|&access_id| access_id != id);
            info!(node_id = %id, "Cleared configuration cache for node");
        } else {
            cache.entries.clear();
            cache.access_order.clear();
            info!("Cleared all configuration cache entries");
        }
    }

    /// Helper methods for protocol-specific operations

    /// Get management IP address from node
    fn get_management_ip(&self, node: &Node) -> Result<IpAddr> {
        // Check if node has a management IP already set
        if let Some(ip) = node.management_ip {
            return Ok(ip);
        }

        // Try to parse FQDN as IP address first
        if let Ok(ip) = node.fqdn.parse::<IpAddr>() {
            return Ok(ip);
        }

        // TODO: Implement DNS resolution for FQDN
        Err(Error::network(
            &node.fqdn,
            format!("DNS resolution not implemented for FQDN: {}", node.fqdn),
        ))
    }

    /// Get vendor-specific configuration command
    fn get_config_command(&self, vendor: &Vendor) -> String {
        match vendor {
            Vendor::Cisco => "show running-config".to_string(),
            Vendor::Juniper => "show configuration | display set".to_string(),
            Vendor::Arista => "show running-config".to_string(),
            Vendor::Generic => "show config".to_string(),
            _ => "show running-config".to_string(),
        }
    }

    /// Get vendor-specific SNMP OIDs for configuration
    fn get_config_oids(&self, vendor: &Vendor) -> Vec<&'static str> {
        match vendor {
            Vendor::Cisco => vec![
                "1.3.6.1.4.1.9.2.1.40.0", // ccmHistoryRunningConfig
                "1.3.6.1.4.1.9.2.1.41.0", // ccmHistoryStartupConfig
            ],
            Vendor::Juniper => vec![
                "1.3.6.1.4.1.2636.1.6.1.1.2.0", // jnxVirtualChassisMemberSWVersion
            ],
            Vendor::Arista => vec![
                "1.3.6.1.4.1.30065.1.2.1.2.1.0", // aristaConfigRevision
            ],
            _ => vec!["1.3.6.1.2.1.1.1.0"], // sysDescr as fallback
        }
    }

    /// Detect configuration format based on content and vendor
    fn detect_config_format(&self, content: &str, vendor: &Vendor) -> ConfigFormat {
        // Check for JSON format
        if content.trim_start().starts_with('{') && content.trim_end().ends_with('}') {
            return ConfigFormat::Json;
        }

        // Check for XML format
        if content.trim_start().starts_with('<') && content.contains("</") {
            return ConfigFormat::Xml;
        }

        // Vendor-specific format detection
        match vendor {
            Vendor::Cisco => {
                if content.contains("interface ") && content.contains("router ") {
                    ConfigFormat::CiscoIos
                } else {
                    ConfigFormat::Generic
                }
            }
            Vendor::Juniper => {
                if content.contains("set ") || content.contains("interfaces {") {
                    ConfigFormat::JuniperJunos
                } else {
                    ConfigFormat::Generic
                }
            }
            Vendor::Arista => {
                if content.contains("interface Ethernet") || content.contains("router bgp") {
                    ConfigFormat::AristaEos
                } else {
                    ConfigFormat::Generic
                }
            }
            _ => ConfigFormat::Generic,
        }
    }

    // Additional helper methods would be implemented here...
    // (SSH execution, SNMP session building, API client building, etc.)
}

// SSH authentication method
#[derive(Debug, Clone)]
enum SshAuthMethod {
    Password(String),
    Key {
        path: String,
        passphrase: Option<String>,
    },
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total number of cached entries
    pub total_entries: usize,
    /// Maximum cache size
    pub max_size: usize,
    /// Cache hit ratio (0.0 to 1.0)
    pub hit_ratio: f64,
    /// Age of oldest entry in seconds
    pub oldest_entry_age: Option<u64>,
    /// Total size of cached data in bytes
    pub total_size_bytes: usize,
}

// Placeholder implementations for helper methods that would require additional dependencies
impl LiveConfigManager {
    async fn execute_ssh_command(
        &self,
        _host: IpAddr,
        _port: u16,
        _username: &str,
        _auth_method: &SshAuthMethod,
        _command: &str,
        _timeout: Duration,
    ) -> Result<String> {
        // TODO: Implement actual SSH client using a crate like tokio-ssh2 or russh
        Err(Error::config(
            "SSH client implementation pending - requires ssh2 or russh crate integration",
        ))
    }

    fn build_snmp_session_config(
        &self,
        address: SocketAddr,
        credentials: &DeviceCredentials,
    ) -> Result<crate::snmp::SessionConfig> {
        let snmp_credentials = match credentials {
            DeviceCredentials::SnmpCommunity { community } => {
                crate::snmp::SnmpCredentials::Community {
                    community: community.clone(),
                }
            }
            DeviceCredentials::SnmpV3 {
                username,
                auth_protocol,
                auth_password,
                privacy_protocol,
                privacy_password,
            } => crate::snmp::SnmpCredentials::UserBased {
                username: username.clone(),
                auth: auth_protocol.as_ref().and_then(|proto| {
                    auth_password
                        .as_ref()
                        .map(|pass| (proto.clone(), pass.clone()))
                }),
                privacy: privacy_protocol.as_ref().and_then(|proto| {
                    privacy_password
                        .as_ref()
                        .map(|pass| (proto.clone(), pass.clone()))
                }),
            },
            _ => {
                return Err(Error::validation(
                    "credentials",
                    "SNMP protocol requires SNMP credentials",
                ));
            }
        };

        Ok(crate::snmp::SessionConfig {
            address,
            version: self.config.snmp.version,
            credentials: snmp_credentials,
            timeout: self.config.snmp.timeout,
            retries: self.config.snmp.retries,
            max_vars_per_request: 10,
        })
    }

    fn build_api_client(&self, _credentials: &DeviceCredentials) -> Result<reqwest::Client> {
        // TODO: Implement HTTP client with proper authentication
        Err(Error::config(
            "HTTP API client implementation pending - requires reqwest integration",
        ))
    }

    fn get_config_api_url(&self, _host: IpAddr, _port: u16, _vendor: &Vendor) -> Result<String> {
        // TODO: Implement vendor-specific API endpoints
        Err(Error::config(
            "Vendor-specific API endpoints not yet implemented",
        ))
    }

    fn snmp_response_to_config(
        &self,
        _response: HashMap<String, crate::snmp::SnmpValue>,
        _vendor: &Vendor,
    ) -> Result<String> {
        // TODO: Convert SNMP response to configuration text
        Err(Error::config("SNMP response parsing not yet implemented"))
    }

    async fn calculate_hit_ratio(&self, _cache: &ConfigCache) -> f64 {
        // TODO: Implement hit ratio calculation based on access statistics
        0.0
    }

    async fn get_oldest_entry_age(&self, cache: &ConfigCache) -> Option<u64> {
        cache
            .entries
            .values()
            .map(|entry| {
                entry
                    .cached_at
                    .elapsed()
                    .unwrap_or(Duration::ZERO)
                    .as_secs()
            })
            .max()
    }
}

impl Clone for LiveConfigManager {
    fn clone(&self) -> Self {
        Self {
            connection_manager: Arc::clone(&self.connection_manager),
            snmp_client: Arc::clone(&self.snmp_client),
            config_cache: Arc::clone(&self.config_cache),
            config: self.config.clone(),
        }
    }
}

impl ConnectionManager {
    /// Acquire connection permit for a device
    async fn acquire_permit(&self, node_id: Uuid) -> Result<ConnectionPermit> {
        // For now, just use a simple permit tracking without complex lifetime management
        let _global_permit =
            self.global_semaphore.acquire().await.map_err(|_| {
                Error::network("connection_pool", "Global connection pool exhausted")
            })?;

        // Get or create device pool
        let device_pool = {
            let mut pools = self.pools.write().await;
            pools
                .entry(node_id)
                .or_insert_with(|| DeviceConnectionPool {
                    semaphore: Semaphore::new(self.config.max_connections_per_device),
                    active_connections: RwLock::new(0),
                    last_activity: RwLock::new(SystemTime::now()),
                })
                .clone()
        };

        // Acquire device-specific permit
        let _device_permit = device_pool.semaphore.acquire().await.map_err(|_| {
            Error::network(&node_id.to_string(), "Device connection pool exhausted")
        })?;

        // Update connection count and activity
        {
            let mut active = device_pool.active_connections.write().await;
            *active += 1;
        }
        {
            let mut activity = device_pool.last_activity.write().await;
            *activity = SystemTime::now();
        }

        // Clone the pool to avoid the move issue with the permit
        let pool_clone = device_pool.clone();

        Ok(ConnectionPermit {
            node_id,
            pool: pool_clone,
        })
    }
}

impl Clone for DeviceConnectionPool {
    fn clone(&self) -> Self {
        Self {
            semaphore: Semaphore::new(self.semaphore.available_permits()),
            active_connections: RwLock::new(0),
            last_activity: RwLock::new(SystemTime::now()),
        }
    }
}

/// Connection permit that manages resource cleanup
pub struct ConnectionPermit {
    node_id: Uuid,
    pool: DeviceConnectionPool,
}

impl Drop for ConnectionPermit {
    fn drop(&mut self) {
        // Update active connection count
        if let Ok(mut active) = self.pool.active_connections.try_write() {
            if *active > 0 {
                *active -= 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DeviceRole, Lifecycle};

    fn create_test_node() -> Node {
        Node {
            id: Uuid::new_v4(),
            name: "test-device".to_string(),
            domain: "example.com".to_string(),
            fqdn: "test-device.example.com".to_string(),
            vendor: Vendor::Cisco,
            model: "ISR4321".to_string(),
            role: DeviceRole::Router,
            lifecycle: Lifecycle::Live,
            management_ip: None,
            location_id: None,
            platform: Some("IOS".to_string()),
            version: Some("16.9.4".to_string()),
            serial_number: Some("ABC123".to_string()),
            asset_tag: None,
            purchase_date: None,
            warranty_expires: None,
            custom_data: serde_json::Value::Null,
        }
    }

    #[test]
    fn test_live_config_manager_creation() {
        let config = LiveConfigManagerConfig::default();
        let manager = LiveConfigManager::new(config);
        assert!(manager.is_ok());
    }

    #[test]
    fn test_config_format_detection() {
        let config = LiveConfigManagerConfig::default();
        let manager = LiveConfigManager::new(config).unwrap();

        // Test JSON detection
        let json_config = r#"{"interface": "eth0"}"#;
        assert_eq!(
            manager.detect_config_format(json_config, &Vendor::Generic),
            ConfigFormat::Json
        );

        // Test XML detection
        let xml_config = r#"<config><interface>eth0</interface></config>"#;
        assert_eq!(
            manager.detect_config_format(xml_config, &Vendor::Generic),
            ConfigFormat::Xml
        );

        // Test Cisco IOS detection
        let cisco_config = "interface GigabitEthernet0/1\n router bgp 65001";
        assert_eq!(
            manager.detect_config_format(cisco_config, &Vendor::Cisco),
            ConfigFormat::CiscoIos
        );

        // Test Juniper detection
        let juniper_config = "set interfaces ge-0/0/0 unit 0 family inet address 10.1.1.1/24";
        assert_eq!(
            manager.detect_config_format(juniper_config, &Vendor::Juniper),
            ConfigFormat::JuniperJunos
        );
    }

    #[test]
    fn test_config_commands() {
        let config = LiveConfigManagerConfig::default();
        let manager = LiveConfigManager::new(config).unwrap();

        assert_eq!(
            manager.get_config_command(&Vendor::Cisco),
            "show running-config"
        );
        assert_eq!(
            manager.get_config_command(&Vendor::Juniper),
            "show configuration | display set"
        );
        assert_eq!(
            manager.get_config_command(&Vendor::Arista),
            "show running-config"
        );
    }

    #[tokio::test]
    async fn test_cache_operations() {
        let config = LiveConfigManagerConfig::default();
        let manager = LiveConfigManager::new(config).unwrap();

        let node = create_test_node();
        let config_result = ConfigResult {
            node_id: node.id,
            config_content: "test config".to_string(),
            retrieved_at: SystemTime::now(),
            protocol: ConfigProtocol::Ssh,
            format: ConfigFormat::CiscoIos,
            size: 11,
            duration: Duration::from_millis(100),
            from_cache: false,
        };

        // Cache the configuration
        manager.cache_config(config_result.clone()).await;

        // Retrieve from cache
        let cached_result = manager.get_cached_config(node.id).await;
        assert!(cached_result.is_some());
        let cached = cached_result.unwrap();
        assert!(cached.from_cache);
        assert_eq!(cached.config_content, "test config");

        // Clear cache
        manager.clear_cache(Some(node.id)).await;
        let cleared_result = manager.get_cached_config(node.id).await;
        assert!(cleared_result.is_none());
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let config = LiveConfigManagerConfig::default();
        let manager = LiveConfigManager::new(config).unwrap();

        let stats = manager.cache_stats().await;
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.total_size_bytes, 0);
    }
}
