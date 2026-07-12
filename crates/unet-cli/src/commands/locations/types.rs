/// Location command types and arguments
use clap::{Args, Subcommand};
use uuid::Uuid;

#[derive(Subcommand)]
pub enum LocationCommands {
    /// Add a new location
    Add(AddLocationArgs),
    /// List all locations
    List(ListLocationArgs),
    /// Show location details
    Show(ShowLocationArgs),
    /// Update an existing location
    Update(UpdateLocationArgs),
    /// Delete a location
    Delete(DeleteLocationArgs),
}

#[derive(Args)]
pub struct AddLocationArgs {
    /// Location name
    #[arg(short, long)]
    pub name: String,

    /// Location type (e.g., datacenter, campus, building, floor, rack)
    #[arg(short, long)]
    pub location_type: String,

    /// Parent location ID
    #[arg(short, long)]
    pub parent_id: Option<Uuid>,

    /// Address
    #[arg(short, long)]
    pub address: Option<String>,

    /// City
    #[arg(short = 'c', long)]
    pub city: Option<String>,

    /// Country
    #[arg(short = 'C', long)]
    pub country: Option<String>,

    /// Custom data as JSON
    #[arg(short = 'j', long)]
    pub custom_data: Option<String>,
}

#[derive(Args)]
pub struct ListLocationArgs {
    /// Filter by location type
    #[arg(long)]
    pub location_type: Option<String>,

    /// Filter by parent ID
    #[arg(long)]
    pub parent_id: Option<Uuid>,

    /// Page number (1-based)
    #[arg(long, default_value = "1")]
    pub page: u64,

    /// Items per page
    #[arg(long, default_value = "20")]
    pub per_page: u64,
}

#[derive(Args)]
pub struct ShowLocationArgs {
    /// Location ID
    pub id: Uuid,
}

#[derive(Args)]
pub struct UpdateLocationArgs {
    /// Location ID
    pub id: Uuid,

    /// Location name
    #[arg(short, long)]
    pub name: Option<String>,

    /// Location type
    #[arg(short, long)]
    pub location_type: Option<String>,

    /// Parent location ID
    #[arg(short, long)]
    pub parent_id: Option<Uuid>,

    /// Address
    #[arg(short, long)]
    pub address: Option<String>,

    /// City
    #[arg(short = 'c', long)]
    pub city: Option<String>,

    /// Country
    #[arg(short = 'C', long)]
    pub country: Option<String>,

    /// Custom data as JSON
    #[arg(short = 'j', long)]
    pub custom_data: Option<String>,
}

#[derive(Args)]
pub struct DeleteLocationArgs {
    /// Location ID
    pub id: Uuid,

    /// Skip confirmation prompt
    #[arg(short = 'y', long)]
    pub yes: bool,
}
