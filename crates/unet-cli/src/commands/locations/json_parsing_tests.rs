/// Tests for JSON parsing and custom data handling
use serde_json::Value as JsonValue;

#[tokio::test]
async fn test_json_parsing_for_location_custom_data() {
    let valid_json_strings = vec![
        r#"{"zone": "production"}"#,
        r#"{"rack_units": 42, "power_circuits": 2}"#,
        r#"{"coordinates": {"lat": 40.7128, "lng": -74.0060}}"#,
        r#"{"contact": {"name": "John Doe", "phone": "555-1234"}}"#,
        r"{}",
    ];

    for json_str in valid_json_strings {
        let result = serde_json::from_str::<JsonValue>(json_str);
        assert!(result.is_ok(), "Failed to parse JSON: {json_str}");
    }
}

#[tokio::test]
async fn test_json_parsing_invalid_location_custom_data() {
    let invalid_json_strings = vec![
        r#"{"zone": production}"#, // missing quotes
        r#"{"rack_units": }"#,     // missing value
        r#"{zone: "production"}"#, // missing quotes on key
        r"invalid json",
        r#"{"incomplete": }"#,
    ];

    for json_str in invalid_json_strings {
        let result = serde_json::from_str::<JsonValue>(json_str);
        assert!(
            result.is_err(),
            "Should have failed to parse JSON: {json_str}"
        );
    }
}

#[tokio::test]
async fn test_custom_data_complex_structures() {
    let complex_json_strings = vec![
        // Nested objects
        r#"{"datacenter": {"region": "us-east", "availability_zone": "1a", "power": {"primary": "grid", "backup": "generator"}}}"#,
        // Arrays
        r#"{"supported_services": ["web", "database", "cache"], "contact_emails": ["admin@example.com", "ops@example.com"]}"#,
        // Mixed data types
        r#"{"location_details": {"floor": 3, "room": "server-room-a", "temperature_controlled": true, "max_capacity": 100.5}}"#,
        // Complex nested structure
        r#"{"infrastructure": {"network": {"switches": 4, "ports_per_switch": 48}, "power": {"circuits": 2, "max_load_kw": 50}, "environmental": {"hvac_zones": ["north", "south"], "fire_suppression": "water_mist"}}}"#,
    ];

    for json_str in complex_json_strings {
        let result = serde_json::from_str::<JsonValue>(json_str);
        assert!(result.is_ok(), "Failed to parse complex JSON: {json_str}");

        let parsed_value = result.unwrap();

        // Verify we can access nested values
        if let JsonValue::Object(obj) = &parsed_value {
            assert!(!obj.is_empty(), "Parsed JSON should not be empty");
        }

        // Verify we can serialize it back
        let serialized = serde_json::to_string(&parsed_value);
        assert!(
            serialized.is_ok(),
            "Should be able to serialize back to JSON"
        );
    }
}
