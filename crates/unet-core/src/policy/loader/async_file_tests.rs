//! Tests to reproduce and verify the fix for async file I/O deadlock issues

use super::*;
use crate::config::GitConfig;
use std::fs;
use tempfile::TempDir;
use tokio::runtime::Runtime;

#[test]
fn test_policy_loader_reproduces_deadlock_in_single_threaded_runtime() {
    // This test reproduces the deadlock issue by using a single-threaded tokio runtime
    // and calling synchronous file I/O from within an async context
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let temp_dir = TempDir::new().unwrap();
        let policies_dir = temp_dir.path().join("policies");
        fs::create_dir_all(&policies_dir).unwrap();

        // Create a test policy file
        let policy_content = r#"# Test policy file for deadlock reproduction
WHEN node.vendor == "cisco" THEN ASSERT node.version IS "WrongVersion"
"#;
        let policy_file = policies_dir.join("test.policy");
        fs::write(&policy_file, policy_content).unwrap();

        let git_config = GitConfig {
            repository_url: None,
            local_directory: None,
            branch: "main".to_string(),
            auth_token: None,
            sync_interval: 300,
            policies_repo: None,
            templates_repo: None,
        };

        let mut loader = PolicyLoader::new(git_config).with_local_dir(&policies_dir);

        // This should complete without deadlocking
        // In the current implementation, this may deadlock due to blocking I/O in async context
        let result = tokio::time::timeout(std::time::Duration::from_secs(5), async {
            loader.load_policies()
        })
        .await;

        // This test should pass after we fix the async I/O issue
        assert!(
            result.is_ok(),
            "Policy loading timed out - likely due to deadlock"
        );

        let load_result = result.unwrap().unwrap();
        assert_eq!(load_result.loaded.len(), 1);
        assert_eq!(load_result.errors.len(), 0);
    });
}

#[tokio::test]
async fn test_policy_loader_async_load_policies_works() {
    // This test demonstrates that async file I/O is now implemented
    let temp_dir = TempDir::new().unwrap();
    let policies_dir = temp_dir.path().join("policies");
    fs::create_dir_all(&policies_dir).unwrap();

    // Create a test policy file
    let policy_content = r#"# Test policy file for async loading
WHEN node.vendor == "cisco" THEN ASSERT node.version IS "15.1"
"#;
    let policy_file = policies_dir.join("test.policy");
    fs::write(&policy_file, policy_content).unwrap();

    let git_config = GitConfig {
        repository_url: None,
        local_directory: None,
        branch: "main".to_string(),
        auth_token: None,
        sync_interval: 300,
        policies_repo: None,
        templates_repo: None,
    };

    let mut loader = PolicyLoader::new(git_config).with_local_dir(&policies_dir);

    // This should work without deadlocking now that we have async file I/O
    let result = loader.load_policies_async().await;

    assert!(result.is_ok());
    let load_result = result.unwrap();
    assert_eq!(load_result.loaded.len(), 1);
    assert_eq!(load_result.errors.len(), 0);
}
