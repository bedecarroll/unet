/// Node creation operations
use anyhow::Result;
use serde_json::Value as JsonValue;
use unet_core::datastore::DataStore;
use unet_core::prelude::*;

use super::types::AddNodeArgs;

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
