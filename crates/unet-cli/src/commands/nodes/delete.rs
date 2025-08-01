/// Node deletion operations
use anyhow::Result;
use unet_core::datastore::DataStore;

use super::types::DeleteNodeArgs;

pub async fn delete_node(
    args: DeleteNodeArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    // Get node first to show confirmation
    let node = datastore.get_node_required(&args.id).await?;

    if !args.yes {
        eprintln!(
            "Are you sure you want to delete node '{}'? [y/N]",
            node.name
        );
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();
        if input != "y" && input != "yes" {
            eprintln!("Cancelled");
            return Ok(());
        }
    }

    datastore.delete_node(&args.id).await?;

    let output = serde_json::json!({
        "message": format!("Node '{}' ({}) deleted successfully", node.name, node.id),
        "node_id": node.id,
        "node_name": node.name
    });

    crate::commands::print_output(&output, output_format)?;

    Ok(())
}
