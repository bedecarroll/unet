//! Monitoring and status operations for nodes

use anyhow::Result;
use unet_core::datastore::DataStore;

use super::types::{MetricsNodeArgs, MonitorNodeArgs, StatusNodeArgs, StatusType};

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

pub fn monitor_node(
    _args: &MonitorNodeArgs,
    _datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    // For now, return a placeholder message
    let output = serde_json::json!({
        "status": "Not implemented",
        "message": "Real-time monitoring not yet implemented",
        "implementation_note": "This would continuously poll and display derived state changes",
        "available_when": "SQLite datastore implementation complete (M2.5.8)"
    });

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
