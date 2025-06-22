//! Derived state data models for μNet
//!
//! This module contains data structures for information derived from SNMP polling
//! and other external sources. Unlike desired state models, these represent the
//! actual state of network devices as discovered through monitoring.

use std::collections::HashMap;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::snmp::SnmpValue;

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
        self.raw_snmp_data = snmp_data.clone();
        
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
    pub fn is_stale(&self, max_age: std::time::Duration) -> bool {
        SystemTime::now()
            .duration_since(self.last_updated)
            .map_or(true, |age| age > max_age)
    }
    
    /// Get uptime from system info if available
    pub fn uptime_seconds(&self) -> Option<u32> {
        self.system_info.as_ref()?.uptime_ticks.map(|ticks| ticks / 100)
    }
    
    /// Get interface by name
    pub fn get_interface(&self, name: &str) -> Option<&InterfaceStatus> {
        self.interfaces.iter().find(|iface| iface.name == name)
    }
    
    /// Get operational interfaces (admin up and oper up)
    pub fn operational_interfaces(&self) -> Vec<&InterfaceStatus> {
        self.interfaces
            .iter()
            .filter(|iface| iface.admin_status == InterfaceAdminStatus::Up 
                         && iface.oper_status == InterfaceOperStatus::Up)
            .collect()
    }
}

/// System information derived from SNMP system group
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemInfo {
    /// System description (sysDescr)
    pub description: Option<String>,
    /// System object identifier (sysObjectID)
    pub object_id: Option<String>,
    /// System uptime in ticks (sysUpTime)
    pub uptime_ticks: Option<u32>,
    /// System contact (sysContact)
    pub contact: Option<String>,
    /// System name (sysName)
    pub name: Option<String>,
    /// System location (sysLocation)
    pub location: Option<String>,
    /// System services (sysServices)
    pub services: Option<u32>,
}

impl SystemInfo {
    /// Extract system information from SNMP data
    pub fn from_snmp(snmp_data: &HashMap<String, SnmpValue>) -> Option<Self> {
        let mut system_info = SystemInfo {
            description: None,
            object_id: None,
            uptime_ticks: None,
            contact: None,
            name: None,
            location: None,
            services: None,
        };
        
        let mut has_data = false;
        
        // Extract system description
        if let Some(value) = snmp_data.get("1.3.6.1.2.1.1.1.0") {
            if let SnmpValue::String(desc) = value {
                system_info.description = Some(desc.clone());
                has_data = true;
            }
        }
        
        // Extract system object ID
        if let Some(value) = snmp_data.get("1.3.6.1.2.1.1.2.0") {
            if let SnmpValue::Oid(oid) = value {
                system_info.object_id = Some(oid.clone());
                has_data = true;
            }
        }
        
        // Extract system uptime
        if let Some(value) = snmp_data.get("1.3.6.1.2.1.1.3.0") {
            if let SnmpValue::TimeTicks(ticks) = value {
                system_info.uptime_ticks = Some(*ticks);
                has_data = true;
            }
        }
        
        // Extract system contact
        if let Some(value) = snmp_data.get("1.3.6.1.2.1.1.4.0") {
            if let SnmpValue::String(contact) = value {
                system_info.contact = Some(contact.clone());
                has_data = true;
            }
        }
        
        // Extract system name
        if let Some(value) = snmp_data.get("1.3.6.1.2.1.1.5.0") {
            if let SnmpValue::String(name) = value {
                system_info.name = Some(name.clone());
                has_data = true;
            }
        }
        
        // Extract system location
        if let Some(value) = snmp_data.get("1.3.6.1.2.1.1.6.0") {
            if let SnmpValue::String(location) = value {
                system_info.location = Some(location.clone());
                has_data = true;
            }
        }
        
        // Extract system services
        if let Some(value) = snmp_data.get("1.3.6.1.2.1.1.7.0") {
            if let SnmpValue::Integer(services) = value {
                system_info.services = Some(*services as u32);
                has_data = true;
            }
        }
        
        if has_data {
            Some(system_info)
        } else {
            None
        }
    }
}

/// Status of a network interface
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterfaceStatus {
    /// Interface index
    pub index: u32,
    /// Interface name/description
    pub name: String,
    /// Interface type (see RFC 1213)
    pub interface_type: u32,
    /// Interface MTU
    pub mtu: Option<u32>,
    /// Interface speed in bits per second
    pub speed: Option<u64>,
    /// Physical address (MAC address)
    pub physical_address: Option<String>,
    /// Administrative status
    pub admin_status: InterfaceAdminStatus,
    /// Operational status
    pub oper_status: InterfaceOperStatus,
    /// Last change timestamp (in ticks)
    pub last_change: Option<u32>,
    /// Input statistics
    pub input_stats: InterfaceStats,
    /// Output statistics
    pub output_stats: InterfaceStats,
}

impl InterfaceStatus {
    /// Extract interface statuses from SNMP data
    pub fn from_snmp(snmp_data: &HashMap<String, SnmpValue>) -> Vec<Self> {
        let mut interfaces = Vec::new();
        let mut interface_indexes = std::collections::BTreeSet::new();
        
        // First, collect all interface indexes
        for (oid, _) in snmp_data {
            if oid.starts_with("1.3.6.1.2.1.2.2.1.1.") {
                if let Some(index_str) = oid.strip_prefix("1.3.6.1.2.1.2.2.1.1.") {
                    if let Ok(index) = index_str.parse::<u32>() {
                        interface_indexes.insert(index);
                    }
                }
            }
        }
        
        // For each interface index, extract all available data
        for index in interface_indexes {
            let mut interface = InterfaceStatus {
                index,
                name: format!("Interface {}", index),
                interface_type: 1, // Default to other
                mtu: None,
                speed: None,
                physical_address: None,
                admin_status: InterfaceAdminStatus::Unknown,
                oper_status: InterfaceOperStatus::Unknown,
                last_change: None,
                input_stats: InterfaceStats::default(),
                output_stats: InterfaceStats::default(),
            };
            
            // Extract interface description
            if let Some(value) = snmp_data.get(&format!("1.3.6.1.2.1.2.2.1.2.{}", index)) {
                if let SnmpValue::String(desc) = value {
                    interface.name = desc.clone();
                }
            }
            
            // Extract interface type
            if let Some(value) = snmp_data.get(&format!("1.3.6.1.2.1.2.2.1.3.{}", index)) {
                if let SnmpValue::Integer(iface_type) = value {
                    interface.interface_type = *iface_type as u32;
                }
            }
            
            // Extract MTU
            if let Some(value) = snmp_data.get(&format!("1.3.6.1.2.1.2.2.1.4.{}", index)) {
                if let SnmpValue::Integer(mtu) = value {
                    interface.mtu = Some(*mtu as u32);
                }
            }
            
            // Extract speed
            if let Some(value) = snmp_data.get(&format!("1.3.6.1.2.1.2.2.1.5.{}", index)) {
                if let SnmpValue::Gauge32(speed) = value {
                    interface.speed = Some(*speed as u64);
                }
            }
            
            // Extract administrative status
            if let Some(value) = snmp_data.get(&format!("1.3.6.1.2.1.2.2.1.7.{}", index)) {
                if let SnmpValue::Integer(status) = value {
                    interface.admin_status = InterfaceAdminStatus::from(*status as u8);
                }
            }
            
            // Extract operational status
            if let Some(value) = snmp_data.get(&format!("1.3.6.1.2.1.2.2.1.8.{}", index)) {
                if let SnmpValue::Integer(status) = value {
                    interface.oper_status = InterfaceOperStatus::from(*status as u8);
                }
            }
            
            // Extract input octets
            if let Some(value) = snmp_data.get(&format!("1.3.6.1.2.1.2.2.1.10.{}", index)) {
                if let SnmpValue::Counter32(octets) = value {
                    interface.input_stats.octets = *octets as u64;
                }
            }
            
            // Extract input packets
            if let Some(value) = snmp_data.get(&format!("1.3.6.1.2.1.2.2.1.11.{}", index)) {
                if let SnmpValue::Counter32(packets) = value {
                    interface.input_stats.packets = *packets as u64;
                }
            }
            
            // Extract input errors
            if let Some(value) = snmp_data.get(&format!("1.3.6.1.2.1.2.2.1.14.{}", index)) {
                if let SnmpValue::Counter32(errors) = value {
                    interface.input_stats.errors = *errors as u64;
                }
            }
            
            // Extract output octets
            if let Some(value) = snmp_data.get(&format!("1.3.6.1.2.1.2.2.1.16.{}", index)) {
                if let SnmpValue::Counter32(octets) = value {
                    interface.output_stats.octets = *octets as u64;
                }
            }
            
            // Extract output packets
            if let Some(value) = snmp_data.get(&format!("1.3.6.1.2.1.2.2.1.17.{}", index)) {
                if let SnmpValue::Counter32(packets) = value {
                    interface.output_stats.packets = *packets as u64;
                }
            }
            
            // Extract output errors
            if let Some(value) = snmp_data.get(&format!("1.3.6.1.2.1.2.2.1.20.{}", index)) {
                if let SnmpValue::Counter32(errors) = value {
                    interface.output_stats.errors = *errors as u64;
                }
            }
            
            interfaces.push(interface);
        }
        
        interfaces
    }
}

/// Interface administrative status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterfaceAdminStatus {
    /// Interface is administratively up
    Up,
    /// Interface is administratively down
    Down,
    /// Interface is in testing mode
    Testing,
    /// Unknown status
    Unknown,
}

impl From<u8> for InterfaceAdminStatus {
    fn from(value: u8) -> Self {
        match value {
            1 => InterfaceAdminStatus::Up,
            2 => InterfaceAdminStatus::Down,
            3 => InterfaceAdminStatus::Testing,
            _ => InterfaceAdminStatus::Unknown,
        }
    }
}

/// Interface operational status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterfaceOperStatus {
    /// Interface is operationally up
    Up,
    /// Interface is operationally down
    Down,
    /// Interface is in testing mode
    Testing,
    /// Status is unknown
    Unknown,
    /// Interface is dormant
    Dormant,
    /// Interface is not present
    NotPresent,
    /// Lower layer is down
    LowerLayerDown,
}

impl From<u8> for InterfaceOperStatus {
    fn from(value: u8) -> Self {
        match value {
            1 => InterfaceOperStatus::Up,
            2 => InterfaceOperStatus::Down,
            3 => InterfaceOperStatus::Testing,
            4 => InterfaceOperStatus::Unknown,
            5 => InterfaceOperStatus::Dormant,
            6 => InterfaceOperStatus::NotPresent,
            7 => InterfaceOperStatus::LowerLayerDown,
            _ => InterfaceOperStatus::Unknown,
        }
    }
}

/// Interface traffic statistics
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterfaceStats {
    /// Number of octets (bytes)
    pub octets: u64,
    /// Number of packets
    pub packets: u64,
    /// Number of errors
    pub errors: u64,
    /// Number of discarded packets
    pub discards: u64,
}

impl Default for InterfaceStats {
    fn default() -> Self {
        Self {
            octets: 0,
            packets: 0,
            errors: 0,
            discards: 0,
        }
    }
}

/// Performance metrics for a device
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// CPU utilization percentage (0-100)
    pub cpu_utilization: Option<f32>,
    /// Memory utilization percentage (0-100)
    pub memory_utilization: Option<f32>,
    /// Total memory in bytes
    pub total_memory: Option<u64>,
    /// Used memory in bytes
    pub used_memory: Option<u64>,
    /// Load average (for Unix-like systems)
    pub load_average: Option<f32>,
}

impl PerformanceMetrics {
    /// Extract performance metrics from SNMP data
    pub fn from_snmp(snmp_data: &HashMap<String, SnmpValue>) -> Option<Self> {
        let mut metrics = PerformanceMetrics {
            cpu_utilization: None,
            memory_utilization: None,
            total_memory: None,
            used_memory: None,
            load_average: None,
        };
        
        let mut has_data = false;
        
        // Try to extract vendor-specific CPU metrics
        // Cisco CPU utilization (example)
        if let Some(value) = snmp_data.get("1.3.6.1.4.1.9.2.1.3.0") {
            if let SnmpValue::Integer(cpu) = value {
                metrics.cpu_utilization = Some(*cpu as f32);
                has_data = true;
            }
        }
        
        // Cisco memory utilization (example)
        if let Some(value) = snmp_data.get("1.3.6.1.4.1.9.2.1.8.0") {
            if let SnmpValue::Integer(mem) = value {
                metrics.memory_utilization = Some(*mem as f32);
                has_data = true;
            }
        }
        
        if has_data {
            Some(metrics)
        } else {
            None
        }
    }
}

/// Environmental metrics for a device
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnvironmentalMetrics {
    /// Temperature sensors
    pub temperatures: Vec<TemperatureSensor>,
    /// Fan sensors
    pub fans: Vec<FanSensor>,
    /// Power supply status
    pub power_supplies: Vec<PowerSupply>,
}

impl EnvironmentalMetrics {
    /// Extract environmental metrics from SNMP data
    pub fn from_snmp(_snmp_data: &HashMap<String, SnmpValue>) -> Option<Self> {
        // TODO: Implement environmental metrics extraction
        // This would require vendor-specific MIB knowledge
        None
    }
}

/// Temperature sensor reading
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemperatureSensor {
    /// Sensor name/location
    pub name: String,
    /// Temperature in Celsius
    pub temperature: f32,
    /// Critical temperature threshold
    pub critical_threshold: Option<f32>,
    /// Warning temperature threshold
    pub warning_threshold: Option<f32>,
}

/// Fan sensor reading
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FanSensor {
    /// Fan name/location
    pub name: String,
    /// Fan speed in RPM
    pub speed_rpm: Option<u32>,
    /// Fan status
    pub status: FanStatus,
}

/// Fan operational status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FanStatus {
    /// Fan is operating normally
    Normal,
    /// Fan has failed
    Failed,
    /// Fan is not present
    NotPresent,
    /// Fan status is unknown
    Unknown,
}

/// Power supply status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PowerSupply {
    /// Power supply name/location
    pub name: String,
    /// Power supply status
    pub status: PowerSupplyStatus,
    /// Power output in watts
    pub power_output: Option<f32>,
}

/// Power supply operational status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PowerSupplyStatus {
    /// Power supply is operating normally
    Normal,
    /// Power supply has failed
    Failed,
    /// Power supply is not present
    NotPresent,
    /// Power supply status is unknown
    Unknown,
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
        snmp_data.insert("1.3.6.1.2.1.1.1.0".to_string(), 
                        SnmpValue::String("Test Device".to_string()));
        snmp_data.insert("1.3.6.1.2.1.1.5.0".to_string(), 
                        SnmpValue::String("test-device".to_string()));
        
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
        snmp_data.insert("1.3.6.1.2.1.1.1.0".to_string(), 
                        SnmpValue::String("Test Device".to_string()));
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
    fn test_system_info_from_snmp() {
        let mut snmp_data = HashMap::new();
        snmp_data.insert("1.3.6.1.2.1.1.1.0".to_string(), 
                        SnmpValue::String("Test Device v1.0".to_string()));
        snmp_data.insert("1.3.6.1.2.1.1.3.0".to_string(), 
                        SnmpValue::TimeTicks(12345678));
        snmp_data.insert("1.3.6.1.2.1.1.5.0".to_string(), 
                        SnmpValue::String("test-router".to_string()));
        
        let system_info = SystemInfo::from_snmp(&snmp_data);
        assert!(system_info.is_some());
        
        let info = system_info.unwrap();
        assert_eq!(info.description, Some("Test Device v1.0".to_string()));
        assert_eq!(info.uptime_ticks, Some(12345678));
        assert_eq!(info.name, Some("test-router".to_string()));
    }
    
    #[test]
    fn test_interface_status_from_snmp() {
        let mut snmp_data = HashMap::new();
        
        // Add interface 1 data
        snmp_data.insert("1.3.6.1.2.1.2.2.1.1.1".to_string(), SnmpValue::Integer(1));
        snmp_data.insert("1.3.6.1.2.1.2.2.1.2.1".to_string(), 
                        SnmpValue::String("eth0".to_string()));
        snmp_data.insert("1.3.6.1.2.1.2.2.1.3.1".to_string(), SnmpValue::Integer(6));
        snmp_data.insert("1.3.6.1.2.1.2.2.1.7.1".to_string(), SnmpValue::Integer(1)); // admin up
        snmp_data.insert("1.3.6.1.2.1.2.2.1.8.1".to_string(), SnmpValue::Integer(1)); // oper up
        
        let interfaces = InterfaceStatus::from_snmp(&snmp_data);
        assert_eq!(interfaces.len(), 1);
        
        let interface = &interfaces[0];
        assert_eq!(interface.index, 1);
        assert_eq!(interface.name, "eth0");
        assert_eq!(interface.admin_status, InterfaceAdminStatus::Up);
        assert_eq!(interface.oper_status, InterfaceOperStatus::Up);
    }
    
    #[test]
    fn test_interface_admin_status_conversion() {
        assert_eq!(InterfaceAdminStatus::from(1), InterfaceAdminStatus::Up);
        assert_eq!(InterfaceAdminStatus::from(2), InterfaceAdminStatus::Down);
        assert_eq!(InterfaceAdminStatus::from(3), InterfaceAdminStatus::Testing);
        assert_eq!(InterfaceAdminStatus::from(255), InterfaceAdminStatus::Unknown);
    }
    
    #[test]
    fn test_interface_oper_status_conversion() {
        assert_eq!(InterfaceOperStatus::from(1), InterfaceOperStatus::Up);
        assert_eq!(InterfaceOperStatus::from(2), InterfaceOperStatus::Down);
        assert_eq!(InterfaceOperStatus::from(7), InterfaceOperStatus::LowerLayerDown);
        assert_eq!(InterfaceOperStatus::from(255), InterfaceOperStatus::Unknown);
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
            uptime_ticks: Some(123456), // 1234.56 seconds
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
                speed: Some(1000000000),
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
                speed: Some(1000000000),
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