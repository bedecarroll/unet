use anyhow::Result;
use clap::{Args, Subcommand};
use unet_core::datastore::DataStore;

#[derive(Subcommand)]
pub enum VendorCommands {
    /// Add a new vendor name
    Add(AddVendorArgs),
    /// List all vendors
    List,
    /// Delete a vendor by name
    Delete(DeleteVendorArgs),
}

#[derive(Args, Debug)]
pub struct AddVendorArgs {
    /// Vendor name
    pub name: String,
}

#[derive(Args, Debug)]
pub struct DeleteVendorArgs {
    /// Vendor name
    pub name: String,
    /// Skip confirmation prompt
    #[arg(short = 'y', long)]
    pub yes: bool,
}

/// Execute vendor subcommands.
///
/// # Errors
/// Returns an error if datastore operations or output formatting fail.
pub async fn execute(
    command: VendorCommands,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    match command {
        VendorCommands::Add(args) => add_vendor(args, datastore, output_format).await,
        VendorCommands::List => list_vendors(datastore, output_format).await,
        VendorCommands::Delete(args) => delete_vendor(args, datastore, output_format).await,
    }
}

async fn add_vendor(
    args: AddVendorArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    datastore.create_vendor(&args.name).await?;
    let output = serde_json::json!({ "message": "Vendor added", "name": args.name });
    crate::commands::print_output(&output, output_format)?;
    Ok(())
}

async fn list_vendors(datastore: &dyn DataStore, output_format: crate::OutputFormat) -> Result<()> {
    let vendors = datastore.list_vendors().await?;
    crate::commands::print_output(&vendors, output_format)?;
    Ok(())
}

async fn delete_vendor(
    args: DeleteVendorArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    if !args.yes {
        let stdin = std::io::stdin();
        let mut lock = stdin.lock();
        let mut reader = std::io::BufReader::new(&mut lock);
        if !confirm_deletion(args.yes, &args.name, &mut reader)? {
            return Ok(());
        }
    }
    datastore.delete_vendor(&args.name).await?;
    let output = serde_json::json!({ "message": "Vendor deleted", "name": args.name });
    crate::commands::print_output(&output, output_format)?;
    Ok(())
}

// Small helper to make interactive confirmation testable
pub(crate) fn confirm_deletion(
    yes: bool,
    name: &str,
    reader: &mut impl std::io::BufRead,
) -> Result<bool> {
    if yes {
        return Ok(true);
    }
    println!("Are you sure you want to delete vendor '{name}' ? [y/N]");
    let mut input = String::new();
    reader.read_line(&mut input)?;
    if !input.trim().to_lowercase().starts_with('y') {
        println!("Cancelled.");
        return Ok(false);
    }
    Ok(true)
}


#[cfg(test)]
#[path = "vendors_tests.rs"]
mod vendors_tests;
