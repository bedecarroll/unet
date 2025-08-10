/// CRUD operations for link management
use anyhow::Result;
use serde_json::Value as JsonValue;
use unet_core::datastore::DataStore;
use unet_core::prelude::*;

use super::types::{AddLinkArgs, DeleteLinkArgs, ListLinkArgs, ShowLinkArgs, UpdateLinkArgs};

pub async fn add_link(
    args: AddLinkArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    // Parse custom data if provided
    let custom_data = if let Some(json_str) = args.custom_data {
        Some(serde_json::from_str::<JsonValue>(&json_str)?)
    } else {
        None
    };

    // Build link
    let mut builder = LinkBuilder::new()
        .name(args.name)
        .source_node_id(args.node_a_id)
        .node_a_interface(args.node_a_interface);

    if let Some(node_z_id) = args.node_z_id {
        builder = builder.dest_node_id(node_z_id);
    }

    if let Some(node_z_interface) = args.node_z_interface {
        builder = builder.node_z_interface(node_z_interface);
    }

    if let Some(bandwidth_bps) = args.bandwidth_bps {
        builder = builder.bandwidth(bandwidth_bps);
    }

    if let Some(description) = args.description {
        builder = builder.description(description);
    }

    if let Some(custom_data) = custom_data {
        builder = builder.custom_data(custom_data);
    }

    let link = builder
        .build()
        .map_err(|e| anyhow::anyhow!("Link validation failed: {}", e))?;

    // Create link in datastore
    let created_link = datastore.create_link(&link).await?;

    crate::commands::print_output(&created_link, output_format)?;

    Ok(())
}

pub async fn list_links(
    args: ListLinkArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    let mut filters = Vec::new();

    if let Some(node_id) = args.node_id {
        // We'll need to filter by either node_a_id or node_z_id
        // For now, just filter by node_a_id as an example
        filters.push(Filter {
            field: "node_a_id".to_owned(),
            operation: FilterOperation::Equals,
            value: FilterValue::Uuid(node_id),
        });
    }

    // Note: For bandwidth filtering, we'd need to extend the CSV datastore
    // to support numeric comparisons. For now, we'll skip this filter.

    let options = QueryOptions {
        filters,
        sort: vec![Sort {
            field: "interface_a".to_owned(),
            direction: SortDirection::Ascending,
        }],
        pagination: Some(Pagination {
            offset: usize::try_from((args.page - 1) * args.per_page)?,
            limit: usize::try_from(args.per_page)?,
        }),
    };

    let result = datastore.list_links(&options).await?;

    crate::commands::print_output(&result, output_format)?;

    Ok(())
}

pub async fn show_link(
    args: ShowLinkArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    let link = datastore.get_link_required(&args.id).await?;

    crate::commands::print_output(&link, output_format)?;

    Ok(())
}

pub async fn update_link(
    args: UpdateLinkArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    let mut link = datastore.get_link_required(&args.id).await?;

    // Update fields that were provided
    if let Some(name) = args.name {
        link.name = name;
    }

    if let Some(node_a_id) = args.node_a_id {
        link.source_node_id = node_a_id;
    }

    if let Some(node_a_interface) = args.node_a_interface {
        link.node_a_interface = node_a_interface;
    }

    if let Some(node_z_id) = args.node_z_id {
        link.dest_node_id = Some(node_z_id);
    }

    if let Some(node_z_interface) = args.node_z_interface {
        link.node_z_interface = Some(node_z_interface);
    }

    if let Some(bandwidth_bps) = args.bandwidth_bps {
        link.bandwidth = Some(bandwidth_bps);
    }

    if let Some(description) = args.description {
        link.description = Some(description);
    }

    if let Some(custom_data_str) = args.custom_data {
        link.custom_data = serde_json::from_str(&custom_data_str)?;
    }

    let updated_link = datastore.update_link(&link).await?;

    crate::commands::print_output(&updated_link, output_format)?;

    Ok(())
}

pub async fn delete_link(
    args: DeleteLinkArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    // Check if link exists first
    let link = datastore.get_link_required(&args.id).await?;

    if !args.yes {
        let stdin = std::io::stdin();
        let mut lock = stdin.lock();
        let mut reader = std::io::BufReader::new(&mut lock);
        if !confirm_link_deletion(false, &link, &mut reader)? {
            return Ok(());
        }
    }

    datastore.delete_link(&args.id).await?;

    let output = serde_json::json!({
        "message": "Link deleted successfully",
        "id": args.id
    });

    crate::commands::print_output(&output, output_format)?;

    Ok(())
}

// Testable confirmation helper for link deletion
pub(crate) fn confirm_link_deletion(
    yes: bool,
    link: &Link,
    reader: &mut impl std::io::BufRead,
) -> Result<bool> {
    if yes {
        return Ok(true);
    }
    println!(
        "Are you sure you want to delete link {} <-> {} (ID: {})? [y/N]",
        link.node_a_interface,
        link
            .node_z_interface
            .as_deref()
            .unwrap_or("internet"),
        link.id
    );
    let mut input = String::new();
    reader.read_line(&mut input)?;
    if !input.trim().to_lowercase().starts_with('y') {
        println!("Cancelled.");
        return Ok(false);
    }
    Ok(true)
}

#[cfg(test)]
mod confirm_tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_confirm_link_deletion_yes_and_no() {
        let link = Link::new(
            "L1".into(),
            Uuid::new_v4(),
            "Gi0/0".into(),
            Uuid::new_v4(),
            "Gi0/1".into(),
        );

        // No
        let mut cur = std::io::Cursor::new(b"no\n".to_vec());
        let res = confirm_link_deletion(false, &link, &mut cur).unwrap();
        assert!(!res);

        // Yes
        let mut cur2 = std::io::Cursor::new(b"y\n".to_vec());
        let res2 = confirm_link_deletion(false, &link, &mut cur2).unwrap();
        assert!(res2);
    }
}
