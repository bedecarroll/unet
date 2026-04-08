use super::*;
use crate::entities::{interface_status, links, locations, node_status, nodes};
use crate::models::derived::{InterfaceAdminStatus, InterfaceOperStatus};
use uuid::Uuid;

fn test_node_entity(domain: Option<&str>, fqdn: Option<&str>) -> nodes::Model {
    nodes::Model {
        id: Uuid::new_v4().to_string(),
        name: "edge-router".to_string(),
        fqdn: fqdn.map(ToString::to_string),
        domain: domain.map(ToString::to_string),
        vendor: "cisco".to_string(),
        model: "ASR1000".to_string(),
        role: "router".to_string(),
        lifecycle: "live".to_string(),
        serial_number: None,
        asset_tag: None,
        location_id: None,
        management_ip: None,
        description: None,
        custom_data: None,
        created_at: "2026-04-07T01:02:03Z".to_string(),
        updated_at: "2026-04-07T01:02:03Z".to_string(),
    }
}

fn test_node_status_entity() -> node_status::Model {
    node_status::Model {
        id: "status-1".to_string(),
        node_id: Uuid::new_v4().to_string(),
        last_updated: "2026-04-07T01:02:03Z".to_string(),
        reachable: true,
        system_info: None,
        performance: None,
        environmental: None,
        vendor_metrics: None,
        raw_snmp_data: None,
        last_snmp_success: None,
        last_error: None,
        consecutive_failures: 0,
    }
}

fn test_interface_status_entity(oper_status: &str) -> interface_status::Model {
    interface_status::Model {
        id: "iface-1".to_string(),
        node_status_id: "status-1".to_string(),
        index: 1,
        name: "GigabitEthernet0/1".to_string(),
        interface_type: 6,
        mtu: Some(1500),
        speed: Some(1_000_000_000),
        physical_address: Some("00:11:22:33:44:55".to_string()),
        admin_status: "testing".to_string(),
        oper_status: oper_status.to_string(),
        last_change: Some(42),
        input_stats: r#"{"octets":1000,"packets":10,"errors":0,"discards":0}"#.to_string(),
        output_stats: r#"{"octets":2000,"packets":20,"errors":0,"discards":0}"#.to_string(),
    }
}

#[test]
fn test_entity_to_node_builds_fqdn_from_domain_when_missing() {
    let node = entity_to_node(test_node_entity(Some("example.com"), None)).unwrap();

    assert_eq!(node.fqdn, "edge-router.example.com");
}

#[test]
fn test_entity_to_node_uses_name_when_domain_and_fqdn_are_missing() {
    let node = entity_to_node(test_node_entity(None, None)).unwrap();

    assert_eq!(node.fqdn, "edge-router");
    assert_eq!(node.domain, "");
}

#[test]
fn test_entity_to_link_tolerates_invalid_optional_values() {
    let node_a_id = Uuid::new_v4();
    let node_b_id = Uuid::new_v4();
    let entity = links::Model {
        id: Uuid::new_v4().to_string(),
        name: "wan-link".to_string(),
        node_a_id: node_a_id.to_string(),
        interface_a: "Gi0/0".to_string(),
        node_b_id: Some(node_b_id.to_string()),
        interface_b: Some("Gi0/1".to_string()),
        capacity: Some(-1),
        utilization: None,
        is_internet_circuit: 1,
        circuit_id: None,
        provider: None,
        description: Some("WAN".to_string()),
        custom_data: Some("{invalid-json".to_string()),
        created_at: "2026-04-07T01:02:03Z".to_string(),
        updated_at: "2026-04-07T01:02:03Z".to_string(),
    };

    let link = entity_to_link(entity).unwrap();

    assert_eq!(link.source_node_id, node_a_id);
    assert_eq!(link.dest_node_id, Some(node_b_id));
    assert_eq!(link.bandwidth, Some(0));
    assert_eq!(link.custom_data, serde_json::Value::Null);
    assert!(link.is_internet_circuit);
}

#[test]
fn test_entity_to_location_rejects_invalid_parent_uuid() {
    let entity = locations::Model {
        id: Uuid::new_v4().to_string(),
        name: "Rack 7".to_string(),
        location_type: "rack".to_string(),
        path: "/dc/rack-7".to_string(),
        parent_id: Some("not-a-uuid".to_string()),
        description: None,
        address: None,
        coordinates: None,
        custom_data: None,
        created_at: "2026-04-07T01:02:03Z".to_string(),
        updated_at: "2026-04-07T01:02:03Z".to_string(),
    };

    let error = entity_to_location(entity).unwrap_err();
    assert!(error.to_string().contains("Invalid parent UUID"));
}

#[test]
fn test_parse_optional_json_reports_invalid_json() {
    let error = parse_optional_json::<serde_json::Value>(Some("{oops".to_string()), "field_name")
        .unwrap_err();

    assert!(error.to_string().contains("Invalid JSON in field_name"));
}

#[test]
fn test_entity_to_node_status_defaults_missing_maps() {
    let status = entity_to_node_status(test_node_status_entity(), Vec::new()).unwrap();

    assert!(status.vendor_metrics.is_empty());
    assert!(status.raw_snmp_data.is_empty());
    assert!(status.last_snmp_success.is_none());
}

#[test]
fn test_entity_to_node_status_rejects_negative_consecutive_failures() {
    let mut entity = test_node_status_entity();
    entity.consecutive_failures = -1;

    let error = entity_to_node_status(entity, Vec::new()).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("Invalid consecutive failure count")
    );
}

#[test]
fn test_entity_to_interface_status_supports_testing_and_lower_layer_down() {
    let interface =
        entity_to_interface_status(test_interface_status_entity("lowerLayerDown")).unwrap();

    assert_eq!(interface.admin_status, InterfaceAdminStatus::Testing);
    assert_eq!(interface.oper_status, InterfaceOperStatus::LowerLayerDown);
}

#[test]
fn test_entity_to_interface_status_rejects_invalid_oper_status() {
    let error = entity_to_interface_status(test_interface_status_entity("broken")).unwrap_err();

    assert!(error.to_string().contains("Invalid interface oper status"));
}
