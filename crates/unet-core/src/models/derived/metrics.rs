//! Performance and environmental metrics for network devices

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::snmp::SnmpValue;

/// Performance metrics for a device
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// CPU utilization percentage (0-100)
    pub cpu_utilization: Option<u8>,
    /// Memory utilization percentage (0-100)
    pub memory_utilization: Option<u8>,
    /// Total memory in bytes
    pub total_memory: Option<u64>,
    /// Used memory in bytes
    pub used_memory: Option<u64>,
    /// Load average (for Unix-like systems)
    pub load_average: Option<f32>,
}

impl PerformanceMetrics {
    /// Convert an integer percentage value to u8 safely
    ///
    /// Clamps the input to valid percentage range (0-100) and converts
    /// to u8 without any precision loss.
    #[must_use]
    fn percentage_to_u8(value: i64) -> Option<u8> {
        // Try to convert i64 to u8, clamping to valid percentage range
        if value < 0 {
            Some(0)
        } else {
            u8::try_from(value.min(100)).ok()
        }
    }

    /// Extract performance metrics from SNMP data
    #[must_use]
    pub fn from_snmp(snmp_data: &HashMap<String, SnmpValue>) -> Option<Self> {
        let mut metrics = Self {
            cpu_utilization: None,
            memory_utilization: None,
            total_memory: None,
            used_memory: None,
            load_average: None,
        };

        let mut has_data = false;

        // Try to extract vendor-specific CPU metrics
        // Cisco CPU utilization (example)
        if let Some(SnmpValue::Integer(cpu)) = snmp_data.get("1.3.6.1.4.1.9.2.1.3.0") {
            metrics.cpu_utilization = Self::percentage_to_u8(*cpu);
            has_data = true;
        }

        // Cisco memory utilization (example)
        if let Some(SnmpValue::Integer(mem)) = snmp_data.get("1.3.6.1.4.1.9.2.1.8.0") {
            metrics.memory_utilization = Self::percentage_to_u8(*mem);
            has_data = true;
        }

        if has_data { Some(metrics) } else { None }
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
    #[must_use]
    pub const fn from_snmp(_snmp_data: &HashMap<String, SnmpValue>) -> Option<Self> {
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
