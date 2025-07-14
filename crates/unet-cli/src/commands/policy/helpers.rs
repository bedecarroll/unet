//! Shared helper functions for policy commands

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
