/// Tests for node show functionality
#[cfg(test)]
mod tests {
    use crate::commands::nodes::types::ShowNodeArgs;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_show_node_args_structure() {
        let node_id = Uuid::new_v4();
        let args = ShowNodeArgs {
            id: node_id,
            include_status: true,
            show_interfaces: true,
            show_system_info: true,
        };

        assert_eq!(args.id, node_id);
        assert!(args.include_status);
        assert!(args.show_interfaces);
        assert!(args.show_system_info);
    }

    #[tokio::test]
    async fn test_show_node_args_all_false() {
        let node_id = Uuid::new_v4();
        let args = ShowNodeArgs {
            id: node_id,
            include_status: false,
            show_interfaces: false,
            show_system_info: false,
        };

        assert_eq!(args.id, node_id);
        assert!(!args.include_status);
        assert!(!args.show_interfaces);
        assert!(!args.show_system_info);
    }

    #[tokio::test]
    async fn test_show_node_mixed_flags() {
        let node_id = Uuid::new_v4();

        let args1 = ShowNodeArgs {
            id: node_id,
            include_status: true,
            show_interfaces: true,
            show_system_info: false,
        };
        assert_eq!(args1.id, node_id);
        assert!(args1.include_status);
        assert!(args1.show_interfaces);
        assert!(!args1.show_system_info);

        let args2 = ShowNodeArgs {
            id: node_id,
            include_status: true,
            show_interfaces: false,
            show_system_info: true,
        };
        assert_eq!(args2.id, node_id);
        assert!(args2.include_status);
        assert!(!args2.show_interfaces);
        assert!(args2.show_system_info);

        let args3 = ShowNodeArgs {
            id: node_id,
            include_status: false,
            show_interfaces: true,
            show_system_info: true,
        };
        assert_eq!(args3.id, node_id);
        assert!(!args3.include_status);
        assert!(args3.show_interfaces);
        assert!(args3.show_system_info);
    }
}

#[cfg(test)]
mod exec_tests {
    use super::super::show::show_node;
    use super::super::types::ShowNodeArgs;
    use unet_core::datastore::{
        MockDataStore,
        testing::{ready_err, ready_ok},
        types::DataStoreError,
    };
    use uuid::Uuid;

    fn store_for_show(node: unet_core::models::Node, fail_status: bool) -> MockDataStore {
        let node_for_lookup = node.clone();
        let node_for_status = node.clone();
        let mut store = MockDataStore::new();
        store
            .expect_get_node_required()
            .returning(move |_| ready_ok(node_for_lookup.clone()));

        if fail_status {
            store.expect_get_node_status().returning(|_| {
                ready_err(DataStoreError::InternalError {
                    message: "status failed".to_string(),
                })
            });
        } else {
            store.expect_get_node_status().returning(move |_| {
                ready_ok(Some(unet_core::models::derived::NodeStatus::new(
                    node_for_status.id,
                )))
            });
        }

        store
            .expect_get_node_interfaces()
            .returning(|_| ready_ok(Vec::<unet_core::models::derived::InterfaceStatus>::new()));
        store
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
    async fn test_show_node_basic_exec() {
        let node = make_node();
        let store = store_for_show(node.clone(), false);
        let args = ShowNodeArgs {
            id: node.id,
            include_status: false,
            show_interfaces: false,
            show_system_info: false,
        };
        let result = show_node(args, &store, crate::OutputFormat::Json).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_show_node_with_status_and_interfaces_exec() {
        let node = make_node();
        let store = store_for_show(node.clone(), false);
        let args = ShowNodeArgs {
            id: node.id,
            include_status: true,
            show_interfaces: true,
            show_system_info: true,
        };
        let result = show_node(args, &store, crate::OutputFormat::Json).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_show_node_status_error_branch_exec() {
        let node = make_node();
        let store = store_for_show(node.clone(), true);
        let args = ShowNodeArgs {
            id: node.id,
            include_status: true,
            show_interfaces: false,
            show_system_info: false,
        };
        let result = show_node(args, &store, crate::OutputFormat::Json).await;
        assert!(result.is_ok());
    }
}
