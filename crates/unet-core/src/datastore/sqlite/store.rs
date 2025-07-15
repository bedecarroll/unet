//! Main `SQLite` store implementation

use super::{links, locations, nodes};

use super::super::DataStore;
use super::super::types::{
    BatchOperation, BatchResult, DataStoreError, DataStoreResult, PagedResult, QueryOptions,
    Transaction,
};
use super::transaction::SqliteTransaction;
use crate::models::{Link, Location, Node};
use async_trait::async_trait;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, TransactionTrait};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

/// SQLite-based `DataStore` implementation
pub struct SqliteStore {
    /// Database connection
    pub(crate) db: DatabaseConnection,
}

impl SqliteStore {
    /// Creates a new `SQLite` store with the given database URL
    ///
    /// # Errors
    /// Returns an error if the database connection cannot be established
    pub async fn new(database_url: &str) -> DataStoreResult<Self> {
        let mut opt = ConnectOptions::new(database_url);
        opt.max_connections(100)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(8))
            .acquire_timeout(Duration::from_secs(8))
            .idle_timeout(Duration::from_secs(8))
            .max_lifetime(Duration::from_secs(8))
            .sqlx_logging(false);

        let db = Database::connect(opt)
            .await
            .map_err(|e| DataStoreError::ConnectionError {
                message: format!("Failed to connect to database: {e}"),
            })?;

        Ok(Self { db })
    }

    /// Get the database connection for testing
    #[must_use]
    pub const fn connection(&self) -> &DatabaseConnection {
        &self.db
    }
}

#[async_trait]
impl DataStore for SqliteStore {
    fn name(&self) -> &'static str {
        "SQLite"
    }

    async fn health_check(&self) -> DataStoreResult<()> {
        self.db
            .ping()
            .await
            .map_err(|e| DataStoreError::ConnectionError {
                message: format!("Database health check failed: {e}"),
            })
    }

    async fn begin_transaction(&self) -> DataStoreResult<Box<dyn Transaction>> {
        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| DataStoreError::TransactionError {
                message: format!("Failed to begin transaction: {e}"),
            })?;

        Ok(Box::new(SqliteTransaction { txn }))
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
        // TODO: Implement proper entity counting
        // For now, return empty counts - this is a placeholder
        Ok(HashMap::new())
    }

    async fn get_statistics(&self) -> DataStoreResult<HashMap<String, serde_json::Value>> {
        // TODO: Implement proper statistics collection
        // For now, return empty stats - this is a placeholder
        Ok(HashMap::new())
    }
}
