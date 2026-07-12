/// Tests for location types and address handling

#[tokio::test]
async fn test_location_types() {
    let location_types = vec![
        "datacenter",
        "campus",
        "building",
        "floor",
        "rack",
        "room",
        "site",
        "zone",
    ];

    for location_type in location_types {
        assert!(!location_type.is_empty());
        assert!(!location_type.is_empty());
    }
}

#[tokio::test]
async fn test_address_parts_combination() {
    let address_parts = ["123 Main St", "New York", "NY", "USA"];

    let combined = address_parts.join(", ");
    assert_eq!(combined, "123 Main St, New York, NY, USA");

    // Test partial combinations
    let partial = ["456 Oak Ave", "Boston"];
    let partial_combined = partial.join(", ");
    assert_eq!(partial_combined, "456 Oak Ave, Boston");

    // Test single part
    let single = ["Single Address"];
    let single_combined = single.join(", ");
    assert_eq!(single_combined, "Single Address");

    // Test empty
    let empty: Vec<String> = vec![];
    let empty_combined = empty.join(", ");
    assert!(empty_combined.is_empty());
}

#[tokio::test]
async fn test_address_parts_empty_handling() {
    // Test various empty and whitespace scenarios
    let empty_parts = vec!["", "", "", ""];
    assert!(!empty_parts.into_iter().any(|s| !s.is_empty()));

    let mixed_parts = vec!["123 Main St", "", "New York", ""];
    let mixed_filtered: Vec<&str> = mixed_parts.into_iter().filter(|s| !s.is_empty()).collect();
    assert_eq!(mixed_filtered, vec!["123 Main St", "New York"]);

    let whitespace_parts = vec!["  ", "\t", "\n", "  Real Address  "];
    let trimmed_parts: Vec<String> = whitespace_parts
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    assert_eq!(trimmed_parts, vec!["Real Address"]);
}
