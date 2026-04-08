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
