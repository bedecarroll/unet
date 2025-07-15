//! `DataStore` trait implementation for CSV store
//!
//! Contains the complete `DataStore` trait implementation for the CSV-based
//! datastore, delegating to specialized modules for entity operations.

use super::super::DataStore;
use super::super::types::{
    BatchOperation, BatchResult, DataStoreError, DataStoreResult, PagedResult, QueryOptions,
    Transaction,
};
use super::store::{CsvData, CsvStore};
use super::transaction::CsvTransaction;
use super::{links, locations, nodes};
use crate::models::{Link, Location, Node};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

#[async_trait]
impl DataStore for CsvStore {
    fn name(&self) -> &'static str {
        "CSV"
    }

    async fn health_check(&self) -> DataStoreResult<()> {
        // Check if base directory is accessible
        if !self.base_path.parent().unwrap_or(&self.base_path).exists() {
            return Err(DataStoreError::ConnectionError {
                message: "Base directory is not accessible".to_string(),
            });
        }
        Ok(())
    }

    async fn begin_transaction(&self) -> DataStoreResult<Box<dyn Transaction>> {
        Ok(Box::new(CsvTransaction {
            store: Arc::new(Self {
                base_path: self.base_path.clone(),
                data: Arc::clone(&self.data),
            }),
            changes: Mutex::new(CsvData::default()),
            committed: Mutex::new(false),
        }))
    }

    // Node operations - delegate to nodes module
    async fn create_node(&self, node: &Node) -> DataStoreResult<Node> {
        nodes::create_node(self, node).await
    }

    async fn get_node(&self, id: &Uuid) -> DataStoreResult<Option<Node>> {
        nodes::get_node(self, id).await
    }

    async fn list_nodes(&self, options: &QueryOptions) -> DataStoreResult<PagedResult<Node>> {
        nodes::list_nodes(self, options).await
    }

    async fn update_node(&self, node: &Node) -> DataStoreResult<Node> {
        nodes::update_node(self, node).await
    }

    async fn delete_node(&self, id: &Uuid) -> DataStoreResult<()> {
        nodes::delete_node(self, id).await
    }

    async fn get_nodes_by_location(&self, location_id: &Uuid) -> DataStoreResult<Vec<Node>> {
        nodes::get_nodes_by_location(self, location_id).await
    }

    async fn search_nodes_by_name(&self, name: &str) -> DataStoreResult<Vec<Node>> {
        nodes::search_nodes_by_name(self, name).await
    }

    async fn batch_nodes(
        &self,
        operations: &[BatchOperation<Node>],
    ) -> DataStoreResult<BatchResult> {
        nodes::batch_nodes(self, operations).await
    }

    // Link operations - delegate to links module
    async fn create_link(&self, link: &Link) -> DataStoreResult<Link> {
        links::create_link(self, link).await
    }

    async fn get_link(&self, id: &Uuid) -> DataStoreResult<Option<Link>> {
        links::get_link(self, id).await
    }

    async fn list_links(&self, options: &QueryOptions) -> DataStoreResult<PagedResult<Link>> {
        links::list_links(self, options).await
    }

    async fn update_link(&self, link: &Link) -> DataStoreResult<Link> {
        links::update_link(self, link).await
    }

    async fn delete_link(&self, id: &Uuid) -> DataStoreResult<()> {
        links::delete_link(self, id).await
    }

    async fn get_links_for_node(&self, node_id: &Uuid) -> DataStoreResult<Vec<Link>> {
        links::get_links_for_node(self, node_id).await
    }

    async fn get_links_between_nodes(
        &self,
        first_node_id: &Uuid,
        second_node_id: &Uuid,
    ) -> DataStoreResult<Vec<Link>> {
        links::get_links_between_nodes(self, first_node_id, second_node_id).await
    }

    async fn batch_links(
        &self,
        operations: &[BatchOperation<Link>],
    ) -> DataStoreResult<BatchResult> {
        links::batch_links(self, operations).await
    }

    // Location operations - delegate to locations module
    async fn create_location(&self, location: &Location) -> DataStoreResult<Location> {
        locations::create_location(self, location).await
    }

    async fn get_location(&self, id: &Uuid) -> DataStoreResult<Option<Location>> {
        locations::get_location(self, id).await
    }

    async fn list_locations(
        &self,
        options: &QueryOptions,
    ) -> DataStoreResult<PagedResult<Location>> {
        locations::list_locations(self, options).await
    }

    async fn update_location(&self, location: &Location) -> DataStoreResult<Location> {
        locations::update_location(self, location).await
    }

    async fn delete_location(&self, id: &Uuid) -> DataStoreResult<()> {
        locations::delete_location(self, id).await
    }

    async fn batch_locations(
        &self,
        operations: &[BatchOperation<Location>],
    ) -> DataStoreResult<BatchResult> {
        locations::batch_locations(self, operations).await
    }

    // Statistics operations
    async fn get_entity_counts(&self) -> DataStoreResult<HashMap<String, usize>> {
        let mut counts = HashMap::new();
        {
            let data = self.data.lock().await;
            counts.insert("nodes".to_string(), data.nodes.len());
            counts.insert("links".to_string(), data.links.len());
            counts.insert("locations".to_string(), data.locations.len());
        }
        Ok(counts)
    }

    async fn get_statistics(&self) -> DataStoreResult<HashMap<String, serde_json::Value>> {
        let mut stats = HashMap::new();
        {
            let data = self.data.lock().await;
            stats.insert("total_nodes".to_string(), data.nodes.len().into());
            stats.insert("total_links".to_string(), data.links.len().into());
            stats.insert("total_locations".to_string(), data.locations.len().into());
        }
        Ok(stats)
    }
}
