//! Tests for node CRUD handlers
//!
//! Contains comprehensive tests for all node CRUD operations including
//! creation, reading, updating, and deletion via API handlers.

#[cfg(test)]
mod tests {
    use super::super::crud::*;
    use crate::api::{ApiResponse, CreateNodeRequest, UpdateNodeRequest};
    use crate::handlers::nodes::types::ListNodesQuery;
    use crate::server::AppState;
    use axum::{
        extract::{Path, Query, State},
        response::Json,
    };
    use migration::{Migrator, MigratorTrait};
    use std::net::IpAddr;
    use std::sync::Arc;
    use unet_core::{
        config::Config,
        datastore::{DataStore, sqlite::SqliteStore},
        models::{DeviceRole, Lifecycle, Node, Vendor},
        policy_integration::PolicyService,
    };
    use uuid::Uuid;

    /// Set up a test app state with in-memory `SQLite`
    async fn setup_test_app_state() -> AppState {
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
    async fn create_test_node(app_state: &AppState) -> Node {
        let node = Node::new(
            "test-node".to_string(),
            "example.com".to_string(),
            Vendor::Cisco,
            DeviceRole::Router,
        );

        app_state.datastore.create_node(&node).await.unwrap()
    }

    /// Create a test `CreateNodeRequest`
    fn create_test_create_request() -> CreateNodeRequest {
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

    /// Create a test `UpdateNodeRequest`
    fn create_test_update_request() -> UpdateNodeRequest {
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

    #[tokio::test]
    async fn test_list_nodes_empty() {
        let app_state = setup_test_app_state().await;
        let query = ListNodesQuery {
            page: None,
            per_page: None,
            lifecycle: None,
            role: None,
            vendor: None,
            include_status: None,
        };

        let result = list_nodes(State(app_state), Query(query)).await;

        assert!(result.is_ok());
        let Json(ApiResponse { data, success, .. }) = result.unwrap();
        assert!(success);
        assert_eq!(data.data.len(), 0);
        assert_eq!(data.total, 0);
        assert_eq!(data.page, 1);
        assert_eq!(data.per_page, 20);
    }

    #[tokio::test]
    async fn test_list_nodes_with_data() {
        let app_state = setup_test_app_state().await;
        let _node = create_test_node(&app_state).await;

        let query = ListNodesQuery {
            page: None,
            per_page: None,
            lifecycle: None,
            role: None,
            vendor: None,
            include_status: None,
        };

        let result = list_nodes(State(app_state), Query(query)).await;

        assert!(result.is_ok());
        let Json(ApiResponse { data, success, .. }) = result.unwrap();
        assert!(success);
        assert_eq!(data.data.len(), 1);
        assert_eq!(data.total, 1);
        assert_eq!(data.data[0].node.name, "test-node");
    }

    #[tokio::test]
    async fn test_list_nodes_with_pagination() {
        let app_state = setup_test_app_state().await;

        // Create multiple nodes
        for i in 1..=5 {
            let node = Node::new(
                format!("node-{i}"),
                "example.com".to_string(),
                Vendor::Cisco,
                DeviceRole::Router,
            );
            app_state.datastore.create_node(&node).await.unwrap();
        }

        let query = ListNodesQuery {
            page: Some(2),
            per_page: Some(2),
            lifecycle: None,
            role: None,
            vendor: None,
            include_status: None,
        };

        let result = list_nodes(State(app_state), Query(query)).await;

        assert!(result.is_ok());
        let Json(ApiResponse { data, success, .. }) = result.unwrap();
        assert!(success);
        assert_eq!(data.data.len(), 2);
        assert_eq!(data.total, 5);
        assert_eq!(data.page, 2);
        assert_eq!(data.per_page, 2);
        assert_eq!(data.total_pages, 3);
        assert!(data.has_prev);
        assert!(data.has_next);
    }

    #[tokio::test]
    async fn test_list_nodes_with_filters() {
        let app_state = setup_test_app_state().await;

        // Create nodes with different vendors
        let cisco_node = Node::new(
            "cisco-router".to_string(),
            "example.com".to_string(),
            Vendor::Cisco,
            DeviceRole::Router,
        );
        let juniper_node = Node::new(
            "juniper-switch".to_string(),
            "example.com".to_string(),
            Vendor::Juniper,
            DeviceRole::Switch,
        );
        app_state.datastore.create_node(&cisco_node).await.unwrap();
        app_state
            .datastore
            .create_node(&juniper_node)
            .await
            .unwrap();

        let query = ListNodesQuery {
            page: None,
            per_page: None,
            lifecycle: None,
            role: Some("switch".to_string()),
            vendor: Some("juniper".to_string()),
            include_status: None,
        };

        let result = list_nodes(State(app_state), Query(query)).await;

        assert!(result.is_ok());
        let Json(ApiResponse { data, success, .. }) = result.unwrap();
        assert!(success);
        assert_eq!(data.data.len(), 1);
        assert_eq!(data.data[0].node.name, "juniper-switch");
        assert_eq!(data.data[0].node.vendor, Vendor::Juniper);
        assert_eq!(data.data[0].node.role, DeviceRole::Switch);
    }

    #[tokio::test]
    async fn test_get_node_success() {
        let app_state = setup_test_app_state().await;
        let node = create_test_node(&app_state).await;

        let result = get_node(State(app_state), Path(node.id)).await;

        assert!(result.is_ok());
        let Json(ApiResponse { data, success, .. }) = result.unwrap();
        assert!(success);
        assert_eq!(data.node.id, node.id);
        assert_eq!(data.node.name, "test-node");
        assert_eq!(data.node.vendor, Vendor::Cisco);
        assert!(data.status.is_none());
    }

    #[tokio::test]
    async fn test_get_node_not_found() {
        let app_state = setup_test_app_state().await;
        let non_existent_id = Uuid::new_v4();

        let result = get_node(State(app_state), Path(non_existent_id)).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_create_node_success() {
        let app_state = setup_test_app_state().await;
        let request = create_test_create_request();

        let result = create_node(State(app_state.clone()), Json(request)).await;

        assert!(result.is_ok());
        let Json(ApiResponse { data, success, .. }) = result.unwrap();
        assert!(success);
        assert_eq!(data.node.name, "test-router");
        assert_eq!(data.node.domain, "test.com");
        assert_eq!(data.node.vendor, Vendor::Cisco);
        assert_eq!(data.node.model, "ISR4431");
        assert_eq!(data.node.role, DeviceRole::Router);
        assert_eq!(data.node.lifecycle, Lifecycle::Live);
        assert_eq!(
            data.node.management_ip,
            Some("192.168.1.1".parse::<IpAddr>().unwrap())
        );
        assert_eq!(data.node.custom_data, serde_json::json!({"rack": "R1"}));

        // Verify it was actually saved to the datastore
        let saved_node = app_state.datastore.get_node(&data.node.id).await.unwrap();
        assert!(saved_node.is_some());
        assert_eq!(saved_node.unwrap().name, "test-router");
    }

    #[tokio::test]
    async fn test_create_node_validation_error() {
        let app_state = setup_test_app_state().await;
        let mut request = create_test_create_request();
        request.name = String::new(); // Invalid empty name

        let result = create_node(State(app_state), Json(request)).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("validation failed"));
    }

    #[tokio::test]
    async fn test_create_node_invalid_ip() {
        let app_state = setup_test_app_state().await;
        let mut request = create_test_create_request();
        request.management_ip = Some("invalid-ip".to_string());

        let result = create_node(State(app_state), Json(request)).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Invalid IP"));
    }

    #[tokio::test]
    async fn test_update_node_success() {
        let app_state = setup_test_app_state().await;
        let node = create_test_node(&app_state).await;
        let request = create_test_update_request();

        let result = update_node(State(app_state.clone()), Path(node.id), Json(request)).await;

        assert!(result.is_ok());
        let Json(ApiResponse { data, success, .. }) = result.unwrap();
        assert!(success);
        assert_eq!(data.node.id, node.id);
        assert_eq!(data.node.name, "updated-router");
        assert_eq!(data.node.domain, "updated.com");
        assert_eq!(data.node.fqdn, "updated-router.updated.com");
        assert_eq!(data.node.vendor, Vendor::Juniper);
        assert_eq!(data.node.model, "EX4300");
        assert_eq!(data.node.role, DeviceRole::Switch);
        assert_eq!(data.node.lifecycle, Lifecycle::Implementing);
        assert_eq!(
            data.node.management_ip,
            Some("192.168.2.1".parse::<IpAddr>().unwrap())
        );
        assert_eq!(data.node.custom_data, serde_json::json!({"rack": "R2"}));

        // Verify the update was persisted
        let updated_node = app_state
            .datastore
            .get_node(&node.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated_node.name, "updated-router");
        assert_eq!(updated_node.vendor, Vendor::Juniper);
    }

    #[tokio::test]
    async fn test_update_node_partial() {
        let app_state = setup_test_app_state().await;
        let node = create_test_node(&app_state).await;
        let original_vendor = node.vendor;
        let original_model = node.model.clone();

        let request = UpdateNodeRequest {
            name: Some("partial-update".to_string()),
            domain: None,
            vendor: None,
            model: None,
            role: None,
            lifecycle: None,
            location_id: None,
            management_ip: None,
            custom_data: None,
        };

        let result = update_node(State(app_state.clone()), Path(node.id), Json(request)).await;

        assert!(result.is_ok());
        let Json(ApiResponse { data, success, .. }) = result.unwrap();
        assert!(success);
        assert_eq!(data.node.name, "partial-update");
        assert_eq!(data.node.vendor, original_vendor); // Should remain unchanged
        assert_eq!(data.node.model, original_model); // Should remain unchanged
    }

    #[tokio::test]
    async fn test_update_node_not_found() {
        let app_state = setup_test_app_state().await;
        let non_existent_id = Uuid::new_v4();
        let request = create_test_update_request();

        let result = update_node(State(app_state), Path(non_existent_id), Json(request)).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_update_node_invalid_ip() {
        let app_state = setup_test_app_state().await;
        let node = create_test_node(&app_state).await;
        let mut request = create_test_update_request();
        request.management_ip = Some("invalid-ip".to_string());

        let result = update_node(State(app_state), Path(node.id), Json(request)).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Invalid management IP"));
    }

    #[tokio::test]
    async fn test_update_node_fqdn_update() {
        let app_state = setup_test_app_state().await;
        let node = create_test_node(&app_state).await;

        let request = UpdateNodeRequest {
            name: Some("new-name".to_string()),
            domain: Some("new-domain.com".to_string()),
            vendor: None,
            model: None,
            role: None,
            lifecycle: None,
            location_id: None,
            management_ip: None,
            custom_data: None,
        };

        let result = update_node(State(app_state), Path(node.id), Json(request)).await;

        assert!(result.is_ok());
        let Json(ApiResponse { data, success, .. }) = result.unwrap();
        assert!(success);
        assert_eq!(data.node.name, "new-name");
        assert_eq!(data.node.domain, "new-domain.com");
        assert_eq!(data.node.fqdn, "new-name.new-domain.com");
    }

    #[tokio::test]
    async fn test_delete_node_success() {
        let app_state = setup_test_app_state().await;
        let node = create_test_node(&app_state).await;

        let result = delete_node(State(app_state.clone()), Path(node.id)).await;

        assert!(result.is_ok());
        let Json(ApiResponse {
            data: (), success, ..
        }) = result.unwrap();
        assert!(success);

        // Verify the node was actually deleted
        let deleted_node = app_state.datastore.get_node(&node.id).await.unwrap();
        assert!(deleted_node.is_none());
    }

    #[tokio::test]
    async fn test_delete_node_not_found() {
        let app_state = setup_test_app_state().await;
        let non_existent_id = Uuid::new_v4();

        let result = delete_node(State(app_state), Path(non_existent_id)).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_list_nodes_include_status_flag() {
        let app_state = setup_test_app_state().await;
        let _node = create_test_node(&app_state).await;

        let query = ListNodesQuery {
            page: None,
            per_page: None,
            lifecycle: None,
            role: None,
            vendor: None,
            include_status: Some(true),
        };

        let result = list_nodes(State(app_state), Query(query)).await;

        assert!(result.is_ok());
        let Json(ApiResponse { data, success, .. }) = result.unwrap();
        assert!(success);
        assert_eq!(data.data.len(), 1);
        // Status should still be None as the TODO indicates it's not implemented yet
        assert!(data.data[0].status.is_none());
    }

    #[tokio::test]
    async fn test_create_node_with_minimal_data() {
        let app_state = setup_test_app_state().await;
        let request = CreateNodeRequest {
            name: "minimal-node".to_string(),
            domain: None,
            vendor: Vendor::Generic,
            model: "Unknown".to_string(),
            role: DeviceRole::Server,
            lifecycle: Lifecycle::Planned,
            location_id: None,
            management_ip: None,
            custom_data: None,
        };

        let result = create_node(State(app_state.clone()), Json(request)).await;

        assert!(result.is_ok());
        let Json(ApiResponse { data, success, .. }) = result.unwrap();
        assert!(success);
        assert_eq!(data.node.name, "minimal-node");
        assert_eq!(data.node.domain, "");
        assert_eq!(data.node.vendor, Vendor::Generic);
        assert_eq!(data.node.management_ip, None);
        assert_eq!(data.node.custom_data, serde_json::Value::Null);
    }

    #[tokio::test]
    async fn test_list_nodes_lifecycle_filter() {
        let app_state = setup_test_app_state().await;

        // Create nodes with different lifecycles
        let mut prod_node = Node::new(
            "prod-node".to_string(),
            "example.com".to_string(),
            Vendor::Cisco,
            DeviceRole::Router,
        );
        prod_node.lifecycle = Lifecycle::Live;

        let mut staging_node = Node::new(
            "staging-node".to_string(),
            "example.com".to_string(),
            Vendor::Cisco,
            DeviceRole::Router,
        );
        staging_node.lifecycle = Lifecycle::Implementing;

        app_state.datastore.create_node(&prod_node).await.unwrap();
        app_state
            .datastore
            .create_node(&staging_node)
            .await
            .unwrap();

        let query = ListNodesQuery {
            page: None,
            per_page: None,
            lifecycle: Some("live".to_string()),
            role: None,
            vendor: None,
            include_status: None,
        };

        let result = list_nodes(State(app_state), Query(query)).await;

        assert!(result.is_ok());
        let Json(ApiResponse { data, success, .. }) = result.unwrap();
        assert!(success);
        assert_eq!(data.data.len(), 1);
        assert_eq!(data.data[0].node.name, "prod-node");
        assert_eq!(data.data[0].node.lifecycle, Lifecycle::Live);
    }

    // Error handling tests using mock DataStore
    #[tokio::test]
    async fn test_get_node_internal_error() {
        use std::sync::Arc;
        use unet_core::datastore::{DataStoreError, MockDataStore};

        let mut mock_datastore = MockDataStore::new();
        mock_datastore.expect_get_node_required().returning(|_| {
            Box::pin(async move {
                Err(DataStoreError::InternalError {
                    message: "Internal database error".to_string(),
                })
            })
        });

        let app_state = AppState {
            datastore: Arc::new(mock_datastore),
            policy_service: PolicyService::new(Config::default().git),
        };

        let node_id = Uuid::new_v4();
        let result = get_node(State(app_state), Path(node_id)).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Internal database error"));
    }

    #[tokio::test]
    async fn test_update_node_internal_error() {
        use std::sync::Arc;
        use unet_core::datastore::{DataStoreError, MockDataStore};

        let mut mock_datastore = MockDataStore::new();
        mock_datastore.expect_get_node_required().returning(|_| {
            Box::pin(async move {
                Err(DataStoreError::ConnectionError {
                    message: "Database connection failed".to_string(),
                })
            })
        });

        let app_state = AppState {
            datastore: Arc::new(mock_datastore),
            policy_service: PolicyService::new(Config::default().git),
        };

        let node_id = Uuid::new_v4();
        let request = UpdateNodeRequest {
            name: Some("updated-name".to_string()),
            domain: None,
            vendor: None,
            model: None,
            role: None,
            lifecycle: None,
            location_id: None,
            management_ip: None,
            custom_data: None,
        };

        let result = update_node(State(app_state), Path(node_id), Json(request)).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Database connection failed"));
    }

    #[tokio::test]
    async fn test_delete_node_internal_error() {
        use std::sync::Arc;
        use unet_core::datastore::{DataStoreError, MockDataStore};

        let mut mock_datastore = MockDataStore::new();
        mock_datastore.expect_delete_node().returning(|_| {
            Box::pin(async move {
                Err(DataStoreError::TransactionError {
                    message: "Transaction failed".to_string(),
                })
            })
        });

        let app_state = AppState {
            datastore: Arc::new(mock_datastore),
            policy_service: PolicyService::new(Config::default().git),
        };

        let node_id = Uuid::new_v4();
        let result = delete_node(State(app_state), Path(node_id)).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Transaction failed"));
    }

    #[tokio::test]
    async fn test_create_node_internal_error() {
        use std::sync::Arc;
        use unet_core::datastore::{DataStoreError, MockDataStore};

        let mut mock_datastore = MockDataStore::new();
        mock_datastore.expect_create_node().returning(|_| {
            Box::pin(async move {
                Err(DataStoreError::ConstraintViolation {
                    message: "Constraint violation".to_string(),
                })
            })
        });

        let app_state = AppState {
            datastore: Arc::new(mock_datastore),
            policy_service: PolicyService::new(Config::default().git),
        };

        let request = CreateNodeRequest {
            name: "test-node".to_string(),
            domain: Some("example.com".to_string()),
            vendor: Vendor::Cisco,
            model: "ISR4321".to_string(),
            role: DeviceRole::Router,
            lifecycle: Lifecycle::Live,
            location_id: None,
            management_ip: None,
            custom_data: None,
        };

        let result = create_node(State(app_state), Json(request)).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Constraint violation"));
    }

    #[tokio::test]
    async fn test_update_node_with_location_id() {
        let app_state = setup_test_app_state().await;
        let node = create_test_node(&app_state).await;
        let location_id = Uuid::new_v4();

        let request = UpdateNodeRequest {
            name: None,
            domain: None,
            vendor: None,
            model: None,
            role: None,
            lifecycle: None,
            location_id: Some(location_id),
            management_ip: None,
            custom_data: None,
        };

        let result = update_node(State(app_state), Path(node.id), Json(request)).await;

        assert!(result.is_ok());
        let Json(ApiResponse { data, success, .. }) = result.unwrap();
        assert!(success);
        assert_eq!(data.node.location_id, Some(location_id));
    }
}
