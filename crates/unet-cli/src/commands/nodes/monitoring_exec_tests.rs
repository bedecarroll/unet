//! Exec-style tests for monitoring and metrics commands

#[cfg(test)]
mod tests {
    use crate::commands::nodes::monitoring::{metrics_node, status_node};
    use crate::commands::nodes::types::{MetricsNodeArgs, StatusNodeArgs, StatusType};
    use mockall::predicate::eq;
    use unet_core::datastore::{DataStoreError, MockDataStore};
    use unet_core::models::derived::{
        InterfaceAdminStatus, InterfaceOperStatus, InterfaceStats, InterfaceStatus, NodeStatus,
        PerformanceMetrics,
    };
    use unet_core::models::{DeviceRole, NodeBuilder, Vendor};
    use uuid::Uuid;

    fn make_node() -> unet_core::models::Node {
        NodeBuilder::new()
            .id(Uuid::new_v4())
            .name("monitor-1")
            .domain("example.com")
            .vendor(Vendor::Cisco)
            .model("CSR1000V")
            .role(DeviceRole::Router)
            .build()
            .unwrap()
    }

    fn sample_status(node_id: Uuid) -> NodeStatus {
        let mut status = NodeStatus::new(node_id);
        status.reachable = true;
        status
    }

    fn sample_interfaces() -> Vec<InterfaceStatus> {
        vec![InterfaceStatus {
            index: 1,
            name: "Gig0/0".to_string(),
            interface_type: 6,
            mtu: Some(1500),
            speed: Some(1_000_000_000),
            physical_address: None,
            admin_status: InterfaceAdminStatus::Up,
            oper_status: InterfaceOperStatus::Up,
            last_change: None,
            input_stats: InterfaceStats::default(),
            output_stats: InterfaceStats::default(),
        }]
    }

    #[tokio::test]
    async fn test_status_basic_ok() {
        let node = make_node();
        let id = node.id;
        let mut mock = MockDataStore::new();
        mock.expect_get_node_required()
            .with(eq(id))
            .returning(move |_| {
                let n = node.clone();
                Box::pin(async move { Ok(n) })
            });

        let args = StatusNodeArgs {
            id,
            status_type: vec![StatusType::Basic],
        };
        let res = status_node(args, &mock, crate::OutputFormat::Json).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_status_all_ok_with_data() {
        let node = make_node();
        let id = node.id;
        let status = sample_status(id);
        let ifaces = sample_interfaces();

        let mut mock = MockDataStore::new();
        mock.expect_get_node_required()
            .with(eq(id))
            .returning(move |_| {
                let n = node.clone();
                Box::pin(async move { Ok(n) })
            });
        mock.expect_get_node_status()
            .with(eq(id))
            .returning(move |_| {
                let s = status.clone();
                Box::pin(async move { Ok(Some(s)) })
            });
        mock.expect_get_node_interfaces()
            .with(eq(id))
            .returning(move |_| {
                let v = ifaces.clone();
                Box::pin(async move { Ok(v) })
            });

        let args = StatusNodeArgs {
            id,
            status_type: vec![StatusType::All],
        };
        let res = status_node(args, &mock, crate::OutputFormat::Yaml).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_status_interfaces_error_branch() {
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
                        message: "ifaces fail".to_string(),
                    })
                })
            });

        let args = StatusNodeArgs {
            id,
            status_type: vec![StatusType::Interfaces],
        };
        let res = status_node(args, &mock, crate::OutputFormat::Table).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_status_system_none_and_error() {
        let node = make_node();
        let id = node.id;
        let mut mock_none = MockDataStore::new();
        mock_none
            .expect_get_node_required()
            .with(eq(id))
            .returning(move |_| {
                let n = node.clone();
                Box::pin(async move { Ok(n) })
            });
        mock_none
            .expect_get_node_status()
            .with(eq(id))
            .returning(move |_| Box::pin(async move { Ok(None) }));
        let args = StatusNodeArgs {
            id,
            status_type: vec![StatusType::System],
        };
        assert!(
            status_node(args, &mock_none, crate::OutputFormat::Json)
                .await
                .is_ok()
        );

        // Error variant
        let node2 = make_node();
        let id2 = node2.id;
        let mut mock_err = MockDataStore::new();
        mock_err
            .expect_get_node_required()
            .with(eq(id2))
            .returning(move |_| {
                let n = node2.clone();
                Box::pin(async move { Ok(n) })
            });
        mock_err
            .expect_get_node_status()
            .with(eq(id2))
            .returning(move |_| {
                Box::pin(async move {
                    Err(DataStoreError::InternalError {
                        message: "status err".into(),
                    })
                })
            });
        let args2 = StatusNodeArgs {
            id: id2,
            status_type: vec![StatusType::System],
        };
        assert!(
            status_node(args2, &mock_err, crate::OutputFormat::Json)
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn test_metrics_history_branch() {
        let node = make_node();
        let id = node.id;
        let mut mock = MockDataStore::new();
        mock.expect_get_node_required()
            .with(eq(id))
            .returning(move |_| {
                let n = node.clone();
                Box::pin(async move { Ok(n) })
            });
        // history=true should not require metrics calls
        let args = MetricsNodeArgs {
            id,
            detailed: false,
            history: true,
        };
        assert!(
            metrics_node(args, &mock, crate::OutputFormat::Json)
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn test_metrics_current_ok_detailed() {
        let node = make_node();
        let id = node.id;
        let mut mock = MockDataStore::new();
        mock.expect_get_node_required()
            .with(eq(id))
            .returning(move |_| {
                let n = node.clone();
                Box::pin(async move { Ok(n) })
            });
        mock.expect_get_node_metrics()
            .with(eq(id))
            .returning(move |_| {
                Box::pin(async move {
                    Ok(Some(PerformanceMetrics {
                        cpu_utilization: Some(10),
                        memory_utilization: Some(20),
                        total_memory: None,
                        used_memory: None,
                        load_average: None,
                    }))
                })
            });
        mock.expect_get_node_interfaces()
            .with(eq(id))
            .returning(move |_| Box::pin(async move { Ok(sample_interfaces()) }));

        let args = MetricsNodeArgs {
            id,
            detailed: true,
            history: false,
        };
        assert!(
            metrics_node(args, &mock, crate::OutputFormat::Yaml)
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn test_metrics_current_none_and_error() {
        let node = make_node();
        let id = node.id;
        // None case
        let mut mock_none = MockDataStore::new();
        mock_none
            .expect_get_node_required()
            .with(eq(id))
            .returning(move |_| {
                let n = node.clone();
                Box::pin(async move { Ok(n) })
            });
        mock_none
            .expect_get_node_metrics()
            .with(eq(id))
            .returning(move |_| Box::pin(async move { Ok(None) }));
        let args = MetricsNodeArgs {
            id,
            detailed: false,
            history: false,
        };
        assert!(
            metrics_node(args, &mock_none, crate::OutputFormat::Json)
                .await
                .is_ok()
        );

        // Error case
        let node2 = make_node();
        let id2 = node2.id;
        let mut mock_err = MockDataStore::new();
        mock_err
            .expect_get_node_required()
            .with(eq(id2))
            .returning(move |_| {
                let n = node2.clone();
                Box::pin(async move { Ok(n) })
            });
        mock_err
            .expect_get_node_metrics()
            .with(eq(id2))
            .returning(move |_| {
                Box::pin(async move {
                    Err(DataStoreError::InternalError {
                        message: "metrics err".into(),
                    })
                })
            });
        let args2 = MetricsNodeArgs {
            id: id2,
            detailed: false,
            history: false,
        };
        assert!(
            metrics_node(args2, &mock_err, crate::OutputFormat::Json)
                .await
                .is_ok()
        );
    }
}
