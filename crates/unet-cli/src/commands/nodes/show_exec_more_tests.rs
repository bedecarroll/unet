#[cfg(test)]
mod tests {
    use crate::commands::nodes::show::show_node;
    use crate::commands::nodes::types::ShowNodeArgs;
    use mockall::predicate::eq;
    use unet_core::datastore::{DataStoreError, MockDataStore};
    use unet_core::models::derived::NodeStatus;
    use unet_core::models::{DeviceRole, NodeBuilder, Vendor};
    use uuid::Uuid;

    fn make_node() -> unet_core::models::Node {
        NodeBuilder::new()
            .id(Uuid::new_v4())
            .name("show-1")
            .domain("example.com")
            .vendor(Vendor::Cisco)
            .model("ISR4321")
            .role(DeviceRole::Router)
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn test_show_node_system_info_none_branch() {
        let node = make_node();
        let id = node.id;
        let mut mock = MockDataStore::new();
        mock.expect_get_node_required()
            .with(eq(id))
            .returning(move |_| {
                let n = node.clone();
                Box::pin(async move { Ok(n) })
            });
        // include_status: Ok(None)
        mock.expect_get_node_status()
            .with(eq(id))
            .returning(move |_| Box::pin(async move { Ok(None) }));

        let args = ShowNodeArgs {
            id,
            include_status: true,
            show_interfaces: false,
            show_system_info: true,
        };
        let res = show_node(args, &mock, crate::OutputFormat::Json).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_show_node_interfaces_error_branch() {
        let node = make_node();
        let id = node.id;
        let mut mock = MockDataStore::new();
        mock.expect_get_node_required()
            .with(eq(id))
            .returning(move |_| {
                let n = node.clone();
                Box::pin(async move { Ok(n) })
            });
        mock.expect_get_node_interfaces()
            .with(eq(id))
            .returning(move |_| {
                Box::pin(async move {
                    Err(DataStoreError::InternalError {
                        message: "iface error".into(),
                    })
                })
            });
        // also set get_node_status for completeness when include_status true
        mock.expect_get_node_status()
            .returning(move |_| Box::pin(async move { Ok(Some(NodeStatus::new(id))) }));

        let args = ShowNodeArgs {
            id,
            include_status: true,
            show_interfaces: true,
            show_system_info: false,
        };
        let res = show_node(args, &mock, crate::OutputFormat::Yaml).await;
        assert!(res.is_ok());
    }
}
