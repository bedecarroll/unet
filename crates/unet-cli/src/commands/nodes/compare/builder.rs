use anyhow::{Result, bail};
use serde_json::Value;
use unet_core::datastore::DataStore;

use super::super::types::{CompareNodeArgs, CompareType};
use super::sections::{compare_basic_nodes, compare_interfaces, compare_metrics, compare_system};
use super::serialize_section;

pub(in crate::commands::nodes) async fn build_compare_output(
    args: CompareNodeArgs,
    datastore: &dyn DataStore,
) -> Result<Value> {
    let Some(node_b_id) = args.node_b else {
        bail!("Historical comparison is not supported yet; compare requires --node-b");
    };

    let node_a = datastore.get_node_required(&args.node_a).await?;
    let node_b = datastore.get_node_required(&node_b_id).await?;

    let include_basic = args.compare_type.contains(&CompareType::All);
    let include_interfaces = include_basic || args.compare_type.contains(&CompareType::Interfaces);
    let include_metrics = include_basic || args.compare_type.contains(&CompareType::Metrics);
    let include_system = include_basic || args.compare_type.contains(&CompareType::System);

    let mut output = serde_json::json!({
        "node_a": {
            "id": args.node_a,
            "name": node_a.name.clone()
        },
        "node_b": {
            "id": node_b_id,
            "name": node_b.name.clone()
        },
        "compare_types": args.compare_type,
        "diff_only": args.diff_only
    });

    if include_basic {
        output["basic_comparison"] =
            serialize_section(compare_basic_nodes(&node_a, &node_b, args.diff_only))?;
    }

    if include_interfaces {
        let interfaces_a = datastore.get_node_interfaces(&args.node_a).await?;
        let interfaces_b = datastore.get_node_interfaces(&node_b_id).await?;
        output["interfaces_comparison"] = serialize_section(compare_interfaces(
            &interfaces_a,
            &interfaces_b,
            args.diff_only,
        ))?;
    }

    if include_metrics {
        let metrics_a = datastore.get_node_metrics(&args.node_a).await?;
        let metrics_b = datastore.get_node_metrics(&node_b_id).await?;
        output["metrics_comparison"] = serialize_section(compare_metrics(
            metrics_a.as_ref(),
            metrics_b.as_ref(),
            args.diff_only,
        ))?;
    }

    if include_system {
        let system_a = datastore
            .get_node_status(&args.node_a)
            .await?
            .and_then(|status| status.system_info);
        let system_b = datastore
            .get_node_status(&node_b_id)
            .await?
            .and_then(|status| status.system_info);
        output["system_comparison"] = serialize_section(compare_system(
            system_a.as_ref(),
            system_b.as_ref(),
            args.diff_only,
        ))?;
    }

    Ok(output)
}
