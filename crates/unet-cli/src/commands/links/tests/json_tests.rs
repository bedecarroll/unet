/// Tests for JSON parsing in link commands

#[tokio::test]
async fn test_json_parsing_for_custom_data() {
    let valid_json_strings = vec![
        r#"{"provider": "ISP"}"#,
        r#"{"vlan": 100, "tagged": true}"#,
        r#"{"circuit_id": "MPLS-12345"}"#,
        r"{}",
    ];

    for json_str in valid_json_strings {
        let result = serde_json::from_str::<serde_json::Value>(json_str);
        assert!(result.is_ok(), "Failed to parse JSON: {json_str}");
    }
}

#[tokio::test]
async fn test_json_parsing_invalid_custom_data() {
    let invalid_json_strings = vec![
        r#"{"provider": ISP}"#, // missing quotes
        r#"{"vlan": }"#,        // missing value
        r#"{provider: "ISP"}"#, // missing quotes on key
        r"invalid json",
    ];

    for json_str in invalid_json_strings {
        let result = serde_json::from_str::<serde_json::Value>(json_str);
        assert!(
            result.is_err(),
            "Should have failed to parse JSON: {json_str}"
        );
    }
}
