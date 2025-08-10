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
        let stdin = std::io::stdin();
        let mut lock = stdin.lock();
        let mut reader = std::io::BufReader::new(&mut lock);
        if !confirm_deletion(false, &node.name, &mut reader)? {
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

// Extracted for testability
pub fn confirm_deletion(
    yes: bool,
    node_name: &str,
    reader: &mut impl std::io::BufRead,
) -> Result<bool> {
    if yes {
        return Ok(true);
    }
    eprintln!("Are you sure you want to delete node '{node_name}'? [y/N]");
    let mut input = String::new();
    reader.read_line(&mut input)?;
    let input = input.trim().to_lowercase();
    if input != "y" && input != "yes" {
        eprintln!("Cancelled");
        return Ok(false);
    }
    Ok(true)
}
