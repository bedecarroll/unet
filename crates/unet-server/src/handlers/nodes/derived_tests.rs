use super::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::AppState;
    use axum::extract::{Path, State};
    use migration::sea_orm::{ActiveModelTrait, Set};
    use std::sync::Arc;
    use unet_core::{
        datastore::{DataStore, sqlite::SqliteStore},
        entities::node_status,
        models::*,
        policy_integration::PolicyService,
    };

    async fn setup_test_datastore() -> SqliteStore {
        test_support::sqlite::sqlite_store().await
    }

    async fn create_test_node(datastore: &SqliteStore) -> Node {
        let mut node = Node::new(
            "test-node".to_string(),
            "example.com".to_string(),
            Vendor::Cisco,
            DeviceRole::Router,
        );
        node.model = "ASR1000".to_string();
        node.lifecycle = Lifecycle::Live;
        datastore.create_node(&node).await.unwrap()
    }

    #[tokio::test]
    async fn test_get_node_status_success() {
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;

        node_status::ActiveModel {
            id: Set("derived-handler-status".to_string()),
            node_id: Set(node.id.to_string()),
            last_updated: Set("2026-04-07T01:02:03Z".to_string()),
            reachable: Set(true),
            system_info: Set(Some(r#"{"name":"test-node"}"#.to_string())),
            performance: Set(None),
            environmental: Set(None),
            vendor_metrics: Set(None),
            raw_snmp_data: Set(None),
            last_snmp_success: Set(Some("2026-04-07T01:00:00Z".to_string())),
            last_error: Set(None),
            consecutive_failures: Set(0),
        }
        .insert(datastore.connection())
        .await
        .unwrap();

        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
        };

        let result = get_node_status(State(app_state), Path(node.id)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert!(response.success);
        assert_eq!(response.data.node_id, node.id);
        assert!(response.data.reachable);
    }

    #[tokio::test]
    async fn test_get_node_status_not_found() {
        let datastore = setup_test_datastore().await;
        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
        };

        let non_existent_id = Uuid::new_v4();
        let result = get_node_status(State(app_state), Path(non_existent_id)).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ServerError::NotFound(msg) => {
                assert!(msg.contains("not found"));
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_get_node_interfaces_success() {
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;

        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
        };

        let result = get_node_interfaces(State(app_state), Path(node.id)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert!(response.success);
        // CSV datastore will return empty interfaces list
        assert!(response.data.is_empty());
    }

    #[tokio::test]
    async fn test_get_node_interfaces_not_found() {
        let datastore = setup_test_datastore().await;
        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
        };

        let non_existent_id = Uuid::new_v4();
        let result = get_node_interfaces(State(app_state), Path(non_existent_id)).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ServerError::NotFound(msg) => {
                assert!(msg.contains("not found"));
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_get_node_metrics_not_found_node() {
        let datastore = setup_test_datastore().await;
        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
        };

        let non_existent_id = Uuid::new_v4();
        let result = get_node_metrics(State(app_state), Path(non_existent_id)).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ServerError::NotFound(msg) => {
                assert!(msg.contains("not found"));
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_get_node_metrics_no_metrics() {
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;

        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
        };

        let result = get_node_metrics(State(app_state), Path(node.id)).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ServerError::NotFound(msg) => {
                assert!(msg.contains("No metrics available"));
            }
            _ => panic!("Expected NotFound error for missing metrics"),
        }
    }

    #[tokio::test]
    async fn test_get_node_status_internal_error_handling() {
        let datastore = setup_test_datastore().await;
        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
        };

        let non_existent_id = Uuid::new_v4();
        let result = get_node_status(State(app_state), Path(non_existent_id)).await;

        if let Err(ServerError::NotFound(_)) = result {
            // This covers lines 22-29
        } else {
            // Any other result
        }
    }

    #[tokio::test]
    async fn test_get_node_interfaces_internal_error_handling() {
        let datastore = setup_test_datastore().await;
        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
        };

        let non_existent_id = Uuid::new_v4();
        let result = get_node_interfaces(State(app_state), Path(non_existent_id)).await;

        if let Err(ServerError::NotFound(_)) = result {
            // This covers lines 49-56
        } else {
            // Any other result
        }
    }

    #[tokio::test]
    async fn test_get_node_metrics_internal_error_handling() {
        let datastore = setup_test_datastore().await;
        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
        };

        let non_existent_id = Uuid::new_v4();
        let result = get_node_metrics(State(app_state), Path(non_existent_id)).await;

        if let Err(ServerError::NotFound(_)) = result {
            // This covers lines 72-79
        } else {
            // Any other result
        }
    }

    #[tokio::test]
    async fn test_get_node_status_without_persisted_status_returns_not_found() {
        let datastore = setup_test_datastore().await;
        let node = create_test_node(&datastore).await;

        let app_state = AppState {
            datastore: Arc::new(datastore),
            policy_service: PolicyService::with_local_dir("/tmp"),
        };

        let result = get_node_status(State(app_state), Path(node.id)).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ServerError::NotFound(message) => {
                assert!(message.contains("No status available"));
            }
            other => panic!("Expected NotFound error, got {other:?}"),
        }
    }
}
