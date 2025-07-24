//! SNMP client statistics and metrics

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// SNMP client statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnmpClientStats {
    /// Number of active sessions
    pub active_sessions: usize,
    /// Maximum allowed connections
    pub max_connections: usize,
    /// Available connection permits
    pub available_permits: usize,
    /// Active connections (for testing compatibility)
    pub active_connections: usize,
    /// Total requests processed
    pub total_requests: usize,
    /// Failed requests count
    pub failed_requests: usize,
    /// Average response time
    pub avg_response_time: Duration,
}

impl Default for SnmpClientStats {
    fn default() -> Self {
        Self {
            active_sessions: 0,
            max_connections: 0,
            available_permits: 0,
            active_connections: 0,
            total_requests: 0,
            failed_requests: 0,
            avg_response_time: Duration::ZERO,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snmp_client_stats_default() {
        let stats = SnmpClientStats::default();
        assert_eq!(stats.active_sessions, 0);
        assert_eq!(stats.max_connections, 0);
        assert_eq!(stats.available_permits, 0);
        assert_eq!(stats.active_connections, 0);
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.failed_requests, 0);
        assert_eq!(stats.avg_response_time, Duration::ZERO);
    }

    #[test]
    fn test_snmp_client_stats_creation() {
        let stats = SnmpClientStats {
            active_sessions: 5,
            max_connections: 100,
            available_permits: 95,
            active_connections: 5,
            total_requests: 1_234,
            failed_requests: 10,
            avg_response_time: Duration::from_millis(250),
        };

        assert_eq!(stats.active_sessions, 5);
        assert_eq!(stats.max_connections, 100);
        assert_eq!(stats.available_permits, 95);
        assert_eq!(stats.active_connections, 5);
        assert_eq!(stats.total_requests, 1_234);
        assert_eq!(stats.failed_requests, 10);
        assert_eq!(stats.avg_response_time, Duration::from_millis(250));
    }

    #[test]
    fn test_snmp_client_stats_serialization() {
        let stats = SnmpClientStats {
            active_sessions: 2,
            max_connections: 25,
            available_permits: 23,
            active_connections: 2,
            total_requests: 500,
            failed_requests: 5,
            avg_response_time: Duration::from_millis(100),
        };

        let serialized = serde_json::to_string(&stats).unwrap();
        let deserialized: SnmpClientStats = serde_json::from_str(&serialized).unwrap();

        assert_eq!(stats.active_sessions, deserialized.active_sessions);
        assert_eq!(stats.max_connections, deserialized.max_connections);
        assert_eq!(stats.available_permits, deserialized.available_permits);
        assert_eq!(stats.active_connections, deserialized.active_connections);
        assert_eq!(stats.total_requests, deserialized.total_requests);
        assert_eq!(stats.failed_requests, deserialized.failed_requests);
        assert_eq!(stats.avg_response_time, deserialized.avg_response_time);
    }

    #[test]
    fn test_snmp_client_stats_debug_format() {
        let stats = SnmpClientStats::default();
        let debug_output = format!("{stats:?}");
        assert!(debug_output.contains("SnmpClientStats"));
        assert!(debug_output.contains("active_sessions"));
        assert!(debug_output.contains("max_connections"));
    }

    #[test]
    fn test_snmp_client_stats_clone() {
        let stats = SnmpClientStats {
            active_sessions: 3,
            max_connections: 50,
            available_permits: 47,
            active_connections: 3,
            total_requests: 150,
            failed_requests: 2,
            avg_response_time: Duration::from_millis(75),
        };

        let cloned_stats = stats.clone();
        assert_eq!(stats.active_sessions, cloned_stats.active_sessions);
        assert_eq!(stats.max_connections, cloned_stats.max_connections);
        assert_eq!(stats.total_requests, cloned_stats.total_requests);
        assert_eq!(stats.avg_response_time, cloned_stats.avg_response_time);
    }
}
