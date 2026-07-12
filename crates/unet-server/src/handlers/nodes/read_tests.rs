//! Tests for node read/list operations

#[cfg(test)]
mod tests {
    use super::super::crud::*;
    use crate::api::ApiResponse;
    use crate::handlers::nodes::crud_tests::test_utils::*;
    use crate::handlers::nodes::types::ListNodesQuery;
    use axum::{
        extract::{Path, Query, State},
        response::Json,
    };
    use unet_core::models::{DeviceRole, Lifecycle, Node, Vendor};
    use uuid::Uuid;

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
    async fn test_list_nodes_lifecycle_filter() {
        let app_state = setup_test_app_state().await;

        // Create nodes with different lifecycles
        let mut live_node = Node::new(
            "live-router".to_string(),
            "live.example.com".to_string(),
            Vendor::Cisco,
            DeviceRole::Router,
        );
        live_node.lifecycle = Lifecycle::Live;

        let mut dev_node = Node::new(
            "dev-router".to_string(),
            "dev.example.com".to_string(),
            Vendor::Cisco,
            DeviceRole::Router,
        );
        dev_node.lifecycle = Lifecycle::Implementing;

        app_state.datastore.create_node(&live_node).await.unwrap();
        app_state.datastore.create_node(&dev_node).await.unwrap();

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
        assert_eq!(data.data[0].node.name, "live-router");
        // Note: Node::new creates nodes with default lifecycle, so we'd need to update them
        // This test would need adjustment based on actual API behavior
    }
}
