/// Node display operations
use anyhow::Result;
use unet_core::datastore::DataStore;

use super::types::ShowNodeArgs;

pub async fn show_node(
    args: ShowNodeArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    let node = datastore.get_node_required(&args.id).await?;

    if args.include_status || args.show_interfaces || args.show_system_info {
        // Create enhanced output with derived state
        let mut output = serde_json::json!({
            "node": node,
            "derived_state": {}
        });

        // Fetch actual derived state data
        if args.include_status {
            match datastore.get_node_status(&args.id).await {
                Ok(Some(status)) => {
                    output["derived_state"]["status"] = serde_json::to_value(&status)?;
                }
                Ok(None) => {
                    output["derived_state"]["status"] = serde_json::json!({
                        "message": "No status data available",
                        "note": "Node has not been polled yet or polling is not configured"
                    });
                }
                Err(e) => {
                    output["derived_state"]["status"] = serde_json::json!({
                        "error": format!("Failed to fetch status: {}", e)
                    });
                }
            }
        }

        if args.show_interfaces {
            match datastore.get_node_interfaces(&args.id).await {
                Ok(interfaces) => {
                    output["derived_state"]["interfaces"] = serde_json::to_value(&interfaces)?;
                }
                Err(e) => {
                    output["derived_state"]["interfaces"] = serde_json::json!({
                        "error": format!("Failed to fetch interfaces: {}", e)
                    });
                }
            }
        }

        if args.show_system_info {
            // Get system info from node status
            match datastore.get_node_status(&args.id).await {
                Ok(Some(status)) => {
                    output["derived_state"]["system_info"] =
                        serde_json::to_value(&status.system_info)?;
                }
                Ok(None) => {
                    output["derived_state"]["system_info"] = serde_json::json!({
                        "message": "No system information available",
                        "note": "Node has not been polled yet or SNMP polling is not configured"
                    });
                }
                Err(e) => {
                    output["derived_state"]["system_info"] = serde_json::json!({
                        "error": format!("Failed to fetch system info: {}", e)
                    });
                }
            }
        }

        crate::commands::print_output(&output, output_format)?;
    } else {
        // Standard node display
        crate::commands::print_output(&node, output_format)?;
    }

    Ok(())
}
