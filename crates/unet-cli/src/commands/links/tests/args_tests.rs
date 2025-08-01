/// Tests for link command arguments
use crate::commands::links::types::*;
use uuid::Uuid;

#[tokio::test]
async fn test_add_link_args_creation() {
    let source_node_id = Uuid::new_v4();
    let target_node_id = Uuid::new_v4();

    let args = AddLinkArgs {
        name: "router-a-to-router-b".to_string(),
        node_a_id: source_node_id,
        node_a_interface: "GigabitEthernet0/1".to_string(),
        node_z_id: Some(target_node_id),
        node_z_interface: Some("GigabitEthernet0/2".to_string()),
        bandwidth_bps: Some(1_000_000_000),
        description: Some("Primary link between routers".to_string()),
        custom_data: Some(r#"{"provider": "ISP"}"#.to_string()),
    };

    assert_eq!(args.name, "router-a-to-router-b");
    assert_eq!(args.node_a_id, source_node_id);
    assert_eq!(args.node_a_interface, "GigabitEthernet0/1");
    assert_eq!(args.node_z_id, Some(target_node_id));
    assert_eq!(
        args.node_z_interface,
        Some("GigabitEthernet0/2".to_string())
    );
    assert_eq!(args.bandwidth_bps, Some(1_000_000_000));
    assert!(args.description.is_some());
    assert!(args.custom_data.is_some());
}

#[tokio::test]
async fn test_add_link_args_minimal() {
    let source_node_id = Uuid::new_v4();

    let args = AddLinkArgs {
        name: "internet-link".to_string(),
        node_a_id: source_node_id,
        node_a_interface: "WAN0".to_string(),
        node_z_id: None,
        node_z_interface: None,
        bandwidth_bps: None,
        description: None,
        custom_data: None,
    };

    assert_eq!(args.name, "internet-link");
    assert_eq!(args.node_a_id, source_node_id);
    assert_eq!(args.node_a_interface, "WAN0");
    assert_eq!(args.node_z_id, None);
    assert_eq!(args.node_z_interface, None);
    assert_eq!(args.bandwidth_bps, None);
    assert_eq!(args.description, None);
    assert_eq!(args.custom_data, None);
}

#[tokio::test]
async fn test_list_link_args_creation() {
    let node_id = Uuid::new_v4();

    let args = ListLinkArgs {
        node_id: Some(node_id),
        min_bandwidth: Some(100_000_000),
        page: 2,
        per_page: 50,
    };

    assert_eq!(args.node_id, Some(node_id));
    assert_eq!(args.min_bandwidth, Some(100_000_000));
    assert_eq!(args.page, 2);
    assert_eq!(args.per_page, 50);
}

#[tokio::test]
async fn test_list_link_args_defaults() {
    let args = ListLinkArgs {
        node_id: None,
        min_bandwidth: None,
        page: 1,
        per_page: 20,
    };

    assert_eq!(args.node_id, None);
    assert_eq!(args.min_bandwidth, None);
    assert_eq!(args.page, 1);
    assert_eq!(args.per_page, 20);
}

#[tokio::test]
async fn test_show_link_args_creation() {
    let link_id = Uuid::new_v4();

    let args = ShowLinkArgs { id: link_id };

    assert_eq!(args.id, link_id);
}

#[tokio::test]
async fn test_update_link_args_creation() {
    let link_id = Uuid::new_v4();
    let source_node_id = Uuid::new_v4();
    let target_node_id = Uuid::new_v4();

    let args = UpdateLinkArgs {
        id: link_id,
        name: Some("updated-link".to_string()),
        node_a_id: Some(source_node_id),
        node_a_interface: Some("FastEthernet0/1".to_string()),
        node_z_id: Some(target_node_id),
        node_z_interface: Some("FastEthernet0/2".to_string()),
        bandwidth_bps: Some(100_000_000),
        description: Some("Updated description".to_string()),
        custom_data: Some(r#"{"updated": true}"#.to_string()),
    };

    assert_eq!(args.id, link_id);
    assert_eq!(args.name, Some("updated-link".to_string()));
    assert_eq!(args.node_a_id, Some(source_node_id));
    assert_eq!(args.node_a_interface, Some("FastEthernet0/1".to_string()));
    assert_eq!(args.node_z_id, Some(target_node_id));
    assert_eq!(args.node_z_interface, Some("FastEthernet0/2".to_string()));
    assert_eq!(args.bandwidth_bps, Some(100_000_000));
    assert_eq!(args.description, Some("Updated description".to_string()));
    assert!(args.custom_data.is_some());
}

#[tokio::test]
async fn test_update_link_args_partial() {
    let link_id = Uuid::new_v4();

    let args = UpdateLinkArgs {
        id: link_id,
        name: Some("partial-update".to_string()),
        node_a_id: None,
        node_a_interface: None,
        node_z_id: None,
        node_z_interface: None,
        bandwidth_bps: Some(50_000_000),
        description: None,
        custom_data: None,
    };

    assert_eq!(args.id, link_id);
    assert_eq!(args.name, Some("partial-update".to_string()));
    assert_eq!(args.node_a_id, None);
    assert_eq!(args.node_a_interface, None);
    assert_eq!(args.node_z_id, None);
    assert_eq!(args.node_z_interface, None);
    assert_eq!(args.bandwidth_bps, Some(50_000_000));
    assert_eq!(args.description, None);
    assert_eq!(args.custom_data, None);
}

#[tokio::test]
async fn test_delete_link_args_creation() {
    let link_id = Uuid::new_v4();

    let args = DeleteLinkArgs {
        id: link_id,
        yes: true,
    };

    assert_eq!(args.id, link_id);
    assert!(args.yes);
}

#[tokio::test]
async fn test_delete_link_args_interactive() {
    let link_id = Uuid::new_v4();

    let args = DeleteLinkArgs {
        id: link_id,
        yes: false,
    };

    assert_eq!(args.id, link_id);
    assert!(!args.yes);
}

#[tokio::test]
async fn test_link_command_variants() {
    // Test that all LinkCommand variants can be constructed
    let source_node_id = Uuid::new_v4();
    let link_id = Uuid::new_v4();

    let add_args = AddLinkArgs {
        name: "test".to_string(),
        node_a_id: source_node_id,
        node_a_interface: "eth0".to_string(),
        node_z_id: None,
        node_z_interface: None,
        bandwidth_bps: None,
        description: None,
        custom_data: None,
    };

    let list_args = ListLinkArgs {
        node_id: None,
        min_bandwidth: None,
        page: 1,
        per_page: 20,
    };

    let show_args = ShowLinkArgs { id: link_id };

    let update_args = UpdateLinkArgs {
        id: link_id,
        name: None,
        node_a_id: None,
        node_a_interface: None,
        node_z_id: None,
        node_z_interface: None,
        bandwidth_bps: None,
        description: None,
        custom_data: None,
    };

    let delete_args = DeleteLinkArgs {
        id: link_id,
        yes: false,
    };

    // Verify the commands can be created successfully
    assert!(matches!(LinkCommands::Add(add_args), LinkCommands::Add(_)));
    assert!(matches!(
        LinkCommands::List(list_args),
        LinkCommands::List(_)
    ));
    assert!(matches!(
        LinkCommands::Show(show_args),
        LinkCommands::Show(_)
    ));
    assert!(matches!(
        LinkCommands::Update(update_args),
        LinkCommands::Update(_)
    ));
    assert!(matches!(
        LinkCommands::Delete(delete_args),
        LinkCommands::Delete(_)
    ));
}
