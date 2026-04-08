/// Execution tests for node history command
#[cfg(test)]
mod tests {
    use super::super::history::{build_history_output, history_node};
    use super::super::types::{HistoryNodeArgs, HistoryType};
    use unet_core::datastore::{MockDataStore, testing::ready_ok};
    use unet_core::models::derived::{
        InterfaceAdminStatus, InterfaceOperStatus, InterfaceStats, InterfaceStatus, NodeStatus,
        PerformanceMetrics, SystemInfo,
    };
    use uuid::Uuid;

    fn store_with_node(node: unet_core::models::Node) -> MockDataStore {
        let mut store = MockDataStore::new();
        store
            .expect_get_node_required()
            .returning(move |_| ready_ok(node.clone()));
        store
            .expect_get_node_status_history()
            .returning(|_, _| ready_ok(Vec::new()));
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

    fn snapshot(node_id: Uuid) -> NodeStatus {
        NodeStatus {
            node_id,
            last_updated: chrono::DateTime::parse_from_rfc3339("2026-04-07T02:00:00Z")
                .unwrap()
                .with_timezone(&chrono::Utc)
                .into(),
            reachable: true,
            system_info: Some(SystemInfo {
                description: Some("edge-1 chassis".to_string()),
                object_id: None,
                uptime_ticks: Some(12_345),
                contact: None,
                name: Some("edge-1".to_string()),
                location: Some("rack-22".to_string()),
                services: Some(72),
            }),
            interfaces: vec![InterfaceStatus {
                index: 1,
                name: "GigabitEthernet0/1".to_string(),
                interface_type: 6,
                mtu: Some(1500),
                speed: Some(1_000_000_000),
                physical_address: Some("00:11:22:33:44:55".to_string()),
                admin_status: InterfaceAdminStatus::Up,
                oper_status: InterfaceOperStatus::Up,
                last_change: Some(10),
                input_stats: InterfaceStats {
                    octets: 1000,
                    packets: 10,
                    errors: 0,
                    discards: 0,
                },
                output_stats: InterfaceStats {
                    octets: 2000,
                    packets: 20,
                    errors: 0,
                    discards: 0,
                },
            }],
            performance: Some(PerformanceMetrics {
                cpu_utilization: Some(55),
                memory_utilization: Some(70),
                total_memory: Some(8192),
                used_memory: Some(4096),
                load_average: Some(0.5),
            }),
            environmental: None,
            vendor_metrics: std::collections::HashMap::new(),
            raw_snmp_data: std::collections::HashMap::new(),
            last_snmp_success: Some(
                chrono::DateTime::parse_from_rfc3339("2026-04-07T01:59:00Z")
                    .unwrap()
                    .with_timezone(&chrono::Utc)
                    .into(),
            ),
            last_error: None,
            consecutive_failures: 0,
        }
    }

    #[tokio::test]
    async fn test_history_all_variants() {
        let node = make_node();
        let store = store_with_node(node.clone());

        for history_type in [
            HistoryType::Status,
            HistoryType::Interfaces,
            HistoryType::Metrics,
            HistoryType::System,
            HistoryType::All,
        ] {
            let args = HistoryNodeArgs {
                id: node.id,
                history_type: history_type.clone(),
                limit: 5,
                last_hours: Some(24),
                detailed: true,
            };
            let result = history_node(args, &store, crate::OutputFormat::Json).await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_build_history_output_uses_real_snapshots_for_all_supported_types() {
        let node = make_node();
        let mut store = MockDataStore::new();
        let history = vec![snapshot(node.id)];

        store
            .expect_get_node_status_history()
            .times(5)
            .returning(move |_, _| ready_ok(history.clone()));

        for (history_type, key) in [
            (HistoryType::Status, "status_history"),
            (HistoryType::Interfaces, "interface_history"),
            (HistoryType::Metrics, "metrics_history"),
            (HistoryType::System, "system_history"),
            (HistoryType::All, "complete_history"),
        ] {
            let output = build_history_output(
                &node,
                HistoryNodeArgs {
                    id: node.id,
                    history_type: history_type.clone(),
                    limit: 5,
                    last_hours: Some(24),
                    detailed: true,
                },
                &store,
            )
            .await
            .unwrap();

            assert_eq!(output["node_id"], serde_json::json!(node.id));
            assert!(output[key].is_array(), "missing array payload for {key}");
            assert_eq!(output[key].as_array().unwrap().len(), 1);
            assert!(output.to_string().contains("edge-1"));
            assert!(!output.to_string().contains("implementation_required"));
        }
    }
}
