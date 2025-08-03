//! Tests for node CRUD handlers
//!
//! Contains comprehensive tests for all node CRUD operations including
//! creation, reading, updating, and deletion via API handlers.
//!
//! Tests are organized into focused modules by operation type.

#[cfg(test)]
#[path = "list_tests.rs"]
mod list_tests;

#[cfg(test)]
#[path = "get_tests.rs"]
mod get_tests;

#[cfg(test)]
#[path = "create_node_tests.rs"]
mod create_node_tests;

#[cfg(test)]
#[path = "update_node_tests.rs"]
mod update_node_tests;

#[cfg(test)]
#[path = "delete_node_tests.rs"]
mod delete_node_tests;

/// Shared test utilities for node CRUD tests
#[cfg(test)]
pub mod test_utils {
    use crate::api::{CreateNodeRequest, UpdateNodeRequest};
    use crate::server::AppState;
    use migration::{Migrator, MigratorTrait};
    use std::sync::Arc;
    use unet_core::{
        config::Config,
        datastore::{DataStore, sqlite::SqliteStore},
        models::{DeviceRole, Lifecycle, Location, Node, Vendor, location::LocationBuilder},
        policy_integration::PolicyService,
    };

    /// Set up a test app state with in-memory `SQLite`
    pub async fn setup_test_app_state() -> AppState {
        let store = SqliteStore::new("sqlite::memory:")
            .await
            .expect("Failed to create test datastore");

        // Run migrations manually
        Migrator::up(store.connection(), None).await.unwrap();

        let datastore: Arc<dyn DataStore + Send + Sync> = Arc::new(store);

        let config = Config::default();
        let policy_service = PolicyService::new(config.git);

        AppState {
            datastore,
            policy_service,
        }
    }

    /// Create a test node in the datastore
    pub async fn create_test_node(app_state: &AppState) -> Node {
        let node = Node::new(
            "test-node".to_string(),
            "example.com".to_string(),
            Vendor::Cisco,
            DeviceRole::Router,
        );

        app_state.datastore.create_node(&node).await.unwrap()
    }

    /// Create a test `CreateNodeRequest`
    pub fn create_test_create_request() -> CreateNodeRequest {
        CreateNodeRequest {
            name: "test-router".to_string(),
            domain: Some("test.com".to_string()),
            vendor: Vendor::Cisco,
            model: "ISR4431".to_string(),
            role: DeviceRole::Router,
            lifecycle: Lifecycle::Live,
            location_id: None,
            management_ip: Some("192.168.1.1".to_string()),
            custom_data: Some(serde_json::json!({"rack": "R1"})),
        }
    }

    /// Create a test location in the datastore
    pub async fn create_test_location(app_state: &AppState) -> Location {
        let location = LocationBuilder::new()
            .name("Test Datacenter".to_string())
            .location_type("datacenter".to_string())
            .build()
            .unwrap();

        app_state
            .datastore
            .create_location(&location)
            .await
            .unwrap()
    }

    /// Create a test `UpdateNodeRequest`
    pub fn create_test_update_request() -> UpdateNodeRequest {
        UpdateNodeRequest {
            name: Some("updated-router".to_string()),
            domain: Some("updated.com".to_string()),
            vendor: Some(Vendor::Juniper),
            model: Some("EX4300".to_string()),
            role: Some(DeviceRole::Switch),
            lifecycle: Some(Lifecycle::Implementing),
            location_id: None,
            management_ip: Some("192.168.2.1".to_string()),
            custom_data: Some(serde_json::json!({"rack": "R2"})),
        }
    }
}
