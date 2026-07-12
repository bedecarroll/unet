//! Node relationship tests for `SQLite` link operations
//!
//! Tests covering link queries between specific nodes and node-specific link retrieval.

use super::super::super::setup::{create_test_node, setup_test_db};
use super::super::helpers::create_test_link;
use crate::datastore::sqlite::links::*;
use uuid::Uuid;

#[tokio::test]
async fn test_get_links_for_node() {
    let test_db = setup_test_db().await;
    let node1_id = Uuid::new_v4();
    let node2_id = Uuid::new_v4();
    let node3_id = Uuid::new_v4();

    // Create nodes
    create_test_node(&test_db.store, node1_id, "node1")
        .await
        .unwrap();
    create_test_node(&test_db.store, node2_id, "node2")
        .await
        .unwrap();
    create_test_node(&test_db.store, node3_id, "node3")
        .await
        .unwrap();

    // Create links involving node1
    let link1 = create_test_link("link1-2", node1_id, Some(node2_id));
    let link2 = create_test_link("link1-3", node1_id, Some(node3_id));
    let link3 = create_test_link("link2-3", node2_id, Some(node3_id)); // Doesn't involve node1

    create_link(&test_db.store, &link1).await.unwrap();
    create_link(&test_db.store, &link2).await.unwrap();
    create_link(&test_db.store, &link3).await.unwrap();

    let result = get_links_for_node(&test_db.store, &node1_id).await;
    assert!(result.is_ok());

    let node_links = result.unwrap();
    assert_eq!(node_links.len(), 2);

    // Verify the links involve node1
    for link in &node_links {
        assert!(link.source_node_id == node1_id || link.dest_node_id == Some(node1_id));
    }
}

#[tokio::test]
async fn test_get_links_for_node_no_links() {
    let test_db = setup_test_db().await;
    let node_id = Uuid::new_v4();

    // Create node but no links
    create_test_node(&test_db.store, node_id, "isolated-node")
        .await
        .unwrap();

    let result = get_links_for_node(&test_db.store, &node_id).await;
    assert!(result.is_ok());

    let links = result.unwrap();
    assert_eq!(links.len(), 0);
}

#[tokio::test]
async fn test_get_links_between_nodes() {
    let test_db = setup_test_db().await;
    let node1_id = Uuid::new_v4();
    let node2_id = Uuid::new_v4();
    let node3_id = Uuid::new_v4();

    // Create nodes
    create_test_node(&test_db.store, node1_id, "node1")
        .await
        .unwrap();
    create_test_node(&test_db.store, node2_id, "node2")
        .await
        .unwrap();
    create_test_node(&test_db.store, node3_id, "node3")
        .await
        .unwrap();

    // Create multiple links between node1 and node2
    let link1 = create_test_link("link1-primary", node1_id, Some(node2_id));
    let link2 = create_test_link("link1-backup", node1_id, Some(node2_id));
    let link3 = create_test_link("link1-3", node1_id, Some(node3_id)); // Different pair

    create_link(&test_db.store, &link1).await.unwrap();
    create_link(&test_db.store, &link2).await.unwrap();
    create_link(&test_db.store, &link3).await.unwrap();

    let result = get_links_between_nodes(&test_db.store, &node1_id, &node2_id).await;
    assert!(result.is_ok());

    let between_links = result.unwrap();
    assert_eq!(between_links.len(), 2);

    // Verify the links are between the specified nodes
    for link in &between_links {
        assert!(
            (link.source_node_id == node1_id && link.dest_node_id == Some(node2_id))
                || (link.source_node_id == node2_id && link.dest_node_id == Some(node1_id))
        );
    }
}

#[tokio::test]
async fn test_get_links_between_nodes_no_links() {
    let test_db = setup_test_db().await;
    let node1_id = Uuid::new_v4();
    let node2_id = Uuid::new_v4();

    // Create nodes but no links between them
    create_test_node(&test_db.store, node1_id, "node1")
        .await
        .unwrap();
    create_test_node(&test_db.store, node2_id, "node2")
        .await
        .unwrap();

    let result = get_links_between_nodes(&test_db.store, &node1_id, &node2_id).await;
    assert!(result.is_ok());

    let links = result.unwrap();
    assert_eq!(links.len(), 0);
}
