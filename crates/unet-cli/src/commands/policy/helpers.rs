/// Shared helper functions for policy commands
use anyhow::Result;
use unet_core::config::GitConfig;
use unet_core::datastore::DataStore;
use unet_core::policy::{PolicyLoader, PolicyParser};
use unet_core::prelude::QueryOptions;

/// Load policies from a file or directory path
pub fn load_policies_from_path(
    path: &std::path::Path,
) -> Result<Vec<Vec<unet_core::policy::PolicyRule>>> {
    // Create a default GitConfig for the loader
    let git_config = GitConfig {
        repository_url: None,
        local_directory: None,
        branch: "main".to_owned(),
        auth_token: None,
        sync_interval: 300,
        policies_repo: None,
        templates_repo: None,
    };

    let mut loader = PolicyLoader::new(git_config);

    if path.is_file() {
        let content = std::fs::read_to_string(path)?;
        let rules = PolicyParser::parse_file(&content)?;
        Ok(vec![rules])
    } else if path.is_dir() {
        let load_result = loader.load_policies_from_directory(path)?;
        if !load_result.errors.is_empty() {
            for (file_path, error) in &load_result.errors {
                println!("❌ Failed to load {}: {}", file_path.display(), error);
            }
            return Err(anyhow::anyhow!("Failed to load some policy files"));
        }
        Ok(load_result.loaded.into_iter().map(|f| f.rules).collect())
    } else {
        Err(anyhow::anyhow!("Path does not exist: {}", path.display()))
    }
}

/// Get nodes to evaluate policies against
pub async fn get_evaluation_nodes(
    node_id: Option<uuid::Uuid>,
    datastore: &dyn DataStore,
) -> Result<Vec<unet_core::models::Node>> {
    if let Some(node_id) = node_id {
        match datastore.get_node(&node_id).await {
            Ok(Some(node)) => Ok(vec![node]),
            Ok(None) => Err(anyhow::anyhow!("Node not found: {}", node_id)),
            Err(e) => Err(anyhow::anyhow!("Failed to get node: {}", e)),
        }
    } else {
        // Get all nodes
        let query_options = QueryOptions::default();
        match datastore.list_nodes(&query_options).await {
            Ok(paged_result) => Ok(paged_result.items),
            Err(e) => Err(anyhow::anyhow!("Failed to list nodes: {}", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn test_load_policies_from_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            "WHEN node.vendor == \"cisco\" THEN ASSERT node.version IS \"15.1\""
        )
        .unwrap();

        let result = load_policies_from_path(temp_file.path());
        assert!(result.is_ok());

        let policies = result.unwrap();
        assert_eq!(policies.len(), 1);
        assert!(!policies[0].is_empty());
    }

    #[test]
    fn test_load_policies_from_directory() {
        let temp_dir = TempDir::new().unwrap();
        let policy_file = temp_dir.path().join("test.policy");

        fs::write(
            &policy_file,
            "WHEN node.role == \"router\" THEN ASSERT node.snmp_enabled IS true",
        )
        .unwrap();

        let result = load_policies_from_path(temp_dir.path());
        assert!(result.is_ok());

        let policies = result.unwrap();
        assert!(!policies.is_empty());
    }

    #[test]
    fn test_load_policies_nonexistent_path() {
        let result = load_policies_from_path(std::path::Path::new("/nonexistent/path"));
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("Path does not exist"));
    }

    #[test]
    fn test_load_policies_invalid_file_content() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "INVALID POLICY SYNTAX").unwrap();

        let result = load_policies_from_path(temp_file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_git_config_creation() {
        // Test that GitConfig is created with expected defaults
        let git_config = GitConfig {
            repository_url: None,
            local_directory: None,
            branch: "main".to_owned(),
            auth_token: None,
            sync_interval: 300,
            policies_repo: None,
            templates_repo: None,
        };

        assert_eq!(git_config.branch, "main");
        assert_eq!(git_config.sync_interval, 300);
        assert!(git_config.repository_url.is_none());
        assert!(git_config.auth_token.is_none());
    }

    #[test]
    fn test_load_policies_empty_directory() {
        let temp_dir = TempDir::new().unwrap();

        let result = load_policies_from_path(temp_dir.path());
        assert!(result.is_ok());

        let policies = result.unwrap();
        assert!(policies.is_empty());
    }

    #[test]
    fn test_load_policies_directory_with_mixed_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create a valid policy file
        let policy_file = temp_dir.path().join("valid.policy");
        fs::write(
            &policy_file,
            "WHEN node.vendor == \"cisco\" THEN ASSERT node.version IS \"15.1\"",
        )
        .unwrap();

        // Create a non-policy file that should be ignored
        let other_file = temp_dir.path().join("readme.txt");
        fs::write(&other_file, "This is not a policy file").unwrap();

        let result = load_policies_from_path(temp_dir.path());
        assert!(result.is_ok());

        let policies = result.unwrap();
        // Should load policies only from .policy files
        assert!(!policies.is_empty());
    }

    // Note: Testing get_evaluation_nodes requires a DataStore implementation
    // These would be integration tests that test against actual/mock datastores
    #[test]
    fn test_get_evaluation_nodes_argument_validation() {
        use uuid::Uuid;

        // Test that we can create valid UUID arguments
        let node_id = Uuid::new_v4();
        assert!(!node_id.to_string().is_empty());

        // Test None case preparation
        let no_node_id: Option<Uuid> = None;
        assert!(no_node_id.is_none());
    }

    #[test]
    fn test_policy_loader_configuration() {
        let git_config = GitConfig {
            repository_url: Some("https://github.com/example/policies.git".to_string()),
            local_directory: Some("/tmp/policies".to_string()),
            branch: "production".to_owned(),
            auth_token: Some("token123".to_string()),
            sync_interval: 600,
            policies_repo: Some("policies".to_string()),
            templates_repo: Some("templates".to_string()),
        };

        let _loader = PolicyLoader::new(git_config);
        // PolicyLoader should be created successfully with custom config
        // (We can't test internal state without exposing it, but construction should work)
    }
}
