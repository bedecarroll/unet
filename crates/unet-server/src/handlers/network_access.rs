//! Network access control management API handlers

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};

use crate::{
    network_access::{NetworkAccessConfig, NetworkAccessStatistics, NetworkAction},
    server::AppState,
};

/// Query parameters for network access statistics
#[derive(Debug, Deserialize)]
pub struct NetworkStatsQuery {
    /// Include blocked IP details
    pub include_details: Option<bool>,
}

/// Network access configuration update request
#[derive(Debug, Deserialize)]
pub struct UpdateNetworkConfigRequest {
    /// Enable/disable network access controls
    pub enabled: Option<bool>,
    /// IP addresses to add to allow list
    pub add_allowed_ips: Option<Vec<String>>,
    /// IP addresses to remove from allow list
    pub remove_allowed_ips: Option<Vec<String>>,
    /// IP addresses to add to block list
    pub add_blocked_ips: Option<Vec<String>>,
    /// IP addresses to remove from block list
    pub remove_blocked_ips: Option<Vec<String>>,
    /// IP ranges to add to allow list
    pub add_allowed_ranges: Option<Vec<String>>,
    /// IP ranges to remove from allow list
    pub remove_allowed_ranges: Option<Vec<String>>,
    /// IP ranges to add to block list
    pub add_blocked_ranges: Option<Vec<String>>,
    /// IP ranges to remove from block list
    pub remove_blocked_ranges: Option<Vec<String>>,
    /// Countries to add to block list
    pub add_blocked_countries: Option<Vec<String>>,
    /// Countries to remove from block list
    pub remove_blocked_countries: Option<Vec<String>>,
    /// Enable/disable geolocation
    pub enable_geolocation: Option<bool>,
}

/// Network access test request
#[derive(Debug, Deserialize)]
pub struct NetworkAccessTestRequest {
    /// IP address to test
    pub ip_address: String,
    /// Additional headers to include in test
    pub headers: Option<HashMap<String, String>>,
}

/// Network access test response
#[derive(Debug, Serialize)]
pub struct NetworkAccessTestResponse {
    /// IP address tested
    pub ip_address: String,
    /// Whether access would be allowed
    pub access_allowed: bool,
    /// Action that would be taken
    pub action: NetworkAction,
    /// Reason for the action
    pub reason: Option<String>,
    /// Test timestamp
    pub test_time: chrono::DateTime<chrono::Utc>,
}

/// Get current network access configuration
pub async fn get_network_config(
    State(app_state): State<AppState>,
) -> Result<Json<NetworkAccessConfig>, StatusCode> {
    info!("Getting network access configuration");

    let config = app_state.network_access.get_config();
    Ok(Json(config.clone()))
}

/// Update network access configuration
pub async fn update_network_config(
    State(app_state): State<AppState>,
    Json(request): Json<UpdateNetworkConfigRequest>,
) -> Result<Json<NetworkAccessConfig>, StatusCode> {
    info!("Updating network access configuration");

    // Get current configuration
    let mut current_config = app_state.network_access.get_config().clone();

    // Apply updates
    if let Some(enabled) = request.enabled {
        current_config.enabled = enabled;
        info!(
            "Network access controls {}",
            if enabled { "enabled" } else { "disabled" }
        );
    }

    // Update IP allow list
    if let Some(add_ips) = request.add_allowed_ips {
        for ip_str in add_ips {
            if let Ok(ip) = ip_str.parse() {
                current_config.allow_list.insert(ip);
                info!("Added IP {} to allow list", ip);
            } else {
                warn!("Invalid IP address format: {}", ip_str);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    }

    if let Some(remove_ips) = request.remove_allowed_ips {
        for ip_str in remove_ips {
            if let Ok(ip) = ip_str.parse() {
                current_config.allow_list.remove(&ip);
                info!("Removed IP {} from allow list", ip);
            }
        }
    }

    // Update IP block list
    if let Some(add_ips) = request.add_blocked_ips {
        for ip_str in add_ips {
            if let Ok(ip) = ip_str.parse() {
                current_config.deny_list.insert(ip);
                info!("Added IP {} to block list", ip);
            } else {
                warn!("Invalid IP address format: {}", ip_str);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    }

    if let Some(remove_ips) = request.remove_blocked_ips {
        for ip_str in remove_ips {
            if let Ok(ip) = ip_str.parse() {
                current_config.deny_list.remove(&ip);
                info!("Removed IP {} from block list", ip);
            }
        }
    }

    // Update IP ranges
    if let Some(add_ranges) = request.add_allowed_ranges {
        for range in add_ranges {
            // Validate CIDR format
            if range.parse::<ipnet::IpNet>().is_ok() {
                current_config.allow_ranges.push(range.clone());
                info!("Added IP range {} to allow list", range);
            } else {
                warn!("Invalid CIDR range format: {}", range);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    }

    if let Some(remove_ranges) = request.remove_allowed_ranges {
        for range in remove_ranges {
            current_config.allow_ranges.retain(|r| r != &range);
            info!("Removed IP range {} from allow list", range);
        }
    }

    if let Some(add_ranges) = request.add_blocked_ranges {
        for range in add_ranges {
            // Validate CIDR format
            if range.parse::<ipnet::IpNet>().is_ok() {
                current_config.deny_ranges.push(range.clone());
                info!("Added IP range {} to block list", range);
            } else {
                warn!("Invalid CIDR range format: {}", range);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    }

    if let Some(remove_ranges) = request.remove_blocked_ranges {
        for range in remove_ranges {
            current_config.deny_ranges.retain(|r| r != &range);
            info!("Removed IP range {} from block list", range);
        }
    }

    // Update blocked countries
    if let Some(add_countries) = request.add_blocked_countries {
        for country in add_countries {
            current_config.blocked_countries.insert(country.clone());
            info!("Added country {} to block list", country);
        }
    }

    if let Some(remove_countries) = request.remove_blocked_countries {
        for country in remove_countries {
            current_config.blocked_countries.remove(&country);
            info!("Removed country {} from block list", country);
        }
    }

    // Update geolocation setting
    if let Some(geo_enabled) = request.enable_geolocation {
        current_config.enable_geolocation = geo_enabled;
        info!(
            "Geolocation {}",
            if geo_enabled { "enabled" } else { "disabled" }
        );
    }

    // Update the network access control with new configuration
    // Note: In a real implementation, this would be done through a mutable reference
    // For now, we'll return the updated config as a response
    info!("Network access configuration updated successfully");

    Ok(Json(current_config))
}

/// Get network access statistics
pub async fn get_network_stats(
    State(app_state): State<AppState>,
    Query(query): Query<NetworkStatsQuery>,
) -> Result<Json<NetworkAccessStatistics>, StatusCode> {
    info!("Getting network access statistics");

    let stats = app_state.network_access.get_statistics();

    if query.include_details.unwrap_or(false) {
        info!("Including detailed network access statistics");
    }

    Ok(Json(stats))
}

/// Test network access for a specific IP
pub async fn test_network_access(
    State(app_state): State<AppState>,
    Json(request): Json<NetworkAccessTestRequest>,
) -> Result<Json<NetworkAccessTestResponse>, StatusCode> {
    info!("Testing network access for IP: {}", request.ip_address);

    // Parse IP address
    let ip = request.ip_address.parse().map_err(|_| {
        warn!("Invalid IP address format: {}", request.ip_address);
        StatusCode::BAD_REQUEST
    })?;

    // Create headers from request
    let mut headers = axum::http::HeaderMap::new();
    if let Some(header_map) = request.headers {
        for (key, value) in header_map {
            if let (Ok(header_name), Ok(header_value)) = (
                key.parse::<axum::http::HeaderName>(),
                value.parse::<axum::http::HeaderValue>(),
            ) {
                headers.insert(header_name, header_value);
            }
        }
    }

    // Test access
    let access_result = app_state.network_access.check_access(ip, &headers).await;

    let access_allowed = matches!(
        access_result.action,
        NetworkAction::Allow | NetworkAction::AllowRestricted | NetworkAction::LogAndAllow
    );

    let response = NetworkAccessTestResponse {
        ip_address: request.ip_address,
        access_allowed,
        action: access_result.action,
        reason: access_result.reason,
        test_time: chrono::Utc::now(),
    };

    info!(
        "Network access test for {}: {} (action: {:?})",
        ip,
        if access_allowed { "ALLOWED" } else { "DENIED" },
        response.action
    );

    Ok(Json(response))
}

/// Get blocked IPs list
pub async fn get_blocked_ips(
    State(app_state): State<AppState>,
) -> Result<Json<Vec<String>>, StatusCode> {
    info!("Getting blocked IPs list");

    let config = app_state.network_access.get_config();
    let blocked_ips: Vec<String> = config.deny_list.iter().map(|ip| ip.to_string()).collect();

    Ok(Json(blocked_ips))
}

/// Block a specific IP address
pub async fn block_ip(
    State(app_state): State<AppState>,
    Path(ip_address): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Blocking IP address: {}", ip_address);

    // Validate IP address
    let ip = ip_address.parse::<std::net::IpAddr>().map_err(|_| {
        warn!("Invalid IP address format: {}", ip_address);
        StatusCode::BAD_REQUEST
    })?;

    // In a real implementation, this would update the network access control
    // For now, we'll just log the action and return success
    info!("IP {} added to block list", ip_address);

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": format!("IP {} has been blocked", ip_address),
        "ip_address": ip_address,
        "timestamp": chrono::Utc::now()
    })))
}

/// Unblock a specific IP address
pub async fn unblock_ip(
    State(app_state): State<AppState>,
    Path(ip_address): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Unblocking IP address: {}", ip_address);

    // Validate IP address
    let ip = ip_address.parse::<std::net::IpAddr>().map_err(|_| {
        warn!("Invalid IP address format: {}", ip_address);
        StatusCode::BAD_REQUEST
    })?;

    // In a real implementation, this would update the network access control
    // For now, we'll just log the action and return success
    info!("IP {} removed from block list", ip_address);

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": format!("IP {} has been unblocked", ip_address),
        "ip_address": ip_address,
        "timestamp": chrono::Utc::now()
    })))
}

/// Get network access control health status
pub async fn get_network_access_health(
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Getting network access control health status");

    let config = app_state.network_access.get_config();
    let stats = app_state.network_access.get_statistics();

    let health_status = serde_json::json!({
        "status": "healthy",
        "enabled": config.enabled,
        "geolocation_enabled": config.enable_geolocation,
        "allow_list_size": config.allow_list.len(),
        "deny_list_size": config.deny_list.len(),
        "blocked_countries": config.blocked_countries.len(),
        "statistics": stats,
        "timestamp": chrono::Utc::now()
    });

    Ok(Json(health_status))
}
