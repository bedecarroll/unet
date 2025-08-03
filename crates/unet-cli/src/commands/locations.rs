/// Location management CLI commands
use anyhow::Result;
use unet_core::datastore::DataStore;

mod crud;
mod types;

pub use types::LocationCommands;

pub async fn execute(
    command: LocationCommands,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    match command {
        LocationCommands::Add(args) => crud::add_location(args, datastore, output_format).await,
        LocationCommands::List(args) => crud::list_locations(args, datastore, output_format).await,
        LocationCommands::Show(args) => crud::show_location(args, datastore, output_format).await,
        LocationCommands::Update(args) => {
            crud::update_location(args, datastore, output_format).await
        }
        LocationCommands::Delete(args) => {
            crud::delete_location(args, datastore, output_format).await
        }
    }
}

#[cfg(test)]
mod tests;
