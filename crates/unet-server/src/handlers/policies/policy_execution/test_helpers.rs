//! Shared test helper functions for policy execution tests

use unet_core::{
    datastore::{DataStore, sqlite::SqliteStore},
    models::*,
};

pub async fn setup_test_datastore() -> SqliteStore {
    
    test_support::sqlite::sqlite_store().await
}

pub async fn create_test_node(datastore: &SqliteStore) -> Node {
    let mut node = Node::new(
        "test-node".to_string(),
        "example.com".to_string(),
        Vendor::Cisco,
        DeviceRole::Router,
    );
    node.model = "ASR1000".to_string();
    node.version = Some("15.1".to_string());
    node.lifecycle = Lifecycle::Live;
    datastore.create_node(&node).await.unwrap()
}
