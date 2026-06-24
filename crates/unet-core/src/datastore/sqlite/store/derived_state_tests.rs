//! Tests for persisted derived-state lookups in `SqliteStore`

use super::super::SqliteStore;
use crate::datastore::DataStore;
use crate::entities;
use crate::models::derived::{
    InterfaceAdminStatus, InterfaceOperStatus, InterfaceStats, InterfaceStatus, NodeStatus,
    PerformanceMetrics, SystemInfo,
};
use crate::models::{DeviceRole, Node, Vendor};
use chrono::{DateTime, Utc};
use migration::Migrator;
use sea_orm::{ActiveModelTrait, ConnectionTrait, Database, DatabaseBackend, Schema, Set};
use sea_orm_migration::MigratorTrait;
use std::time::SystemTime;

fn test_node() -> Node {
    Node::new(
        "derived-node".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    )
}

async fn setup_schema_store() -> SqliteStore {
    let db = Database::connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory database");
    apply_entity_schema(&db).await;
    SqliteStore::from_connection(db)
}

async fn apply_entity_schema(connection: &impl ConnectionTrait) {
    let schema = Schema::new(DatabaseBackend::Sqlite);

    for stmt in [
        schema.create_table_from_entity(entities::vendors::Entity),
        schema.create_table_from_entity(entities::locations::Entity),
        schema.create_table_from_entity(entities::nodes::Entity),
        schema.create_table_from_entity(entities::links::Entity),
        schema.create_table_from_entity(entities::node_status::Entity),
        schema.create_table_from_entity(entities::interface_status::Entity),
        schema.create_table_from_entity(entities::polling_tasks::Entity),
    ] {
        connection
            .execute(connection.get_database_backend().build(&stmt))
            .await
            .unwrap();
    }
}

async fn setup_migrated_store() -> SqliteStore {
    let store = SqliteStore::new("sqlite::memory:").await.unwrap();
    Migrator::up(store.connection(), None).await.unwrap();
    store
}

fn parse_timestamp(value: &str) -> SystemTime {
    DateTime::parse_from_rfc3339(value)
        .unwrap()
        .with_timezone(&Utc)
        .into()
}

fn snapshot(
    node_id: uuid::Uuid,
    last_updated: &str,
    reachable: bool,
    name: &str,
    oper_status: InterfaceOperStatus,
    cpu: u8,
) -> NodeStatus {
    NodeStatus {
        node_id,
        last_updated: parse_timestamp(last_updated),
        reachable,
        system_info: Some(SystemInfo {
            description: Some(format!("{name} chassis")),
            object_id: None,
            uptime_ticks: Some(12_345),
            contact: None,
            name: Some(name.to_string()),
            location: Some("rack-1".to_string()),
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
            oper_status,
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
            cpu_utilization: Some(cpu),
            memory_utilization: Some(70),
            total_memory: Some(8192),
            used_memory: Some(4096),
            load_average: Some(0.5),
        }),
        environmental: None,
        vendor_metrics: std::collections::HashMap::new(),
        raw_snmp_data: std::collections::HashMap::new(),
        last_snmp_success: Some(parse_timestamp(last_updated)),
        last_error: reachable
            .then(String::new)
            .filter(|value| !value.is_empty()),
        consecutive_failures: if reachable { 0 } else { 2 },
    }
}

#[tokio::test]
async fn test_get_node_status_returns_persisted_status() {
    let store = setup_schema_store().await;
    let node = test_node();
    store.create_node(&node).await.unwrap();

    crate::entities::node_status::ActiveModel {
        id: Set("status-derived-node".to_string()),
        node_id: Set(node.id.to_string()),
        last_updated: Set("2026-04-07T01:02:03Z".to_string()),
        reachable: Set(true),
        system_info: Set(Some(
            r#"{"description":"Test Router","name":"derived-node","uptime_ticks":1200}"#
                .to_string(),
        )),
        performance: Set(None),
        environmental: Set(None),
        vendor_metrics: Set(Some(
            r#"{"1.3.6.1.4.1.9.2.1.3.0":{"Integer":55}}"#.to_string(),
        )),
        raw_snmp_data: Set(Some(
            r#"{"1.3.6.1.2.1.1.5.0":{"String":"derived-node"}}"#.to_string(),
        )),
        last_snmp_success: Set(Some("2026-04-07T01:00:00Z".to_string())),
        last_error: Set(None),
        consecutive_failures: Set(0),
    }
    .insert(store.connection())
    .await
    .unwrap();

    let status = store.get_node_status(&node.id).await.unwrap().unwrap();
    assert!(status.reachable);
    assert_eq!(status.node_id, node.id);
    assert_eq!(
        status.system_info.unwrap().name.as_deref(),
        Some("derived-node")
    );
    assert_eq!(
        status
            .raw_snmp_data
            .get("1.3.6.1.2.1.1.5.0")
            .unwrap()
            .to_string(),
        "derived-node"
    );
}

#[tokio::test]
async fn test_get_node_interfaces_returns_persisted_interfaces() {
    let store = setup_schema_store().await;
    let node = test_node();
    store.create_node(&node).await.unwrap();

    crate::entities::node_status::ActiveModel {
        id: Set("status-derived-iface".to_string()),
        node_id: Set(node.id.to_string()),
        last_updated: Set("2026-04-07T01:02:03Z".to_string()),
        reachable: Set(true),
        system_info: Set(None),
        performance: Set(None),
        environmental: Set(None),
        vendor_metrics: Set(None),
        raw_snmp_data: Set(None),
        last_snmp_success: Set(None),
        last_error: Set(None),
        consecutive_failures: Set(0),
    }
    .insert(store.connection())
    .await
    .unwrap();

    crate::entities::interface_status::ActiveModel {
        id: Set("iface-1".to_string()),
        node_status_id: Set("status-derived-iface".to_string()),
        index: Set(1),
        name: Set("GigabitEthernet0/1".to_string()),
        interface_type: Set(6),
        mtu: Set(Some(1500)),
        speed: Set(Some(1_000_000_000)),
        physical_address: Set(Some("00:11:22:33:44:55".to_string())),
        admin_status: Set("up".to_string()),
        oper_status: Set("down".to_string()),
        last_change: Set(Some(42)),
        input_stats: Set(r#"{"octets":1000,"packets":10,"errors":1,"discards":0}"#.to_string()),
        output_stats: Set(r#"{"octets":2000,"packets":20,"errors":2,"discards":1}"#.to_string()),
    }
    .insert(store.connection())
    .await
    .unwrap();

    let interfaces = store.get_node_interfaces(&node.id).await.unwrap();
    assert_eq!(interfaces.len(), 1);
    assert_eq!(interfaces[0].name, "GigabitEthernet0/1");
    assert_eq!(interfaces[0].input_stats.octets, 1000);
    assert_eq!(interfaces[0].output_stats.errors, 2);
}

#[tokio::test]
async fn test_get_node_metrics_returns_persisted_performance_metrics() {
    let store = setup_schema_store().await;
    let node = test_node();
    store.create_node(&node).await.unwrap();

    crate::entities::node_status::ActiveModel {
        id: Set("status-derived-metrics".to_string()),
        node_id: Set(node.id.to_string()),
        last_updated: Set("2026-04-07T01:02:03Z".to_string()),
        reachable: Set(true),
        system_info: Set(None),
        performance: Set(Some(
            r#"{"cpu_utilization":55,"memory_utilization":70,"used_memory":2048}"#.to_string(),
        )),
        environmental: Set(None),
        vendor_metrics: Set(None),
        raw_snmp_data: Set(None),
        last_snmp_success: Set(None),
        last_error: Set(None),
        consecutive_failures: Set(0),
    }
    .insert(store.connection())
    .await
    .unwrap();

    let metrics = store.get_node_metrics(&node.id).await.unwrap().unwrap();
    assert_eq!(metrics.cpu_utilization, Some(55));
    assert_eq!(metrics.memory_utilization, Some(70));
    assert_eq!(metrics.used_memory, Some(2048));
}

#[tokio::test]
async fn test_get_node_status_returns_none_without_persisted_status() {
    let store = setup_schema_store().await;
    let node = test_node();
    store.create_node(&node).await.unwrap();

    let status = store.get_node_status(&node.id).await.unwrap();
    assert!(status.is_none());
}

#[tokio::test]
async fn test_get_node_interfaces_returns_empty_without_persisted_status() {
    let store = setup_schema_store().await;
    let node = test_node();
    store.create_node(&node).await.unwrap();

    let interfaces = store.get_node_interfaces(&node.id).await.unwrap();
    assert!(interfaces.is_empty());
}

#[tokio::test]
async fn test_get_node_metrics_returns_none_without_persisted_status() {
    let store = setup_schema_store().await;
    let node = test_node();
    store.create_node(&node).await.unwrap();

    let metrics = store.get_node_metrics(&node.id).await.unwrap();
    assert!(metrics.is_none());
}

#[tokio::test]
async fn test_store_node_status_snapshot_supports_history_queries_after_migrations() {
    let store = setup_migrated_store().await;
    let node = test_node();
    store.create_node(&node).await.unwrap();

    let older = snapshot(
        node.id,
        "2026-04-07T01:00:00Z",
        false,
        "derived-node-old",
        InterfaceOperStatus::Down,
        15,
    );
    let newer = snapshot(
        node.id,
        "2026-04-07T02:00:00Z",
        true,
        "derived-node-new",
        InterfaceOperStatus::Up,
        55,
    );

    store.store_node_status_snapshot(&older).await.unwrap();
    store.store_node_status_snapshot(&newer).await.unwrap();

    let latest = store.get_node_status(&node.id).await.unwrap().unwrap();
    assert!(latest.reachable);
    assert_eq!(
        latest.system_info.unwrap().name.as_deref(),
        Some("derived-node-new")
    );
    assert_eq!(latest.interfaces[0].oper_status, InterfaceOperStatus::Up);

    let history = store
        .get_node_status_history(
            &node.id,
            &crate::datastore::HistoryQueryOptions {
                limit: 10,
                since: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(
        history[0].system_info.as_ref().unwrap().name.as_deref(),
        Some("derived-node-new")
    );
    assert_eq!(
        history[1].system_info.as_ref().unwrap().name.as_deref(),
        Some("derived-node-old")
    );

    let metrics = store.get_node_metrics(&node.id).await.unwrap().unwrap();
    assert_eq!(metrics.cpu_utilization, Some(55));
}

#[tokio::test]
async fn test_get_node_status_history_applies_limit_and_since_filter() {
    let store = setup_migrated_store().await;
    let node = test_node();
    store.create_node(&node).await.unwrap();

    for (timestamp, reachable, cpu) in [
        ("2026-04-07T01:00:00Z", false, 10),
        ("2026-04-07T02:00:00Z", true, 20),
        ("2026-04-07T03:00:00Z", true, 30),
    ] {
        let snapshot = snapshot(
            node.id,
            timestamp,
            reachable,
            "derived-node-filter",
            InterfaceOperStatus::Up,
            cpu,
        );
        store.store_node_status_snapshot(&snapshot).await.unwrap();
    }

    let limited = store
        .get_node_status_history(
            &node.id,
            &crate::datastore::HistoryQueryOptions {
                limit: 1,
                since: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(limited.len(), 1);
    assert_eq!(
        limited[0].performance.as_ref().unwrap().cpu_utilization,
        Some(30)
    );

    let filtered = store
        .get_node_status_history(
            &node.id,
            &crate::datastore::HistoryQueryOptions {
                limit: 10,
                since: Some(parse_timestamp("2026-04-07T01:30:00Z")),
            },
        )
        .await
        .unwrap();
    assert_eq!(filtered.len(), 2);
    assert_eq!(
        filtered[0].performance.as_ref().unwrap().cpu_utilization,
        Some(30)
    );
    assert_eq!(
        filtered[1].performance.as_ref().unwrap().cpu_utilization,
        Some(20)
    );
}
