/// Node history operations.
use anyhow::{Result, anyhow};
use chrono::{Duration, SecondsFormat, Utc};
use serde_json::{Value, json};
use std::time::SystemTime;
use unet_core::datastore::{DataStore, HistoryQueryOptions};
use unet_core::models::{Node, derived::NodeStatus};

use super::types::{HistoryNodeArgs, HistoryType};

pub async fn history_node(
    args: HistoryNodeArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    let node = datastore.get_node_required(&args.id).await?;
    let output = build_history_output(&node, args, datastore).await?;
    crate::commands::print_output(&output, output_format)?;
    Ok(())
}

pub(super) async fn build_history_output(
    node: &Node,
    args: HistoryNodeArgs,
    datastore: &dyn DataStore,
) -> Result<Value> {
    let history = datastore
        .get_node_status_history(
            &args.id,
            &HistoryQueryOptions {
                limit: args.limit,
                since: history_since(args.last_hours)?,
            },
        )
        .await?;

    let mut output = json!({
        "node_id": args.id,
        "node_name": node.name.clone(),
        "history_type": format!("{:?}", args.history_type),
        "limit": args.limit,
        "last_hours": args.last_hours,
        "detailed": args.detailed,
    });

    match args.history_type {
        HistoryType::Status => {
            output["status_history"] = Value::Array(
                history
                    .iter()
                    .map(|snapshot| status_entry(snapshot, args.detailed))
                    .collect::<Vec<_>>(),
            );
            output["entry_count"] = json!(history.len());
        }
        HistoryType::Interfaces => {
            output["interface_history"] = Value::Array(
                history
                    .iter()
                    .map(|snapshot| interface_entry(snapshot, args.detailed))
                    .collect::<Vec<_>>(),
            );
            output["entry_count"] = json!(history.len());
        }
        HistoryType::Metrics => {
            let metrics_history = history.iter().filter_map(metrics_entry).collect::<Vec<_>>();
            output["metrics_history"] = Value::Array(metrics_history.clone());
            output["entry_count"] = json!(metrics_history.len());
        }
        HistoryType::System => {
            let system_history = history.iter().filter_map(system_entry).collect::<Vec<_>>();
            output["system_history"] = Value::Array(system_history.clone());
            output["entry_count"] = json!(system_history.len());
        }
        HistoryType::All => {
            output["complete_history"] = Value::Array(
                history
                    .iter()
                    .map(|snapshot| complete_entry(snapshot, args.detailed))
                    .collect::<Vec<_>>(),
            );
            output["entry_count"] = json!(history.len());
        }
    }

    Ok(output)
}

fn history_since(last_hours: Option<u64>) -> Result<Option<SystemTime>> {
    last_hours
        .map(|hours| {
            i64::try_from(hours)
                .map_err(|_| anyhow!("--last-hours value is too large: {hours}"))
                .map(|hours| (Utc::now() - Duration::hours(hours)).into())
        })
        .transpose()
}

fn status_entry(snapshot: &NodeStatus, detailed: bool) -> Value {
    let mut entry = json!({
        "last_updated": format_timestamp(snapshot.last_updated),
        "reachable": snapshot.reachable,
        "last_snmp_success": snapshot.last_snmp_success.map(format_timestamp),
        "last_error": snapshot.last_error.clone(),
        "consecutive_failures": snapshot.consecutive_failures,
    });

    if detailed {
        entry["system_info"] = json!(&snapshot.system_info);
        entry["performance"] = json!(&snapshot.performance);
        entry["interface_count"] = json!(snapshot.interfaces.len());
    }

    entry
}

fn interface_entry(snapshot: &NodeStatus, detailed: bool) -> Value {
    let interfaces = if detailed {
        json!(&snapshot.interfaces)
    } else {
        json!(
            snapshot
                .interfaces
                .iter()
                .map(|interface| {
                    json!({
                        "index": interface.index,
                        "name": interface.name.clone(),
                        "admin_status": interface.admin_status,
                        "oper_status": interface.oper_status,
                    })
                })
                .collect::<Vec<_>>()
        )
    };

    json!({
        "last_updated": format_timestamp(snapshot.last_updated),
        "interfaces": interfaces,
    })
}

fn metrics_entry(snapshot: &NodeStatus) -> Option<Value> {
    snapshot.performance.as_ref().map(|performance| {
        json!({
            "last_updated": format_timestamp(snapshot.last_updated),
            "metrics": performance,
        })
    })
}

fn system_entry(snapshot: &NodeStatus) -> Option<Value> {
    snapshot.system_info.as_ref().map(|system_info| {
        json!({
            "last_updated": format_timestamp(snapshot.last_updated),
            "system_info": system_info,
        })
    })
}

fn complete_entry(snapshot: &NodeStatus, detailed: bool) -> Value {
    let mut entry = json!({
        "last_updated": format_timestamp(snapshot.last_updated),
        "reachable": snapshot.reachable,
        "system_info": &snapshot.system_info,
        "interfaces": if detailed {
            json!(&snapshot.interfaces)
        } else {
            json!(snapshot.interfaces.iter().map(|interface| {
                json!({
                    "index": interface.index,
                    "name": interface.name.clone(),
                    "admin_status": interface.admin_status,
                    "oper_status": interface.oper_status,
                })
            }).collect::<Vec<_>>())
        },
        "performance": &snapshot.performance,
        "environmental": &snapshot.environmental,
        "last_snmp_success": snapshot.last_snmp_success.map(format_timestamp),
        "last_error": snapshot.last_error.clone(),
        "consecutive_failures": snapshot.consecutive_failures,
    });

    if detailed {
        entry["vendor_metrics"] = json!(&snapshot.vendor_metrics);
        entry["raw_snmp_data"] = json!(&snapshot.raw_snmp_data);
    }

    entry
}

fn format_timestamp(value: SystemTime) -> String {
    chrono::DateTime::<Utc>::from(value).to_rfc3339_opts(SecondsFormat::Secs, true)
}
