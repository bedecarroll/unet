use anyhow::Result;
use clap::{Args, Subcommand};
use serde_json::Value as JsonValue;
use unet_core::datastore::DataStore;
use unet_core::prelude::*;
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
    name: String,

    /// Location type (e.g., datacenter, campus, building, floor, rack)
    #[arg(short, long)]
    location_type: String,

    /// Parent location ID
    #[arg(short, long)]
    parent_id: Option<Uuid>,

    /// Address
    #[arg(short, long)]
    address: Option<String>,

    /// City
    #[arg(short = 'c', long)]
    city: Option<String>,

    /// Country
    #[arg(short = 'C', long)]
    country: Option<String>,

    /// Custom data as JSON
    #[arg(short = 'j', long)]
    custom_data: Option<String>,
}

#[derive(Args)]
pub struct ListLocationArgs {
    /// Filter by location type
    #[arg(long)]
    location_type: Option<String>,

    /// Filter by parent ID
    #[arg(long)]
    parent_id: Option<Uuid>,

    /// Page number (1-based)
    #[arg(long, default_value = "1")]
    page: u64,

    /// Items per page
    #[arg(long, default_value = "20")]
    per_page: u64,
}

#[derive(Args)]
pub struct ShowLocationArgs {
    /// Location ID
    id: Uuid,
}

#[derive(Args)]
pub struct UpdateLocationArgs {
    /// Location ID
    id: Uuid,

    /// Location name
    #[arg(short, long)]
    name: Option<String>,

    /// Location type
    #[arg(short, long)]
    location_type: Option<String>,

    /// Parent location ID
    #[arg(short, long)]
    parent_id: Option<Uuid>,

    /// Address
    #[arg(short, long)]
    address: Option<String>,

    /// City
    #[arg(short = 'c', long)]
    city: Option<String>,

    /// Country
    #[arg(short = 'C', long)]
    country: Option<String>,

    /// Custom data as JSON
    #[arg(short = 'j', long)]
    custom_data: Option<String>,
}

#[derive(Args)]
pub struct DeleteLocationArgs {
    /// Location ID
    id: Uuid,

    /// Skip confirmation prompt
    #[arg(short = 'y', long)]
    yes: bool,
}

pub async fn execute(
    command: LocationCommands,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    match command {
        LocationCommands::Add(args) => add_location(args, datastore, output_format).await,
        LocationCommands::List(args) => list_locations(args, datastore, output_format).await,
        LocationCommands::Show(args) => show_location(args, datastore, output_format).await,
        LocationCommands::Update(args) => update_location(args, datastore, output_format).await,
        LocationCommands::Delete(args) => delete_location(args, datastore, output_format).await,
    }
}

async fn add_location(
    args: AddLocationArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    // Parse custom data if provided
    let custom_data = if let Some(json_str) = args.custom_data {
        Some(serde_json::from_str::<JsonValue>(&json_str)?)
    } else {
        None
    };

    // Build location
    let mut builder = LocationBuilder::new()
        .name(args.name)
        .location_type(args.location_type);

    if let Some(parent_id) = args.parent_id {
        builder = builder.parent_id(parent_id);
    }

    // Combine address, city, country into a single address field
    let mut address_parts = Vec::new();
    if let Some(address) = args.address {
        address_parts.push(address);
    }
    if let Some(city) = args.city {
        address_parts.push(city);
    }
    if let Some(country) = args.country {
        address_parts.push(country);
    }
    if !address_parts.is_empty() {
        builder = builder.address(address_parts.join(", "));
    }

    if let Some(custom_data) = custom_data {
        builder = builder.custom_data(custom_data);
    }

    let location = builder
        .build()
        .map_err(|e| anyhow::anyhow!("Location validation failed: {}", e))?;

    // Create location in datastore
    let created_location = datastore.create_location(&location).await?;

    crate::commands::print_output(&created_location, output_format)?;

    Ok(())
}

async fn list_locations(
    args: ListLocationArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    let mut filters = Vec::new();

    if let Some(location_type) = args.location_type {
        filters.push(Filter {
            field: "location_type".to_owned(),
            operation: FilterOperation::Equals,
            value: FilterValue::String(location_type),
        });
    }

    if let Some(parent_id) = args.parent_id {
        filters.push(Filter {
            field: "parent_id".to_owned(),
            operation: FilterOperation::Equals,
            value: FilterValue::Uuid(parent_id),
        });
    }

    let options = QueryOptions {
        filters,
        sort: vec![Sort {
            field: "name".to_owned(),
            direction: SortDirection::Ascending,
        }],
        pagination: Some(Pagination {
            offset: ((args.page - 1) * args.per_page) as usize,
            limit: args.per_page as usize,
        }),
    };

    let result = datastore.list_locations(&options).await?;

    crate::commands::print_output(&result, output_format)?;

    Ok(())
}

async fn show_location(
    args: ShowLocationArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    let location = datastore.get_location_required(&args.id).await?;

    crate::commands::print_output(&location, output_format)?;

    Ok(())
}

async fn update_location(
    args: UpdateLocationArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    let mut location = datastore.get_location_required(&args.id).await?;

    // Update fields that were provided
    if let Some(name) = args.name {
        location.name = name;
    }

    if let Some(location_type) = args.location_type {
        location.location_type = location_type;
    }

    if let Some(parent_id) = args.parent_id {
        location.parent_id = Some(parent_id);
    }

    // Update address (combining address, city, country like in add command)
    let mut address_parts = Vec::new();
    if let Some(address) = args.address {
        address_parts.push(address);
    }
    if let Some(city) = args.city {
        address_parts.push(city);
    }
    if let Some(country) = args.country {
        address_parts.push(country);
    }
    if !address_parts.is_empty() {
        location.address = Some(address_parts.join(", "));
    }

    if let Some(custom_data_str) = args.custom_data {
        location.custom_data = serde_json::from_str(&custom_data_str)?;
    }

    let updated_location = datastore.update_location(&location).await?;

    crate::commands::print_output(&updated_location, output_format)?;

    Ok(())
}

async fn delete_location(
    args: DeleteLocationArgs,
    datastore: &dyn DataStore,
    _output_format: crate::OutputFormat,
) -> Result<()> {
    // Check if location exists first
    let location = datastore.get_location_required(&args.id).await?;

    if !args.yes {
        // Ask for confirmation
        println!(
            "Are you sure you want to delete location '{}' (ID: {})? [y/N]",
            location.name, location.id
        );

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if !input.trim().to_lowercase().starts_with('y') {
            println!("Cancelled.");
            return Ok(());
        }
    }

    datastore.delete_location(&args.id).await?;

    println!("Location '{}' deleted successfully.", location.name);

    Ok(())
}
