/// Import processing functions for different entity types
use anyhow::Result;
use tracing::info;
use unet_core::datastore::DataStore;
use unet_core::prelude::*;

use super::ImportArgs;
use super::loaders::{load_links, load_locations, load_nodes};
use super::stats::{ImportStats, process_import_item};

/// Import locations from source path
pub async fn import_locations(
    args: &ImportArgs,
    datastore: &dyn DataStore,
    stats: &mut ImportStats,
) -> Result<()> {
    let Some(locations) = load_locations(&args.from).await? else {
        return Ok(());
    };

    info!("Importing {} locations...", locations.len());
    for location in locations {
        process_import_item(
            || import_location(&location, datastore, args.dry_run),
            &format!("location '{}'", location.name),
            args.continue_on_error,
            stats,
        )
        .await?;
    }
    Ok(())
}

/// Import nodes from source path
pub async fn import_nodes(
    args: &ImportArgs,
    datastore: &dyn DataStore,
    stats: &mut ImportStats,
) -> Result<()> {
    let Some(nodes) = load_nodes(&args.from).await? else {
        return Ok(());
    };

    info!("Importing {} nodes...", nodes.len());
    for node in nodes {
        process_import_item(
            || import_node(&node, datastore, args.dry_run),
            &format!("node '{}'", node.name),
            args.continue_on_error,
            stats,
        )
        .await?;
    }
    Ok(())
}

/// Import links from source path
pub async fn import_links(
    args: &ImportArgs,
    datastore: &dyn DataStore,
    stats: &mut ImportStats,
) -> Result<()> {
    let Some(links) = load_links(&args.from).await? else {
        return Ok(());
    };

    info!("Importing {} links...", links.len());
    for link in links {
        process_import_item(
            || import_link(&link, datastore, args.dry_run),
            &format!("link '{}'", link.name),
            args.continue_on_error,
            stats,
        )
        .await?;
    }
    Ok(())
}

/// Import a single location
async fn import_location(
    location: &Location,
    datastore: &dyn DataStore,
    dry_run: bool,
) -> Result<()> {
    if dry_run {
        info!("Would import location: {}", location.name);
        return Ok(());
    }

    datastore.create_location(location).await?;
    Ok(())
}

/// Import a single node
async fn import_node(node: &Node, datastore: &dyn DataStore, dry_run: bool) -> Result<()> {
    if dry_run {
        info!("Would import node: {}", node.name);
        return Ok(());
    }

    datastore.create_node(node).await?;
    Ok(())
}

/// Import a single link
async fn import_link(link: &Link, datastore: &dyn DataStore, dry_run: bool) -> Result<()> {
    if dry_run {
        info!("Would import link: {}", link.name);
        return Ok(());
    }

    datastore.create_link(link).await?;
    Ok(())
}
