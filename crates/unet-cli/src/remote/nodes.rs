//! Remote execution for node commands.

use anyhow::Result;
use reqwest::Method;
use serde_json::json;
use unet_core::{
    datastore::PagedResult,
    models::{DeviceRole, Lifecycle, Vendor},
};

use crate::{
    OutputFormat,
    commands::nodes::{NodeCommands, types::StatusType},
};

use super::{
    RemoteClient,
    node_api::{
        RemoteNodeResponse, RemotePage, fetch_interfaces, fetch_metrics, fetch_node, fetch_status,
        parse_value,
    },
    parse_json_arg, print_remote_output,
};

pub(super) async fn dispatch(
    command: NodeCommands,
    client: &RemoteClient,
    output: OutputFormat,
) -> Result<()> {
    match command {
        NodeCommands::Add(args) => add(args, client, output).await,
        NodeCommands::List(args) => list(args, client, output).await,
        NodeCommands::Show(args) => show(args, client, output).await,
        NodeCommands::Update(args) => update(args, client, output).await,
        NodeCommands::Delete(args) => delete(args, client, output).await,
        NodeCommands::Status(args) => status(args, client, output).await,
        NodeCommands::Metrics(args) => metrics(args, client, output).await,
        NodeCommands::Compare(_) | NodeCommands::Polling(_) | NodeCommands::History(_) => {
            Err(anyhow::anyhow!(
                "Remote mode does not support compare, polling, or history node commands yet"
            ))
        }
    }
}

async fn list(
    args: crate::commands::nodes::types::ListNodeArgs,
    client: &RemoteClient,
    output: OutputFormat,
) -> Result<()> {
    let mut request = client.request(Method::GET, "/api/v1/nodes").query(&[
        ("page", args.page.to_string()),
        ("per_page", args.per_page.to_string()),
    ]);

    if let Some(vendor) = args.vendor {
        request = request.query(&[("vendor", vendor)]);
    }
    if let Some(role) = args.role {
        request = request.query(&[("role", role)]);
    }
    if let Some(lifecycle) = args.lifecycle {
        request = request.query(&[("lifecycle", lifecycle)]);
    }

    let response: RemotePage<RemoteNodeResponse> = client.send(request).await?;
    let paged = PagedResult {
        items: response.data.into_iter().map(|node| node.node).collect(),
        total_count: usize::try_from(response.total)?,
        page_size: usize::try_from(response.per_page)?,
        page: usize::try_from(response.page)?,
        total_pages: usize::try_from(response.total_pages)?,
        has_next: response.has_next,
        has_previous: response.has_prev,
    };

    print_remote_output(&paged, output)
}

async fn add(
    args: crate::commands::nodes::types::AddNodeArgs,
    client: &RemoteClient,
    output: OutputFormat,
) -> Result<()> {
    let vendor = parse_value::<Vendor>(&args.vendor, "vendor")?;
    let role = parse_value::<DeviceRole>(&args.role, "role")?;
    let lifecycle = parse_value::<Lifecycle>(&args.lifecycle, "lifecycle")?;
    let custom_data = parse_json_arg(args.custom_data)?;

    let payload = json!({
        "name": args.name,
        "domain": args.domain,
        "vendor": vendor,
        "model": args.model,
        "role": role,
        "lifecycle": lifecycle,
        "location_id": args.location_id,
        "management_ip": args.management_ip,
        "custom_data": custom_data,
    });

    let response: RemoteNodeResponse = client
        .send(client.request(Method::POST, "/api/v1/nodes").json(&payload))
        .await?;
    print_remote_output(&response.node, output)
}

async fn show(
    args: crate::commands::nodes::types::ShowNodeArgs,
    client: &RemoteClient,
    output: OutputFormat,
) -> Result<()> {
    let node = fetch_node(client, args.id).await?;
    if !args.include_status && !args.show_interfaces && !args.show_system_info {
        return print_remote_output(&node.node, output);
    }

    let mut response = json!({ "node": node.node, "derived_state": {} });
    if args.include_status {
        response["derived_state"]["status"] =
            serde_json::to_value(fetch_status(client, args.id).await?)?;
    }
    if args.show_interfaces {
        response["derived_state"]["interfaces"] =
            serde_json::to_value(fetch_interfaces(client, args.id).await?)?;
    }
    if args.show_system_info {
        response["derived_state"]["system_info"] =
            serde_json::to_value(fetch_status(client, args.id).await?.system_info)?;
    }

    print_remote_output(&response, output)
}

async fn update(
    args: crate::commands::nodes::types::UpdateNodeArgs,
    client: &RemoteClient,
    output: OutputFormat,
) -> Result<()> {
    let custom_data = parse_json_arg(args.custom_data)?;
    let payload = json!({
        "name": args.name,
        "domain": args.domain,
        "vendor": args.vendor,
        "model": args.model,
        "role": args.role,
        "lifecycle": args.lifecycle,
        "location_id": args.location_id,
        "management_ip": args.management_ip,
        "custom_data": custom_data,
    });

    let response: RemoteNodeResponse = client
        .send(
            client
                .request(Method::PUT, &format!("/api/v1/nodes/{}", args.id))
                .json(&payload),
        )
        .await?;
    print_remote_output(&response.node, output)
}

async fn delete(
    args: crate::commands::nodes::types::DeleteNodeArgs,
    client: &RemoteClient,
    output: OutputFormat,
) -> Result<()> {
    let node = fetch_node(client, args.id).await?;

    if !args.yes {
        eprintln!(
            "Are you sure you want to delete node '{}' ? [y/N]",
            node.node.name
        );
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let confirmation = input.trim().to_lowercase();
        if confirmation != "y" && confirmation != "yes" {
            eprintln!("Cancelled");
            return Ok(());
        }
    }

    let _: () = client
        .send(client.request(Method::DELETE, &format!("/api/v1/nodes/{}", args.id)))
        .await?;

    let result = json!({
        "message": format!("Node '{}' ({}) deleted successfully", node.node.name, node.node.id),
        "node_id": node.node.id,
        "node_name": node.node.name,
    });
    print_remote_output(&result, output)
}

async fn status(
    args: crate::commands::nodes::types::StatusNodeArgs,
    client: &RemoteClient,
    output: OutputFormat,
) -> Result<()> {
    let node = fetch_node(client, args.id).await?.node;
    let mut response = json!({
        "node_id": args.id,
        "node_name": node.name,
        "status_types": args.status_type,
    });

    for status_type in args.status_type {
        match status_type {
            StatusType::Basic => response["basic"] = serde_json::to_value(&node)?,
            StatusType::All => {
                response["node"] = serde_json::to_value(&node)?;
                response["status"] = serde_json::to_value(fetch_status(client, args.id).await?)?;
                response["interfaces"] =
                    serde_json::to_value(fetch_interfaces(client, args.id).await?)?;
            }
            StatusType::Interfaces => {
                response["interfaces"] =
                    serde_json::to_value(fetch_interfaces(client, args.id).await?)?;
            }
            StatusType::System => {
                response["system_info"] =
                    serde_json::to_value(fetch_status(client, args.id).await?.system_info)?;
            }
            StatusType::Polling => {
                response["polling"] = json!({
                    "message": "Polling task status display not yet implemented",
                    "note": "This will show SNMP polling task status for the node",
                });
            }
        }
    }

    print_remote_output(&response, output)
}

async fn metrics(
    args: crate::commands::nodes::types::MetricsNodeArgs,
    client: &RemoteClient,
    output: OutputFormat,
) -> Result<()> {
    let node = fetch_node(client, args.id).await?.node;
    let mut response = json!({
        "node_id": args.id,
        "node_name": node.name,
        "detailed": args.detailed,
        "historical": args.history,
    });

    if args.history {
        response["message"] = json!({
            "message": "Historical metrics display not yet implemented",
            "note": "This requires time-series data storage implementation",
        });
        return print_remote_output(&response, output);
    }

    match fetch_metrics(client, args.id).await {
        Ok(Some(metrics)) => {
            response["metrics"] = serde_json::to_value(&metrics)?;
            if args.detailed {
                let interfaces = fetch_interfaces(client, args.id).await?;
                response["interface_metrics"] = serde_json::Value::Array(
                    interfaces
                        .into_iter()
                        .map(|interface| {
                            json!({
                                "name": interface.name,
                                "admin_status": interface.admin_status,
                                "oper_status": interface.oper_status,
                            })
                        })
                        .collect(),
                );
            }
        }
        Ok(None) => {
            response["metrics"] = json!({
                "message": "No metrics available for this node",
                "note": "Node has not been polled yet or SNMP polling is not configured",
            });
        }
        Err(error) => {
            response["error"] = json!({ "message": error.to_string() });
        }
    }

    print_remote_output(&response, output)
}
