//! CSV store core implementation
//!
//! Contains the main `CsvStore` structure and data management operations
//! for file-based data persistence.

use super::super::types::{DataStoreError, DataStoreResult};
use crate::models::{Link, Location, Node};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Shared data structure for CSV store
#[derive(Debug, Default)]
pub(crate) struct CsvData {
    /// Nodes storage
    pub(crate) nodes: HashMap<Uuid, Node>,
    /// Links storage
    pub(crate) links: HashMap<Uuid, Link>,
    /// Locations storage
    pub(crate) locations: HashMap<Uuid, Location>,
}

/// CSV-based `DataStore` implementation
#[derive(Debug)]
pub struct CsvStore {
    /// Base directory path for CSV files
    pub(crate) base_path: PathBuf,
    /// In-memory data storage
    pub(crate) data: Arc<Mutex<CsvData>>,
}

impl CsvStore {
    /// Creates a new CSV store with the given base directory
    ///
    /// # Errors
    /// Returns an error if the directory cannot be created or accessed
    pub async fn new<P: AsRef<Path>>(base_path: P) -> DataStoreResult<Self> {
        let base_path = base_path.as_ref().to_path_buf();

        // Create directory if it doesn't exist
        if let Some(parent) = base_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| DataStoreError::ConnectionError {
                    message: format!("Failed to create directory: {e}"),
                })?;
        }

        let store = Self {
            base_path,
            data: Arc::new(Mutex::new(CsvData::default())),
        };

        // Load existing data
        store.load_data().await?;

        Ok(store)
    }

    /// Loads data from CSV files
    pub(crate) async fn load_data(&self) -> DataStoreResult<()> {
        // For simplicity, we'll use JSON files instead of CSV for demo
        // In a real implementation, you'd use a CSV library
        let nodes_file = self.base_path.join("nodes.json");
        let links_file = self.base_path.join("links.json");
        let locations_file = self.base_path.join("locations.json");

        // Load all data first (without holding the lock)
        let nodes = if nodes_file.exists() {
            let content = fs::read_to_string(&nodes_file).await.map_err(|e| {
                DataStoreError::ConnectionError {
                    message: format!("Failed to read nodes file: {e}"),
                }
            })?;
            serde_json::from_str::<Vec<Node>>(&content).map_err(|e| {
                DataStoreError::InternalError {
                    message: format!("Failed to parse nodes: {e}"),
                }
            })?
        } else {
            Vec::new()
        };

        let links = if links_file.exists() {
            let content = fs::read_to_string(&links_file).await.map_err(|e| {
                DataStoreError::ConnectionError {
                    message: format!("Failed to read links file: {e}"),
                }
            })?;
            serde_json::from_str::<Vec<Link>>(&content).map_err(|e| {
                DataStoreError::InternalError {
                    message: format!("Failed to parse links: {e}"),
                }
            })?
        } else {
            Vec::new()
        };

        let locations = if locations_file.exists() {
            let content = fs::read_to_string(&locations_file).await.map_err(|e| {
                DataStoreError::ConnectionError {
                    message: format!("Failed to read locations file: {e}"),
                }
            })?;
            serde_json::from_str::<Vec<Location>>(&content).map_err(|e| {
                DataStoreError::InternalError {
                    message: format!("Failed to parse locations: {e}"),
                }
            })?
        } else {
            Vec::new()
        };

        // Now acquire lock and update data
        {
            let mut data = self.data.lock().await;
            for node in nodes {
                data.nodes.insert(node.id, node);
            }
            for link in links {
                data.links.insert(link.id, link);
            }
            for location in locations {
                data.locations.insert(location.id, location);
            }
        }

        Ok(())
    }

    /// Saves data to CSV files
    pub(crate) async fn save_data(&self) -> DataStoreResult<()> {
        // Collect data first (minimizing lock time)
        let (nodes_content, links_content, locations_content) = {
            let data = self.data.lock().await;
            let nodes: Vec<Node> = data.nodes.values().cloned().collect();
            let links: Vec<Link> = data.links.values().cloned().collect();
            let locations: Vec<Location> = data.locations.values().cloned().collect();
            drop(data);

            let nodes_content = serde_json::to_string_pretty(&nodes).map_err(|e| {
                DataStoreError::InternalError {
                    message: format!("Failed to serialize nodes: {e}"),
                }
            })?;
            let links_content = serde_json::to_string_pretty(&links).map_err(|e| {
                DataStoreError::InternalError {
                    message: format!("Failed to serialize links: {e}"),
                }
            })?;
            let locations_content = serde_json::to_string_pretty(&locations).map_err(|e| {
                DataStoreError::InternalError {
                    message: format!("Failed to serialize locations: {e}"),
                }
            })?;

            (nodes_content, links_content, locations_content)
        };

        // Ensure the directory exists before writing files
        fs::create_dir_all(&self.base_path)
            .await
            .map_err(|e| DataStoreError::ConnectionError {
                message: format!("Failed to create directory: {e}"),
            })?;

        // Now write files without holding the lock
        fs::write(self.base_path.join("nodes.json"), nodes_content)
            .await
            .map_err(|e| DataStoreError::ConnectionError {
                message: format!("Failed to write nodes file: {e}"),
            })?;

        fs::write(self.base_path.join("links.json"), links_content)
            .await
            .map_err(|e| DataStoreError::ConnectionError {
                message: format!("Failed to write links file: {e}"),
            })?;

        fs::write(self.base_path.join("locations.json"), locations_content)
            .await
            .map_err(|e| DataStoreError::ConnectionError {
                message: format!("Failed to write locations file: {e}"),
            })?;

        Ok(())
    }
}
