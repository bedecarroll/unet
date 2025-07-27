//! Shared test helper functions for node update tests

use migration::{Migrator, MigratorTrait};
use unet_core::{
    datastore::{DataStore, sqlite::SqliteStore},
    models::*,
};

pub async fn setup_test_datastore() -> SqliteStore {
    let store = SqliteStore::new("sqlite::memory:").await.unwrap();
    Migrator::up(store.connection(), None).await.unwrap();
    store
}

pub async fn create_test_node(datastore: &SqliteStore) -> Node {
    let mut node = Node::new(
        "test-update-node".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );
    node.model = "ASR1000".to_string();
    node.lifecycle = Lifecycle::Live;
    node.management_ip = Some("192.168.1.1".parse().unwrap());
    datastore.create_node(&node).await.unwrap()
}
