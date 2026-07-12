/// Link command types and arguments
use clap::{Args, Subcommand};
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
    pub name: String,

    /// First node ID
    #[arg(short = 'a', long)]
    pub node_a_id: Uuid,

    /// Interface name on first node
    #[arg(short = 'A', long)]
    pub node_a_interface: String,

    /// Second node ID (optional for internet circuits)
    #[arg(short = 'z', long)]
    pub node_z_id: Option<Uuid>,

    /// Interface name on second node
    #[arg(short = 'Z', long)]
    pub node_z_interface: Option<String>,

    /// Bandwidth in bits per second
    #[arg(short, long)]
    pub bandwidth_bps: Option<u64>,

    /// Link description
    #[arg(short, long)]
    pub description: Option<String>,

    /// Custom data as JSON
    #[arg(short = 'j', long)]
    pub custom_data: Option<String>,
}

#[derive(Args)]
pub struct ListLinkArgs {
    /// Filter by node ID (either `node_a` or `node_z`)
    #[arg(long)]
    pub node_id: Option<Uuid>,

    /// Filter by minimum bandwidth
    #[arg(long)]
    pub min_bandwidth: Option<u64>,

    /// Page number (1-based)
    #[arg(long, default_value = "1")]
    pub page: u64,

    /// Items per page
    #[arg(long, default_value = "20")]
    pub per_page: u64,
}

#[derive(Args)]
pub struct ShowLinkArgs {
    /// Link ID
    pub id: Uuid,
}

#[derive(Args)]
pub struct UpdateLinkArgs {
    /// Link ID
    pub id: Uuid,

    /// Link name
    #[arg(short, long)]
    pub name: Option<String>,

    /// First node ID
    #[arg(short = 'a', long)]
    pub node_a_id: Option<Uuid>,

    /// Interface name on first node
    #[arg(short = 'A', long)]
    pub node_a_interface: Option<String>,

    /// Second node ID
    #[arg(short = 'z', long)]
    pub node_z_id: Option<Uuid>,

    /// Interface name on second node
    #[arg(short = 'Z', long)]
    pub node_z_interface: Option<String>,

    /// Bandwidth in bits per second
    #[arg(short, long)]
    pub bandwidth_bps: Option<u64>,

    /// Link description
    #[arg(short, long)]
    pub description: Option<String>,

    /// Custom data as JSON
    #[arg(short = 'j', long)]
    pub custom_data: Option<String>,
}

#[derive(Args)]
pub struct DeleteLinkArgs {
    /// Link ID
    pub id: Uuid,

    /// Skip confirmation prompt
    #[arg(short = 'y', long)]
    pub yes: bool,
}
