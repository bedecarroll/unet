//! Tests for Node struct and methods
//!
//! Comprehensive test suite covering node creation, validation,
//! FQDN management, and custom data operations.

#[cfg(test)]
mod node_tests {
    use super::super::Node;
    use crate::models::{DeviceRole, Lifecycle, Vendor};
    use serde_json::{Value, json};

    fn create_test_node() -> Node {
        Node::new(
            "test-router".to_string(),
            "example.com".to_string(),
            Vendor::Cisco,
            DeviceRole::Router,
        )
    }

    #[test]
    fn test_node_new() {
        let node = Node::new(
            "router-1".to_string(),
            "corp.com".to_string(),
            Vendor::Juniper,
            DeviceRole::Switch,
        );

        assert_eq!(node.name, "router-1");
        assert_eq!(node.domain, "corp.com");
        assert_eq!(node.fqdn, "router-1.corp.com");
        assert_eq!(node.vendor, Vendor::Juniper);
        assert_eq!(node.role, DeviceRole::Switch);
        assert_eq!(node.lifecycle, Lifecycle::Planned);
        assert!(node.model.is_empty());
        assert!(node.management_ip.is_none());
        assert!(node.location_id.is_none());
        assert!(node.platform.is_none());
        assert!(node.version.is_none());
        assert!(node.serial_number.is_none());
        assert!(node.asset_tag.is_none());
        assert!(node.purchase_date.is_none());
        assert!(node.warranty_expires.is_none());
        assert_eq!(node.custom_data, Value::Null);
    }

    #[test]
    fn test_node_new_empty_domain() {
        let node = Node::new(
            "standalone-device".to_string(),
            String::new(),
            Vendor::Cisco,
            DeviceRole::AccessPoint,
        );

        assert_eq!(node.name, "standalone-device");
        assert_eq!(node.domain, "");
        assert_eq!(node.fqdn, "standalone-device");
    }

    #[test]
    fn test_node_validate_success() {
        let mut node = create_test_node();
        node.model = "ISR4451".to_string();

        let result = node.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_node_validate_empty_name() {
        let mut node = create_test_node();
        node.name = String::new();
        node.model = "ISR4451".to_string();

        let result = node.validate();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Node name cannot be empty");
    }

    #[test]
    fn test_node_validate_invalid_name_characters() {
        let mut node = create_test_node();
        node.name = "test router!".to_string();
        node.model = "ISR4451".to_string();

        let result = node.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("alphanumeric characters"));
    }

    #[test]
    fn test_node_validate_valid_name_characters() {
        let mut node = create_test_node();
        node.name = "test-router_01".to_string();
        node.model = "ISR4451".to_string();
        node.update_fqdn();

        let result = node.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_node_validate_invalid_domain() {
        let mut node = create_test_node();
        node.domain = "invalid@domain".to_string();
        node.model = "ISR4451".to_string();
        node.update_fqdn();

        let result = node.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Domain must contain only"));
    }

    #[test]
    fn test_node_validate_fqdn_mismatch() {
        let mut node = create_test_node();
        node.fqdn = "different.example.com".to_string();
        node.model = "ISR4451".to_string();

        let result = node.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("FQDN"));
    }

    #[test]
    fn test_node_validate_empty_model() {
        let node = create_test_node(); // model defaults to empty

        let result = node.validate();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Node model cannot be empty");
    }

    #[test]
    fn test_update_fqdn_with_domain() {
        let mut node = create_test_node();
        node.name = "updated-router".to_string();
        node.domain = "new-domain.com".to_string();

        node.update_fqdn();

        assert_eq!(node.fqdn, "updated-router.new-domain.com");
    }

    #[test]
    fn test_update_fqdn_empty_domain() {
        let mut node = create_test_node();
        node.name = "standalone".to_string();
        node.domain = String::new();

        node.update_fqdn();

        assert_eq!(node.fqdn, "standalone");
    }

    #[test]
    fn test_get_custom_data_null_data() {
        let node = create_test_node();

        let result = node.get_custom_data("any.path");
        assert!(result.is_none());
    }

    #[test]
    fn test_get_custom_data_simple_path() {
        let mut node = create_test_node();
        node.custom_data = json!({
            "config": {
                "snmp": {
                    "community": "public"
                }
            }
        });

        let result = node.get_custom_data("config.snmp.community");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), &Value::String("public".to_string()));
    }

    #[test]
    fn test_get_custom_data_missing_path() {
        let mut node = create_test_node();
        node.custom_data = json!({
            "config": {
                "snmp": {
                    "community": "public"
                }
            }
        });

        let result = node.get_custom_data("config.snmp.missing");
        assert!(result.is_none());
    }

    #[test]
    fn test_get_custom_data_invalid_path() {
        let mut node = create_test_node();
        node.custom_data = json!({
            "config": "not_an_object"
        });

        let result = node.get_custom_data("config.snmp.community");
        assert!(result.is_none());
    }

    #[test]
    fn test_set_custom_data_null_to_object() {
        let mut node = create_test_node();
        assert_eq!(node.custom_data, Value::Null);

        let result = node.set_custom_data(
            "config.snmp.community",
            Value::String("private".to_string()),
        );
        assert!(result.is_ok());

        let retrieved = node.get_custom_data("config.snmp.community");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), &Value::String("private".to_string()));
    }

    #[test]
    fn test_set_custom_data_existing_object() {
        let mut node = create_test_node();
        node.custom_data = json!({
            "existing": "value"
        });

        let result = node.set_custom_data("config.timeout", Value::Number(30.into()));
        assert!(result.is_ok());

        let retrieved = node.get_custom_data("config.timeout");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), &Value::Number(30.into()));

        // Existing data should still be there
        let existing = node.get_custom_data("existing");
        assert!(existing.is_some());
        assert_eq!(existing.unwrap(), &Value::String("value".to_string()));
    }

    #[test]
    fn test_set_custom_data_empty_path() {
        let mut node = create_test_node();

        let result = node.set_custom_data("", Value::String("test".to_string()));
        // Empty string split on '.' gives [""], not empty vec, so this actually succeeds
        // but sets a key named "" (empty string) in the root object
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_custom_data_overwrite_existing() {
        let mut node = create_test_node();
        node.custom_data = json!({
            "config": {
                "timeout": 10
            }
        });

        let result = node.set_custom_data("config.timeout", Value::Number(60.into()));
        assert!(result.is_ok());

        let retrieved = node.get_custom_data("config.timeout");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), &Value::Number(60.into()));
    }

    #[test]
    fn test_set_custom_data_set_on_non_object() {
        let mut node = create_test_node();
        node.custom_data = Value::String("not_an_object".to_string());

        let result = node.set_custom_data("timeout", Value::Number(30.into()));
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Cannot set value on non-object")
        );
    }

    #[test]
    fn test_set_custom_data_deep_nesting() {
        let mut node = create_test_node();

        let result = node.set_custom_data(
            "level1.level2.level3.level4.value",
            Value::String("deep".to_string()),
        );
        assert!(result.is_ok());

        let retrieved = node.get_custom_data("level1.level2.level3.level4.value");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), &Value::String("deep".to_string()));
    }

    #[test]
    fn test_node_serialization() {
        let node = create_test_node();
        let serialized = serde_json::to_string(&node);
        assert!(serialized.is_ok());
    }

    #[test]
    fn test_node_with_management_ip() {
        let mut node = create_test_node();
        node.management_ip = Some("192.168.1.1".parse().unwrap());

        assert!(node.management_ip.is_some());
        assert_eq!(node.management_ip.unwrap().to_string(), "192.168.1.1");
    }

    #[test]
    fn test_node_with_all_optional_fields() {
        let mut node = create_test_node();
        node.model = "ISR4451".to_string();
        node.platform = Some("IOS-XE".to_string());
        node.version = Some("16.12.03".to_string());
        node.serial_number = Some("ABC123456".to_string());
        node.asset_tag = Some("ASSET-001".to_string());
        node.purchase_date = Some("2023-01-15".to_string());
        node.warranty_expires = Some("2026-01-15".to_string());
        node.custom_data = json!({"test": "value"});

        let validation_result = node.validate();
        assert!(validation_result.is_ok());

        assert_eq!(node.platform.unwrap(), "IOS-XE");
        assert_eq!(node.version.unwrap(), "16.12.03");
        assert_eq!(node.serial_number.unwrap(), "ABC123456");
        assert_eq!(node.asset_tag.unwrap(), "ASSET-001");
        assert_eq!(node.purchase_date.unwrap(), "2023-01-15");
        assert_eq!(node.warranty_expires.unwrap(), "2026-01-15");
    }
}
