//! Tests for node list operations

#[cfg(test)]
mod tests {
    use crate::api::ApiResponse;
    use crate::handlers::nodes::crud::*;
    use crate::handlers::nodes::crud_tests::test_utils::*;
    use crate::handlers::nodes::types::ListNodesQuery;
    use axum::{
        extract::{Query, State},
        response::Json,
    };
    use migration::sea_orm::{ActiveModelTrait, Set};
    use unet_core::entities::node_status;
    use unet_core::models::{DeviceRole, Node, Vendor};

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
    async fn test_list_nodes_include_status_flag() {
        let (connection, app_state) = setup_test_app_state_with_connection().await;
        let node = create_test_node(&app_state).await;

        node_status::ActiveModel {
            id: Set("node-list-status".to_string()),
            node_id: Set(node.id.to_string()),
            last_updated: Set("2026-04-07T01:02:03Z".to_string()),
            reachable: Set(true),
            system_info: Set(Some(
                r#"{"description":"Listed router","name":"test-node"}"#.to_string(),
            )),
            performance: Set(None),
            environmental: Set(None),
            vendor_metrics: Set(None),
            raw_snmp_data: Set(None),
            last_snmp_success: Set(Some("2026-04-07T01:00:00Z".to_string())),
            last_error: Set(None),
            consecutive_failures: Set(0),
        }
        .insert(&connection)
        .await
        .unwrap();

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
        assert!(data.data[0].status.is_some());
        let status = data.data[0].status.as_ref().unwrap();
        assert!(status.reachable);
        assert_eq!(
            status.system_info.as_ref().unwrap().name.as_deref(),
            Some("test-node")
        );
    }

    #[tokio::test]
    async fn test_list_nodes_lifecycle_filter() {
        let app_state = setup_test_app_state().await;

        // Create nodes with different lifecycles
        let mut live_node = Node::new(
            "live-router".to_string(),
            "example.com".to_string(),
            Vendor::Cisco,
            DeviceRole::Router,
        );
        live_node.lifecycle = unet_core::models::Lifecycle::Live;

        let mut implementing_node = Node::new(
            "implementing-switch".to_string(),
            "example.com".to_string(),
            Vendor::Juniper,
            DeviceRole::Switch,
        );
        implementing_node.lifecycle = unet_core::models::Lifecycle::Implementing;

        app_state.datastore.create_node(&live_node).await.unwrap();
        app_state
            .datastore
            .create_node(&implementing_node)
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
        assert_eq!(data.data[0].node.name, "live-router");
        assert_eq!(
            data.data[0].node.lifecycle,
            unet_core::models::Lifecycle::Live
        );
    }
}
