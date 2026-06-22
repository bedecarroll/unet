use std::collections::{BTreeMap, BTreeSet};

use unet_core::models::Node;
use unet_core::models::derived::{InterfaceStatus, PerformanceMetrics, SystemInfo};

use super::model::{ComparisonSection, SectionBuilder};

pub(super) fn compare_basic_nodes(
    node_a: &Node,
    node_b: &Node,
    diff_only: bool,
) -> ComparisonSection {
    let mut builder = SectionBuilder::new(diff_only);

    builder.record("node", "name", &node_a.name, &node_b.name);
    builder.record("node", "domain", &node_a.domain, &node_b.domain);
    builder.record("node", "fqdn", &node_a.fqdn, &node_b.fqdn);
    builder.record("node", "vendor", node_a.vendor, node_b.vendor);
    builder.record("node", "model", &node_a.model, &node_b.model);
    builder.record("node", "role", node_a.role, node_b.role);
    builder.record("node", "lifecycle", node_a.lifecycle, node_b.lifecycle);
    builder.record(
        "node",
        "management_ip",
        node_a.management_ip,
        node_b.management_ip,
    );
    builder.record(
        "node",
        "location_id",
        node_a.location_id,
        node_b.location_id,
    );
    builder.record("node", "platform", &node_a.platform, &node_b.platform);
    builder.record("node", "version", &node_a.version, &node_b.version);
    builder.record(
        "node",
        "serial_number",
        &node_a.serial_number,
        &node_b.serial_number,
    );
    builder.record("node", "asset_tag", &node_a.asset_tag, &node_b.asset_tag);
    builder.record(
        "node",
        "purchase_date",
        &node_a.purchase_date,
        &node_b.purchase_date,
    );
    builder.record(
        "node",
        "warranty_expires",
        &node_a.warranty_expires,
        &node_b.warranty_expires,
    );
    builder.record(
        "node",
        "custom_data",
        &node_a.custom_data,
        &node_b.custom_data,
    );

    builder.finish()
}

pub(super) fn compare_interfaces(
    interface_statuses_a: &[InterfaceStatus],
    interface_statuses_b: &[InterfaceStatus],
    diff_only: bool,
) -> ComparisonSection {
    let mut builder = SectionBuilder::new(diff_only);
    let interfaces_a = interface_statuses_a
        .iter()
        .map(|interface| (interface.name.as_str(), interface))
        .collect::<BTreeMap<_, _>>();
    let interfaces_b = interface_statuses_b
        .iter()
        .map(|interface| (interface.name.as_str(), interface))
        .collect::<BTreeMap<_, _>>();
    let names = interfaces_a
        .keys()
        .chain(interfaces_b.keys())
        .copied()
        .collect::<BTreeSet<_>>();

    for name in names {
        match (interfaces_a.get(name), interfaces_b.get(name)) {
            (Some(interface_on_a), Some(interface_on_b)) => {
                builder.record(name, "present", true, true);
                builder.record(
                    name,
                    "interface_type",
                    interface_on_a.interface_type,
                    interface_on_b.interface_type,
                );
                builder.record(name, "mtu", interface_on_a.mtu, interface_on_b.mtu);
                builder.record(name, "speed", interface_on_a.speed, interface_on_b.speed);
                builder.record(
                    name,
                    "admin_status",
                    interface_on_a.admin_status,
                    interface_on_b.admin_status,
                );
                builder.record(
                    name,
                    "oper_status",
                    interface_on_a.oper_status,
                    interface_on_b.oper_status,
                );
                builder.record(
                    name,
                    "input_octets",
                    interface_on_a.input_stats.octets,
                    interface_on_b.input_stats.octets,
                );
                builder.record(
                    name,
                    "input_packets",
                    interface_on_a.input_stats.packets,
                    interface_on_b.input_stats.packets,
                );
                builder.record(
                    name,
                    "input_errors",
                    interface_on_a.input_stats.errors,
                    interface_on_b.input_stats.errors,
                );
                builder.record(
                    name,
                    "output_octets",
                    interface_on_a.output_stats.octets,
                    interface_on_b.output_stats.octets,
                );
                builder.record(
                    name,
                    "output_packets",
                    interface_on_a.output_stats.packets,
                    interface_on_b.output_stats.packets,
                );
                builder.record(
                    name,
                    "output_errors",
                    interface_on_a.output_stats.errors,
                    interface_on_b.output_stats.errors,
                );
            }
            (Some(_), None) => builder.record(name, "present", true, false),
            (None, Some(_)) => builder.record(name, "present", false, true),
            (None, None) => {}
        }
    }

    builder.finish()
}

pub(super) fn compare_metrics(
    metrics_a: Option<&PerformanceMetrics>,
    metrics_b: Option<&PerformanceMetrics>,
    diff_only: bool,
) -> ComparisonSection {
    let mut builder = SectionBuilder::new(diff_only);

    builder.record(
        "metrics",
        "cpu_utilization",
        metrics_a
            .as_ref()
            .and_then(|metrics| metrics.cpu_utilization),
        metrics_b
            .as_ref()
            .and_then(|metrics| metrics.cpu_utilization),
    );
    builder.record(
        "metrics",
        "memory_utilization",
        metrics_a
            .as_ref()
            .and_then(|metrics| metrics.memory_utilization),
        metrics_b
            .as_ref()
            .and_then(|metrics| metrics.memory_utilization),
    );
    builder.record(
        "metrics",
        "total_memory",
        metrics_a.as_ref().and_then(|metrics| metrics.total_memory),
        metrics_b.as_ref().and_then(|metrics| metrics.total_memory),
    );
    builder.record(
        "metrics",
        "used_memory",
        metrics_a.as_ref().and_then(|metrics| metrics.used_memory),
        metrics_b.as_ref().and_then(|metrics| metrics.used_memory),
    );
    builder.record(
        "metrics",
        "load_average",
        metrics_a.as_ref().and_then(|metrics| metrics.load_average),
        metrics_b.as_ref().and_then(|metrics| metrics.load_average),
    );

    builder.finish()
}

pub(super) fn compare_system(
    system_a: Option<&SystemInfo>,
    system_b: Option<&SystemInfo>,
    diff_only: bool,
) -> ComparisonSection {
    let mut builder = SectionBuilder::new(diff_only);

    builder.record(
        "system",
        "description",
        system_a
            .as_ref()
            .and_then(|system| system.description.clone()),
        system_b
            .as_ref()
            .and_then(|system| system.description.clone()),
    );
    builder.record(
        "system",
        "object_id",
        system_a
            .as_ref()
            .and_then(|system| system.object_id.clone()),
        system_b
            .as_ref()
            .and_then(|system| system.object_id.clone()),
    );
    builder.record(
        "system",
        "uptime_ticks",
        system_a.as_ref().and_then(|system| system.uptime_ticks),
        system_b.as_ref().and_then(|system| system.uptime_ticks),
    );
    builder.record(
        "system",
        "contact",
        system_a.as_ref().and_then(|system| system.contact.clone()),
        system_b.as_ref().and_then(|system| system.contact.clone()),
    );
    builder.record(
        "system",
        "name",
        system_a.as_ref().and_then(|system| system.name.clone()),
        system_b.as_ref().and_then(|system| system.name.clone()),
    );
    builder.record(
        "system",
        "location",
        system_a.as_ref().and_then(|system| system.location.clone()),
        system_b.as_ref().and_then(|system| system.location.clone()),
    );
    builder.record(
        "system",
        "services",
        system_a.as_ref().and_then(|system| system.services),
        system_b.as_ref().and_then(|system| system.services),
    );

    builder.finish()
}
