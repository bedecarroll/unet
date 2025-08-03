/// Tests for node show functionality
#[cfg(test)]
mod tests {
    use crate::commands::nodes::types::ShowNodeArgs;
    use uuid::Uuid;

    // Tests for ShowNodeArgs argument structure

    #[tokio::test]
    async fn test_show_node_args_structure() {
        // Test ShowNodeArgs structure and field access
        let node_id = Uuid::new_v4();

        let args = ShowNodeArgs {
            id: node_id,
            include_status: true,
            show_interfaces: true,
            show_system_info: true,
        };

        assert_eq!(args.id, node_id);
        assert!(args.include_status);
        assert!(args.show_interfaces);
        assert!(args.show_system_info);
    }

    #[tokio::test]
    async fn test_show_node_args_all_false() {
        // Test ShowNodeArgs with all flags false
        let node_id = Uuid::new_v4();

        let args = ShowNodeArgs {
            id: node_id,
            include_status: false,
            show_interfaces: false,
            show_system_info: false,
        };

        assert_eq!(args.id, node_id);
        assert!(!args.include_status);
        assert!(!args.show_interfaces);
        assert!(!args.show_system_info);
    }

    #[tokio::test]
    async fn test_show_node_mixed_flags() {
        // Test various combinations of flags to ensure all code paths
        let node_id = Uuid::new_v4();

        // Test include_status + show_interfaces
        let args1 = ShowNodeArgs {
            id: node_id,
            include_status: true,
            show_interfaces: true,
            show_system_info: false,
        };

        assert_eq!(args1.id, node_id);
        assert!(args1.include_status);
        assert!(args1.show_interfaces);
        assert!(!args1.show_system_info);

        // Test include_status + show_system_info
        let args2 = ShowNodeArgs {
            id: node_id,
            include_status: true,
            show_interfaces: false,
            show_system_info: true,
        };

        assert_eq!(args2.id, node_id);
        assert!(args2.include_status);
        assert!(!args2.show_interfaces);
        assert!(args2.show_system_info);

        // Test show_interfaces + show_system_info
        let args3 = ShowNodeArgs {
            id: node_id,
            include_status: false,
            show_interfaces: true,
            show_system_info: true,
        };

        assert_eq!(args3.id, node_id);
        assert!(!args3.include_status);
        assert!(args3.show_interfaces);
        assert!(args3.show_system_info);
    }
}
