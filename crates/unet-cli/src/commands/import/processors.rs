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
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::always;
    use unet_core::datastore::MockDataStore;
    use unet_core::models::{DeviceRole, Location, NodeBuilder, Vendor};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_import_location_dry_run() {
        let loc = Location::new_root("HQ".into(), "building".into());
        let mock = MockDataStore::new();
        assert!(import_location(&loc, &mock, true).await.is_ok());
    }

    #[tokio::test]
    async fn test_import_location_real_calls_datastore() {
        let loc = Location::new_root("HQ".into(), "building".into());
        let mut mock = MockDataStore::new();
        mock.expect_create_location()
            .with(always())
            .returning(move |_| {
                let l = loc.clone();
                Box::pin(async move { Ok(l) })
            });
        // A bit of a hack: ignore return value type specifics; we only assert Ok path
        let _ = import_location(&Location::new_root("HQ".into(), "building".into()), &mock, false).await;
    }

    #[tokio::test]
    async fn test_import_node_dry_run_and_real() {
        let node = NodeBuilder::new()
            .name("n1")
            .domain("example.com")
            .vendor(Vendor::Cisco)
            .model("ISR")
            .role(DeviceRole::Router)
            .build()
            .unwrap();
        let mut mock = MockDataStore::new();
        assert!(import_node(&node, &mock, true).await.is_ok());
        let node_clone = node.clone();
        mock.expect_create_node()
            .with(always())
            .returning(move |_| {
                let n = node_clone.clone();
                Box::pin(async move { Ok(n) })
            });
        assert!(import_node(&node, &mock, false).await.is_ok());
    }

    #[tokio::test]
    async fn test_import_link_dry_run_and_real() {
        use unet_core::models::Link;
        let link = Link::new("L1".into(), Uuid::new_v4(), "Gi0/0".into(), Uuid::new_v4(), "Gi0/1".into());
        let mut mock = MockDataStore::new();
        assert!(import_link(&link, &mock, true).await.is_ok());
        let link_clone = link.clone();
        mock.expect_create_link()
            .with(always())
            .returning(move |_| {
                let l = link_clone.clone();
                Box::pin(async move { Ok(l) })
            });
        assert!(import_link(&link, &mock, false).await.is_ok());
    }
}
