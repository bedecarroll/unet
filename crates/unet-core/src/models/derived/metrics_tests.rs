use super::*;
use crate::snmp::SnmpValue;
use std::collections::HashMap;

#[test]
fn test_performance_metrics_percentage_to_u8() {
    assert_eq!(PerformanceMetrics::percentage_to_u8(0), Some(0));
    assert_eq!(PerformanceMetrics::percentage_to_u8(50), Some(50));
    assert_eq!(PerformanceMetrics::percentage_to_u8(100), Some(100));
    assert_eq!(PerformanceMetrics::percentage_to_u8(-10), Some(0));
    assert_eq!(PerformanceMetrics::percentage_to_u8(150), Some(100));
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

    let metrics = PerformanceMetrics::from_snmp(&snmp_data).unwrap();
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

    let metrics = PerformanceMetrics::from_snmp(&snmp_data).unwrap();
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

    let metrics = PerformanceMetrics::from_snmp(&snmp_data).unwrap();
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

    let metrics = PerformanceMetrics::from_snmp(&snmp_data).unwrap();
    assert_eq!(metrics.cpu_utilization, Some(0));
    assert_eq!(metrics.memory_utilization, Some(100));
}

#[test]
fn test_performance_metrics_from_snmp_non_integer_values() {
    let mut snmp_data = HashMap::new();
    snmp_data.insert(
        "1.3.6.1.4.1.9.2.1.3.0".to_string(),
        SnmpValue::String("75".to_string()),
    );
    snmp_data.insert("1.3.6.1.4.1.9.2.1.8.0".to_string(), SnmpValue::Gauge32(85));

    let metrics = PerformanceMetrics::from_snmp(&snmp_data);
    assert!(metrics.is_none());
}

#[test]
fn test_environmental_metrics_from_snmp() {
    let mut snmp_data = HashMap::new();
    snmp_data.insert(
        "1.3.6.1.4.1.9.9.13.1.3.1.3.7".to_string(),
        SnmpValue::Integer(42),
    );
    snmp_data.insert(
        "1.3.6.1.4.1.2636.3.1.13.1.7.9".to_string(),
        SnmpValue::Integer(37),
    );

    let metrics = EnvironmentalMetrics::from_snmp(&snmp_data).unwrap();
    assert_eq!(metrics.temperatures.len(), 2);
    assert!(metrics.fans.is_empty());
    assert!(metrics.power_supplies.is_empty());
    assert_eq!(metrics.temperatures[0].name, "Temperature Sensor 7");
    assert!((metrics.temperatures[0].temperature - 42.0).abs() < f32::EPSILON);
    assert_eq!(metrics.temperatures[1].name, "Temperature Sensor 9");
    assert!((metrics.temperatures[1].temperature - 37.0).abs() < f32::EPSILON);
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
    assert!((sensor.temperature - 42.5).abs() < f32::EPSILON);
    assert!((sensor.critical_threshold.unwrap() - 80.0).abs() < f32::EPSILON);
    assert!((sensor.warning_threshold.unwrap() - 70.0).abs() < f32::EPSILON);
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
    for status in [
        FanStatus::Normal,
        FanStatus::Failed,
        FanStatus::NotPresent,
        FanStatus::Unknown,
    ] {
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
    assert!((psu.power_output.unwrap() - 200.0).abs() < f32::EPSILON);
}

#[test]
fn test_power_supply_status_variants() {
    for status in [
        PowerSupplyStatus::Normal,
        PowerSupplyStatus::Failed,
        PowerSupplyStatus::NotPresent,
        PowerSupplyStatus::Unknown,
    ] {
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
