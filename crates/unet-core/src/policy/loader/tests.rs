//! Tests for policy loader functionality

use crate::config::GitConfig;
use crate::policy::{PolicyError, PolicyLoader};
use std::fs;
use std::time::Duration;
use tempfile::TempDir;

fn create_test_git_config() -> GitConfig {
    GitConfig {
        repository_url: None,
        local_directory: None,
        branch: "main".to_string(),
        auth_token: None,
        sync_interval: 300,
        policies_repo: None,
        templates_repo: None,
    }
}

#[test]
fn test_policy_loader_local_directory() {
    let temp_dir = TempDir::new().unwrap();
    let policies_dir = temp_dir.path().join("policies");
    fs::create_dir_all(&policies_dir).unwrap();

    // Create a test policy file
    let policy_content = r#"# Test policy file
WHEN node.vendor == "cisco" THEN ASSERT node.version IS "15.1"
WHEN node.role == "router" THEN SET custom_data.managed TO true
"#;
    let policy_file = policies_dir.join("test.policy");
    fs::write(&policy_file, policy_content).unwrap();

    // Create policy loader
    let git_config = create_test_git_config();
    let mut loader = PolicyLoader::new(git_config).with_local_dir(policies_dir);

    // Load policies
    let result = loader.load_policies().unwrap();

    assert_eq!(result.loaded.len(), 1);
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.total_files, 1);

    let policy_file = &result.loaded[0];
    assert_eq!(policy_file.rules.len(), 2);
}

#[test]
fn test_policy_file_validation() {
    let git_config = create_test_git_config();
    let loader = PolicyLoader::new(git_config);

    let valid_content = r#"# Valid policy file
WHEN node.vendor == "cisco" THEN ASSERT node.version IS "15.1"
WHEN node.role == "router" THEN SET custom_data.managed TO true
"#;

    let result = loader.validate_policy_file(valid_content);
    assert!(result.is_valid());
    assert_eq!(result.valid_rules, 2);
    assert_eq!(result.error_count(), 0);

    let invalid_content = r#"# Invalid policy file
WHEN node.vendor == "cisco" THEN ASSERT node.version IS "15.1"
INVALID SYNTAX HERE
WHEN node.role == "router" THEN SET custom_data.managed TO true
"#;

    let result = loader.validate_policy_file(invalid_content);
    assert!(!result.is_valid());
    assert_eq!(result.valid_rules, 2);
    assert_eq!(result.error_count(), 1);
}

#[test]
fn test_policy_caching() {
    let temp_dir = TempDir::new().unwrap();
    let policy_file = temp_dir.path().join("test.policy");

    let policy_content = r#"WHEN node.vendor == "cisco" THEN ASSERT node.version IS "15.1""#;
    fs::write(&policy_file, policy_content).unwrap();

    let git_config = create_test_git_config();
    let mut loader = PolicyLoader::new(git_config).with_cache_ttl(Duration::from_secs(60));

    // Load file first time
    let result1 = loader.load_policy_file(&policy_file).unwrap();
    assert_eq!(result1.rules.len(), 1);

    // Check cache stats
    let stats = loader.cache_stats();
    assert_eq!(stats.total_entries, 1);
    assert_eq!(stats.valid_entries, 1);

    // Load file second time (should use cache)
    let result2 = loader.load_policy_file(&policy_file).unwrap();
    assert_eq!(result2.rules.len(), 1);

    // Modify file to invalidate cache
    std::thread::sleep(Duration::from_millis(10)); // Ensure different mtime
    fs::write(&policy_file, policy_content).unwrap();

    // Load file third time (should reload from disk)
    let result3 = loader.load_policy_file(&policy_file).unwrap();
    assert_eq!(result3.rules.len(), 1);
}

#[test]
fn test_policy_loader_directory_not_found() {
    let git_config = create_test_git_config();
    let non_existent_dir = std::path::Path::new("/non/existent/directory");
    let mut loader = PolicyLoader::new(git_config).with_local_dir(non_existent_dir);

    let result = loader.load_policies();
    assert!(result.is_err());

    if let Err(PolicyError::Io(io_error)) = result {
        assert_eq!(io_error.kind(), std::io::ErrorKind::NotFound);
    } else {
        panic!("Expected IO error for non-existent directory");
    }
}

#[test]
fn test_policy_loader_clear_expired_cache() {
    let temp_dir = TempDir::new().unwrap();
    let policy_file = temp_dir.path().join("test.policy");

    let policy_content = r#"WHEN node.vendor == "cisco" THEN ASSERT node.version IS "15.1""#;
    fs::write(&policy_file, policy_content).unwrap();

    let git_config = create_test_git_config();
    let mut loader = PolicyLoader::new(git_config).with_cache_ttl(Duration::from_millis(1));

    // Load file to add to cache
    let _result = loader.load_policy_file(&policy_file).unwrap();

    // Wait for cache to expire
    std::thread::sleep(Duration::from_millis(10));

    // Clear expired cache entries
    let cleared_count = loader.clear_expired_cache();
    assert_eq!(cleared_count, 1);

    // Check cache is now empty
    let stats = loader.cache_stats();
    assert_eq!(stats.total_entries, 0);
}

#[test]
fn test_policy_loader_invalid_policy_file() {
    let temp_dir = TempDir::new().unwrap();
    let policies_dir = temp_dir.path().join("policies");
    fs::create_dir_all(&policies_dir).unwrap();

    // Create an invalid policy file
    let invalid_policy = policies_dir.join("invalid.policy");
    fs::write(&invalid_policy, "COMPLETELY INVALID SYNTAX").unwrap();

    let git_config = create_test_git_config();
    let mut loader = PolicyLoader::new(git_config).with_local_dir(policies_dir);

    let result = loader.load_policies().unwrap();
    assert_eq!(result.loaded.len(), 0);
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.total_files, 1);
}

#[test]
fn test_policy_loader_mixed_valid_invalid_files() {
    let temp_dir = TempDir::new().unwrap();
    let policies_dir = temp_dir.path().join("policies");
    fs::create_dir_all(&policies_dir).unwrap();

    // Create valid policy file
    let valid_policy = r#"WHEN node.vendor == "cisco" THEN ASSERT node.version IS "15.1""#;
    fs::write(policies_dir.join("valid.policy"), valid_policy).unwrap();

    // Create invalid policy file
    fs::write(policies_dir.join("invalid.policy"), "INVALID SYNTAX").unwrap();

    let git_config = create_test_git_config();
    let mut loader = PolicyLoader::new(git_config).with_local_dir(policies_dir);

    let result = loader.load_policies().unwrap();
    assert_eq!(result.loaded.len(), 1);
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.total_files, 2);
}
