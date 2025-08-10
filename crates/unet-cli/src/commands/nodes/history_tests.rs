/// Tests for node history functionality
#[cfg(test)]
mod tests {
    use crate::commands::nodes::types::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_history_node_args_creation() {
        let node_id = Uuid::new_v4();

        let args = HistoryNodeArgs {
            id: node_id,
            history_type: HistoryType::Status,
            limit: 10,
            last_hours: None,
            detailed: false,
        };

        assert_eq!(args.id, node_id);
        assert!(matches!(args.history_type, HistoryType::Status));
        assert_eq!(args.limit, 10);
        assert_eq!(args.last_hours, None);
        assert!(!args.detailed);
    }

    #[tokio::test]
    async fn test_history_type_variants() {
        assert!(matches!(HistoryType::Status, HistoryType::Status));
        assert!(matches!(HistoryType::Interfaces, HistoryType::Interfaces));
        assert!(matches!(HistoryType::Metrics, HistoryType::Metrics));
        assert!(matches!(HistoryType::System, HistoryType::System));
        assert!(matches!(HistoryType::All, HistoryType::All));
    }

    #[tokio::test]
    async fn test_history_node_interfaces_type() {
        let node_id = Uuid::new_v4();

        let args = HistoryNodeArgs {
            id: node_id,
            history_type: HistoryType::Interfaces,
            limit: 20,
            last_hours: Some(24),
            detailed: true,
        };

        assert_eq!(args.id, node_id);
        assert!(matches!(args.history_type, HistoryType::Interfaces));
        assert_eq!(args.limit, 20);
        assert_eq!(args.last_hours, Some(24));
        assert!(args.detailed);
    }

    #[tokio::test]
    async fn test_history_node_with_last_hours() {
        let node_id = Uuid::new_v4();

        let args = HistoryNodeArgs {
            id: node_id,
            history_type: HistoryType::Metrics,
            limit: 500,
            last_hours: Some(336), // 2 weeks
            detailed: true,
        };

        assert_eq!(args.id, node_id);
        assert!(matches!(args.history_type, HistoryType::Metrics));
        assert_eq!(args.limit, 500);
        assert_eq!(args.last_hours, Some(336));
        assert!(args.detailed);
    }

    #[tokio::test]
    async fn test_history_type_debug_format() {
        let history_type = HistoryType::System;
        let debug_str = format!("{history_type:?}");
        assert!(debug_str.contains("System"));
    }

    #[tokio::test]
    async fn test_large_values() {
        let node_id = Uuid::new_v4();

        // Test with large limit and last_hours values
        let args = HistoryNodeArgs {
            id: node_id,
            history_type: HistoryType::Interfaces,
            limit: 1_000_000,
            last_hours: Some(8760), // 1 year
            detailed: true,
        };

        assert_eq!(args.limit, 1_000_000);
        assert_eq!(args.last_hours, Some(8760));
    }
}

#[cfg(test)]
mod exec_tests {
    use crate::commands::nodes::history::history_node;
    use crate::commands::nodes::types::{HistoryNodeArgs, HistoryType};
    use mockall::predicate::eq;
    use unet_core::datastore::MockDataStore;
    use unet_core::models::{DeviceRole, NodeBuilder, Vendor};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_history_node_all_variants_execute() {
        // Arrange: a single node setup
        let node_id = Uuid::new_v4();
        let node = NodeBuilder::new()
            .id(node_id)
            .name("node-hist")
            .domain("example.com")
            .vendor(Vendor::Juniper)
            .model("MX")
            .role(DeviceRole::Router)
            .build()
            .unwrap();

        let mut mock = MockDataStore::new();
        mock.expect_get_node_required()
            .with(eq(node_id))
            .returning(move |_| {
                let node = node.clone();
                Box::pin(async move { Ok(node) })
            });

        // Exercise each HistoryType arm
        for history_type in [
            HistoryType::Status,
            HistoryType::Interfaces,
            HistoryType::Metrics,
            HistoryType::System,
            HistoryType::All,
        ] {
            let args = HistoryNodeArgs {
                id: node_id,
                history_type: history_type.clone(),
                limit: 5,
                last_hours: Some(1),
                detailed: true,
            };
            let res = history_node(args, &mock, crate::OutputFormat::Json).await;
            assert!(res.is_ok());
        }
    }
}
