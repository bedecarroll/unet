//! Hierarchy operations tests for `Location` model

use crate::models::*;
use uuid::Uuid;

#[test]
fn test_location_hierarchy_relationships() {
    let root = Location::new_root("USA".to_string(), "country".to_string());

    let mut child = Location::new_child("California".to_string(), "state".to_string(), "USA");
    child.parent_id = Some(root.id);

    let mut grandchild = Location::new_child(
        "San Francisco".to_string(),
        "city".to_string(),
        "USA/California",
    );
    grandchild.parent_id = Some(child.id);

    // Test ancestor relationships
    assert!(root.is_ancestor_of(&child));
    assert!(root.is_ancestor_of(&grandchild));
    assert!(child.is_ancestor_of(&grandchild));
    assert!(!child.is_ancestor_of(&root));
    assert!(!grandchild.is_ancestor_of(&root));

    // Test descendant relationships
    assert!(child.is_descendant_of(&root));
    assert!(grandchild.is_descendant_of(&root));
    assert!(grandchild.is_descendant_of(&child));
    assert!(!root.is_descendant_of(&child));

    // Test parent-child relationships
    assert!(child.is_child_of(&root));
    assert!(grandchild.is_child_of(&child));
    assert!(root.is_parent_of(&child));
    assert!(child.is_parent_of(&grandchild));
    assert!(!root.is_child_of(&child));
    assert!(!child.is_parent_of(&root));
}

#[test]
fn test_location_detect_circular_reference() {
    let root = Location::new_root("USA".to_string(), "country".to_string());

    let mut child = Location::new_child("California".to_string(), "state".to_string(), "USA");
    child.parent_id = Some(root.id);

    let locations = vec![root.clone(), child.clone()];

    // Test self-reference
    assert!(Location::detect_circular_reference(
        &locations, root.id, root.id
    ));

    // Test valid parent-child
    assert!(!Location::detect_circular_reference(
        &locations, root.id, child.id
    ));

    // Test circular reference (child becoming parent of its ancestor)
    assert!(Location::detect_circular_reference(
        &locations, child.id, root.id
    ));
}

#[test]
fn test_location_get_ancestors() {
    let root = Location::new_root("USA".to_string(), "country".to_string());

    let mut child = Location::new_child("California".to_string(), "state".to_string(), "USA");
    child.parent_id = Some(root.id);

    let mut grandchild = Location::new_child(
        "San Francisco".to_string(),
        "city".to_string(),
        "USA/California",
    );
    grandchild.parent_id = Some(child.id);

    let locations = vec![root.clone(), child.clone(), grandchild.clone()];

    // Root has no ancestors
    let root_ancestors = root.get_ancestors(&locations);
    assert!(root_ancestors.is_empty());

    // Child has root as ancestor
    let child_ancestors = child.get_ancestors(&locations);
    assert_eq!(child_ancestors.len(), 1);
    assert_eq!(child_ancestors[0].id, root.id);

    // Grandchild has child and root as ancestors
    let grandchild_ancestors = grandchild.get_ancestors(&locations);
    assert_eq!(grandchild_ancestors.len(), 2);
    assert_eq!(grandchild_ancestors[0].id, child.id);
    assert_eq!(grandchild_ancestors[1].id, root.id);
}

#[test]
fn test_location_get_descendants() {
    let root = Location::new_root("USA".to_string(), "country".to_string());

    let mut child1 = Location::new_child("California".to_string(), "state".to_string(), "USA");
    child1.parent_id = Some(root.id);

    let mut child2 = Location::new_child("Texas".to_string(), "state".to_string(), "USA");
    child2.parent_id = Some(root.id);

    let mut grandchild = Location::new_child(
        "San Francisco".to_string(),
        "city".to_string(),
        "USA/California",
    );
    grandchild.parent_id = Some(child1.id);

    let locations = vec![
        root.clone(),
        child1.clone(),
        child2.clone(),
        grandchild.clone(),
    ];

    // Root has all others as descendants
    let root_descendants = root.get_descendants(&locations);
    assert_eq!(root_descendants.len(), 3);

    // Child1 has only grandchild as descendant
    let child1_descendants = child1.get_descendants(&locations);
    assert_eq!(child1_descendants.len(), 1);
    assert_eq!(child1_descendants[0].id, grandchild.id);

    // Child2 has no descendants
    let child2_descendants = child2.get_descendants(&locations);
    assert!(child2_descendants.is_empty());

    // Grandchild has no descendants
    let grandchild_descendants = grandchild.get_descendants(&locations);
    assert!(grandchild_descendants.is_empty());
}

#[test]
fn test_location_get_children() {
    let root = Location::new_root("USA".to_string(), "country".to_string());

    let mut child1 = Location::new_child("California".to_string(), "state".to_string(), "USA");
    child1.parent_id = Some(root.id);

    let mut child2 = Location::new_child("Texas".to_string(), "state".to_string(), "USA");
    child2.parent_id = Some(root.id);

    let mut grandchild = Location::new_child(
        "San Francisco".to_string(),
        "city".to_string(),
        "USA/California",
    );
    grandchild.parent_id = Some(child1.id);

    let locations = vec![
        root.clone(),
        child1.clone(),
        child2.clone(),
        grandchild.clone(),
    ];

    // Root has two direct children
    let root_children = root.get_children(&locations);
    assert_eq!(root_children.len(), 2);
    let child_ids: Vec<Uuid> = root_children.iter().map(|l| l.id).collect();
    assert!(child_ids.contains(&child1.id));
    assert!(child_ids.contains(&child2.id));

    // Child1 has one direct child
    let child1_children = child1.get_children(&locations);
    assert_eq!(child1_children.len(), 1);
    assert_eq!(child1_children[0].id, grandchild.id);

    // Child2 has no direct children
    let child2_children = child2.get_children(&locations);
    assert!(child2_children.is_empty());
}
