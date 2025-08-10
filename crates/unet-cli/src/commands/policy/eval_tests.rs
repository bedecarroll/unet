/// Execution tests for policy evaluation CLI
#[cfg(test)]
mod tests {
    use crate::commands::policy::EvalPolicyArgs;
    use crate::commands::policy::eval::eval_policy;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use tempfile::{NamedTempFile, TempDir};
    use unet_core::datastore::DataStore;
    use uuid::Uuid;

    #[derive(Clone, Default)]
    struct Store {
        node: Option<unet_core::models::Node>,
        nodes: Vec<unet_core::models::Node>,
    }

    #[async_trait]
    impl DataStore for Store {
        fn name(&self) -> &'static str {
            "store"
        }
        async fn health_check(&self) -> unet_core::datastore::DataStoreResult<()> {
            Ok(())
        }
        async fn begin_transaction(
            &self,
        ) -> unet_core::datastore::DataStoreResult<Box<dyn unet_core::datastore::Transaction>>
        {
            unimplemented!("not needed")
        }
        async fn create_node(
            &self,
            _node: &unet_core::models::Node,
        ) -> unet_core::datastore::DataStoreResult<unet_core::models::Node> {
            unimplemented!("not needed")
        }
        async fn get_node(
            &self,
            _id: &Uuid,
        ) -> unet_core::datastore::DataStoreResult<Option<unet_core::models::Node>> {
            Ok(self.node.clone())
        }
        async fn list_nodes(
            &self,
            _options: &unet_core::datastore::QueryOptions,
        ) -> unet_core::datastore::DataStoreResult<
            unet_core::datastore::types::PagedResult<unet_core::models::Node>,
        > {
            Ok(unet_core::datastore::types::PagedResult::new(
                self.nodes.clone(),
                self.nodes.len(),
                None,
            ))
        }
        async fn update_node(
            &self,
            node: &unet_core::models::Node,
        ) -> unet_core::datastore::DataStoreResult<unet_core::models::Node> {
            Ok(node.clone())
        }
        async fn delete_node(&self, _id: &Uuid) -> unet_core::datastore::DataStoreResult<()> {
            unimplemented!("not needed")
        }
        async fn get_nodes_by_location(
            &self,
            _location_id: &Uuid,
        ) -> unet_core::datastore::DataStoreResult<Vec<unet_core::models::Node>> {
            unimplemented!("not needed")
        }
        async fn search_nodes_by_name(
            &self,
            _name: &str,
        ) -> unet_core::datastore::DataStoreResult<Vec<unet_core::models::Node>> {
            unimplemented!("not needed")
        }
        async fn create_link(
            &self,
            _link: &unet_core::models::Link,
        ) -> unet_core::datastore::DataStoreResult<unet_core::models::Link> {
            unimplemented!("not needed")
        }
        async fn get_link(
            &self,
            _id: &Uuid,
        ) -> unet_core::datastore::DataStoreResult<Option<unet_core::models::Link>> {
            Ok(None)
        }
        async fn list_links(
            &self,
            _options: &unet_core::datastore::QueryOptions,
        ) -> unet_core::datastore::DataStoreResult<
            unet_core::datastore::types::PagedResult<unet_core::models::Link>,
        > {
            unimplemented!("not needed")
        }
        async fn update_link(
            &self,
            _link: &unet_core::models::Link,
        ) -> unet_core::datastore::DataStoreResult<unet_core::models::Link> {
            unimplemented!("not needed")
        }
        async fn delete_link(&self, _id: &Uuid) -> unet_core::datastore::DataStoreResult<()> {
            unimplemented!("not needed")
        }
        async fn get_links_for_node(
            &self,
            _node_id: &Uuid,
        ) -> unet_core::datastore::DataStoreResult<Vec<unet_core::models::Link>> {
            unimplemented!("not needed")
        }
        async fn get_links_between_nodes(
            &self,
            _first_node_id: &Uuid,
            _second_node_id: &Uuid,
        ) -> unet_core::datastore::DataStoreResult<Vec<unet_core::models::Link>> {
            unimplemented!("not needed")
        }
        async fn create_location(
            &self,
            _location: &unet_core::models::Location,
        ) -> unet_core::datastore::DataStoreResult<unet_core::models::Location> {
            unimplemented!("not needed")
        }
        async fn get_location(
            &self,
            _id: &Uuid,
        ) -> unet_core::datastore::DataStoreResult<Option<unet_core::models::Location>> {
            Ok(None)
        }
        async fn list_locations(
            &self,
            _options: &unet_core::datastore::QueryOptions,
        ) -> unet_core::datastore::DataStoreResult<
            unet_core::datastore::types::PagedResult<unet_core::models::Location>,
        > {
            unimplemented!("not needed")
        }
        async fn update_location(
            &self,
            _location: &unet_core::models::Location,
        ) -> unet_core::datastore::DataStoreResult<unet_core::models::Location> {
            unimplemented!("not needed")
        }
        async fn delete_location(&self, _id: &Uuid) -> unet_core::datastore::DataStoreResult<()> {
            unimplemented!("not needed")
        }
        async fn create_vendor(&self, _name: &str) -> unet_core::datastore::DataStoreResult<()> {
            unimplemented!("not needed")
        }
        async fn list_vendors(&self) -> unet_core::datastore::DataStoreResult<Vec<String>> {
            unimplemented!("not needed")
        }
        async fn delete_vendor(&self, _name: &str) -> unet_core::datastore::DataStoreResult<()> {
            unimplemented!("not needed")
        }
        async fn batch_nodes(
            &self,
            _operations: &[unet_core::datastore::BatchOperation<unet_core::models::Node>],
        ) -> unet_core::datastore::DataStoreResult<unet_core::datastore::types::BatchResult>
        {
            unimplemented!("not needed")
        }
        async fn batch_links(
            &self,
            _operations: &[unet_core::datastore::BatchOperation<unet_core::models::Link>],
        ) -> unet_core::datastore::DataStoreResult<unet_core::datastore::types::BatchResult>
        {
            unimplemented!("not needed")
        }
        async fn batch_locations(
            &self,
            _operations: &[unet_core::datastore::BatchOperation<unet_core::models::Location>],
        ) -> unet_core::datastore::DataStoreResult<unet_core::datastore::types::BatchResult>
        {
            unimplemented!("not needed")
        }
        async fn get_entity_counts(
            &self,
        ) -> unet_core::datastore::DataStoreResult<HashMap<String, usize>> {
            unimplemented!("not needed")
        }
        async fn get_statistics(
            &self,
        ) -> unet_core::datastore::DataStoreResult<HashMap<String, serde_json::Value>> {
            unimplemented!("not needed")
        }
    }

    fn make_node() -> unet_core::models::Node {
        use unet_core::models::*;
        let id = Uuid::new_v4();
        NodeBuilder::new()
            .id(id)
            .name("edge-1")
            .domain("example.com")
            .vendor(Vendor::Cisco)
            .model("ISR4321")
            .role(DeviceRole::Router)
            .lifecycle(Lifecycle::Live)
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn test_eval_policy_with_single_node_and_simple_rule() {
        use std::io::Write;
        let node = make_node();
        let store = Store {
            node: Some(node.clone()),
            nodes: vec![],
        };

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            "WHEN node.vendor == \"cisco\" THEN ASSERT node.model IS \"ISR4321\""
        )
        .unwrap();

        let args = EvalPolicyArgs {
            path: temp_file.path().to_path_buf(),
            node_id: Some(node.id),
            verbose: true,
            failures_only: false,
        };
        let result = eval_policy(args, &store).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_eval_policy_all_nodes_from_dir() {
        let node = make_node();
        let store = Store {
            node: None,
            nodes: vec![node.clone()],
        };
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("pol.policy");
        std::fs::write(
            &file,
            "WHEN node.name == \"edge-1\" THEN ASSERT node.vendor IS \"cisco\"",
        )
        .unwrap();

        let args = EvalPolicyArgs {
            path: dir.path().to_path_buf(),
            node_id: None,
            verbose: false,
            failures_only: true,
        };
        let result = eval_policy(args, &store).await;
        assert!(result.is_ok());
    }
}
