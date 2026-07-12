//! Tests for configuration-related functionality

use super::super::service::PolicyService;
use crate::config::GitConfig;

#[tokio::test]
async fn test_policy_service_load_policies_empty_directory() {
    // Create a service with a non-existent directory
    let mut service = PolicyService::with_local_dir("/tmp/non_existent_policies_dir_12345");

    let result = service.load_policies();

    // Should return error when trying to load from non-existent directory
    assert!(result.is_err());
}

#[test]
fn test_git_config_construction_variants() {
    // Test with repository URL
    let git_config_with_repo = GitConfig {
        repository_url: Some("https://github.com/test/policies".to_string()),
        local_directory: Some("./policies".to_string()),
        branch: "develop".to_string(),
        auth_token: Some("token123".to_string()),
        sync_interval: 600,
        policies_repo: Some("policies-repo".to_string()),
        templates_repo: Some("templates-repo".to_string()),
    };

    let service = PolicyService::new(git_config_with_repo);
    let _loader = service.loader();

    // Test with minimal config
    let minimal_config = GitConfig {
        repository_url: None,
        local_directory: None,
        branch: "main".to_string(),
        auth_token: None,
        sync_interval: 300,
        policies_repo: None,
        templates_repo: None,
    };

    let service2 = PolicyService::new(minimal_config);
    let _loader2 = service2.loader();
}
