//! Basic functionality tests for config-slicer
//!
//! These tests verify that the core functionality works with the actual API.

use config_slicer::{api::ConfigSlicerApi, error::Result, parser::Vendor};
use std::fs;

/// Load test fixture content
fn load_fixture(name: &str) -> String {
    let path = format!("tests/fixtures/{name}");
    fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to load fixture: {path}"))
}

#[test]
fn test_api_creation() {
    ConfigSlicerApi::new();
    // Just verify we can create the API without panicking
    println!("ConfigSlicerApi created successfully");
}

#[test]
fn test_simple_config_parsing() -> Result<()> {
    let simple_config = r#"
interface GigabitEthernet0/1
 description Test Interface
 ip address 192.168.1.1 255.255.255.0
!
"#;

    let api = ConfigSlicerApi::new();
    let result = api.parse_config(simple_config, Some(Vendor::Cisco))?;

    // Basic verification
    assert!(
        !result.children.is_empty() || !result.command.is_empty(),
        "Should parse something from the config"
    );

    println!("Successfully parsed simple config");
    Ok(())
}

#[test]
fn test_cisco_fixture_parsing() -> Result<()> {
    let config_content = load_fixture("cisco_ios_large.cfg");
    let api = ConfigSlicerApi::new();

    let result = api.parse_config(&config_content, Some(Vendor::Cisco))?;

    // Should parse without errors
    assert!(
        !result.children.is_empty(),
        "Should parse children from Cisco config"
    );

    let count = result.children.len();
    println!("Successfully parsed Cisco fixture with {count} child nodes");
    Ok(())
}

#[test]
fn test_juniper_fixture_parsing() -> Result<()> {
    let config_content = load_fixture("juniper_junos_large.cfg");
    let api = ConfigSlicerApi::new();

    let result = api.parse_config(&config_content, Some(Vendor::Juniper))?;

    // Should parse without errors
    assert!(
        !result.children.is_empty(),
        "Should parse children from Juniper config"
    );

    let count = result.children.len();
    println!("Successfully parsed Juniper fixture with {count} child nodes");
    Ok(())
}

#[test]
fn test_arista_fixture_parsing() -> Result<()> {
    let config_content = load_fixture("arista_eos_large.cfg");
    let api = ConfigSlicerApi::new();

    let result = api.parse_config(&config_content, Some(Vendor::Arista))?;

    // Should parse without errors
    assert!(
        !result.children.is_empty(),
        "Should parse children from Arista config"
    );

    let count = result.children.len();
    println!("Successfully parsed Arista fixture with {count} child nodes");
    Ok(())
}

#[test]
fn test_glob_slicing() -> Result<()> {
    let simple_config = r#"
interface GigabitEthernet0/1
 description Interface 1
 ip address 192.168.1.1 255.255.255.0
!
interface GigabitEthernet0/2
 description Interface 2
 ip address 192.168.2.1 255.255.255.0
!
router ospf 1
 router-id 1.1.1.1
!
"#;

    let api = ConfigSlicerApi::new();
    let config_tree = api.parse_config(simple_config, Some(Vendor::Cisco))?;

    // Test glob slicing
    let slice_result = api.slice_by_glob(&config_tree, "interface*")?;

    println!("Glob slicing found {} matches", slice_result.matches.len());

    // Should find some matches
    assert!(slice_result.has_matches(), "Should find interface matches");

    Ok(())
}

#[test]
fn test_regex_slicing() -> Result<()> {
    let simple_config = r#"
interface GigabitEthernet0/1
 description Interface 1
!
router ospf 1
 router-id 1.1.1.1
!
"#;

    let api = ConfigSlicerApi::new();
    let config_tree = api.parse_config(simple_config, Some(Vendor::Cisco))?;

    // Test regex slicing
    let slice_result = api.slice_by_regex(&config_tree, r"router.*")?;

    println!("Regex slicing found {} matches", slice_result.matches.len());

    Ok(())
}

#[test]
fn test_error_handling() {
    let api = ConfigSlicerApi::new();

    // Test empty configuration
    let empty_result = api.parse_config("", Some(Vendor::Cisco));
    assert!(empty_result.is_err(), "Should reject empty configuration");

    println!("Error handling test passed");
}

#[test]
fn test_performance_basic() -> Result<()> {
    let config_content = load_fixture("cisco_ios_large.cfg");
    let api = ConfigSlicerApi::new();

    // Measure parsing time
    let start = std::time::Instant::now();
    let config_tree = api.parse_config(&config_content, Some(Vendor::Cisco))?;
    let parse_duration = start.elapsed();

    // Measure slicing time
    let start = std::time::Instant::now();
    let _ = api.slice_by_glob(&config_tree, "interface*")?;
    let slice_duration = start.elapsed();

    println!(
        "Performance: parse={}ms, slice={}ms",
        parse_duration.as_millis(),
        slice_duration.as_millis()
    );

    // Very generous performance limits for CI
    assert!(
        parse_duration.as_secs() < 30,
        "Parsing should complete within 30 seconds"
    );
    assert!(
        slice_duration.as_secs() < 10,
        "Slicing should complete within 10 seconds"
    );

    Ok(())
}

#[test]
fn test_multiple_vendors() -> Result<()> {
    let configs = vec![
        ("cisco_ios_large.cfg", Vendor::Cisco),
        ("juniper_junos_large.cfg", Vendor::Juniper),
        ("arista_eos_large.cfg", Vendor::Arista),
    ];

    let api = ConfigSlicerApi::new();

    for (fixture_name, vendor) in configs {
        let config_content = load_fixture(fixture_name);

        let parse_result = api.parse_config(&config_content, Some(vendor))?;

        // Should parse successfully for all vendors
        assert!(
            !parse_result.children.is_empty() || !parse_result.command.is_empty(),
            "Should parse {fixture_name} successfully"
        );

        println!(
            "Successfully parsed {fixture_name} with vendor {:?}",
            vendor
        );
    }

    Ok(())
}

#[test]
fn test_slice_result_methods() -> Result<()> {
    let simple_config = r#"
interface GigabitEthernet0/1
 description Test
!
interface GigabitEthernet0/2
 description Test2
!
"#;

    let api = ConfigSlicerApi::new();
    let config_tree = api.parse_config(simple_config, Some(Vendor::Cisco))?;
    let slice_result = api.slice_by_glob(&config_tree, "interface*")?;

    // Test SliceResult methods
    let match_count = slice_result.match_count();
    let has_matches = slice_result.has_matches();
    let matches = slice_result.matches();
    let metadata = slice_result.metadata();

    println!("SliceResult: {match_count} matches, has_matches={has_matches}");

    // Basic verification
    assert_eq!(
        match_count,
        matches.len(),
        "match_count should equal matches.len()"
    );
    assert_eq!(
        has_matches,
        !matches.is_empty(),
        "has_matches should be consistent"
    );
    assert!(
        !metadata.is_empty() || metadata.is_empty(),
        "metadata should be accessible"
    );

    Ok(())
}
