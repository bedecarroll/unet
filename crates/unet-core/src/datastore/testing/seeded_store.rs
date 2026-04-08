//! Seeded in-memory `DataStore` fake for tests.

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use uuid::Uuid;

use crate::datastore::{
    BatchOperation, BatchResult, DataStore, DataStoreResult, PagedResult, QueryOptions, Transaction,
};
use crate::models::{Link, Location, Node};

use super::unexpected_call;

#[derive(Clone, Default)]
struct SeededData {
    nodes: HashMap<Uuid, Node>,
    links: HashMap<Uuid, Link>,
    locations: HashMap<Uuid, Location>,
}

/// Seeded in-memory `DataStore` fake with explicit failures for unconfigured writes.
#[derive(Default)]
pub struct SeededDataStore {
    data: Mutex<SeededData>,
}

impl SeededDataStore {
    /// Creates an empty seeded store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Seeds a node into the fake store.
    #[must_use]
    pub fn with_node(mut self, node: Node) -> Self {
        self.data_mut().nodes.insert(node.id, node);
        self
    }

    /// Seeds a link into the fake store.
    #[must_use]
    pub fn with_link(mut self, link: Link) -> Self {
        self.data_mut().links.insert(link.id, link);
        self
    }

    /// Seeds a location into the fake store.
    #[must_use]
    pub fn with_location(mut self, location: Location) -> Self {
        self.data_mut().locations.insert(location.id, location);
        self
    }

    fn snapshot(&self) -> SeededData {
        self.data
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    fn data_mut(&mut self) -> &mut SeededData {
        self.data
            .get_mut()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[async_trait]
impl DataStore for SeededDataStore {
    fn name(&self) -> &'static str {
        "SeededDataStore"
    }

    async fn health_check(&self) -> DataStoreResult<()> {
        Ok(())
    }

    async fn begin_transaction(&self) -> DataStoreResult<Box<dyn Transaction>> {
        unexpected_call(self.name(), "begin_transaction")
    }

    async fn create_node(&self, _node: &Node) -> DataStoreResult<Node> {
        unexpected_call(self.name(), "create_node")
    }

    async fn get_node(&self, id: &Uuid) -> DataStoreResult<Option<Node>> {
        Ok(self.snapshot().nodes.get(id).cloned())
    }

    async fn list_nodes(&self, options: &QueryOptions) -> DataStoreResult<PagedResult<Node>> {
        let items = self.snapshot().nodes.into_values().collect::<Vec<_>>();
        Ok(PagedResult::new(
            items.clone(),
            items.len(),
            options.pagination.as_ref(),
        ))
    }

    async fn update_node(&self, node: &Node) -> DataStoreResult<Node> {
        self.data
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .nodes
            .insert(node.id, node.clone());
        Ok(node.clone())
    }

    async fn delete_node(&self, _id: &Uuid) -> DataStoreResult<()> {
        unexpected_call(self.name(), "delete_node")
    }

    async fn get_nodes_by_location(&self, _location_id: &Uuid) -> DataStoreResult<Vec<Node>> {
        unexpected_call(self.name(), "get_nodes_by_location")
    }

    async fn search_nodes_by_name(&self, _name: &str) -> DataStoreResult<Vec<Node>> {
        unexpected_call(self.name(), "search_nodes_by_name")
    }

    async fn create_link(&self, _link: &Link) -> DataStoreResult<Link> {
        unexpected_call(self.name(), "create_link")
    }

    async fn get_link(&self, id: &Uuid) -> DataStoreResult<Option<Link>> {
        Ok(self.snapshot().links.get(id).cloned())
    }

    async fn list_links(&self, options: &QueryOptions) -> DataStoreResult<PagedResult<Link>> {
        let items = self.snapshot().links.into_values().collect::<Vec<_>>();
        Ok(PagedResult::new(
            items.clone(),
            items.len(),
            options.pagination.as_ref(),
        ))
    }

    async fn update_link(&self, _link: &Link) -> DataStoreResult<Link> {
        unexpected_call(self.name(), "update_link")
    }

    async fn delete_link(&self, _id: &Uuid) -> DataStoreResult<()> {
        unexpected_call(self.name(), "delete_link")
    }

    async fn get_links_for_node(&self, _node_id: &Uuid) -> DataStoreResult<Vec<Link>> {
        unexpected_call(self.name(), "get_links_for_node")
    }

    async fn get_links_between_nodes(
        &self,
        _first_node_id: &Uuid,
        _second_node_id: &Uuid,
    ) -> DataStoreResult<Vec<Link>> {
        unexpected_call(self.name(), "get_links_between_nodes")
    }

    async fn create_location(&self, _location: &Location) -> DataStoreResult<Location> {
        unexpected_call(self.name(), "create_location")
    }

    async fn get_location(&self, id: &Uuid) -> DataStoreResult<Option<Location>> {
        Ok(self.snapshot().locations.get(id).cloned())
    }

    async fn list_locations(
        &self,
        options: &QueryOptions,
    ) -> DataStoreResult<PagedResult<Location>> {
        let items = self.snapshot().locations.into_values().collect::<Vec<_>>();
        Ok(PagedResult::new(
            items.clone(),
            items.len(),
            options.pagination.as_ref(),
        ))
    }

    async fn update_location(&self, _location: &Location) -> DataStoreResult<Location> {
        unexpected_call(self.name(), "update_location")
    }

    async fn delete_location(&self, _id: &Uuid) -> DataStoreResult<()> {
        unexpected_call(self.name(), "delete_location")
    }

    async fn create_vendor(&self, _name: &str) -> DataStoreResult<()> {
        unexpected_call(self.name(), "create_vendor")
    }

    async fn list_vendors(&self) -> DataStoreResult<Vec<String>> {
        unexpected_call(self.name(), "list_vendors")
    }

    async fn delete_vendor(&self, _name: &str) -> DataStoreResult<()> {
        unexpected_call(self.name(), "delete_vendor")
    }

    async fn batch_nodes(
        &self,
        _operations: &[BatchOperation<Node>],
    ) -> DataStoreResult<BatchResult> {
        unexpected_call(self.name(), "batch_nodes")
    }

    async fn batch_links(
        &self,
        _operations: &[BatchOperation<Link>],
    ) -> DataStoreResult<BatchResult> {
        unexpected_call(self.name(), "batch_links")
    }

    async fn batch_locations(
        &self,
        _operations: &[BatchOperation<Location>],
    ) -> DataStoreResult<BatchResult> {
        unexpected_call(self.name(), "batch_locations")
    }

    async fn get_entity_counts(&self) -> DataStoreResult<HashMap<String, usize>> {
        unexpected_call(self.name(), "get_entity_counts")
    }

    async fn get_statistics(&self) -> DataStoreResult<HashMap<String, serde_json::Value>> {
        unexpected_call(self.name(), "get_statistics")
    }
}
