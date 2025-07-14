//! CRUD operations for node management

use anyhow::Result;
use serde_json::Value as JsonValue;
use unet_core::datastore::DataStore;
use unet_core::prelude::*;

use super::types::{AddNodeArgs, DeleteNodeArgs, ListNodeArgs, ShowNodeArgs, UpdateNodeArgs};

pub async fn add_node(
    args: AddNodeArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    // Parse vendor and role
    let vendor = args
        .vendor
        .parse::<Vendor>()
        .map_err(|e| anyhow::anyhow!("Invalid vendor '{}': {}", args.vendor, e))?;

    let role = args
        .role
        .parse::<DeviceRole>()
        .map_err(|e| anyhow::anyhow!("Invalid role '{}': {}", args.role, e))?;

    let lifecycle = args
        .lifecycle
        .parse::<Lifecycle>()
        .map_err(|e| anyhow::anyhow!("Invalid lifecycle '{}': {}", args.lifecycle, e))?;

    // Parse custom data if provided
    let custom_data = if let Some(json_str) = args.custom_data {
        Some(serde_json::from_str::<JsonValue>(&json_str)?)
    } else {
        None
    };

    // Build node
    let mut builder = NodeBuilder::new()
        .name(args.name)
        .domain(args.domain)
        .vendor(vendor)
        .model(args.model)
        .role(role)
        .lifecycle(lifecycle);

    if let Some(location_id) = args.location_id {
        builder = builder.location_id(location_id);
    }

    if let Some(management_ip_str) = args.management_ip {
        let management_ip = management_ip_str
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid management IP '{}': {}", management_ip_str, e))?;
        builder = builder.management_ip(management_ip);
    }

    if let Some(custom_data) = custom_data {
        builder = builder.custom_data(custom_data);
    }

    let node = builder
        .build()
        .map_err(|e| anyhow::anyhow!("Node validation failed: {}", e))?;

    // Create node in datastore
    let created_node = datastore.create_node(&node).await?;

    crate::commands::print_output(&created_node, output_format)?;

    Ok(())
}

pub async fn list_nodes(
    args: ListNodeArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    let mut filters = Vec::new();

    if let Some(lifecycle) = args.lifecycle {
        filters.push(Filter {
            field: "lifecycle".to_owned(),
            operation: FilterOperation::Equals,
            value: FilterValue::String(lifecycle),
        });
    }

    if let Some(role) = args.role {
        filters.push(Filter {
            field: "role".to_owned(),
            operation: FilterOperation::Equals,
            value: FilterValue::String(role),
        });
    }

    if let Some(vendor) = args.vendor {
        filters.push(Filter {
            field: "vendor".to_owned(),
            operation: FilterOperation::Equals,
            value: FilterValue::String(vendor),
        });
    }

    let options = QueryOptions {
        filters,
        sort: vec![Sort {
            field: "name".to_owned(),
            direction: SortDirection::Ascending,
        }],
        pagination: Some(Pagination {
            offset: usize::try_from((args.page - 1) * args.per_page)?,
            limit: usize::try_from(args.per_page)?,
        }),
    };

    let result = datastore.list_nodes(&options).await?;

    crate::commands::print_output(&result, output_format)?;

    Ok(())
}

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

pub async fn update_node(
    args: UpdateNodeArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    let mut node = datastore.get_node_required(&args.id).await?;

    // Update fields that were provided
    if let Some(name) = args.name {
        node.name = name;
    }

    if let Some(domain) = args.domain {
        node.domain = domain;
        node.fqdn = format!("{}.{}", node.name, node.domain);
    }

    if let Some(vendor_str) = args.vendor {
        node.vendor = vendor_str
            .parse::<Vendor>()
            .map_err(|e| anyhow::anyhow!("Invalid vendor '{}': {}", vendor_str, e))?;
    }

    if let Some(model) = args.model {
        node.model = model;
    }

    if let Some(role_str) = args.role {
        node.role = role_str
            .parse::<DeviceRole>()
            .map_err(|e| anyhow::anyhow!("Invalid role '{}': {}", role_str, e))?;
    }

    if let Some(lifecycle_str) = args.lifecycle {
        node.lifecycle = lifecycle_str
            .parse::<Lifecycle>()
            .map_err(|e| anyhow::anyhow!("Invalid lifecycle '{}': {}", lifecycle_str, e))?;
    }

    if let Some(location_id) = args.location_id {
        node.location_id = Some(location_id);
    }

    if let Some(management_ip_str) = args.management_ip {
        let management_ip = management_ip_str
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid management IP '{}': {}", management_ip_str, e))?;
        node.management_ip = Some(management_ip);
    }

    if let Some(custom_data_str) = args.custom_data {
        let custom_data = serde_json::from_str::<JsonValue>(&custom_data_str)?;
        node.custom_data = custom_data;
    }

    let updated_node = datastore.update_node(&node).await?;

    crate::commands::print_output(&updated_node, output_format)?;

    Ok(())
}

pub async fn delete_node(
    args: DeleteNodeArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    // Get node first to show confirmation
    let node = datastore.get_node_required(&args.id).await?;

    if !args.yes {
        eprintln!(
            "Are you sure you want to delete node '{}'? [y/N]",
            node.name
        );
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();
        if input != "y" && input != "yes" {
            eprintln!("Cancelled");
            return Ok(());
        }
    }

    datastore.delete_node(&args.id).await?;

    let output = serde_json::json!({
        "message": format!("Node '{}' ({}) deleted successfully", node.name, node.id),
        "node_id": node.id,
        "node_name": node.name
    });

    crate::commands::print_output(&output, output_format)?;

    Ok(())
}
