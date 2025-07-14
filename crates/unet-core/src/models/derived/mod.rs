//! Derived state data models for μNet
//!
//! This module contains data structures for information derived from SNMP polling
//! and other external sources. Unlike desired state models, these represent the
//! actual state of network devices as discovered through monitoring.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;
use uuid::Uuid;

use crate::snmp::SnmpValue;

// Re-export all public types for backward compatibility
pub use self::interfaces::*;
pub use self::metrics::*;
pub use self::system::*;

mod interfaces;
mod metrics;
mod system;

/// Current status and derived state for a network node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeStatus {
    /// Node ID this status refers to
    pub node_id: Uuid,
    /// Timestamp when this status was last updated
    pub last_updated: SystemTime,
    /// Whether the node is currently reachable
    pub reachable: bool,
    /// System information derived from SNMP
    pub system_info: Option<SystemInfo>,
    /// Interface status information
    pub interfaces: Vec<InterfaceStatus>,
    /// Performance metrics
    pub performance: Option<PerformanceMetrics>,
    /// Environmental sensors (temperature, fan speed, etc.)
    pub environmental: Option<EnvironmentalMetrics>,
    /// Device-specific metrics from vendor MIBs
    pub vendor_metrics: HashMap<String, SnmpValue>,
    /// Raw SNMP data (for debugging and extensibility)
    pub raw_snmp_data: HashMap<String, SnmpValue>,
    /// Timestamp of last successful SNMP poll
    pub last_snmp_success: Option<SystemTime>,
    /// Error message from last failed poll attempt
    pub last_error: Option<String>,
    /// Number of consecutive polling failures
    pub consecutive_failures: u32,
}

impl NodeStatus {
    /// Create new node status for a node
    #[must_use]
    pub fn new(node_id: Uuid) -> Self {
        Self {
            node_id,
            last_updated: SystemTime::now(),
            reachable: false,
            system_info: None,
            interfaces: Vec::new(),
            performance: None,
            environmental: None,
            vendor_metrics: HashMap::new(),
            raw_snmp_data: HashMap::new(),
            last_snmp_success: None,
            last_error: None,
            consecutive_failures: 0,
        }
    }

    /// Update status with successful SNMP poll result
    pub fn update_from_snmp(&mut self, snmp_data: HashMap<String, SnmpValue>) {
        self.last_updated = SystemTime::now();
        self.last_snmp_success = Some(SystemTime::now());
        self.reachable = true;
        self.consecutive_failures = 0;
        self.last_error = None;

        // Store raw SNMP data
        self.raw_snmp_data.clone_from(&snmp_data);

        // Extract system information
        self.system_info = SystemInfo::from_snmp(&snmp_data);

        // Extract interface information
        self.interfaces = InterfaceStatus::from_snmp(&snmp_data);

        // Extract performance metrics
        self.performance = PerformanceMetrics::from_snmp(&snmp_data);

        // Extract environmental metrics
        self.environmental = EnvironmentalMetrics::from_snmp(&snmp_data);

        // Separate vendor-specific metrics
        self.vendor_metrics = snmp_data
            .into_iter()
            .filter(|(oid, _)| oid.starts_with("1.3.6.1.4.1")) // Enterprise OIDs
            .collect();
    }

    /// Mark polling failure
    pub fn mark_polling_failure(&mut self, error: String) {
        self.last_updated = SystemTime::now();
        self.consecutive_failures += 1;
        self.last_error = Some(error);

        // Mark as unreachable after multiple failures
        if self.consecutive_failures >= 3 {
            self.reachable = false;
        }
    }

    /// Check if node status is stale (hasn't been updated recently)
    #[must_use]
    pub fn is_stale(&self, max_age: std::time::Duration) -> bool {
        SystemTime::now()
            .duration_since(self.last_updated)
            .map_or(true, |age| age > max_age)
    }

    /// Get uptime from system info if available
    #[must_use]
    pub fn uptime_seconds(&self) -> Option<u32> {
        self.system_info
            .as_ref()?
            .uptime_ticks
            .map(|ticks| ticks / 100)
    }

    /// Get interface by name
    #[must_use]
    pub fn get_interface(&self, name: &str) -> Option<&InterfaceStatus> {
        self.interfaces.iter().find(|iface| iface.name == name)
    }

    /// Get operational interfaces (admin up and oper up)
    #[must_use]
    pub fn operational_interfaces(&self) -> Vec<&InterfaceStatus> {
        self.interfaces
            .iter()
            .filter(|iface| {
                iface.admin_status == InterfaceAdminStatus::Up
                    && iface.oper_status == InterfaceOperStatus::Up
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_node_status_creation() {
        let node_id = Uuid::new_v4();
        let status = NodeStatus::new(node_id);

        assert_eq!(status.node_id, node_id);
        assert!(!status.reachable);
        assert_eq!(status.consecutive_failures, 0);
        assert!(status.interfaces.is_empty());
    }

    #[test]
    fn test_node_status_snmp_update() {
        let node_id = Uuid::new_v4();
        let mut status = NodeStatus::new(node_id);

        let mut snmp_data = HashMap::new();
        snmp_data.insert(
            "1.3.6.1.2.1.1.1.0".to_string(),
            SnmpValue::String("Test Device".to_string()),
        );
        snmp_data.insert(
            "1.3.6.1.2.1.1.5.0".to_string(),
            SnmpValue::String("test-device".to_string()),
        );

        status.update_from_snmp(snmp_data);

        assert!(status.reachable);
        assert_eq!(status.consecutive_failures, 0);
        assert!(status.last_snmp_success.is_some());
        assert!(status.system_info.is_some());
    }

    #[test]
    fn test_node_status_polling_failure() {
        let node_id = Uuid::new_v4();
        let mut status = NodeStatus::new(node_id);

        // Initially unreachable until successful poll
        assert!(!status.reachable);

        // Mark successful poll first to make it reachable
        let mut snmp_data = HashMap::new();
        snmp_data.insert(
            "1.3.6.1.2.1.1.1.0".to_string(),
            SnmpValue::String("Test Device".to_string()),
        );
        status.update_from_snmp(snmp_data);
        assert!(status.reachable);

        // Mark multiple failures
        status.mark_polling_failure("Timeout".to_string());
        assert_eq!(status.consecutive_failures, 1);
        assert!(status.reachable); // Still reachable after 1 failure

        status.mark_polling_failure("Timeout".to_string());
        status.mark_polling_failure("Timeout".to_string());
        assert_eq!(status.consecutive_failures, 3);
        assert!(!status.reachable); // Unreachable after 3 failures
    }

    #[test]
    fn test_node_status_uptime() {
        let node_id = Uuid::new_v4();
        let mut status = NodeStatus::new(node_id);

        // Test without system info
        assert_eq!(status.uptime_seconds(), None);

        // Add system info with uptime
        status.system_info = Some(SystemInfo {
            description: None,
            object_id: None,
            uptime_ticks: Some(123_456), // 1234.56 seconds
            contact: None,
            name: None,
            location: None,
            services: None,
        });

        assert_eq!(status.uptime_seconds(), Some(1234));
    }

    #[test]
    fn test_node_status_interface_operations() {
        let node_id = Uuid::new_v4();
        let mut status = NodeStatus::new(node_id);

        // Add some interfaces
        status.interfaces = vec![
            InterfaceStatus {
                index: 1,
                name: "eth0".to_string(),
                interface_type: 6,
                mtu: Some(1500),
                speed: Some(1_000_000_000),
                physical_address: None,
                admin_status: InterfaceAdminStatus::Up,
                oper_status: InterfaceOperStatus::Up,
                last_change: None,
                input_stats: InterfaceStats::default(),
                output_stats: InterfaceStats::default(),
            },
            InterfaceStatus {
                index: 2,
                name: "eth1".to_string(),
                interface_type: 6,
                mtu: Some(1500),
                speed: Some(1_000_000_000),
                physical_address: None,
                admin_status: InterfaceAdminStatus::Down,
                oper_status: InterfaceOperStatus::Down,
                last_change: None,
                input_stats: InterfaceStats::default(),
                output_stats: InterfaceStats::default(),
            },
        ];

        // Test get_interface
        let eth0 = status.get_interface("eth0");
        assert!(eth0.is_some());
        assert_eq!(eth0.unwrap().index, 1);

        let nonexistent = status.get_interface("eth99");
        assert!(nonexistent.is_none());

        // Test operational_interfaces
        let operational = status.operational_interfaces();
        assert_eq!(operational.len(), 1);
        assert_eq!(operational[0].name, "eth0");
    }
}
