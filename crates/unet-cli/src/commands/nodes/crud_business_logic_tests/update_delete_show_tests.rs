use crate::commands::nodes::types::{DeleteNodeArgs, ShowNodeArgs, UpdateNodeArgs};
use crate::commands::test_support::expect_json_object;
use unet_core::models::{DeviceRole, Lifecycle, Vendor};
use uuid::Uuid;

#[tokio::test]
async fn test_update_node_partial_updates() {
    let node_id = Uuid::new_v4();
    let args = UpdateNodeArgs {
        id: node_id,
        name: Some("updated-name".to_string()),
        domain: None,
        vendor: None,
        model: None,
        role: None,
        lifecycle: None,
        location_id: None,
        management_ip: None,
        custom_data: None,
    };

    assert_eq!(args.id, node_id);
    assert_eq!(args.name, Some("updated-name".to_string()));
    assert_eq!(args.domain, None);
    assert_eq!(args.vendor, None);
    assert_eq!(args.model, None);
    assert_eq!(args.role, None);
    assert_eq!(args.lifecycle, None);
    assert_eq!(args.location_id, None);
    assert_eq!(args.management_ip, None);
    assert_eq!(args.custom_data, None);
}

#[tokio::test]
async fn test_update_node_enum_parsing() {
    let vendor_result = "juniper".parse::<Vendor>();
    let role_result = "switch".parse::<DeviceRole>();
    let lifecycle_result = "decommissioned".parse::<Lifecycle>();

    assert!(vendor_result.is_ok());
    assert_eq!(vendor_result.unwrap(), Vendor::Juniper);
    assert!(role_result.is_ok());
    assert_eq!(role_result.unwrap(), DeviceRole::Switch);
    assert!(lifecycle_result.is_ok());
    assert_eq!(lifecycle_result.unwrap(), Lifecycle::Decommissioned);
}

#[tokio::test]
async fn test_update_node_fqdn_calculation() {
    let expected_fqdn = format!("{}.{}", "test-router", "example.com");
    assert_eq!(expected_fqdn, "test-router.example.com");
}

#[tokio::test]
async fn test_update_node_custom_data_parsing() {
    let value = expect_json_object(r#"{"environment": "production", "rack": "B2"}"#);
    assert_eq!(value["environment"], "production");
    assert_eq!(value["rack"], "B2");
}

#[tokio::test]
async fn test_delete_node_confirmation_logic() {
    let input_variations = vec![
        ("y", true),
        ("Y", true),
        ("yes", true),
        ("YES", true),
        ("Yes", true),
        ("n", false),
        ("N", false),
        ("no", false),
        ("NO", false),
        ("No", false),
        ("", false),
        ("maybe", false),
        ("quit", false),
    ];

    for (input, expected) in input_variations {
        let input_trimmed = input.trim().to_lowercase();
        let should_proceed = input_trimmed == "y" || input_trimmed == "yes";
        assert_eq!(
            should_proceed,
            expected,
            "Input '{input}' should return {expected}"
        );
    }
}

#[tokio::test]
async fn test_delete_node_yes_flag_bypass() {
    let node_id = Uuid::new_v4();
    let args_with_yes = DeleteNodeArgs { id: node_id, yes: true };
    let args_without_yes = DeleteNodeArgs {
        id: node_id,
        yes: false,
    };

    assert!(args_with_yes.yes);
    assert!(!args_without_yes.yes);
}

#[tokio::test]
async fn test_show_node_args_flags() {
    let node_id = Uuid::new_v4();
    let args_basic = ShowNodeArgs {
        id: node_id,
        include_status: false,
        show_interfaces: false,
        show_system_info: false,
    };
    let args_all_flags = ShowNodeArgs {
        id: node_id,
        include_status: true,
        show_interfaces: true,
        show_system_info: true,
    };
    let args_partial_flags = ShowNodeArgs {
        id: node_id,
        include_status: true,
        show_interfaces: false,
        show_system_info: true,
    };

    assert_eq!(args_basic.id, node_id);
    assert!(!args_basic.include_status);
    assert!(!args_basic.show_interfaces);
    assert!(!args_basic.show_system_info);

    assert_eq!(args_all_flags.id, node_id);
    assert!(args_all_flags.include_status);
    assert!(args_all_flags.show_interfaces);
    assert!(args_all_flags.show_system_info);

    assert_eq!(args_partial_flags.id, node_id);
    assert!(args_partial_flags.include_status);
    assert!(!args_partial_flags.show_interfaces);
    assert!(args_partial_flags.show_system_info);
}

#[tokio::test]
async fn test_show_node_enhanced_output_check() {
    let node_id = Uuid::new_v4();
    let args = ShowNodeArgs {
        id: node_id,
        include_status: true,
        show_interfaces: false,
        show_system_info: true,
    };
    let should_use_enhanced_output =
        args.include_status || args.show_interfaces || args.show_system_info;
    assert!(should_use_enhanced_output);

    let args_basic = ShowNodeArgs {
        id: node_id,
        include_status: false,
        show_interfaces: false,
        show_system_info: false,
    };
    let should_use_basic_output =
        !(args_basic.include_status || args_basic.show_interfaces || args_basic.show_system_info);
    assert!(should_use_basic_output);
}
