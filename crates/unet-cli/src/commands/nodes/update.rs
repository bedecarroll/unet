/// Node update operations
use anyhow::Result;
use serde_json::Value as JsonValue;
use unet_core::datastore::DataStore;
use unet_core::prelude::*;

use super::types::UpdateNodeArgs;

pub async fn update_node(
    args: UpdateNodeArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    let mut node = datastore.get_node_required(&args.id).await?;

    // Track if fields affecting FQDN changed
    let mut name_changed = false;
    let mut domain_changed = false;

    // Update fields that were provided
    if let Some(name) = args.name {
        node.name = name;
        name_changed = true;
    }

    if let Some(domain) = args.domain {
        node.domain = domain;
        domain_changed = true;
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

    // Recompute FQDN if name or domain changed
    if name_changed || domain_changed {
        node.fqdn = if node.domain.is_empty() {
            node.name.clone()
        } else {
            format!("{}.{}", node.name, node.domain)
        };
    }

    let updated_node = datastore.update_node(&node).await?;

    crate::commands::print_output(&updated_node, output_format)?;

    Ok(())
}
