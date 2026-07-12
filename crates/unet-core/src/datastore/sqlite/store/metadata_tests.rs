use super::super::SqliteStore;
use crate::datastore::DataStore;
use crate::entities;
use crate::models::{DeviceRole, Location, Node, Vendor};
use sea_orm::ActiveModelTrait;
use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Schema};
use serde_json::Value;

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

#[tokio::test]
async fn test_get_entity_counts_returns_real_counts() {
    let store = setup_schema_store().await;
    let node = Node::new(
        "count-node".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );
    let location = Location::new_root("Count Location".to_string(), "room".to_string());

    store.create_node(&node).await.unwrap();
    store.create_location(&location).await.unwrap();

    crate::entities::node_status::ActiveModel {
        id: sea_orm::Set("status-count-node".to_string()),
        node_id: sea_orm::Set(node.id.to_string()),
        last_updated: sea_orm::Set("2026-04-07T01:02:03Z".to_string()),
        reachable: sea_orm::Set(true),
        system_info: sea_orm::Set(None),
        performance: sea_orm::Set(None),
        environmental: sea_orm::Set(None),
        vendor_metrics: sea_orm::Set(None),
        raw_snmp_data: sea_orm::Set(None),
        last_snmp_success: sea_orm::Set(None),
        last_error: sea_orm::Set(None),
        consecutive_failures: sea_orm::Set(0),
    }
    .insert(store.connection())
    .await
    .unwrap();

    let counts = store.get_entity_counts().await.unwrap();
    assert_eq!(counts.get("nodes"), Some(&1));
    assert_eq!(counts.get("locations"), Some(&1));
    assert_eq!(counts.get("node_status"), Some(&1));
    assert_eq!(counts.get("links"), Some(&0));
    assert_eq!(counts.get("interface_status"), Some(&0));
    assert_eq!(counts.get("polling_tasks"), Some(&0));
}

#[tokio::test]
async fn test_get_statistics_returns_real_datastore_summary() {
    let store = setup_schema_store().await;
    let node = Node::new(
        "stats-node".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );
    store.create_node(&node).await.unwrap();

    crate::entities::node_status::ActiveModel {
        id: sea_orm::Set("status-stats-node".to_string()),
        node_id: sea_orm::Set(node.id.to_string()),
        last_updated: sea_orm::Set("2026-04-07T01:02:03Z".to_string()),
        reachable: sea_orm::Set(true),
        system_info: sea_orm::Set(None),
        performance: sea_orm::Set(Some(r#"{"cpu_utilization":55}"#.to_string())),
        environmental: sea_orm::Set(None),
        vendor_metrics: sea_orm::Set(None),
        raw_snmp_data: sea_orm::Set(None),
        last_snmp_success: sea_orm::Set(Some("2026-04-07T01:00:00Z".to_string())),
        last_error: sea_orm::Set(None),
        consecutive_failures: sea_orm::Set(0),
    }
    .insert(store.connection())
    .await
    .unwrap();

    let stats = store.get_statistics().await.unwrap();
    assert_eq!(stats.get("datastore").unwrap(), "sqlite");
    assert_eq!(stats.get("nodes").unwrap(), &Value::from(1));
    assert_eq!(stats.get("nodes_with_status").unwrap(), &Value::from(1));
    assert_eq!(stats.get("reachable_nodes").unwrap(), &Value::from(1));
    assert_eq!(
        stats.get("latest_status_update").unwrap(),
        &Value::from("2026-04-07T01:02:03Z")
    );
}

#[tokio::test]
async fn test_get_statistics_tracks_unreachable_nodes_and_interface_rows() {
    let store = setup_schema_store().await;
    let reachable = Node::new(
        "reachable-node".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );
    let unreachable = Node::new(
        "unreachable-node".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );
    store.create_node(&reachable).await.unwrap();
    store.create_node(&unreachable).await.unwrap();

    crate::entities::node_status::ActiveModel {
        id: sea_orm::Set("status-reachable".to_string()),
        node_id: sea_orm::Set(reachable.id.to_string()),
        last_updated: sea_orm::Set("2026-04-07T01:02:03Z".to_string()),
        reachable: sea_orm::Set(true),
        system_info: sea_orm::Set(None),
        performance: sea_orm::Set(None),
        environmental: sea_orm::Set(None),
        vendor_metrics: sea_orm::Set(None),
        raw_snmp_data: sea_orm::Set(None),
        last_snmp_success: sea_orm::Set(None),
        last_error: sea_orm::Set(None),
        consecutive_failures: sea_orm::Set(0),
    }
    .insert(store.connection())
    .await
    .unwrap();

    crate::entities::node_status::ActiveModel {
        id: sea_orm::Set("status-unreachable".to_string()),
        node_id: sea_orm::Set(unreachable.id.to_string()),
        last_updated: sea_orm::Set("2026-04-07T01:04:05Z".to_string()),
        reachable: sea_orm::Set(false),
        system_info: sea_orm::Set(None),
        performance: sea_orm::Set(None),
        environmental: sea_orm::Set(None),
        vendor_metrics: sea_orm::Set(None),
        raw_snmp_data: sea_orm::Set(None),
        last_snmp_success: sea_orm::Set(None),
        last_error: sea_orm::Set(Some("timeout".to_string())),
        consecutive_failures: sea_orm::Set(2),
    }
    .insert(store.connection())
    .await
    .unwrap();

    crate::entities::interface_status::ActiveModel {
        id: sea_orm::Set("iface-status".to_string()),
        node_status_id: sea_orm::Set("status-unreachable".to_string()),
        index: sea_orm::Set(7),
        name: sea_orm::Set("GigabitEthernet0/7".to_string()),
        interface_type: sea_orm::Set(6),
        mtu: sea_orm::Set(Some(1500)),
        speed: sea_orm::Set(Some(1_000_000_000)),
        physical_address: sea_orm::Set(None),
        admin_status: sea_orm::Set("up".to_string()),
        oper_status: sea_orm::Set("down".to_string()),
        last_change: sea_orm::Set(None),
        input_stats: sea_orm::Set(
            r#"{"octets":1000,"packets":10,"errors":0,"discards":0}"#.to_string(),
        ),
        output_stats: sea_orm::Set(
            r#"{"octets":2000,"packets":20,"errors":0,"discards":0}"#.to_string(),
        ),
    }
    .insert(store.connection())
    .await
    .unwrap();

    let stats = store.get_statistics().await.unwrap();
    assert_eq!(stats.get("nodes_with_status").unwrap(), &Value::from(2));
    assert_eq!(stats.get("reachable_nodes").unwrap(), &Value::from(1));
    assert_eq!(stats.get("unreachable_nodes").unwrap(), &Value::from(1));
    assert_eq!(stats.get("interfaces_monitored").unwrap(), &Value::from(1));
    assert_eq!(
        stats.get("latest_status_update").unwrap(),
        &Value::from("2026-04-07T01:04:05Z")
    );
}

#[tokio::test]
async fn test_get_statistics_returns_null_latest_status_without_status_rows() {
    let store = setup_schema_store().await;
    let node = Node::new(
        "inventory-only-node".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );
    store.create_node(&node).await.unwrap();

    let stats = store.get_statistics().await.unwrap();
    assert_eq!(stats.get("nodes").unwrap(), &Value::from(1));
    assert_eq!(stats.get("nodes_with_status").unwrap(), &Value::from(0));
    assert_eq!(stats.get("reachable_nodes").unwrap(), &Value::from(0));
    assert_eq!(stats.get("unreachable_nodes").unwrap(), &Value::from(0));
    assert_eq!(stats.get("latest_status_update").unwrap(), &Value::Null);
}
