/// Execution tests for node polling command
#[cfg(test)]
mod tests {
    use super::super::polling::{polling_node, polling_status_value};
    use super::super::types::{PollingAction, PollingNodeArgs};
    use crate::OutputFormat;
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::time::Duration;
    use unet_core::datastore::DataStore;
    use unet_core::snmp::{PollingTask, SessionConfig};
    use uuid::Uuid;

    fn make_node() -> unet_core::models::Node {
        use unet_core::models::*;
        NodeBuilder::new()
            .id(Uuid::new_v4())
            .name("edge-1")
            .domain("example.com")
            .vendor(Vendor::Cisco)
            .model("ISR4321")
            .role(DeviceRole::Router)
            .lifecycle(Lifecycle::Live)
            .management_ip(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 20)))
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn test_polling_start_creates_persisted_task() {
        test_support::sqlite::with_savepoint(
            "polling_start_creates_persisted_task",
            |store| async move {
                let node = make_node();
                store.create_node(&node).await.unwrap();

                let args = PollingNodeArgs {
                    id: node.id,
                    action: PollingAction::Start,
                    detailed: false,
                };

                let result = polling_node(args, &store, OutputFormat::Json).await;
                assert!(result.is_ok());

                let task = unet_core::entities::polling_tasks::Entity::find()
                    .filter(
                        unet_core::entities::polling_tasks::Column::NodeId.eq(node.id.to_string()),
                    )
                    .one(store.connection())
                    .await
                    .unwrap();

                assert!(task.is_some());
                assert!(task.unwrap().enabled);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn test_polling_stop_disables_existing_task() {
        test_support::sqlite::with_savepoint(
            "polling_stop_disables_existing_task",
            |store| async move {
                let node = make_node();
                store.create_node(&node).await.unwrap();

                polling_node(
                    PollingNodeArgs {
                        id: node.id,
                        action: PollingAction::Start,
                        detailed: false,
                    },
                    &store,
                    OutputFormat::Json,
                )
                .await
                .unwrap();

                polling_node(
                    PollingNodeArgs {
                        id: node.id,
                        action: PollingAction::Stop,
                        detailed: false,
                    },
                    &store,
                    OutputFormat::Json,
                )
                .await
                .unwrap();

                let task = unet_core::entities::polling_tasks::Entity::find()
                    .filter(
                        unet_core::entities::polling_tasks::Column::NodeId.eq(node.id.to_string()),
                    )
                    .one(store.connection())
                    .await
                    .unwrap()
                    .unwrap();

                assert!(!task.enabled);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn test_polling_restart_reenables_existing_task() {
        test_support::sqlite::with_savepoint(
            "polling_restart_reenables_existing_task",
            |store| async move {
                let node = make_node();
                store.create_node(&node).await.unwrap();

                polling_node(
                    PollingNodeArgs {
                        id: node.id,
                        action: PollingAction::Start,
                        detailed: false,
                    },
                    &store,
                    OutputFormat::Json,
                )
                .await
                .unwrap();

                polling_node(
                    PollingNodeArgs {
                        id: node.id,
                        action: PollingAction::Stop,
                        detailed: false,
                    },
                    &store,
                    OutputFormat::Json,
                )
                .await
                .unwrap();

                polling_node(
                    PollingNodeArgs {
                        id: node.id,
                        action: PollingAction::Restart,
                        detailed: false,
                    },
                    &store,
                    OutputFormat::Json,
                )
                .await
                .unwrap();

                let task = unet_core::entities::polling_tasks::Entity::find()
                    .filter(
                        unet_core::entities::polling_tasks::Column::NodeId.eq(node.id.to_string()),
                    )
                    .one(store.connection())
                    .await
                    .unwrap()
                    .unwrap();

                assert!(task.enabled);
            },
        )
        .await;
    }

    #[test]
    fn test_polling_status_value_reports_missing_task() {
        let status = polling_status_value(None, false);

        assert_eq!(status["state"], "not_configured");
        assert_eq!(status["enabled"], false);
        assert!(status.get("oids").is_none());
    }

    #[test]
    fn test_polling_status_value_reports_enabled_task() {
        let task = PollingTask::new(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 20)), 161),
            Uuid::new_v4(),
            vec!["1.3.6.1.2.1.1.1.0".to_string()],
            Duration::from_secs(300),
            SessionConfig::default(),
        );

        let status = polling_status_value(Some(&task), true);

        assert_eq!(status["state"], "enabled");
        assert_eq!(status["enabled"], true);
        assert_eq!(status["target"], "192.0.2.20:161");
        assert_eq!(status["oid_count"], 1);
        assert_eq!(status["interval_seconds"], 300);
        assert_eq!(status["oids"][0], "1.3.6.1.2.1.1.1.0");
    }
}
