/// File loading functions for import operations
use anyhow::Result;
use std::path::Path;
use unet_core::prelude::*;

/// Load locations from JSON file
pub async fn load_locations(base_path: &Path) -> Result<Option<Vec<Location>>> {
    let locations_file = base_path.join("locations.json");
    if !locations_file.exists() {
        return Ok(None);
    }

    let content = tokio::fs::read_to_string(&locations_file).await?;
    let locations: Vec<Location> = serde_json::from_str(&content)?;
    Ok(Some(locations))
}

/// Load nodes from JSON file
pub async fn load_nodes(base_path: &Path) -> Result<Option<Vec<Node>>> {
    let nodes_file = base_path.join("nodes.json");
    if !nodes_file.exists() {
        return Ok(None);
    }

    let content = tokio::fs::read_to_string(&nodes_file).await?;
    let nodes: Vec<Node> = serde_json::from_str(&content)?;
    Ok(Some(nodes))
}

/// Load links from JSON file
pub async fn load_links(base_path: &Path) -> Result<Option<Vec<Link>>> {
    let links_file = base_path.join("links.json");
    if !links_file.exists() {
        return Ok(None);
    }

    let content = tokio::fs::read_to_string(&links_file).await?;
    let links: Vec<Link> = serde_json::from_str(&content)?;
    Ok(Some(links))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;
    use tempfile::TempDir;
    use unet_core::models::{DeviceRole, Location, Vendor};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_load_locations_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let result = load_locations(temp_dir.path()).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_load_locations_valid_file() {
        let temp_dir = TempDir::new().unwrap();
        let locations_file = temp_dir.path().join("locations.json");

        let location = Location {
            id: Uuid::new_v4(),
            name: "Test Location".to_string(),
            location_type: "datacenter".to_string(),
            parent_id: None,
            path: "Test Location".to_string(),
            description: Some("Test description".to_string()),
            address: Some("123 Test St".to_string()),
            custom_data: serde_json::Value::Null,
        };

        let locations_json = serde_json::to_string_pretty(&vec![location.clone()]).unwrap();
        tokio::fs::write(&locations_file, locations_json)
            .await
            .unwrap();

        let result = load_locations(temp_dir.path()).await.unwrap();
        assert!(result.is_some());
        let loaded_locations = result.unwrap();
        assert_eq!(loaded_locations.len(), 1);
        assert_eq!(loaded_locations[0].name, "Test Location");
    }

    #[tokio::test]
    async fn test_load_nodes_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let result = load_nodes(temp_dir.path()).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_load_nodes_valid_file() {
        let temp_dir = TempDir::new().unwrap();
        let nodes_file = temp_dir.path().join("nodes.json");

        let mut node = Node::new(
            "test-node".to_string(),
            "example.com".to_string(),
            Vendor::Cisco,
            DeviceRole::Router,
        );
        node.model = "Test Model".to_string();
        node.management_ip = Some(IpAddr::V4("192.168.1.1".parse().unwrap()));

        let nodes_json = serde_json::to_string_pretty(&vec![node.clone()]).unwrap();
        tokio::fs::write(&nodes_file, nodes_json).await.unwrap();

        let result = load_nodes(temp_dir.path()).await.unwrap();
        assert!(result.is_some());
        let loaded_nodes = result.unwrap();
        assert_eq!(loaded_nodes.len(), 1);
        assert_eq!(loaded_nodes[0].name, "test-node");
    }

    #[tokio::test]
    async fn test_load_links_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let result = load_links(temp_dir.path()).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_load_links_valid_file() {
        let temp_dir = TempDir::new().unwrap();
        let links_file = temp_dir.path().join("links.json");

        let link = Link::new(
            "test-link".to_string(),
            Uuid::new_v4(),
            "eth0".to_string(),
            Uuid::new_v4(),
            "eth1".to_string(),
        );

        let links_json = serde_json::to_string_pretty(&vec![link.clone()]).unwrap();
        tokio::fs::write(&links_file, links_json).await.unwrap();

        let result = load_links(temp_dir.path()).await.unwrap();
        assert!(result.is_some());
        let loaded_links = result.unwrap();
        assert_eq!(loaded_links.len(), 1);
        assert_eq!(loaded_links[0].name, "test-link");
    }
}
