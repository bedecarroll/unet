/// Command types and argument structures for node management
use clap::{Args, Subcommand};
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
    pub name: String,

    /// Domain name  
    #[arg(short, long)]
    pub domain: String,

    /// Vendor
    #[arg(short = 'V', long)]
    pub vendor: String,

    /// Model
    #[arg(short, long)]
    pub model: String,

    /// Device role
    #[arg(short, long)]
    pub role: String,

    /// Lifecycle state
    #[arg(short, long, default_value = "planned")]
    pub lifecycle: String,

    /// Location ID
    #[arg(short = 'L', long)]
    pub location_id: Option<Uuid>,

    /// Management IP address
    #[arg(short = 'i', long)]
    pub management_ip: Option<String>,

    /// Custom data as JSON
    #[arg(short = 'c', long)]
    pub custom_data: Option<String>,
}

#[derive(Args)]
pub struct ListNodeArgs {
    /// Filter by lifecycle
    #[arg(long)]
    pub lifecycle: Option<String>,

    /// Filter by role
    #[arg(long)]
    pub role: Option<String>,

    /// Filter by vendor
    #[arg(long)]
    pub vendor: Option<String>,

    /// Page number (1-based)
    #[arg(long, default_value = "1")]
    pub page: u64,

    /// Items per page
    #[arg(long, default_value = "20")]
    pub per_page: u64,
}

#[derive(Args)]
pub struct ShowNodeArgs {
    /// Node ID
    pub id: Uuid,

    /// Include derived state (SNMP polling data) in output
    #[arg(long)]
    pub include_status: bool,

    /// Show interface details from derived state
    #[arg(long)]
    pub show_interfaces: bool,

    /// Show system information from derived state
    #[arg(long)]
    pub show_system_info: bool,
}

#[derive(Args)]
pub struct UpdateNodeArgs {
    /// Node ID
    pub id: Uuid,

    /// Node name
    #[arg(short, long)]
    pub name: Option<String>,

    /// Domain name
    #[arg(short, long)]
    pub domain: Option<String>,

    /// Vendor
    #[arg(short = 'V', long)]
    pub vendor: Option<String>,

    /// Model
    #[arg(short, long)]
    pub model: Option<String>,

    /// Device role
    #[arg(short, long)]
    pub role: Option<String>,

    /// Lifecycle state
    #[arg(short, long)]
    pub lifecycle: Option<String>,

    /// Location ID
    #[arg(short = 'L', long)]
    pub location_id: Option<Uuid>,

    /// Management IP address
    #[arg(short = 'i', long)]
    pub management_ip: Option<String>,

    /// Custom data as JSON
    #[arg(short = 'c', long)]
    pub custom_data: Option<String>,
}

#[derive(Args)]
pub struct DeleteNodeArgs {
    /// Node ID
    pub id: Uuid,

    /// Skip confirmation prompt
    #[arg(short = 'y', long)]
    pub yes: bool,
}

#[derive(Args)]
pub struct StatusNodeArgs {
    /// Node ID
    pub id: Uuid,

    /// Types of status information to show (can be specified multiple times)
    #[arg(long, value_enum, default_value = "basic")]
    pub status_type: Vec<StatusType>,
}

/// Types of status information that can be displayed
#[derive(Debug, Clone, PartialEq, Eq, clap::ValueEnum, serde::Serialize)]
pub enum StatusType {
    /// Show basic node information only
    Basic,
    /// Show all available status information
    All,
    /// Show detailed interface status
    Interfaces,
    /// Show system information
    System,
    /// Show polling task status
    Polling,
}

#[derive(Args)]
pub struct MetricsNodeArgs {
    /// Node ID
    pub id: Uuid,

    /// Show detailed performance metrics
    #[arg(long)]
    pub detailed: bool,

    /// Show historical metrics (not yet implemented)
    #[arg(long)]
    pub history: bool,
}

#[derive(Args)]
pub struct CompareNodeArgs {
    /// First node ID to compare
    #[arg(short = 'a', long)]
    pub node_a: Uuid,

    /// Second node ID to compare (optional for historical comparison)
    #[arg(short = 'b', long)]
    pub node_b: Option<Uuid>,

    /// Types of data to compare (can be specified multiple times)
    #[arg(long, value_enum, default_value = "all")]
    pub compare_type: Vec<CompareType>,

    /// Show detailed differences only
    #[arg(long)]
    pub diff_only: bool,
}

/// Types of node data that can be compared
#[derive(Debug, Clone, PartialEq, Eq, clap::ValueEnum, serde::Serialize)]
pub enum CompareType {
    /// Compare all available data
    All,
    /// Compare interface status
    Interfaces,
    /// Compare performance metrics
    Metrics,
    /// Compare system information
    System,
}

#[derive(Args)]
pub struct PollingNodeArgs {
    /// Node ID
    pub id: Uuid,

    /// Polling action
    #[arg(value_enum)]
    pub action: PollingAction,

    /// Show detailed polling task information
    #[arg(long)]
    pub detailed: bool,
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
    pub id: Uuid,

    /// History type to display
    #[arg(value_enum, default_value = "status")]
    pub history_type: HistoryType,

    /// Number of historical entries to show
    #[arg(short, long, default_value = "10")]
    pub limit: usize,

    /// Show only entries from the last N hours
    #[arg(long)]
    pub last_hours: Option<u64>,

    /// Show detailed historical data
    #[arg(long)]
    pub detailed: bool,
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
