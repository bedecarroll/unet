/// Monitoring and status operations for nodes
use anyhow::Result;
use unet_core::datastore::DataStore;

use super::types::{MetricsNodeArgs, StatusNodeArgs, StatusType};

pub async fn status_node(
    args: StatusNodeArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    // Verify node exists first
    let node = datastore.get_node_required(&args.id).await?;

    let mut output = serde_json::json!({
        "node_id": args.id,
        "node_name": node.name,
        "status_types": args.status_type
    });

    for status_type in &args.status_type {
        match status_type {
            StatusType::Basic => {
                output["basic"] = serde_json::to_value(&node)?;
            }
            StatusType::All => {
                // Fetch all available status information
                output["node"] = serde_json::to_value(&node)?;

                if let Ok(Some(status)) = datastore.get_node_status(&args.id).await {
                    output["status"] = serde_json::to_value(&status)?;
                }

                if let Ok(interfaces) = datastore.get_node_interfaces(&args.id).await {
                    output["interfaces"] = serde_json::to_value(&interfaces)?;
                }
            }
            StatusType::Interfaces => match datastore.get_node_interfaces(&args.id).await {
                Ok(interfaces) => {
                    output["interfaces"] = serde_json::to_value(&interfaces)?;
                }
                Err(e) => {
                    output["interfaces"] = serde_json::json!({
                        "error": format!("Failed to fetch interfaces: {}", e)
                    });
                }
            },
            StatusType::System => match datastore.get_node_status(&args.id).await {
                Ok(Some(status)) => {
                    output["system_info"] = serde_json::to_value(&status.system_info)?;
                }
                Ok(None) => {
                    output["system_info"] = serde_json::json!({
                        "message": "No system information available"
                    });
                }
                Err(e) => {
                    output["system_info"] = serde_json::json!({
                        "error": format!("Failed to fetch system info: {}", e)
                    });
                }
            },
            StatusType::Polling => {
                // TODO: Add polling task status when implemented
                output["polling"] = serde_json::json!({
                    "message": "Polling task status display not yet implemented",
                    "note": "This will show SNMP polling task status for the node"
                });
            }
        }
    }

    crate::commands::print_output(&output, output_format)?;

    Ok(())
}

pub async fn metrics_node(
    args: MetricsNodeArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    // Verify node exists first
    let node = datastore.get_node_required(&args.id).await?;

    let mut output = serde_json::json!({
        "node_id": args.id,
        "node_name": node.name,
        "detailed": args.detailed,
        "historical": args.history
    });

    if args.history {
        output["message"] = serde_json::json!({
            "message": "Historical metrics display not yet implemented",
            "note": "This requires time-series data storage implementation"
        });
    } else {
        // Get current metrics
        match datastore.get_node_metrics(&args.id).await {
            Ok(Some(metrics)) => {
                output["metrics"] = serde_json::to_value(&metrics)?;

                if args.detailed {
                    // Add detailed breakdown if available
                    if let Ok(interfaces) = datastore.get_node_interfaces(&args.id).await {
                        output["interface_metrics"] = serde_json::Value::Array(
                            interfaces
                                .iter()
                                .map(|iface| {
                                    serde_json::json!({
                                        "name": iface.name,
                                        "admin_status": iface.admin_status,
                                        "oper_status": iface.oper_status
                                    })
                                })
                                .collect::<Vec<_>>(),
                        );
                    }
                }
            }
            Ok(None) => {
                output["metrics"] = serde_json::json!({
                    "message": "No metrics available for this node",
                    "note": "Node has not been polled yet or SNMP polling is not configured"
                });
            }
            Err(e) => {
                output["error"] = serde_json::json!({
                    "message": format!("Failed to fetch metrics: {}", e)
                });
            }
        }
    }

    crate::commands::print_output(&output, output_format)?;

    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::commands::nodes::types::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_status_node_args_creation() {
        let node_id = Uuid::new_v4();

        let args = StatusNodeArgs {
            id: node_id,
            status_type: vec![StatusType::Basic],
        };

        assert_eq!(args.id, node_id);
        assert_eq!(args.status_type, vec![StatusType::Basic]);
    }

    #[tokio::test]
    async fn test_status_node_args_multiple_types() {
        let node_id = Uuid::new_v4();

        let args = StatusNodeArgs {
            id: node_id,
            status_type: vec![
                StatusType::Basic,
                StatusType::Interfaces,
                StatusType::System,
            ],
        };

        assert_eq!(args.id, node_id);
        assert_eq!(args.status_type.len(), 3);
        assert!(args.status_type.contains(&StatusType::Basic));
        assert!(args.status_type.contains(&StatusType::Interfaces));
        assert!(args.status_type.contains(&StatusType::System));
    }

    #[tokio::test]
    async fn test_status_node_args_all_type() {
        let node_id = Uuid::new_v4();

        let args = StatusNodeArgs {
            id: node_id,
            status_type: vec![StatusType::All],
        };

        assert_eq!(args.id, node_id);
        assert_eq!(args.status_type, vec![StatusType::All]);
    }

    #[tokio::test]
    async fn test_metrics_node_args_creation() {
        let node_id = Uuid::new_v4();

        let args = MetricsNodeArgs {
            id: node_id,
            detailed: false,
            history: false,
        };

        assert_eq!(args.id, node_id);
        assert!(!args.detailed);
        assert!(!args.history);
    }

    #[tokio::test]
    async fn test_metrics_node_args_detailed() {
        let node_id = Uuid::new_v4();

        let args = MetricsNodeArgs {
            id: node_id,
            detailed: true,
            history: false,
        };

        assert_eq!(args.id, node_id);
        assert!(args.detailed);
        assert!(!args.history);
    }

    #[tokio::test]
    async fn test_metrics_node_args_history() {
        let node_id = Uuid::new_v4();

        let args = MetricsNodeArgs {
            id: node_id,
            detailed: false,
            history: true,
        };

        assert_eq!(args.id, node_id);
        assert!(!args.detailed);
        assert!(args.history);
    }

    #[tokio::test]
    async fn test_metrics_node_args_detailed_and_history() {
        let node_id = Uuid::new_v4();

        let args = MetricsNodeArgs {
            id: node_id,
            detailed: true,
            history: true,
        };

        assert_eq!(args.id, node_id);
        assert!(args.detailed);
        assert!(args.history);
    }

    #[tokio::test]
    async fn test_status_type_basic() {
        let status_type = StatusType::Basic;
        assert!(matches!(status_type, StatusType::Basic));
    }

    #[tokio::test]
    async fn test_status_type_all() {
        let status_type = StatusType::All;
        assert!(matches!(status_type, StatusType::All));
    }

    #[tokio::test]
    async fn test_status_type_interfaces() {
        let status_type = StatusType::Interfaces;
        assert!(matches!(status_type, StatusType::Interfaces));
    }

    #[tokio::test]
    async fn test_status_type_system() {
        let status_type = StatusType::System;
        assert!(matches!(status_type, StatusType::System));
    }

    #[tokio::test]
    async fn test_status_type_polling() {
        let status_type = StatusType::Polling;
        assert!(matches!(status_type, StatusType::Polling));
    }

    #[tokio::test]
    async fn test_status_type_equality() {
        assert_eq!(StatusType::Basic, StatusType::Basic);
        assert_eq!(StatusType::All, StatusType::All);
        assert_eq!(StatusType::Interfaces, StatusType::Interfaces);
        assert_eq!(StatusType::System, StatusType::System);
        assert_eq!(StatusType::Polling, StatusType::Polling);

        assert_ne!(StatusType::Basic, StatusType::All);
        assert_ne!(StatusType::Interfaces, StatusType::System);
        assert_ne!(StatusType::Polling, StatusType::Basic);
    }

    #[tokio::test]
    async fn test_status_type_clone() {
        let original = StatusType::Basic;
        let cloned = original.clone();
        assert_eq!(original, cloned);

        let original_all = StatusType::All;
        let cloned_all = original_all.clone();
        assert_eq!(original_all, cloned_all);
    }
}
