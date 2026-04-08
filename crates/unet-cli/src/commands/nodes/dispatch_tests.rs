#[cfg(test)]
mod tests {
    use crate::commands::nodes::{execute, types::*};
    use unet_core::datastore::{MockDataStore, testing::ready_ok};
    use unet_core::models::{DeviceRole, Node, NodeBuilder, Vendor};
    use uuid::Uuid;

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

    fn store(node: Node) -> MockDataStore {
        let node_for_get = node.clone();
        let node_for_update = node.clone();
        let node_for_status = node.clone();

        let mut store = MockDataStore::new();
        store
            .expect_create_node()
            .returning(|node| ready_ok(node.clone()));
        store
            .expect_get_node_required()
            .returning(move |_| ready_ok(node_for_get.clone()));
        store.expect_list_nodes().returning(|_| {
            ready_ok(unet_core::datastore::types::PagedResult::new(
                vec![],
                0,
                None,
            ))
        });
        store
            .expect_update_node()
            .returning(move |_| ready_ok(node_for_update.clone()));
        store.expect_delete_node().returning(|_| ready_ok(()));
        store.expect_get_node_status().returning(move |_| {
            ready_ok(Some(unet_core::models::derived::NodeStatus::new(
                node_for_status.id,
            )))
        });
        store
            .expect_get_node_polling_task()
            .returning(|_| ready_ok(None));
        store
            .expect_get_node_interfaces()
            .returning(|_| ready_ok(Vec::<unet_core::models::derived::InterfaceStatus>::new()));
        store
            .expect_get_node_metrics()
            .returning(|_| ready_ok(None));
        store
    }

    #[tokio::test]
    async fn test_dispatch_add_and_list() {
        let node = make_node();
        let datastore = store(node);
        let add = AddNodeArgs {
            name: "n1".into(),
            domain: "example.com".into(),
            vendor: "cisco".into(),
            model: "ISR".into(),
            role: "router".into(),
            lifecycle: "planned".into(),
            location_id: None,
            management_ip: None,
            custom_data: None,
        };
        assert!(
            execute(
                NodeCommands::Add(add),
                &datastore,
                crate::OutputFormat::Json
            )
            .await
            .is_ok()
        );

        let list = ListNodeArgs {
            lifecycle: None,
            role: None,
            vendor: None,
            page: 1,
            per_page: 20,
        };
        assert!(
            execute(
                NodeCommands::List(list),
                &datastore,
                crate::OutputFormat::Json
            )
            .await
            .is_ok()
        );
    }

    #[tokio::test]
    async fn test_dispatch_status_and_polling() {
        let node = make_node();
        let datastore = store(node.clone());
        let status = StatusNodeArgs {
            id: node.id,
            status_type: vec![StatusType::Basic],
        };
        assert!(
            execute(
                NodeCommands::Status(status),
                &datastore,
                crate::OutputFormat::Json
            )
            .await
            .is_ok()
        );

        let poll = PollingNodeArgs {
            id: node.id,
            action: PollingAction::Status,
            detailed: false,
        };
        assert!(
            execute(
                NodeCommands::Polling(poll),
                &datastore,
                crate::OutputFormat::Json
            )
            .await
            .is_ok()
        );

        let metrics = MetricsNodeArgs {
            id: node.id,
            detailed: false,
            history: false,
        };
        assert!(
            execute(
                NodeCommands::Metrics(metrics),
                &datastore,
                crate::OutputFormat::Json
            )
            .await
            .is_ok()
        );
    }

    #[tokio::test]
    async fn test_dispatch_compare_and_delete() {
        let node = make_node();
        let datastore = store(node.clone());
        let compare = CompareNodeArgs {
            node_a: node.id,
            node_b: None,
            compare_type: vec![CompareType::All],
            diff_only: false,
        };
        assert!(
            execute(
                NodeCommands::Compare(compare),
                &datastore,
                crate::OutputFormat::Json,
            )
            .await
            .is_ok()
        );

        let delete = DeleteNodeArgs {
            id: node.id,
            yes: true,
        };
        assert!(
            execute(
                NodeCommands::Delete(delete),
                &datastore,
                crate::OutputFormat::Json,
            )
            .await
            .is_ok()
        );

        let history = HistoryNodeArgs {
            id: node.id,
            history_type: HistoryType::Status,
            limit: 10,
            last_hours: None,
            detailed: false,
        };
        assert!(
            execute(
                NodeCommands::History(history),
                &datastore,
                crate::OutputFormat::Json,
            )
            .await
            .is_ok()
        );
    }

    #[tokio::test]
    async fn test_dispatch_update_and_show() {
        let node = make_node();
        let datastore = store(node.clone());
        let update = UpdateNodeArgs {
            id: node.id,
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
        assert!(
            execute(
                NodeCommands::Update(update),
                &datastore,
                crate::OutputFormat::Json,
            )
            .await
            .is_ok()
        );

        let show = ShowNodeArgs {
            id: node.id,
            include_status: false,
            show_interfaces: false,
            show_system_info: false,
        };
        assert!(
            execute(
                NodeCommands::Show(show),
                &datastore,
                crate::OutputFormat::Json
            )
            .await
            .is_ok()
        );
    }
}
