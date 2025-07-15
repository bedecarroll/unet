//! Node management CLI commands
//!
//! This module provides comprehensive CLI commands for managing network nodes,
//! including CRUD operations, status monitoring, and derived state operations.

use anyhow::Result;
use unet_core::datastore::DataStore;

pub use types::NodeCommands;

mod advanced;
mod crud;
mod monitoring;
mod types;

/// Execute a node command
pub async fn execute(
    command: NodeCommands,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    match command {
        NodeCommands::Add(args) => crud::add_node(args, datastore, output_format).await,
        NodeCommands::List(args) => crud::list_nodes(args, datastore, output_format).await,
        NodeCommands::Show(args) => crud::show_node(args, datastore, output_format).await,
        NodeCommands::Update(args) => crud::update_node(args, datastore, output_format).await,
        NodeCommands::Delete(args) => crud::delete_node(args, datastore, output_format).await,
        NodeCommands::Status(args) => monitoring::status_node(args, datastore, output_format).await,
        NodeCommands::Metrics(args) => {
            monitoring::metrics_node(args, datastore, output_format).await
        }
        NodeCommands::Compare(args) => {
            advanced::compare_nodes(args, datastore, output_format).await
        }
        NodeCommands::Polling(args) => advanced::polling_node(args, datastore, output_format).await,
        NodeCommands::History(args) => advanced::history_node(args, datastore, output_format).await,
    }
}
