/// Node polling operations
use anyhow::{Result, anyhow};
use std::net::SocketAddr;
use std::time::SystemTime;
use unet_core::config::defaults;
use unet_core::datastore::DataStore;
use unet_core::models::Node;
use unet_core::snmp::{PollingConfig, PollingTask, SessionConfig, StandardOid};

use super::types::{PollingAction, PollingNodeArgs};

pub(super) fn polling_status_value(
    task: Option<&PollingTask>,
    detailed: bool,
) -> serde_json::Value {
    task.map_or_else(
        || {
            serde_json::json!({
                "state": "not_configured",
                "enabled": false,
                "message": "No polling task configured for this node"
            })
        },
        |task| {
            let mut output = serde_json::json!({
                "state": if task.enabled { "enabled" } else { "disabled" },
                "enabled": task.enabled,
                "task_id": task.id.to_string(),
                "target": task.target.to_string(),
                "interval_seconds": task.interval.as_secs(),
                "oid_count": task.oids.len(),
                "priority": task.priority,
                "created_at": format_timestamp(task.created_at),
                "last_success": task.last_success.map(format_timestamp),
                "last_error": task.last_error.clone(),
                "consecutive_failures": task.consecutive_failures
            });

            if detailed {
                output["oids"] = serde_json::json!(task.oids.clone());
                output["session_config"] =
                    serde_json::to_value(&task.session_config).unwrap_or(serde_json::Value::Null);
            }

            output
        },
    )
}

pub async fn polling_node(
    args: PollingNodeArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    // Verify node exists first
    let node = datastore.get_node_required(&args.id).await?;

    let mut output = serde_json::json!({
        "node_id": args.id,
        "node_name": node.name.clone(),
        "action": format!("{:?}", args.action),
        "detailed": args.detailed
    });

    match args.action {
        PollingAction::Status => {
            let task = datastore.get_node_polling_task(&args.id).await?;
            output["polling"] = polling_status_value(task.as_ref(), args.detailed);
        }
        PollingAction::Start => {
            let task = enable_polling_task(datastore, &node, false).await?;
            output["result"] = serde_json::json!({
                "message": "Polling task enabled"
            });
            output["polling"] = polling_status_value(Some(&task), args.detailed);
        }
        PollingAction::Stop => {
            let task = disable_polling_task(datastore, &args.id).await?;
            output["result"] = serde_json::json!({
                "message": if task.is_some() {
                    "Polling task disabled"
                } else {
                    "No polling task configured for this node"
                }
            });
            output["polling"] = polling_status_value(task.as_ref(), args.detailed);
        }
        PollingAction::Restart => {
            let task = enable_polling_task(datastore, &node, true).await?;
            output["result"] = serde_json::json!({
                "message": "Polling task restarted"
            });
            output["polling"] = polling_status_value(Some(&task), args.detailed);
        }
    }

    crate::commands::print_output(&output, output_format)?;

    Ok(())
}

async fn enable_polling_task(
    datastore: &dyn DataStore,
    node: &Node,
    reset_runtime_state: bool,
) -> Result<PollingTask> {
    let task = match datastore.get_node_polling_task(&node.id).await? {
        Some(mut task) => {
            refresh_task_target(&mut task, node);
            task.enabled = true;
            if reset_runtime_state {
                task.last_error = None;
                task.consecutive_failures = 0;
            }
            task
        }
        None => build_polling_task(node)?,
    };

    datastore
        .upsert_polling_task(&task)
        .await
        .map_err(Into::into)
}

async fn disable_polling_task(
    datastore: &dyn DataStore,
    node_id: &uuid::Uuid,
) -> Result<Option<PollingTask>> {
    let Some(mut task) = datastore.get_node_polling_task(node_id).await? else {
        return Ok(None);
    };

    task.enabled = false;
    datastore
        .upsert_polling_task(&task)
        .await
        .map(Some)
        .map_err(Into::into)
}

fn build_polling_task(node: &Node) -> Result<PollingTask> {
    let management_ip = node.management_ip.ok_or_else(|| {
        anyhow!(
            "Node '{}' has no management IP configured; cannot create a polling task",
            node.name
        )
    })?;
    let target = SocketAddr::new(management_ip, defaults::network::SNMP_DEFAULT_PORT);
    let session_config = SessionConfig {
        address: target,
        ..SessionConfig::default()
    };

    Ok(PollingTask::new(
        target,
        node.id,
        StandardOid::system_oids()
            .into_iter()
            .map(|oid| oid.oid().to_string())
            .collect(),
        PollingConfig::default().default_interval,
        session_config,
    ))
}

const fn refresh_task_target(task: &mut PollingTask, node: &Node) {
    if let Some(management_ip) = node.management_ip {
        task.target = SocketAddr::new(management_ip, defaults::network::SNMP_DEFAULT_PORT);
        task.session_config.address = task.target;
    }
}

fn format_timestamp(value: SystemTime) -> String {
    chrono::DateTime::<chrono::Utc>::from(value).to_rfc3339()
}
