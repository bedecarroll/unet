/// Node comparison functionality.
use anyhow::Result;
use serde_json::Value;
use unet_core::datastore::DataStore;

use super::types::CompareNodeArgs;

mod builder;
mod model;
mod sections;

pub(in crate::commands::nodes) use builder::build_compare_output;

pub async fn compare_nodes(
    args: CompareNodeArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    let output = build_compare_output(args, datastore).await?;
    crate::commands::print_output(&output, output_format)?;
    Ok(())
}

pub(super) fn serialize_section<T>(section: T) -> Result<Value>
where
    T: serde::Serialize,
{
    Ok(serde_json::to_value(section)?)
}
