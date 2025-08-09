#[cfg(test)]
mod tests {
    use crate::commands::policy::{execute, DiffPolicyArgs, EvalPolicyArgs, ListPolicyArgs, PolicyCommands, ShowPolicyArgs, ValidatePolicyArgs};
    use mockall::predicate::eq;
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};
    use unet_core::datastore::{types::PagedResult, MockDataStore};
    use unet_core::models::{DeviceRole, NodeBuilder, Vendor};
    use uuid::Uuid;

    fn make_node() -> unet_core::models::Node {
        NodeBuilder::new()
            .id(Uuid::new_v4())
            .name("pol-node")
            .domain("example.com")
            .vendor(Vendor::Cisco)
            .model("CSR")
            .role(DeviceRole::Router)
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn test_execute_validate() {
        let mut tf = NamedTempFile::new().unwrap();
        writeln!(
            tf,
            "WHEN node.vendor == \"cisco\" THEN ASSERT node.version IS \"15.1\""
        )
        .unwrap();

        let args = ValidatePolicyArgs {
            path: tf.path().into(),
            verbose: true,
        };

        let mock = MockDataStore::new();
        let res = execute(PolicyCommands::Validate(args), &mock).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_execute_eval_no_nodes() {
        let mut tf = NamedTempFile::new().unwrap();
        writeln!(tf, "WHEN node.vendor == \"cisco\" THEN ASSERT node.version IS \"15.1\"").unwrap();

        let args = EvalPolicyArgs {
            path: tf.path().into(),
            node_id: None,
            verbose: false,
            failures_only: false,
        };

        let mut mock = MockDataStore::new();
        mock.expect_list_nodes()
            .returning(|_| Box::pin(async { Ok(PagedResult::new(vec![], 0, None)) }));

        let res = execute(PolicyCommands::Eval(args), &mock).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_execute_diff_with_node() {
        let mut tf = NamedTempFile::new().unwrap();
        writeln!(tf, "WHEN node.vendor == \"cisco\" THEN ASSERT node.version IS \"15.1\"").unwrap();

        let node = make_node();
        let node_id = node.id;

        let args = DiffPolicyArgs {
            path: tf.path().into(),
            node_id,
            verbose: true,
        };

        let mut mock = MockDataStore::new();
        let node_for_get = node.clone();
        mock.expect_get_node()
            .with(eq(node_id))
            .returning(move |_| {
                let n = node_for_get.clone();
                Box::pin(async move { Ok(Some(n)) })
            });

        let res = execute(PolicyCommands::Diff(args), &mock).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_execute_list() {
        let dir = TempDir::new().unwrap();
        let policy_file = dir.path().join("valid.policy");
        std::fs::write(&policy_file, "WHEN TRUE THEN ASSERT node.role IS \"router\"").unwrap();

        let args = ListPolicyArgs {
            path: dir.path().into(),
            verbose: true,
        };

        let mock = MockDataStore::new();
        let res = execute(PolicyCommands::List(args), &mock).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_execute_show() {
        let mut tf = NamedTempFile::new().unwrap();
        writeln!(tf, "WHEN node.vendor == \"cisco\" THEN ASSERT node.version IS \"15.1\"").unwrap();

        let args = ShowPolicyArgs {
            path: tf.path().into(),
            ast: true,
        };

        let mock = MockDataStore::new();
        let res = execute(PolicyCommands::Show(args), &mock).await;
        assert!(res.is_ok());
    }
}
