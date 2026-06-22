#[cfg(test)]
pub(super) mod helpers {
    use std::{collections::HashMap, time::SystemTime};

    use mockall::predicate::eq;
    use serde_json::Value;
    use unet_core::datastore::{MockDataStore, testing::ready_ok};
    use unet_core::models::derived::{
        InterfaceAdminStatus, InterfaceOperStatus, InterfaceStats, InterfaceStatus, NodeStatus,
        PerformanceMetrics, SystemInfo,
    };
    use unet_core::models::{DeviceRole, NodeBuilder, Vendor};
    use uuid::Uuid;

    pub(in crate::commands::nodes) fn make_node(
        id: Uuid,
        name: &str,
        model: &str,
        role: DeviceRole,
    ) -> unet_core::models::Node {
        NodeBuilder::new()
            .id(id)
            .name(name)
            .domain("example.com")
            .vendor(Vendor::Cisco)
            .model(model)
            .role(role)
            .build()
            .unwrap()
    }

    pub(in crate::commands::nodes) fn make_status(
        node_id: Uuid,
        system_name: &str,
        location: &str,
        uptime_ticks: u32,
    ) -> NodeStatus {
        NodeStatus {
            node_id,
            last_updated: SystemTime::UNIX_EPOCH,
            reachable: true,
            system_info: Some(SystemInfo {
                description: Some("Cisco router".to_string()),
                object_id: Some("1.3.6.1.4.1.9".to_string()),
                uptime_ticks: Some(uptime_ticks),
                contact: Some("noc@example.com".to_string()),
                name: Some(system_name.to_string()),
                location: Some(location.to_string()),
                services: Some(72),
            }),
            interfaces: Vec::new(),
            performance: None,
            environmental: None,
            vendor_metrics: HashMap::default(),
            raw_snmp_data: HashMap::default(),
            last_snmp_success: None,
            last_error: None,
            consecutive_failures: 0,
        }
    }

    pub(in crate::commands::nodes) fn find_entry<'a>(
        section: &'a Value,
        scope: &str,
        field: &str,
    ) -> &'a Value {
        section["entries"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["scope"] == scope && entry["field"] == field)
            .unwrap()
    }

    pub(in crate::commands::nodes) fn configure_all_compare_store(
        primary_node: &unet_core::models::Node,
        comparison_node: &unet_core::models::Node,
    ) -> MockDataStore {
        let mut store = MockDataStore::new();
        let primary_node_id = primary_node.id;
        let comparison_node_id = comparison_node.id;

        let primary_node_for_required = primary_node.clone();
        let comparison_node_for_required = comparison_node.clone();
        store
            .expect_get_node_required()
            .with(eq(primary_node_id))
            .returning(move |_| ready_ok(primary_node_for_required.clone()));
        store
            .expect_get_node_required()
            .with(eq(comparison_node_id))
            .returning(move |_| ready_ok(comparison_node_for_required.clone()));
        store
            .expect_get_node_interfaces()
            .with(eq(primary_node_id))
            .returning(|_| {
                ready_ok(vec![
                    make_interface(
                        "wan0",
                        InterfaceAdminStatus::Up,
                        InterfaceOperStatus::Up,
                        0,
                        0,
                    ),
                    make_interface(
                        "lan0",
                        InterfaceAdminStatus::Up,
                        InterfaceOperStatus::Down,
                        1,
                        0,
                    ),
                ])
            });
        store
            .expect_get_node_interfaces()
            .with(eq(comparison_node_id))
            .returning(|_| {
                ready_ok(vec![
                    make_interface(
                        "wan0",
                        InterfaceAdminStatus::Up,
                        InterfaceOperStatus::Down,
                        0,
                        2,
                    ),
                    make_interface(
                        "dmz0",
                        InterfaceAdminStatus::Down,
                        InterfaceOperStatus::Down,
                        0,
                        0,
                    ),
                ])
            });
        store
            .expect_get_node_metrics()
            .with(eq(primary_node_id))
            .returning(|_| {
                ready_ok(Some(PerformanceMetrics {
                    cpu_utilization: Some(15),
                    memory_utilization: Some(60),
                    total_memory: Some(1_024),
                    used_memory: Some(600),
                    load_average: Some(0.4),
                }))
            });
        store
            .expect_get_node_metrics()
            .with(eq(comparison_node_id))
            .returning(|_| {
                ready_ok(Some(PerformanceMetrics {
                    cpu_utilization: Some(15),
                    memory_utilization: Some(75),
                    total_memory: Some(1_024),
                    used_memory: Some(768),
                    load_average: Some(0.9),
                }))
            });
        store
            .expect_get_node_status()
            .with(eq(primary_node_id))
            .returning(move |_| {
                ready_ok(Some(make_status(primary_node_id, "edge-a", "DC1", 12_000)))
            });
        store
            .expect_get_node_status()
            .with(eq(comparison_node_id))
            .returning(move |_| {
                ready_ok(Some(make_status(
                    comparison_node_id,
                    "edge-b",
                    "DC2",
                    16_000,
                )))
            });

        store
    }

    fn make_interface(
        name: &str,
        admin_status: InterfaceAdminStatus,
        oper_status: InterfaceOperStatus,
        input_errors: u64,
        output_errors: u64,
    ) -> InterfaceStatus {
        InterfaceStatus {
            index: 1,
            name: name.to_string(),
            interface_type: 6,
            mtu: Some(1_500),
            speed: Some(1_000_000_000),
            physical_address: None,
            admin_status,
            oper_status,
            last_change: None,
            input_stats: InterfaceStats {
                octets: 10,
                packets: 20,
                errors: input_errors,
                discards: 0,
            },
            output_stats: InterfaceStats {
                octets: 30,
                packets: 40,
                errors: output_errors,
                discards: 0,
            },
        }
    }
}
