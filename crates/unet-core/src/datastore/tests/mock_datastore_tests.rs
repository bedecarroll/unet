//! Mock `DataStore` implementation for testing trait defaults

use crate::datastore::{
    DataStore,
    types::{BatchOperation, BatchResult, PagedResult, QueryOptions, Transaction},
};
use crate::models::{Link, Location, Node};
use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;

/// Mock `DataStore` implementation for testing trait defaults
pub struct MockDataStore {
    nodes: HashMap<Uuid, Node>,
    links: HashMap<Uuid, Link>,
    locations: HashMap<Uuid, Location>,
}

impl MockDataStore {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            links: HashMap::new(),
            locations: HashMap::new(),
        }
    }

    pub fn with_node(mut self, node: Node) -> Self {
        self.nodes.insert(node.id, node);
        self
    }

    pub fn with_link(mut self, link: Link) -> Self {
        self.links.insert(link.id, link);
        self
    }

    pub fn with_location(mut self, location: Location) -> Self {
        self.locations.insert(location.id, location);
        self
    }
}

#[async_trait]
impl DataStore for MockDataStore {
    fn name(&self) -> &'static str {
        "MockDataStore"
    }

    async fn health_check(&self) -> crate::datastore::types::DataStoreResult<()> {
        Ok(())
    }

    async fn begin_transaction(
        &self,
    ) -> crate::datastore::types::DataStoreResult<Box<dyn Transaction>> {
        unimplemented!("Not needed for trait default tests")
    }

    async fn create_node(&self, _node: &Node) -> crate::datastore::types::DataStoreResult<Node> {
        unimplemented!("Not needed for trait default tests")
    }

    async fn get_node(&self, id: &Uuid) -> crate::datastore::types::DataStoreResult<Option<Node>> {
        Ok(self.nodes.get(id).cloned())
    }

    async fn list_nodes(
        &self,
        _options: &QueryOptions,
    ) -> crate::datastore::types::DataStoreResult<PagedResult<Node>> {
        let items: Vec<Node> = self.nodes.values().cloned().collect();
        Ok(PagedResult {
            items,
            total_count: self.nodes.len(),
            page: 1,
            page_size: 10,
            total_pages: 1,
            has_next: false,
            has_previous: false,
        })
    }

    async fn update_node(&self, node: &Node) -> crate::datastore::types::DataStoreResult<Node> {
        // For testing purposes, just return the node as if it was updated
        Ok(node.clone())
    }

    async fn delete_node(&self, _id: &Uuid) -> crate::datastore::types::DataStoreResult<()> {
        unimplemented!("Not needed for trait default tests")
    }

    async fn get_nodes_by_location(
        &self,
        _location_id: &Uuid,
    ) -> crate::datastore::types::DataStoreResult<Vec<Node>> {
        unimplemented!("Not needed for trait default tests")
    }

    async fn search_nodes_by_name(
        &self,
        _name: &str,
    ) -> crate::datastore::types::DataStoreResult<Vec<Node>> {
        unimplemented!("Not needed for trait default tests")
    }

    async fn create_link(&self, _link: &Link) -> crate::datastore::types::DataStoreResult<Link> {
        unimplemented!("Not needed for trait default tests")
    }

    async fn get_link(&self, id: &Uuid) -> crate::datastore::types::DataStoreResult<Option<Link>> {
        Ok(self.links.get(id).cloned())
    }

    async fn list_links(
        &self,
        _options: &QueryOptions,
    ) -> crate::datastore::types::DataStoreResult<PagedResult<Link>> {
        unimplemented!("Not needed for trait default tests")
    }

    async fn update_link(&self, _link: &Link) -> crate::datastore::types::DataStoreResult<Link> {
        unimplemented!("Not needed for trait default tests")
    }

    async fn delete_link(&self, _id: &Uuid) -> crate::datastore::types::DataStoreResult<()> {
        unimplemented!("Not needed for trait default tests")
    }

    async fn get_links_for_node(
        &self,
        _node_id: &Uuid,
    ) -> crate::datastore::types::DataStoreResult<Vec<Link>> {
        unimplemented!("Not needed for trait default tests")
    }

    async fn get_links_between_nodes(
        &self,
        _first_node_id: &Uuid,
        _second_node_id: &Uuid,
    ) -> crate::datastore::types::DataStoreResult<Vec<Link>> {
        unimplemented!("Not needed for trait default tests")
    }

    async fn create_location(
        &self,
        _location: &Location,
    ) -> crate::datastore::types::DataStoreResult<Location> {
        unimplemented!("Not needed for trait default tests")
    }

    async fn get_location(
        &self,
        id: &Uuid,
    ) -> crate::datastore::types::DataStoreResult<Option<Location>> {
        Ok(self.locations.get(id).cloned())
    }

    async fn list_locations(
        &self,
        _options: &QueryOptions,
    ) -> crate::datastore::types::DataStoreResult<PagedResult<Location>> {
        unimplemented!("Not needed for trait default tests")
    }

    async fn update_location(
        &self,
        _location: &Location,
    ) -> crate::datastore::types::DataStoreResult<Location> {
        unimplemented!("Not needed for trait default tests")
    }

    async fn delete_location(&self, _id: &Uuid) -> crate::datastore::types::DataStoreResult<()> {
        unimplemented!("Not needed for trait default tests")
    }

    async fn create_vendor(&self, _name: &str) -> crate::datastore::types::DataStoreResult<()> {
        unimplemented!("Not needed for trait default tests")
    }

    async fn list_vendors(&self) -> crate::datastore::types::DataStoreResult<Vec<String>> {
        unimplemented!("Not needed for trait default tests")
    }

    async fn delete_vendor(&self, _name: &str) -> crate::datastore::types::DataStoreResult<()> {
        unimplemented!("Not needed for trait default tests")
    }

    async fn batch_nodes(
        &self,
        _operations: &[BatchOperation<Node>],
    ) -> crate::datastore::types::DataStoreResult<BatchResult> {
        unimplemented!("Not needed for trait default tests")
    }

    async fn batch_links(
        &self,
        _operations: &[BatchOperation<Link>],
    ) -> crate::datastore::types::DataStoreResult<BatchResult> {
        unimplemented!("Not needed for trait default tests")
    }

    async fn batch_locations(
        &self,
        _operations: &[BatchOperation<Location>],
    ) -> crate::datastore::types::DataStoreResult<BatchResult> {
        unimplemented!("Not needed for trait default tests")
    }

    async fn get_entity_counts(
        &self,
    ) -> crate::datastore::types::DataStoreResult<HashMap<String, usize>> {
        unimplemented!("Not needed for trait default tests")
    }

    async fn get_statistics(
        &self,
    ) -> crate::datastore::types::DataStoreResult<HashMap<String, serde_json::Value>> {
        unimplemented!("Not needed for trait default tests")
    }
}
