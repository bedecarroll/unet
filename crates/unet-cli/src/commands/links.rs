use anyhow::Result;
use clap::{Args, Subcommand};
use serde_json::Value as JsonValue;
use unet_core::datastore::DataStore;
use unet_core::prelude::*;
use uuid::Uuid;

#[derive(Subcommand)]
pub enum LinkCommands {
    /// Add a new link
    Add(AddLinkArgs),
    /// List all links
    List(ListLinkArgs),
    /// Show link details
    Show(ShowLinkArgs),
    /// Update an existing link
    Update(UpdateLinkArgs),
    /// Delete a link
    Delete(DeleteLinkArgs),
}

#[derive(Args)]
pub struct AddLinkArgs {
    /// Link name
    #[arg(short, long)]
    name: String,

    /// First node ID
    #[arg(short = 'a', long)]
    node_a_id: Uuid,

    /// Interface name on first node
    #[arg(short = 'A', long)]
    node_a_interface: String,

    /// Second node ID (optional for internet circuits)
    #[arg(short = 'z', long)]
    node_z_id: Option<Uuid>,

    /// Interface name on second node
    #[arg(short = 'Z', long)]
    node_z_interface: Option<String>,

    /// Bandwidth in bits per second
    #[arg(short, long)]
    bandwidth_bps: Option<u64>,

    /// Link description
    #[arg(short, long)]
    description: Option<String>,

    /// Custom data as JSON
    #[arg(short = 'j', long)]
    custom_data: Option<String>,
}

#[derive(Args)]
pub struct ListLinkArgs {
    /// Filter by node ID (either `node_a` or `node_z`)
    #[arg(long)]
    node_id: Option<Uuid>,

    /// Filter by minimum bandwidth
    #[arg(long)]
    min_bandwidth: Option<u64>,

    /// Page number (1-based)
    #[arg(long, default_value = "1")]
    page: u64,

    /// Items per page
    #[arg(long, default_value = "20")]
    per_page: u64,
}

#[derive(Args)]
pub struct ShowLinkArgs {
    /// Link ID
    id: Uuid,
}

#[derive(Args)]
pub struct UpdateLinkArgs {
    /// Link ID
    id: Uuid,

    /// Link name
    #[arg(short, long)]
    name: Option<String>,

    /// First node ID
    #[arg(short = 'a', long)]
    node_a_id: Option<Uuid>,

    /// Interface name on first node
    #[arg(short = 'A', long)]
    node_a_interface: Option<String>,

    /// Second node ID
    #[arg(short = 'z', long)]
    node_z_id: Option<Uuid>,

    /// Interface name on second node
    #[arg(short = 'Z', long)]
    node_z_interface: Option<String>,

    /// Bandwidth in bits per second
    #[arg(short, long)]
    bandwidth_bps: Option<u64>,

    /// Link description
    #[arg(short, long)]
    description: Option<String>,

    /// Custom data as JSON
    #[arg(short = 'j', long)]
    custom_data: Option<String>,
}

#[derive(Args)]
pub struct DeleteLinkArgs {
    /// Link ID
    id: Uuid,

    /// Skip confirmation prompt
    #[arg(short = 'y', long)]
    yes: bool,
}

pub async fn execute(
    command: LinkCommands,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    match command {
        LinkCommands::Add(args) => add_link(args, datastore, output_format).await,
        LinkCommands::List(args) => list_links(args, datastore, output_format).await,
        LinkCommands::Show(args) => show_link(args, datastore, output_format).await,
        LinkCommands::Update(args) => update_link(args, datastore, output_format).await,
        LinkCommands::Delete(args) => delete_link(args, datastore, output_format).await,
    }
}

async fn add_link(
    args: AddLinkArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    // Parse custom data if provided
    let custom_data = if let Some(json_str) = args.custom_data {
        Some(serde_json::from_str::<JsonValue>(&json_str)?)
    } else {
        None
    };

    // Build link
    let mut builder = LinkBuilder::new()
        .name(args.name)
        .source_node_id(args.node_a_id)
        .node_a_interface(args.node_a_interface);

    if let Some(node_z_id) = args.node_z_id {
        builder = builder.dest_node_id(node_z_id);
    }

    if let Some(node_z_interface) = args.node_z_interface {
        builder = builder.node_z_interface(node_z_interface);
    }

    if let Some(bandwidth_bps) = args.bandwidth_bps {
        builder = builder.bandwidth(bandwidth_bps);
    }

    if let Some(description) = args.description {
        builder = builder.description(description);
    }

    if let Some(custom_data) = custom_data {
        builder = builder.custom_data(custom_data);
    }

    let link = builder
        .build()
        .map_err(|e| anyhow::anyhow!("Link validation failed: {}", e))?;

    // Create link in datastore
    let created_link = datastore.create_link(&link).await?;

    crate::commands::print_output(&created_link, output_format)?;

    Ok(())
}

async fn list_links(
    args: ListLinkArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    let mut filters = Vec::new();

    if let Some(node_id) = args.node_id {
        // We'll need to filter by either node_a_id or node_z_id
        // For now, just filter by node_a_id as an example
        filters.push(Filter {
            field: "node_a_id".to_owned(),
            operation: FilterOperation::Equals,
            value: FilterValue::Uuid(node_id),
        });
    }

    // Note: For bandwidth filtering, we'd need to extend the CSV datastore
    // to support numeric comparisons. For now, we'll skip this filter.

    let options = QueryOptions {
        filters,
        sort: vec![Sort {
            field: "node_a_interface".to_owned(),
            direction: SortDirection::Ascending,
        }],
        pagination: Some(Pagination {
            offset: usize::try_from((args.page - 1) * args.per_page)?,
            limit: usize::try_from(args.per_page)?,
        }),
    };

    let result = datastore.list_links(&options).await?;

    crate::commands::print_output(&result, output_format)?;

    Ok(())
}

async fn show_link(
    args: ShowLinkArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    let link = datastore.get_link_required(&args.id).await?;

    crate::commands::print_output(&link, output_format)?;

    Ok(())
}

async fn update_link(
    args: UpdateLinkArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    let mut link = datastore.get_link_required(&args.id).await?;

    // Update fields that were provided
    if let Some(name) = args.name {
        link.name = name;
    }

    if let Some(node_a_id) = args.node_a_id {
        link.source_node_id = node_a_id;
    }

    if let Some(node_a_interface) = args.node_a_interface {
        link.node_a_interface = node_a_interface;
    }

    if let Some(node_z_id) = args.node_z_id {
        link.dest_node_id = Some(node_z_id);
    }

    if let Some(node_z_interface) = args.node_z_interface {
        link.node_z_interface = Some(node_z_interface);
    }

    if let Some(bandwidth_bps) = args.bandwidth_bps {
        link.bandwidth = Some(bandwidth_bps);
    }

    if let Some(description) = args.description {
        link.description = Some(description);
    }

    if let Some(custom_data_str) = args.custom_data {
        link.custom_data = serde_json::from_str(&custom_data_str)?;
    }

    let updated_link = datastore.update_link(&link).await?;

    crate::commands::print_output(&updated_link, output_format)?;

    Ok(())
}

async fn delete_link(
    args: DeleteLinkArgs,
    datastore: &dyn DataStore,
    _output_format: crate::OutputFormat,
) -> Result<()> {
    // Check if link exists first
    let link = datastore.get_link_required(&args.id).await?;

    if !args.yes {
        // Ask for confirmation
        println!(
            "Are you sure you want to delete link {} <-> {} (ID: {})? [y/N]",
            link.node_a_interface,
            link.node_z_interface.as_deref().unwrap_or("internet"),
            link.id
        );

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if !input.trim().to_lowercase().starts_with('y') {
            println!("Cancelled.");
            return Ok(());
        }
    }

    datastore.delete_link(&args.id).await?;

    println!("Link deleted successfully.");

    Ok(())
}
