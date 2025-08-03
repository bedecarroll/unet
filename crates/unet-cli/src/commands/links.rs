/// Link management CLI commands
use anyhow::Result;
use unet_core::datastore::DataStore;

mod crud;
mod types;

pub use types::LinkCommands;

pub async fn execute(
    command: LinkCommands,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    match command {
        LinkCommands::Add(args) => crud::add_link(args, datastore, output_format).await,
        LinkCommands::List(args) => crud::list_links(args, datastore, output_format).await,
        LinkCommands::Show(args) => crud::show_link(args, datastore, output_format).await,
        LinkCommands::Update(args) => crud::update_link(args, datastore, output_format).await,
        LinkCommands::Delete(args) => crud::delete_link(args, datastore, output_format).await,
    }
}

#[cfg(test)]
mod tests;
