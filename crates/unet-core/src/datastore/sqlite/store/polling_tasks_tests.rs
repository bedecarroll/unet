use crate::config::defaults;
use crate::datastore::DataStore;
use crate::datastore::sqlite::SqliteStore;
use crate::entities::{locations, nodes, polling_tasks, vendors};
use crate::models::{DeviceRole, Lifecycle, NodeBuilder, Vendor};
use crate::snmp::{PollingTask, SessionConfig};
use sea_orm::{
    ColumnTrait, ConnectionTrait, Database, DatabaseBackend, EntityTrait, QueryFilter, Schema,
};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
use uuid::Uuid;

async fn with_store<F, Fut>(f: F)
where
    F: FnOnce(SqliteStore) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let connection = Database::connect("sqlite::memory:")
        .await
        .expect("connect sqlite::memory:");
    let schema = Schema::new(DatabaseBackend::Sqlite);

    for stmt in [
        schema.create_table_from_entity(vendors::Entity),
        schema.create_table_from_entity(locations::Entity),
        schema.create_table_from_entity(nodes::Entity),
        schema.create_table_from_entity(polling_tasks::Entity),
    ] {
        connection
            .execute(connection.get_database_backend().build(&stmt))
            .await
            .expect("apply schema");
    }

    f(SqliteStore::from_connection(connection)).await;
}

#[tokio::test]
async fn test_get_node_polling_task_returns_persisted_task() {
    with_store(|store| async move {
        let node = NodeBuilder::new()
            .id(Uuid::new_v4())
            .name("edge-store-1")
            .domain("example.com")
            .vendor(Vendor::Cisco)
            .model("ISR4321")
            .role(DeviceRole::Router)
            .lifecycle(Lifecycle::Live)
            .management_ip(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 10)))
            .build()
            .unwrap();
        store.create_node(&node).await.unwrap();

        let task = PollingTask::new(
            SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(192, 0, 2, 10)),
                defaults::network::SNMP_DEFAULT_PORT,
            ),
            node.id,
            vec!["1.3.6.1.2.1.1.1.0".to_string()],
            Duration::from_secs(120),
            SessionConfig::default(),
        );

        store.upsert_polling_task(&task).await.unwrap();

        let saved = store
            .get_node_polling_task(&node.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(saved.node_id, node.id);
        assert_eq!(saved.target, task.target);
        assert!(saved.enabled);
        assert_eq!(saved.interval, Duration::from_secs(120));
    })
    .await;
}

#[tokio::test]
async fn test_upsert_polling_task_updates_existing_row() {
    with_store(|store| async move {
        let node = NodeBuilder::new()
            .id(Uuid::new_v4())
            .name("edge-store-2")
            .domain("example.com")
            .vendor(Vendor::Cisco)
            .model("ISR4321")
            .role(DeviceRole::Router)
            .lifecycle(Lifecycle::Live)
            .management_ip(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 11)))
            .build()
            .unwrap();
        store.create_node(&node).await.unwrap();

        let mut task = PollingTask::new(
            SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(192, 0, 2, 11)),
                defaults::network::SNMP_DEFAULT_PORT,
            ),
            node.id,
            vec!["1.3.6.1.2.1.1.1.0".to_string()],
            Duration::from_secs(300),
            SessionConfig::default(),
        );
        store.upsert_polling_task(&task).await.unwrap();

        task.enabled = false;
        task.last_error = Some("disabled for maintenance".to_string());
        task.consecutive_failures = 2;
        store.upsert_polling_task(&task).await.unwrap();

        let persisted = crate::entities::polling_tasks::Entity::find()
            .filter(crate::entities::polling_tasks::Column::NodeId.eq(node.id.to_string()))
            .one(store.connection())
            .await
            .unwrap()
            .unwrap();

        assert!(!persisted.enabled);
        assert_eq!(
            persisted.last_error.as_deref(),
            Some("disabled for maintenance")
        );
        assert_eq!(persisted.consecutive_failures, 2);
    })
    .await;
}
