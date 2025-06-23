pub mod export;
pub mod import;
pub mod links;
pub mod locations;
pub mod nodes;
pub mod policy;

use anyhow::Result;
use serde_json;
use serde_yaml;

/// Common output formatting utilities
pub fn format_output<T: serde::Serialize>(data: &T, format: crate::OutputFormat) -> Result<String> {
    match format {
        crate::OutputFormat::Json => Ok(serde_json::to_string_pretty(data)?),
        crate::OutputFormat::Yaml => Ok(serde_yaml::to_string(data)?),
        crate::OutputFormat::Table => {
            // For table format, we'll implement per-type formatting in each command module
            Ok(serde_json::to_string_pretty(data)?)
        }
    }
}

/// Print data in the specified format
pub fn print_output<T: serde::Serialize>(data: &T, format: crate::OutputFormat) -> Result<()> {
    let output = format_output(data, format)?;
    println!("{}", output);
    Ok(())
}
