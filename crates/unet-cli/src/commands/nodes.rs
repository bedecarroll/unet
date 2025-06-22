use anyhow::Result;
use clap::{Args, Subcommand};
use serde_json::Value as JsonValue;
use unet_core::datastore::DataStore;
use unet_core::prelude::*;
use uuid::Uuid;

#[derive(Subcommand)]
pub enum NodeCommands {
    /// Add a new node
    Add(AddNodeArgs),
    /// List all nodes  
    List(ListNodeArgs),
    /// Show node details
    Show(ShowNodeArgs),
    /// Update an existing node
    Update(UpdateNodeArgs),
    /// Delete a node
    Delete(DeleteNodeArgs),
    /// Show node derived state status
    Status(StatusNodeArgs),
    /// Monitor node derived state changes
    Monitor(MonitorNodeArgs),
    /// Show node performance metrics
    Metrics(MetricsNodeArgs),
    /// Compare derived state between nodes or time periods
    Compare(CompareNodeArgs),
    /// Control and view polling task status
    Polling(PollingNodeArgs),
    /// View derived state history
    History(HistoryNodeArgs),
}

#[derive(Args)]
pub struct AddNodeArgs {
    /// Node name
    #[arg(short, long)]
    name: String,

    /// Domain name  
    #[arg(short, long)]
    domain: String,

    /// Vendor
    #[arg(short = 'V', long)]
    vendor: String,

    /// Model
    #[arg(short, long)]
    model: String,

    /// Device role
    #[arg(short, long)]
    role: String,

    /// Lifecycle state
    #[arg(short, long, default_value = "planned")]
    lifecycle: String,

    /// Location ID
    #[arg(short = 'L', long)]
    location_id: Option<Uuid>,

    /// Management IP address
    #[arg(short = 'i', long)]
    management_ip: Option<String>,

    /// Custom data as JSON
    #[arg(short = 'c', long)]
    custom_data: Option<String>,
}

#[derive(Args)]
pub struct ListNodeArgs {
    /// Filter by lifecycle
    #[arg(long)]
    lifecycle: Option<String>,

    /// Filter by role
    #[arg(long)]
    role: Option<String>,

    /// Filter by vendor
    #[arg(long)]
    vendor: Option<String>,

    /// Page number (1-based)
    #[arg(long, default_value = "1")]
    page: u64,

    /// Items per page
    #[arg(long, default_value = "20")]
    per_page: u64,
}

#[derive(Args)]
pub struct ShowNodeArgs {
    /// Node ID
    id: Uuid,

    /// Include derived state (SNMP polling data) in output
    #[arg(long)]
    include_status: bool,

    /// Show interface details from derived state
    #[arg(long)]
    show_interfaces: bool,

    /// Show system information from derived state
    #[arg(long)]
    show_system_info: bool,
}

#[derive(Args)]
pub struct UpdateNodeArgs {
    /// Node ID
    id: Uuid,

    /// Node name
    #[arg(short, long)]
    name: Option<String>,

    /// Domain name
    #[arg(short, long)]
    domain: Option<String>,

    /// Vendor
    #[arg(short = 'V', long)]
    vendor: Option<String>,

    /// Model
    #[arg(short, long)]
    model: Option<String>,

    /// Device role
    #[arg(short, long)]
    role: Option<String>,

    /// Lifecycle state
    #[arg(short, long)]
    lifecycle: Option<String>,

    /// Location ID
    #[arg(short = 'L', long)]
    location_id: Option<Uuid>,

    /// Management IP address
    #[arg(short = 'i', long)]
    management_ip: Option<String>,

    /// Custom data as JSON
    #[arg(short = 'c', long)]
    custom_data: Option<String>,
}

#[derive(Args)]
pub struct DeleteNodeArgs {
    /// Node ID
    id: Uuid,

    /// Skip confirmation prompt
    #[arg(short = 'y', long)]
    yes: bool,
}

#[derive(Args)]
pub struct StatusNodeArgs {
    /// Node ID
    id: Uuid,

    /// Show detailed interface status
    #[arg(long)]
    interfaces: bool,

    /// Show system information
    #[arg(long)]
    system: bool,

    /// Show polling task status
    #[arg(long)]
    polling: bool,
}

#[derive(Args)]
pub struct MonitorNodeArgs {
    /// Node ID
    id: Uuid,

    /// Monitor interval in seconds
    #[arg(short, long, default_value = "5")]
    interval: u64,

    /// Number of updates to show (0 = infinite)
    #[arg(short, long, default_value = "0")]
    count: u64,

    /// Monitor interfaces only
    #[arg(long)]
    interfaces_only: bool,
}

#[derive(Args)]
pub struct MetricsNodeArgs {
    /// Node ID
    id: Uuid,

    /// Show detailed performance metrics
    #[arg(long)]
    detailed: bool,

    /// Show historical metrics (not yet implemented)
    #[arg(long)]
    history: bool,
}

#[derive(Args)]
pub struct CompareNodeArgs {
    /// First node ID to compare
    #[arg(short = 'a', long)]
    node_a: Uuid,

    /// Second node ID to compare (optional for historical comparison)
    #[arg(short = 'b', long)]
    node_b: Option<Uuid>,

    /// Compare interface status
    #[arg(long)]
    interfaces: bool,

    /// Compare performance metrics
    #[arg(long)]
    metrics: bool,

    /// Compare system information
    #[arg(long)]
    system: bool,

    /// Show detailed differences only
    #[arg(long)]
    diff_only: bool,
}

#[derive(Args)]
pub struct PollingNodeArgs {
    /// Node ID
    id: Uuid,

    /// Polling action
    #[arg(value_enum)]
    action: PollingAction,

    /// Show detailed polling task information
    #[arg(long)]
    detailed: bool,
}

#[derive(Debug, clap::ValueEnum, Clone)]
pub enum PollingAction {
    /// Show current polling status
    Status,
    /// Start polling for this node
    Start,
    /// Stop polling for this node
    Stop,
    /// Restart polling for this node
    Restart,
    /// Show polling task history
    History,
}

#[derive(Args)]
pub struct HistoryNodeArgs {
    /// Node ID
    id: Uuid,

    /// History type to display
    #[arg(value_enum, default_value = "status")]
    history_type: HistoryType,

    /// Number of historical entries to show
    #[arg(short, long, default_value = "10")]
    limit: usize,

    /// Show only entries from the last N hours
    #[arg(long)]
    last_hours: Option<u64>,

    /// Show detailed historical data
    #[arg(long)]
    detailed: bool,
}

#[derive(Debug, clap::ValueEnum, Clone)]
pub enum HistoryType {
    /// Node status history
    Status,
    /// Interface status history
    Interfaces,
    /// Performance metrics history
    Metrics,
    /// System information changes
    System,
    /// All history types
    All,
}

pub async fn execute(
    command: NodeCommands,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    match command {
        NodeCommands::Add(args) => add_node(args, datastore, output_format).await,
        NodeCommands::List(args) => list_nodes(args, datastore, output_format).await,
        NodeCommands::Show(args) => show_node(args, datastore, output_format).await,
        NodeCommands::Update(args) => update_node(args, datastore, output_format).await,
        NodeCommands::Delete(args) => delete_node(args, datastore, output_format).await,
        NodeCommands::Status(args) => status_node(args, datastore, output_format).await,
        NodeCommands::Monitor(args) => monitor_node(args, datastore, output_format).await,
        NodeCommands::Metrics(args) => metrics_node(args, datastore, output_format).await,
        NodeCommands::Compare(args) => compare_nodes(args, datastore, output_format).await,
        NodeCommands::Polling(args) => polling_node(args, datastore, output_format).await,
        NodeCommands::History(args) => history_node(args, datastore, output_format).await,
    }
}

async fn add_node(
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

async fn list_nodes(
    args: ListNodeArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    let mut filters = Vec::new();

    if let Some(lifecycle) = args.lifecycle {
        filters.push(Filter {
            field: "lifecycle".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::String(lifecycle),
        });
    }

    if let Some(role) = args.role {
        filters.push(Filter {
            field: "role".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::String(role),
        });
    }

    if let Some(vendor) = args.vendor {
        filters.push(Filter {
            field: "vendor".to_string(),
            operation: FilterOperation::Equals,
            value: FilterValue::String(vendor),
        });
    }

    let options = QueryOptions {
        filters,
        sort: vec![Sort {
            field: "name".to_string(),
            direction: SortDirection::Ascending,
        }],
        pagination: Some(Pagination {
            offset: ((args.page - 1) * args.per_page) as usize,
            limit: args.per_page as usize,
        }),
    };

    let result = datastore.list_nodes(&options).await?;

    crate::commands::print_output(&result, output_format)?;

    Ok(())
}

async fn show_node(
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

async fn update_node(
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
    }

    if let Some(vendor_str) = args.vendor {
        node.vendor = vendor_str
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid vendor '{}': {}", vendor_str, e))?;
    }

    if let Some(model) = args.model {
        node.model = model;
    }

    if let Some(role_str) = args.role {
        node.role = role_str
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid role '{}': {}", role_str, e))?;
    }

    if let Some(lifecycle_str) = args.lifecycle {
        node.lifecycle = lifecycle_str
            .parse()
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
        node.custom_data = serde_json::from_str(&custom_data_str)?;
    }

    let updated_node = datastore.update_node(&node).await?;

    crate::commands::print_output(&updated_node, output_format)?;

    Ok(())
}

async fn delete_node(
    args: DeleteNodeArgs,
    datastore: &dyn DataStore,
    _output_format: crate::OutputFormat,
) -> Result<()> {
    // Check if node exists first
    let node = datastore.get_node_required(&args.id).await?;

    if !args.yes {
        // Ask for confirmation
        println!(
            "Are you sure you want to delete node '{}' (ID: {})? [y/N]",
            node.name, node.id
        );

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if !input.trim().to_lowercase().starts_with('y') {
            println!("Cancelled.");
            return Ok(());
        }
    }

    datastore.delete_node(&args.id).await?;

    println!("Node '{}' deleted successfully.", node.name);

    Ok(())
}

async fn status_node(
    args: StatusNodeArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    let node = datastore.get_node_required(&args.id).await?;

    // Get node status from datastore
    let node_status = datastore.get_node_status(&args.id).await.unwrap_or(None);

    let mut output = serde_json::json!({
        "node_id": node.id,
        "node_name": node.name,
        "has_derived_state": node_status.is_some()
    });

    // Add node status if available
    if let Some(status) = &node_status {
        output["status"] = serde_json::json!({
            "reachable": status.reachable,
            "last_updated": status.last_updated,
            "consecutive_failures": status.consecutive_failures,
            "last_snmp_success": status.last_snmp_success,
            "last_error": status.last_error
        });
    } else {
        output["status"] = serde_json::json!({
            "message": "No status data available",
            "note": "Node has not been polled yet or polling is not configured"
        });
    }

    if args.interfaces {
        match datastore.get_node_interfaces(&args.id).await {
            Ok(interfaces) => {
                output["interfaces"] = serde_json::to_value(&interfaces)?;
            }
            Err(e) => {
                output["interfaces"] = serde_json::json!({
                    "error": format!("Failed to fetch interfaces: {}", e)
                });
            }
        }
    }

    if args.system {
        if let Some(status) = &node_status {
            output["system_info"] = serde_json::to_value(&status.system_info)?;
        } else {
            output["system_info"] = serde_json::json!({
                "message": "No system information available",
                "note": "Node has not been polled yet or SNMP polling is not configured"
            });
        }
    }

    if args.polling {
        // TODO: Add polling task status when implemented
        output["polling_tasks"] = serde_json::json!({
            "message": "Polling task status display not yet implemented",
            "note": "Will be available in future milestone"
        });
    }

    crate::commands::print_output(&output, output_format)?;

    Ok(())
}

async fn monitor_node(
    args: MonitorNodeArgs,
    _datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    let output = serde_json::json!({
        "monitor_request": {
            "node_id": args.id,
            "interval_seconds": args.interval,
            "count": args.count,
            "interfaces_only": args.interfaces_only
        },
        "status": "Not implemented",
        "message": "Real-time monitoring requires SQLite datastore with derived state",
        "implementation_note": "This would continuously poll and display derived state changes",
        "available_when": "SQLite datastore implementation complete (M2.5.8)"
    });

    crate::commands::print_output(&output, output_format)?;

    Ok(())
}

async fn metrics_node(
    args: MetricsNodeArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    // Verify node exists
    let node = datastore.get_node_required(&args.id).await?;

    let mut output = serde_json::json!({
        "node_id": node.id,
        "node_name": node.name
    });

    // Get performance metrics
    match datastore.get_node_metrics(&args.id).await {
        Ok(Some(metrics)) => {
            if args.detailed {
                output["metrics"] = serde_json::to_value(&metrics)?;
            } else {
                // Show summary metrics
                output["metrics"] = serde_json::json!({
                    "cpu_utilization": metrics.cpu_utilization,
                    "memory_utilization": metrics.memory_utilization,
                    "load_average": metrics.load_average
                });
            }
        }
        Ok(None) => {
            output["metrics"] = serde_json::json!({
                "message": "No performance metrics available",
                "note": "Node has not been polled yet or performance metrics are not configured"
            });
        }
        Err(e) => {
            output["metrics"] = serde_json::json!({
                "error": format!("Failed to fetch metrics: {}", e)
            });
        }
    }

    if args.history {
        output["history"] = serde_json::json!({
            "message": "Historical metrics display not yet implemented",
            "note": "Will be available in future milestone"
        });
    }

    crate::commands::print_output(&output, output_format)?;

    Ok(())
}

async fn compare_nodes(
    args: CompareNodeArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    // Get the first node
    let node_a = datastore.get_node_required(&args.node_a).await?;

    let mut output = serde_json::json!({
        "comparison_type": if args.node_b.is_some() { "node_to_node" } else { "historical" },
        "node_a": {
            "id": node_a.id,
            "name": node_a.name
        }
    });

    if let Some(node_b_id) = args.node_b {
        // Node-to-node comparison
        let node_b = datastore.get_node_required(&node_b_id).await?;
        output["node_b"] = serde_json::json!({
            "id": node_b.id,
            "name": node_b.name
        });

        // Compare basic node properties
        let mut differences = Vec::new();

        if node_a.vendor != node_b.vendor {
            differences.push(serde_json::json!({
                "field": "vendor",
                "node_a_value": node_a.vendor,
                "node_b_value": node_b.vendor
            }));
        }

        if node_a.model != node_b.model {
            differences.push(serde_json::json!({
                "field": "model",
                "node_a_value": node_a.model,
                "node_b_value": node_b.model
            }));
        }

        if node_a.role != node_b.role {
            differences.push(serde_json::json!({
                "field": "role",
                "node_a_value": node_a.role,
                "node_b_value": node_b.role
            }));
        }

        if !args.diff_only || !differences.is_empty() {
            output["basic_differences"] = serde_json::to_value(&differences)?;
        }

        // Compare derived state if requested
        if args.interfaces {
            let interfaces_a = datastore
                .get_node_interfaces(&args.node_a)
                .await
                .unwrap_or_default();
            let interfaces_b = datastore
                .get_node_interfaces(&node_b_id)
                .await
                .unwrap_or_default();

            output["interface_comparison"] = serde_json::json!({
                "node_a_count": interfaces_a.len(),
                "node_b_count": interfaces_b.len(),
                "note": "Detailed interface comparison not yet implemented"
            });
        }

        if args.metrics {
            let metrics_a = datastore
                .get_node_metrics(&args.node_a)
                .await
                .unwrap_or(None);
            let metrics_b = datastore.get_node_metrics(&node_b_id).await.unwrap_or(None);

            output["metrics_comparison"] = serde_json::json!({
                "node_a_has_metrics": metrics_a.is_some(),
                "node_b_has_metrics": metrics_b.is_some(),
                "note": "Detailed metrics comparison not yet implemented"
            });
        }

        if args.system {
            let status_a = datastore
                .get_node_status(&args.node_a)
                .await
                .unwrap_or(None);
            let status_b = datastore.get_node_status(&node_b_id).await.unwrap_or(None);

            output["system_comparison"] = serde_json::json!({
                "node_a_reachable": status_a.map(|s| s.reachable),
                "node_b_reachable": status_b.map(|s| s.reachable),
                "note": "Detailed system comparison not yet implemented"
            });
        }
    } else {
        // Historical comparison (not yet implemented)
        output["historical_comparison"] = serde_json::json!({
            "message": "Historical comparison not yet implemented",
            "note": "This would compare current state with previous snapshots"
        });
    }

    crate::commands::print_output(&output, output_format)?;

    Ok(())
}

async fn polling_node(
    args: PollingNodeArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    // Verify node exists
    let node = datastore.get_node_required(&args.id).await?;

    let mut output = serde_json::json!({
        "node_id": node.id,
        "node_name": node.name,
        "action": format!("{:?}", args.action)
    });

    match args.action {
        PollingAction::Status => {
            // Show current polling status
            // TODO: Implement actual polling task status when datastore supports it
            output["polling_status"] = serde_json::json!({
                "message": "Polling status display not yet fully implemented",
                "note": "This would show current polling task configuration and status",
                "basic_info": {
                    "node_has_status": datastore.get_node_status(&args.id).await.unwrap_or(None).is_some(),
                    "last_updated": datastore.get_node_status(&args.id).await.unwrap_or(None).map(|s| s.last_updated)
                }
            });
        }
        PollingAction::Start => {
            output["result"] = serde_json::json!({
                "message": "Polling start command not yet implemented",
                "note": "This would create or enable polling tasks for the node",
                "action_required": "Implement polling task management in DataStore"
            });
        }
        PollingAction::Stop => {
            output["result"] = serde_json::json!({
                "message": "Polling stop command not yet implemented",
                "note": "This would disable or remove polling tasks for the node",
                "action_required": "Implement polling task management in DataStore"
            });
        }
        PollingAction::Restart => {
            output["result"] = serde_json::json!({
                "message": "Polling restart command not yet implemented",
                "note": "This would stop and then start polling tasks for the node",
                "action_required": "Implement polling task management in DataStore"
            });
        }
        PollingAction::History => {
            output["history"] = serde_json::json!({
                "message": "Polling history display not yet implemented",
                "note": "This would show historical polling task results and errors",
                "action_required": "Implement polling task history tracking"
            });
        }
    }

    if args.detailed {
        output["detailed_info"] = serde_json::json!({
            "node_details": {
                "management_ip": node.management_ip,
                "vendor": node.vendor,
                "model": node.model
            },
            "note": "Additional polling configuration details would be shown here"
        });
    }

    crate::commands::print_output(&output, output_format)?;

    Ok(())
}

async fn history_node(
    args: HistoryNodeArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    // Verify node exists
    let node = datastore.get_node_required(&args.id).await?;

    let mut output = serde_json::json!({
        "node_id": node.id,
        "node_name": node.name,
        "history_type": format!("{:?}", args.history_type),
        "limit": args.limit,
        "last_hours": args.last_hours
    });

    match args.history_type {
        HistoryType::Status => {
            output["status_history"] = serde_json::json!({
                "message": "Status history display not yet implemented",
                "note": "This would show historical changes in node reachability, SNMP polling status, and error states",
                "implementation_required": "Historical status tracking in database",
                "current_status": datastore.get_node_status(&args.id).await.unwrap_or(None).map(|s| serde_json::json!({
                    "reachable": s.reachable,
                    "last_updated": s.last_updated,
                    "consecutive_failures": s.consecutive_failures
                }))
            });
        }
        HistoryType::Interfaces => {
            output["interface_history"] = serde_json::json!({
                "message": "Interface history display not yet implemented",
                "note": "This would show historical changes in interface status, traffic counters, and configuration",
                "implementation_required": "Interface status change tracking",
                "current_interfaces": datastore.get_node_interfaces(&args.id).await.unwrap_or_default().len()
            });
        }
        HistoryType::Metrics => {
            output["metrics_history"] = serde_json::json!({
                "message": "Metrics history display not yet implemented",
                "note": "This would show historical performance metrics like CPU, memory, and load averages",
                "implementation_required": "Performance metrics time series storage",
                "current_metrics": datastore.get_node_metrics(&args.id).await.unwrap_or(None).map(|m| serde_json::json!({
                    "cpu_utilization": m.cpu_utilization,
                    "memory_utilization": m.memory_utilization
                }))
            });
        }
        HistoryType::System => {
            output["system_history"] = serde_json::json!({
                "message": "System information history not yet implemented",
                "note": "This would show historical changes in system info like uptime, description, and contact information",
                "implementation_required": "System information change tracking",
                "current_system": datastore.get_node_status(&args.id).await.unwrap_or(None).and_then(|s| s.system_info)
            });
        }
        HistoryType::All => {
            output["all_history"] = serde_json::json!({
                "message": "Complete history display not yet implemented",
                "note": "This would show a unified timeline of all derived state changes",
                "implementation_required": "Comprehensive historical data aggregation",
                "available_data": {
                    "has_status": datastore.get_node_status(&args.id).await.unwrap_or(None).is_some(),
                    "interface_count": datastore.get_node_interfaces(&args.id).await.unwrap_or_default().len(),
                    "has_metrics": datastore.get_node_metrics(&args.id).await.unwrap_or(None).is_some()
                }
            });
        }
    }

    if args.detailed {
        output["implementation_notes"] = serde_json::json!({
            "storage_requirements": "Historical data requires time-series database or audit tables",
            "data_retention": "Need to define retention policies for different data types",
            "performance_considerations": "Large datasets may require pagination and indexing",
            "suggested_next_steps": [
                "Add timestamp-based audit tables for each derived state type",
                "Implement background cleanup for old historical data",
                "Add time-range filtering and aggregation capabilities"
            ]
        });
    }

    crate::commands::print_output(&output, output_format)?;

    Ok(())
}
