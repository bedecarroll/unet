#[cfg(test)]
mod tests {
    use crate::commands::nodes::{execute, types::*};
    use async_trait::async_trait;
    use std::collections::HashMap;
    use unet_core::datastore::{types::PagedResult, DataStore};
    use unet_core::models::derived::{InterfaceStatus, NodeStatus, PerformanceMetrics};
    use unet_core::models::{DeviceRole, Node, NodeBuilder, Vendor};
    use uuid::Uuid;

    #[derive(Clone)]
    struct FakeStore {
        node: Node,
    }

    #[async_trait]
    impl DataStore for FakeStore {
        fn name(&self) -> &'static str { "fake" }
        async fn health_check(&self) -> unet_core::datastore::DataStoreResult<()> { Ok(()) }
        async fn begin_transaction(&self) -> unet_core::datastore::DataStoreResult<Box<dyn unet_core::datastore::Transaction>> { unimplemented!("not needed") }

        // Node ops used by tests
        async fn create_node(&self, node: &Node) -> unet_core::datastore::DataStoreResult<Node> { Ok(node.clone()) }
        async fn get_node(&self, _id: &Uuid) -> unet_core::datastore::DataStoreResult<Option<Node>> { Ok(Some(self.node.clone())) }
        async fn list_nodes(&self, _o: &unet_core::datastore::QueryOptions) -> unet_core::datastore::DataStoreResult<PagedResult<Node>> { Ok(PagedResult::new(vec![], 0, None)) }
        async fn update_node(&self, node: &Node) -> unet_core::datastore::DataStoreResult<Node> { Ok(node.clone()) }
        async fn delete_node(&self, _id: &Uuid) -> unet_core::datastore::DataStoreResult<()> { Ok(()) }

        // Unused ops
        async fn get_nodes_by_location(&self, _l: &Uuid) -> unet_core::datastore::DataStoreResult<Vec<Node>> { Ok(vec![]) }
        async fn search_nodes_by_name(&self, _n: &str) -> unet_core::datastore::DataStoreResult<Vec<Node>> { Ok(vec![]) }
        async fn create_link(&self, _l: &unet_core::models::Link) -> unet_core::datastore::DataStoreResult<unet_core::models::Link> { unimplemented!("not needed") }
        async fn get_link(&self, _i: &Uuid) -> unet_core::datastore::DataStoreResult<Option<unet_core::models::Link>> { Ok(None) }
        async fn list_links(&self, _o: &unet_core::datastore::QueryOptions) -> unet_core::datastore::DataStoreResult<PagedResult<unet_core::models::Link>> { Ok(PagedResult::new(vec![], 0, None)) }
        async fn update_link(&self, _l: &unet_core::models::Link) -> unet_core::datastore::DataStoreResult<unet_core::models::Link> { unimplemented!("not needed") }
        async fn delete_link(&self, _i: &Uuid) -> unet_core::datastore::DataStoreResult<()> { Ok(()) }
        async fn get_links_for_node(&self, _n: &Uuid) -> unet_core::datastore::DataStoreResult<Vec<unet_core::models::Link>> { Ok(vec![]) }
        async fn get_links_between_nodes(&self, _a: &Uuid, _b: &Uuid) -> unet_core::datastore::DataStoreResult<Vec<unet_core::models::Link>> { Ok(vec![]) }
        async fn create_location(&self, _l: &unet_core::models::Location) -> unet_core::datastore::DataStoreResult<unet_core::models::Location> { unimplemented!("not needed") }
        async fn get_location(&self, _i: &Uuid) -> unet_core::datastore::DataStoreResult<Option<unet_core::models::Location>> { Ok(None) }
        async fn list_locations(&self, _o: &unet_core::datastore::QueryOptions) -> unet_core::datastore::DataStoreResult<PagedResult<unet_core::models::Location>> { Ok(PagedResult::new(vec![], 0, None)) }
        async fn update_location(&self, _l: &unet_core::models::Location) -> unet_core::datastore::DataStoreResult<unet_core::models::Location> { unimplemented!("not needed") }
        async fn delete_location(&self, _i: &Uuid) -> unet_core::datastore::DataStoreResult<()> { Ok(()) }
        async fn create_vendor(&self, _name: &str) -> unet_core::datastore::DataStoreResult<()> { Ok(()) }
        async fn list_vendors(&self) -> unet_core::datastore::DataStoreResult<Vec<String>> { Ok(vec![]) }
        async fn delete_vendor(&self, _n: &str) -> unet_core::datastore::DataStoreResult<()> { Ok(()) }
        async fn batch_nodes(&self, _ops: &[unet_core::datastore::BatchOperation<Node>]) -> unet_core::datastore::DataStoreResult<unet_core::datastore::BatchResult> { Ok(unet_core::datastore::BatchResult { success_count: 0, error_count: 0, errors: vec![] }) }
        async fn batch_links(&self, _ops: &[unet_core::datastore::BatchOperation<unet_core::models::Link>]) -> unet_core::datastore::DataStoreResult<unet_core::datastore::BatchResult> { Ok(unet_core::datastore::BatchResult { success_count: 0, error_count: 0, errors: vec![] }) }
        async fn batch_locations(&self, _ops: &[unet_core::datastore::BatchOperation<unet_core::models::Location>]) -> unet_core::datastore::DataStoreResult<unet_core::datastore::BatchResult> { Ok(unet_core::datastore::BatchResult { success_count: 0, error_count: 0, errors: vec![] }) }
        async fn get_entity_counts(&self) -> unet_core::datastore::DataStoreResult<HashMap<String, usize>> { Ok(HashMap::new()) }
        async fn get_statistics(&self) -> unet_core::datastore::DataStoreResult<HashMap<String, serde_json::Value>> { Ok(HashMap::new()) }
        async fn get_node_status(&self, node_id: &Uuid) -> unet_core::datastore::DataStoreResult<Option<NodeStatus>> { Ok(Some(NodeStatus::new(*node_id))) }
        async fn get_node_interfaces(&self, _node_id: &Uuid) -> unet_core::datastore::DataStoreResult<Vec<InterfaceStatus>> { Ok(vec![]) }
        async fn get_node_metrics(&self, _node_id: &Uuid) -> unet_core::datastore::DataStoreResult<Option<PerformanceMetrics>> { Ok(None) }
        async fn store_policy_result(&self, _n: &Uuid, _r: &str, _res: &unet_core::policy::PolicyExecutionResult) -> unet_core::datastore::DataStoreResult<()> { Ok(()) }
        async fn get_policy_results(&self, _n: &Uuid) -> unet_core::datastore::DataStoreResult<Vec<unet_core::policy::PolicyExecutionResult>> { Ok(vec![]) }
        async fn get_latest_policy_results(&self, _n: &Uuid) -> unet_core::datastore::DataStoreResult<Vec<unet_core::policy::PolicyExecutionResult>> { Ok(vec![]) }
        async fn get_rule_results(&self, _r: &str) -> unet_core::datastore::DataStoreResult<Vec<(Uuid, unet_core::policy::PolicyExecutionResult)>> { Ok(vec![]) }
        async fn update_node_custom_data(&self, _n: &Uuid, _v: &serde_json::Value) -> unet_core::datastore::DataStoreResult<()> { Ok(()) }
        async fn get_nodes_for_policy_evaluation(&self) -> unet_core::datastore::DataStoreResult<Vec<Node>> { Ok(vec![self.node.clone()]) }
    }

    fn make_node() -> Node {
        let id = Uuid::new_v4();
        NodeBuilder::new()
            .id(id)
            .name("edge-1")
            .domain("example.com")
            .vendor(Vendor::Cisco)
            .model("ISR4321")
            .role(DeviceRole::Router)
            .build()
            .unwrap()
    }

    fn store() -> FakeStore { FakeStore { node: make_node() } }

    #[tokio::test]
    async fn test_dispatch_add_and_list() {
        let ds = store();
        let add = AddNodeArgs { name: "n1".into(), domain: "example.com".into(), vendor: "cisco".into(), model: "ISR".into(), role: "router".into(), lifecycle: "planned".into(), location_id: None, management_ip: None, custom_data: None };
        assert!(execute(NodeCommands::Add(add), &ds, crate::OutputFormat::Json).await.is_ok());

        let list = ListNodeArgs { lifecycle: None, role: None, vendor: None, page: 1, per_page: 20 };
        assert!(execute(NodeCommands::List(list), &ds, crate::OutputFormat::Json).await.is_ok());
    }

    #[tokio::test]
    async fn test_dispatch_status_and_polling() {
        let ds = store();
        let id = ds.node.id;
        let status = StatusNodeArgs { id, status_type: vec![StatusType::Basic] };
        assert!(execute(NodeCommands::Status(status), &ds, crate::OutputFormat::Json).await.is_ok());

        let poll = PollingNodeArgs { id, action: PollingAction::Status, detailed: false };
        assert!(execute(NodeCommands::Polling(poll), &ds, crate::OutputFormat::Json).await.is_ok());

        // Also exercise Metrics mapping
        let metrics = MetricsNodeArgs { id, detailed: false, history: false };
        assert!(execute(NodeCommands::Metrics(metrics), &ds, crate::OutputFormat::Json).await.is_ok());
    }

    #[tokio::test]
    async fn test_dispatch_compare_and_delete() {
        let ds = store();
        let id = ds.node.id;
        let cmp = CompareNodeArgs { node_a: id, node_b: None, compare_type: vec![CompareType::All], diff_only: false };
        assert!(execute(NodeCommands::Compare(cmp), &ds, crate::OutputFormat::Json).await.is_ok());

        let del = DeleteNodeArgs { id, yes: true };
        assert!(execute(NodeCommands::Delete(del), &ds, crate::OutputFormat::Json).await.is_ok());

        // Exercise History mapping
        let hist = HistoryNodeArgs { id, history_type: HistoryType::Status, limit: 10, last_hours: None, detailed: false };
        assert!(execute(NodeCommands::History(hist), &ds, crate::OutputFormat::Json).await.is_ok());
    }

    #[tokio::test]
    async fn test_dispatch_update_and_show() {
        let ds = store();
        let id = ds.node.id;
        let upd = UpdateNodeArgs {
            id,
            name: Some("edge-1a".into()),
            domain: Some("example.com".into()),
            vendor: Some("cisco".into()),
            model: Some("ISR4321".into()),
            role: Some("router".into()),
            lifecycle: Some("live".into()),
            location_id: None,
            management_ip: Some("192.0.2.1".into()),
            custom_data: Some("{}".into()),
        };
        assert!(execute(NodeCommands::Update(upd), &ds, crate::OutputFormat::Json).await.is_ok());

        let show = ShowNodeArgs { id, include_status: false, show_interfaces: false, show_system_info: false };
        assert!(execute(NodeCommands::Show(show), &ds, crate::OutputFormat::Json).await.is_ok());
    }
}
