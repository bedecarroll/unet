//! Network access control middleware and configuration
//!
//! This module provides comprehensive network-level security controls including
//! IP allow/deny lists, network interface restrictions, geolocation-based blocking,
//! and network security policies for fine-grained access control.

use axum::{
    extract::{ConnectInfo, Request},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    net::{IpAddr, SocketAddr},
    sync::Arc,
};
use tracing::{debug, info, warn};

/// Network access control configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkAccessConfig {
    /// Enable network access controls
    pub enabled: bool,
    /// IP address allow list (whitelist)
    pub allow_list: HashSet<IpAddr>,
    /// IP address deny list (blacklist)
    pub deny_list: HashSet<IpAddr>,
    /// IP address range allow list (CIDR notation)
    pub allow_ranges: Vec<String>,
    /// IP address range deny list (CIDR notation)
    pub deny_ranges: Vec<String>,
    /// Country codes to block (ISO 3166-1 alpha-2)
    pub blocked_countries: HashSet<String>,
    /// Country codes to allow (if specified, only these are allowed)
    pub allowed_countries: Option<HashSet<String>>,
    /// Network segments configuration
    pub network_segments: HashMap<String, NetworkSegmentPolicy>,
    /// Default action for unknown IPs
    pub default_action: NetworkAction,
    /// Enable geolocation lookup
    pub enable_geolocation: bool,
    /// Maximum request size from untrusted networks
    pub untrusted_max_request_size: usize,
    /// Enable network-based rate limiting multipliers
    pub enable_network_rate_limits: bool,
}

impl Default for NetworkAccessConfig {
    fn default() -> Self {
        let mut allow_list = HashSet::new();
        allow_list.insert("127.0.0.1".parse().unwrap());
        allow_list.insert("::1".parse().unwrap());

        Self {
            enabled: true,
            allow_list,
            deny_list: HashSet::new(),
            allow_ranges: vec!["192.168.0.0/16".to_string(), "10.0.0.0/8".to_string()],
            deny_ranges: vec![],
            blocked_countries: HashSet::new(),
            allowed_countries: None,
            network_segments: HashMap::new(),
            default_action: NetworkAction::Allow,
            enable_geolocation: false,
            untrusted_max_request_size: 64 * 1024, // 64KB for untrusted networks
            enable_network_rate_limits: true,
        }
    }
}

/// Network access actions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NetworkAction {
    /// Allow the request
    Allow,
    /// Deny the request
    Deny,
    /// Allow with restrictions (rate limiting, size limits)
    AllowRestricted,
    /// Log and allow (for monitoring)
    LogAndAllow,
}

/// Network segment policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSegmentPolicy {
    /// CIDR ranges for this segment
    pub cidr_ranges: Vec<String>,
    /// Action for this network segment
    pub action: NetworkAction,
    /// Rate limit multiplier (1.0 = normal, 0.5 = half rate, 2.0 = double rate)
    pub rate_limit_multiplier: f64,
    /// Maximum request size for this segment
    pub max_request_size: Option<usize>,
    /// Additional security headers to add
    pub additional_headers: HashMap<String, String>,
    /// Description of this network segment
    pub description: String,
}

impl Default for NetworkSegmentPolicy {
    fn default() -> Self {
        Self {
            cidr_ranges: vec![],
            action: NetworkAction::Allow,
            rate_limit_multiplier: 1.0,
            max_request_size: None,
            additional_headers: HashMap::new(),
            description: "Default network segment".to_string(),
        }
    }
}

/// Geolocation information for an IP address
#[derive(Debug, Clone)]
pub struct GeolocationInfo {
    /// Country code (ISO 3166-1 alpha-2)
    pub country_code: Option<String>,
    /// Country name
    pub country_name: Option<String>,
    /// Region/state
    pub region: Option<String>,
    /// City
    pub city: Option<String>,
    /// ISP/Organization
    pub isp: Option<String>,
    /// Whether this is a known VPN/proxy
    pub is_proxy: bool,
    /// Whether this is a known Tor exit node
    pub is_tor: bool,
}

/// Network access control manager
#[derive(Debug)]
pub struct NetworkAccessControl {
    config: NetworkAccessConfig,
    compiled_ranges: CompiledRanges,
}

/// Pre-compiled IP ranges for efficient lookup
#[derive(Debug)]
struct CompiledRanges {
    allow_ranges: Vec<ipnet::IpNet>,
    deny_ranges: Vec<ipnet::IpNet>,
    segment_ranges: HashMap<String, Vec<ipnet::IpNet>>,
}

impl NetworkAccessControl {
    /// Create a new network access control manager
    pub fn new(config: NetworkAccessConfig) -> Result<Self, NetworkAccessError> {
        let compiled_ranges = Self::compile_ranges(&config)?;

        Ok(Self {
            config,
            compiled_ranges,
        })
    }

    /// Compile IP ranges for efficient lookup
    fn compile_ranges(config: &NetworkAccessConfig) -> Result<CompiledRanges, NetworkAccessError> {
        let mut allow_ranges = Vec::new();
        let mut deny_ranges = Vec::new();
        let mut segment_ranges = HashMap::new();

        // Compile allow ranges
        for range_str in &config.allow_ranges {
            let range = range_str
                .parse::<ipnet::IpNet>()
                .map_err(|e| NetworkAccessError::InvalidCidr(range_str.clone(), e.to_string()))?;
            allow_ranges.push(range);
        }

        // Compile deny ranges
        for range_str in &config.deny_ranges {
            let range = range_str
                .parse::<ipnet::IpNet>()
                .map_err(|e| NetworkAccessError::InvalidCidr(range_str.clone(), e.to_string()))?;
            deny_ranges.push(range);
        }

        // Compile segment ranges
        for (segment_name, segment_policy) in &config.network_segments {
            let mut ranges = Vec::new();
            for range_str in &segment_policy.cidr_ranges {
                let range = range_str.parse::<ipnet::IpNet>().map_err(|e| {
                    NetworkAccessError::InvalidCidr(range_str.clone(), e.to_string())
                })?;
                ranges.push(range);
            }
            segment_ranges.insert(segment_name.clone(), ranges);
        }

        Ok(CompiledRanges {
            allow_ranges,
            deny_ranges,
            segment_ranges,
        })
    }

    /// Check if a request should be allowed
    pub async fn check_access(&self, ip: IpAddr, headers: &HeaderMap) -> NetworkAccessResult {
        if !self.config.enabled {
            return NetworkAccessResult::new(NetworkAction::Allow, None, None);
        }

        // 1. Check explicit deny list first
        if self.config.deny_list.contains(&ip) {
            debug!("IP {} found in deny list", ip);
            return NetworkAccessResult::new(
                NetworkAction::Deny,
                Some("IP address is blacklisted".to_string()),
                None,
            );
        }

        // 2. Check explicit allow list
        if self.config.allow_list.contains(&ip) {
            debug!("IP {} found in allow list", ip);
            return NetworkAccessResult::new(NetworkAction::Allow, None, None);
        }

        // 3. Check deny ranges
        for range in &self.compiled_ranges.deny_ranges {
            if range.contains(&ip) {
                debug!("IP {} found in deny range {}", ip, range);
                return NetworkAccessResult::new(
                    NetworkAction::Deny,
                    Some(format!("IP address is in blocked range {}", range)),
                    None,
                );
            }
        }

        // 4. Check allow ranges
        for range in &self.compiled_ranges.allow_ranges {
            if range.contains(&ip) {
                debug!("IP {} found in allow range {}", ip, range);
                return NetworkAccessResult::new(NetworkAction::Allow, None, None);
            }
        }

        // 5. Check network segments
        for (segment_name, ranges) in &self.compiled_ranges.segment_ranges {
            for range in ranges {
                if range.contains(&ip) {
                    if let Some(policy) = self.config.network_segments.get(segment_name) {
                        debug!(
                            "IP {} found in network segment {} ({})",
                            ip, segment_name, policy.description
                        );
                        return NetworkAccessResult::new(
                            policy.action.clone(),
                            Some(format!("Network segment: {}", segment_name)),
                            Some(policy.clone()),
                        );
                    }
                }
            }
        }

        // 6. Check geolocation if enabled
        if self.config.enable_geolocation {
            if let Ok(geo_info) = self.get_geolocation_info(ip).await {
                if let Some(country_code) = &geo_info.country_code {
                    // Check blocked countries
                    if self.config.blocked_countries.contains(country_code) {
                        debug!("IP {} from blocked country {}", ip, country_code);
                        return NetworkAccessResult::new(
                            NetworkAction::Deny,
                            Some(format!("Country {} is blocked", country_code)),
                            None,
                        );
                    }

                    // Check allowed countries (if specified)
                    if let Some(ref allowed_countries) = self.config.allowed_countries {
                        if !allowed_countries.contains(country_code) {
                            debug!("IP {} from non-allowed country {}", ip, country_code);
                            return NetworkAccessResult::new(
                                NetworkAction::Deny,
                                Some(format!("Country {} is not in allowed list", country_code)),
                                None,
                            );
                        }
                    }

                    // Block known proxies/VPNs if configured
                    if geo_info.is_proxy || geo_info.is_tor {
                        warn!("IP {} identified as proxy/VPN/Tor", ip);
                        return NetworkAccessResult::new(
                            NetworkAction::AllowRestricted,
                            Some("Proxy/VPN/Tor detected".to_string()),
                            None,
                        );
                    }
                }
            }
        }

        // 7. Apply default action
        debug!(
            "IP {} using default action: {:?}",
            ip, self.config.default_action
        );
        NetworkAccessResult::new(self.config.default_action.clone(), None, None)
    }

    /// Get geolocation information for an IP address
    async fn get_geolocation_info(
        &self,
        ip: IpAddr,
    ) -> Result<GeolocationInfo, NetworkAccessError> {
        // In a real implementation, this would query a geolocation service
        // like MaxMind GeoIP2, IP2Location, or a similar service
        // For now, we'll return a placeholder implementation

        // Skip geolocation for private/local addresses
        let is_private = match ip {
            IpAddr::V4(ipv4) => ipv4.is_private(),
            IpAddr::V6(ipv6) => ipv6.is_loopback() || ipv6.segments()[0] & 0xfe00 == 0xfc00, // Unique local addresses
        };

        if ip.is_loopback() || is_private {
            return Ok(GeolocationInfo {
                country_code: Some("XX".to_string()), // Unknown/Private
                country_name: Some("Private Network".to_string()),
                region: None,
                city: None,
                isp: None,
                is_proxy: false,
                is_tor: false,
            });
        }

        // Placeholder implementation - in production, integrate with real geolocation service
        info!("Geolocation lookup for {} (placeholder implementation)", ip);
        Ok(GeolocationInfo {
            country_code: Some("US".to_string()), // Default to US for placeholder
            country_name: Some("United States".to_string()),
            region: None,
            city: None,
            isp: None,
            is_proxy: false,
            is_tor: false,
        })
    }

    /// Update configuration dynamically
    pub fn update_config(&mut self, config: NetworkAccessConfig) -> Result<(), NetworkAccessError> {
        let compiled_ranges = Self::compile_ranges(&config)?;
        self.config = config;
        self.compiled_ranges = compiled_ranges;
        info!("Network access control configuration updated");
        Ok(())
    }

    /// Get current configuration
    pub const fn get_config(&self) -> &NetworkAccessConfig {
        &self.config
    }

    /// Get statistics about blocked/allowed requests
    pub fn get_statistics(&self) -> NetworkAccessStatistics {
        // In a real implementation, this would return actual statistics
        // For now, return placeholder data
        NetworkAccessStatistics {
            total_requests: 0,
            allowed_requests: 0,
            denied_requests: 0,
            restricted_requests: 0,
            blocked_countries: self.config.blocked_countries.len(),
            blocked_ips: self.config.deny_list.len(),
            allowed_ips: self.config.allow_list.len(),
        }
    }
}

/// Result of network access check
#[derive(Debug, Clone)]
pub struct NetworkAccessResult {
    pub action: NetworkAction,
    pub reason: Option<String>,
    pub segment_policy: Option<NetworkSegmentPolicy>,
}

impl NetworkAccessResult {
    const fn new(
        action: NetworkAction,
        reason: Option<String>,
        segment_policy: Option<NetworkSegmentPolicy>,
    ) -> Self {
        Self {
            action,
            reason,
            segment_policy,
        }
    }
}

/// Network access statistics
#[derive(Debug, Serialize)]
pub struct NetworkAccessStatistics {
    pub total_requests: u64,
    pub allowed_requests: u64,
    pub denied_requests: u64,
    pub restricted_requests: u64,
    pub blocked_countries: usize,
    pub blocked_ips: usize,
    pub allowed_ips: usize,
}

/// Network access control errors
#[derive(Debug, thiserror::Error)]
pub enum NetworkAccessError {
    #[error("Invalid CIDR range '{0}': {1}")]
    InvalidCidr(String, String),
    #[error("Geolocation service error: {0}")]
    GeolocationError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Network access control middleware
pub async fn network_access_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Create default network access control (in production, this would be injected)
    static NETWORK_ACCESS: tokio::sync::OnceCell<Arc<NetworkAccessControl>> =
        tokio::sync::OnceCell::const_new();
    let network_access = NETWORK_ACCESS
        .get_or_init(|| async {
            let config = NetworkAccessConfig::default();
            Arc::new(NetworkAccessControl::new(config).unwrap())
        })
        .await;

    let ip = addr.ip();

    // Check network access
    let access_result = network_access.check_access(ip, &headers).await;

    match access_result.action {
        NetworkAction::Allow => {
            debug!("Network access allowed for {}", ip);
            Ok(next.run(request).await)
        }
        NetworkAction::Deny => {
            let reason = access_result
                .reason
                .unwrap_or_else(|| "Access denied".to_string());
            warn!("Network access denied for {}: {}", ip, reason);
            Err(StatusCode::FORBIDDEN)
        }
        NetworkAction::AllowRestricted => {
            let reason = access_result
                .reason
                .clone()
                .unwrap_or_else(|| "Restricted access".to_string());
            warn!("Network access restricted for {}: {}", ip, reason);
            // Add headers to indicate restricted access
            let mut response = next.run(request).await;
            response
                .headers_mut()
                .insert("X-Access-Restricted", "true".parse().unwrap());
            if let Some(reason) = access_result.reason {
                response.headers_mut().insert(
                    "X-Access-Reason",
                    reason
                        .parse()
                        .unwrap_or_else(|_| "Unknown".parse().unwrap()),
                );
            }
            Ok(response)
        }
        NetworkAction::LogAndAllow => {
            let reason = access_result
                .reason
                .unwrap_or_else(|| "Logged access".to_string());
            info!("Network access logged for {}: {}", ip, reason);
            Ok(next.run(request).await)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[tokio::test]
    async fn test_network_access_allow_list() {
        let mut config = NetworkAccessConfig::default();
        config.allow_list.insert("192.168.1.100".parse().unwrap());

        let network_access = NetworkAccessControl::new(config).unwrap();
        let headers = HeaderMap::new();

        let result = network_access
            .check_access("192.168.1.100".parse().unwrap(), &headers)
            .await;
        assert_eq!(result.action, NetworkAction::Allow);

        let result = network_access
            .check_access("192.168.1.101".parse().unwrap(), &headers)
            .await;
        assert_eq!(result.action, NetworkAction::Allow); // In allow range
    }

    #[tokio::test]
    async fn test_network_access_deny_list() {
        let mut config = NetworkAccessConfig::default();
        config.deny_list.insert("10.0.0.100".parse().unwrap());

        let network_access = NetworkAccessControl::new(config).unwrap();
        let headers = HeaderMap::new();

        let result = network_access
            .check_access("10.0.0.100".parse().unwrap(), &headers)
            .await;
        assert_eq!(result.action, NetworkAction::Deny);

        let result = network_access
            .check_access("10.0.0.101".parse().unwrap(), &headers)
            .await;
        assert_eq!(result.action, NetworkAction::Allow); // In allow range, not in deny list
    }

    #[tokio::test]
    async fn test_network_access_ranges() {
        let mut config = NetworkAccessConfig::default();
        config.deny_ranges.push("203.0.113.0/24".to_string()); // TEST-NET-3
        config.allow_ranges.clear(); // Remove default allow ranges
        config.default_action = NetworkAction::Deny;

        let network_access = NetworkAccessControl::new(config).unwrap();
        let headers = HeaderMap::new();

        let result = network_access
            .check_access("203.0.113.50".parse().unwrap(), &headers)
            .await;
        assert_eq!(result.action, NetworkAction::Deny);

        let result = network_access
            .check_access("8.8.8.8".parse().unwrap(), &headers)
            .await;
        assert_eq!(result.action, NetworkAction::Deny); // Default deny
    }

    #[tokio::test]
    async fn test_network_segments() {
        let mut config = NetworkAccessConfig::default();

        let mut segment_policy = NetworkSegmentPolicy::default();
        segment_policy.action = NetworkAction::AllowRestricted;
        segment_policy.cidr_ranges.push("172.16.0.0/16".to_string());
        segment_policy.description = "Corporate network".to_string();

        config
            .network_segments
            .insert("corporate".to_string(), segment_policy);

        let network_access = NetworkAccessControl::new(config).unwrap();
        let headers = HeaderMap::new();

        let result = network_access
            .check_access("172.16.10.50".parse().unwrap(), &headers)
            .await;
        assert_eq!(result.action, NetworkAction::AllowRestricted);
        assert!(result.segment_policy.is_some());
        assert_eq!(
            result.segment_policy.unwrap().description,
            "Corporate network"
        );
    }

    #[test]
    fn test_invalid_cidr_range() {
        let mut config = NetworkAccessConfig::default();
        config.allow_ranges.push("invalid-cidr".to_string());

        let result = NetworkAccessControl::new(config);
        assert!(result.is_err());
    }
}
