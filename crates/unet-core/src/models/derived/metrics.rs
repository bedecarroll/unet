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
    pub(crate) fn percentage_to_u8(value: i64) -> Option<u8> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snmp::SnmpValue;
    use std::collections::HashMap;

    #[test]
    fn test_performance_metrics_percentage_to_u8() {
        // Test valid percentages
        assert_eq!(PerformanceMetrics::percentage_to_u8(0), Some(0));
        assert_eq!(PerformanceMetrics::percentage_to_u8(50), Some(50));
        assert_eq!(PerformanceMetrics::percentage_to_u8(100), Some(100));
        
        // Test clamping
        assert_eq!(PerformanceMetrics::percentage_to_u8(-10), Some(0));
        assert_eq!(PerformanceMetrics::percentage_to_u8(150), Some(100));
        
        // Test overflow/underflow
        assert_eq!(PerformanceMetrics::percentage_to_u8(i64::MAX), Some(100));
        assert_eq!(PerformanceMetrics::percentage_to_u8(i64::MIN), Some(0));
    }

    #[test]
    fn test_performance_metrics_from_snmp_empty() {
        let snmp_data = HashMap::new();
        let metrics = PerformanceMetrics::from_snmp(&snmp_data);
        assert!(metrics.is_none());
    }

    #[test]
    fn test_performance_metrics_from_snmp_with_cpu() {
        let mut snmp_data = HashMap::new();
        snmp_data.insert("1.3.6.1.4.1.9.2.1.3.0".to_string(), SnmpValue::Integer(75));
        
        let metrics = PerformanceMetrics::from_snmp(&snmp_data);
        assert!(metrics.is_some());
        
        let metrics = metrics.unwrap();
        assert_eq!(metrics.cpu_utilization, Some(75));
        assert_eq!(metrics.memory_utilization, None);
        assert_eq!(metrics.total_memory, None);
        assert_eq!(metrics.used_memory, None);
        assert_eq!(metrics.load_average, None);
    }

    #[test]
    fn test_performance_metrics_from_snmp_with_memory() {
        let mut snmp_data = HashMap::new();
        snmp_data.insert("1.3.6.1.4.1.9.2.1.8.0".to_string(), SnmpValue::Integer(85));
        
        let metrics = PerformanceMetrics::from_snmp(&snmp_data);
        assert!(metrics.is_some());
        
        let metrics = metrics.unwrap();
        assert_eq!(metrics.cpu_utilization, None);
        assert_eq!(metrics.memory_utilization, Some(85));
        assert_eq!(metrics.total_memory, None);
        assert_eq!(metrics.used_memory, None);
        assert_eq!(metrics.load_average, None);
    }

    #[test]
    fn test_performance_metrics_from_snmp_with_both() {
        let mut snmp_data = HashMap::new();
        snmp_data.insert("1.3.6.1.4.1.9.2.1.3.0".to_string(), SnmpValue::Integer(45));
        snmp_data.insert("1.3.6.1.4.1.9.2.1.8.0".to_string(), SnmpValue::Integer(65));
        
        let metrics = PerformanceMetrics::from_snmp(&snmp_data);
        assert!(metrics.is_some());
        
        let metrics = metrics.unwrap();
        assert_eq!(metrics.cpu_utilization, Some(45));
        assert_eq!(metrics.memory_utilization, Some(65));
        assert_eq!(metrics.total_memory, None);
        assert_eq!(metrics.used_memory, None);
        assert_eq!(metrics.load_average, None);
    }

    #[test]
    fn test_performance_metrics_from_snmp_invalid_percentage() {
        let mut snmp_data = HashMap::new();
        snmp_data.insert("1.3.6.1.4.1.9.2.1.3.0".to_string(), SnmpValue::Integer(-5));
        snmp_data.insert("1.3.6.1.4.1.9.2.1.8.0".to_string(), SnmpValue::Integer(150));
        
        let metrics = PerformanceMetrics::from_snmp(&snmp_data);
        assert!(metrics.is_some());
        
        let metrics = metrics.unwrap();
        assert_eq!(metrics.cpu_utilization, Some(0));
        assert_eq!(metrics.memory_utilization, Some(100));
    }

    #[test]
    fn test_performance_metrics_from_snmp_non_integer_values() {
        let mut snmp_data = HashMap::new();
        snmp_data.insert("1.3.6.1.4.1.9.2.1.3.0".to_string(), SnmpValue::String("75".to_string()));
        snmp_data.insert("1.3.6.1.4.1.9.2.1.8.0".to_string(), SnmpValue::Gauge32(85));
        
        let metrics = PerformanceMetrics::from_snmp(&snmp_data);
        assert!(metrics.is_none());
    }

    #[test]
    fn test_environmental_metrics_from_snmp() {
        let snmp_data = HashMap::new();
        let metrics = EnvironmentalMetrics::from_snmp(&snmp_data);
        assert!(metrics.is_none());
    }

    #[test]
    fn test_temperature_sensor_creation() {
        let sensor = TemperatureSensor {
            name: "CPU Temp".to_string(),
            temperature: 42.5,
            critical_threshold: Some(80.0),
            warning_threshold: Some(70.0),
        };
        
        assert_eq!(sensor.name, "CPU Temp");
        assert_eq!(sensor.temperature, 42.5);
        assert_eq!(sensor.critical_threshold, Some(80.0));
        assert_eq!(sensor.warning_threshold, Some(70.0));
    }

    #[test]
    fn test_fan_sensor_creation() {
        let sensor = FanSensor {
            name: "Chassis Fan".to_string(),
            speed_rpm: Some(2800),
            status: FanStatus::Normal,
        };
        
        assert_eq!(sensor.name, "Chassis Fan");
        assert_eq!(sensor.speed_rpm, Some(2800));
        assert_eq!(sensor.status, FanStatus::Normal);
    }

    #[test]
    fn test_fan_status_variants() {
        let statuses = vec![
            FanStatus::Normal,
            FanStatus::Failed,
            FanStatus::NotPresent,
            FanStatus::Unknown,
        ];
        
        for status in statuses {
            let sensor = FanSensor {
                name: "Test Fan".to_string(),
                speed_rpm: None,
                status,
            };
            assert_eq!(sensor.status, status);
        }
    }

    #[test]
    fn test_power_supply_creation() {
        let psu = PowerSupply {
            name: "Power Supply 1".to_string(),
            status: PowerSupplyStatus::Normal,
            power_output: Some(200.0),
        };
        
        assert_eq!(psu.name, "Power Supply 1");
        assert_eq!(psu.status, PowerSupplyStatus::Normal);
        assert_eq!(psu.power_output, Some(200.0));
    }

    #[test]
    fn test_power_supply_status_variants() {
        let statuses = vec![
            PowerSupplyStatus::Normal,
            PowerSupplyStatus::Failed,
            PowerSupplyStatus::NotPresent,
            PowerSupplyStatus::Unknown,
        ];
        
        for status in statuses {
            let psu = PowerSupply {
                name: "Test PSU".to_string(),
                status,
                power_output: None,
            };
            assert_eq!(psu.status, status);
        }
    }

    #[test]
    fn test_environmental_metrics_empty() {
        let metrics = EnvironmentalMetrics {
            temperatures: vec![],
            fans: vec![],
            power_supplies: vec![],
        };
        
        assert!(metrics.temperatures.is_empty());
        assert!(metrics.fans.is_empty());
        assert!(metrics.power_supplies.is_empty());
    }

    #[test]
    fn test_serde_serialization() {
        let metrics = PerformanceMetrics {
            cpu_utilization: Some(50),
            memory_utilization: Some(75),
            total_memory: Some(8 * 1024 * 1024 * 1024),
            used_memory: Some(6 * 1024 * 1024 * 1024),
            load_average: Some(2.5),
        };
        
        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: PerformanceMetrics = serde_json::from_str(&json).unwrap();
        
        assert_eq!(metrics, deserialized);
    }

    #[test]
    fn test_environmental_metrics_serde() {
        let temp_sensor = TemperatureSensor {
            name: "CPU".to_string(),
            temperature: 45.5,
            critical_threshold: Some(85.0),
            warning_threshold: Some(75.0),
        };
        
        let json = serde_json::to_string(&temp_sensor).unwrap();
        let deserialized: TemperatureSensor = serde_json::from_str(&json).unwrap();
        
        assert_eq!(temp_sensor, deserialized);
    }
}
