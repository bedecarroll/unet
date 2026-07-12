/// Execution tests for policy evaluation CLI
#[cfg(test)]
mod tests {
    use crate::commands::policy::EvalPolicyArgs;
    use crate::commands::policy::eval::eval_policy;
    use tempfile::{NamedTempFile, TempDir};
    use unet_core::datastore::{MockDataStore, testing::ready_ok};
    use uuid::Uuid;

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
        let mut store = MockDataStore::new();
        store
            .expect_get_node()
            .returning(move |_| ready_ok(Some(node.clone())));

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            "WHEN node.vendor == \"cisco\" THEN ASSERT node.model IS \"ISR4321\""
        )
        .unwrap();

        let args = EvalPolicyArgs {
            path: temp_file.path().to_path_buf(),
            node_id: Some(Uuid::new_v4()),
            verbose: true,
            failures_only: false,
        };
        let result = eval_policy(args, &store).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_eval_policy_all_nodes_from_dir() {
        let node = make_node();
        let mut store = MockDataStore::new();
        store.expect_list_nodes().returning(move |_| {
            ready_ok(unet_core::datastore::types::PagedResult::new(
                vec![node.clone()],
                1,
                None,
            ))
        });

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
