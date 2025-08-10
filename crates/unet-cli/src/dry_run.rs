use std::sync::Arc;
use async_trait::async_trait;
use tracing::info;
use unet_core::datastore::types::{BatchResult, DataStoreError, DataStoreResult, PagedResult, QueryOptions};
use unet_core::datastore::{BatchOperation, DataStore, Transaction};
use unet_core::models::{Link, Location, Node};
use unet_core::policy::PolicyExecutionResult;
use uuid::Uuid;

pub struct DryRunStore {
    inner: Arc<dyn DataStore>,
}

impl DryRunStore {
    pub fn new(inner: Arc<dyn DataStore>) -> Self { Self { inner } }
}

#[async_trait]
impl DataStore for DryRunStore {
    fn name(&self) -> &'static str { "dry-run" }

    async fn health_check(&self) -> DataStoreResult<()> { self.inner.health_check().await }
    async fn begin_transaction(&self) -> DataStoreResult<Box<dyn Transaction>> { self.inner.begin_transaction().await }

    // Node ops
    async fn create_node(&self, node: &Node) -> DataStoreResult<Node> {
        info!("[dry-run] create_node: {}", node.name);
        Ok(node.clone())
    }
    async fn get_node(&self, id: &Uuid) -> DataStoreResult<Option<Node>> { self.inner.get_node(id).await }
    async fn list_nodes(&self, options: &QueryOptions) -> DataStoreResult<PagedResult<Node>> { self.inner.list_nodes(options).await }
    async fn update_node(&self, node: &Node) -> DataStoreResult<Node> {
        info!("[dry-run] update_node: {}", node.name);
        Ok(node.clone())
    }
    async fn delete_node(&self, id: &Uuid) -> DataStoreResult<()> {
        info!("[dry-run] delete_node: {}", id);
        Ok(())
    }
    async fn get_nodes_by_location(&self, location_id: &Uuid) -> DataStoreResult<Vec<Node>> { self.inner.get_nodes_by_location(location_id).await }
    async fn search_nodes_by_name(&self, name: &str) -> DataStoreResult<Vec<Node>> { self.inner.search_nodes_by_name(name).await }

    // Link ops
    async fn create_link(&self, link: &Link) -> DataStoreResult<Link> {
        info!("[dry-run] create_link: {}", link.name);
        Ok(link.clone())
    }
    async fn get_link(&self, id: &Uuid) -> DataStoreResult<Option<Link>> { self.inner.get_link(id).await }
    async fn list_links(&self, options: &QueryOptions) -> DataStoreResult<PagedResult<Link>> { self.inner.list_links(options).await }
    async fn update_link(&self, link: &Link) -> DataStoreResult<Link> {
        info!("[dry-run] update_link: {}", link.name);
        Ok(link.clone())
    }
    async fn delete_link(&self, id: &Uuid) -> DataStoreResult<()> {
        info!("[dry-run] delete_link: {}", id);
        Ok(())
    }
    async fn get_links_for_node(&self, node_id: &Uuid) -> DataStoreResult<Vec<Link>> { self.inner.get_links_for_node(node_id).await }
    async fn get_links_between_nodes(&self, a: &Uuid, b: &Uuid) -> DataStoreResult<Vec<Link>> { self.inner.get_links_between_nodes(a, b).await }

    // Location ops
    async fn create_location(&self, location: &Location) -> DataStoreResult<Location> {
        info!("[dry-run] create_location: {}", location.name);
        Ok(location.clone())
    }
    async fn get_location(&self, id: &Uuid) -> DataStoreResult<Option<Location>> { self.inner.get_location(id).await }
    async fn list_locations(&self, options: &QueryOptions) -> DataStoreResult<PagedResult<Location>> { self.inner.list_locations(options).await }
    async fn update_location(&self, location: &Location) -> DataStoreResult<Location> {
        info!("[dry-run] update_location: {}", location.name);
        Ok(location.clone())
    }
    async fn delete_location(&self, id: &Uuid) -> DataStoreResult<()> {
        info!("[dry-run] delete_location: {}", id);
        Ok(())
    }

    // Vendors
    async fn create_vendor(&self, name: &str) -> DataStoreResult<()> {
        info!("[dry-run] create_vendor: {}", name);
        Ok(())
    }
    async fn list_vendors(&self) -> DataStoreResult<Vec<String>> { self.inner.list_vendors().await }
    async fn delete_vendor(&self, name: &str) -> DataStoreResult<()> {
        info!("[dry-run] delete_vendor: {}", name);
        Ok(())
    }

    // Batch
    async fn batch_nodes(&self, operations: &[BatchOperation<Node>]) -> DataStoreResult<BatchResult> {
        info!("[dry-run] batch_nodes: {} ops", operations.len());
        Ok(BatchResult { success_count: operations.len(), error_count: 0, errors: vec![] })
    }
    async fn batch_links(&self, operations: &[BatchOperation<Link>]) -> DataStoreResult<BatchResult> {
        info!("[dry-run] batch_links: {} ops", operations.len());
        Ok(BatchResult { success_count: operations.len(), error_count: 0, errors: vec![] })
    }
    async fn batch_locations(&self, operations: &[BatchOperation<Location>]) -> DataStoreResult<BatchResult> {
        info!("[dry-run] batch_locations: {} ops", operations.len());
        Ok(BatchResult { success_count: operations.len(), error_count: 0, errors: vec![] })
    }

    // Stats
    async fn get_entity_counts(&self) -> DataStoreResult<std::collections::HashMap<String, usize>> { self.inner.get_entity_counts().await }
    async fn get_statistics(&self) -> DataStoreResult<std::collections::HashMap<String, serde_json::Value>> { self.inner.get_statistics().await }

    // Derived state
    async fn get_node_status(&self, node_id: &Uuid) -> DataStoreResult<Option<unet_core::models::derived::NodeStatus>> { self.inner.get_node_status(node_id).await }
    async fn get_node_interfaces(&self, node_id: &Uuid) -> DataStoreResult<Vec<unet_core::models::derived::InterfaceStatus>> { self.inner.get_node_interfaces(node_id).await }
    async fn get_node_metrics(&self, node_id: &Uuid) -> DataStoreResult<Option<unet_core::models::derived::PerformanceMetrics>> { self.inner.get_node_metrics(node_id).await }

    // Policy
    async fn store_policy_result(&self, node_id: &Uuid, rule_id: &str, result: &PolicyExecutionResult) -> DataStoreResult<()> {
        info!("[dry-run] store_policy_result: node={} rule={} result={:?}", node_id, rule_id, result);
        Ok(())
    }
    async fn get_policy_results(&self, node_id: &Uuid) -> DataStoreResult<Vec<PolicyExecutionResult>> { self.inner.get_policy_results(node_id).await }
    async fn get_latest_policy_results(&self, node_id: &Uuid) -> DataStoreResult<Vec<PolicyExecutionResult>> { self.inner.get_latest_policy_results(node_id).await }
    async fn get_rule_results(&self, rule_id: &str) -> DataStoreResult<Vec<(Uuid, PolicyExecutionResult)>> { self.inner.get_rule_results(rule_id).await }

    async fn update_node_custom_data(&self, node_id: &Uuid, custom_data: &serde_json::Value) -> DataStoreResult<()> {
        info!("[dry-run] update_node_custom_data: {} -> {}", node_id, custom_data);
        // Optionally verify node exists
        if self.inner.get_node(node_id).await?.is_none() {
            return Err(DataStoreError::NotFound { entity_type: "Node".into(), id: node_id.to_string() });
        }
        Ok(())
    }

    async fn get_nodes_for_policy_evaluation(&self) -> DataStoreResult<Vec<Node>> { self.inner.get_nodes_for_policy_evaluation().await }
}

#[cfg(test)]
mod tests {
    use super::*;
    use unet_core::datastore::MockDataStore;
    use unet_core::models::{DeviceRole, NodeBuilder, Vendor};

    #[tokio::test]
    async fn test_dry_run_create_update_delete_node_ok() {
        let node = NodeBuilder::new()
            .name("n1").domain("example.com").vendor(Vendor::Cisco).model("ISR").role(DeviceRole::Router)
            .build().unwrap();
        let mock = MockDataStore::new();
        let store = DryRunStore::new(Arc::new(mock));
        let created = store.create_node(&node).await.unwrap();
        assert_eq!(created.name, node.name);
        let updated = store.update_node(&node).await.unwrap();
        assert_eq!(updated.name, node.name);
        assert!(store.delete_node(&node.id).await.is_ok());
    }
}
