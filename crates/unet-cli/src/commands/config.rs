use anyhow::{Context, Result};
use clap::{Args, Subcommand, ValueEnum};
use config_slicer::{ConfigSlicerApi, DiffDisplay, DiffEngine, DisplayOptions};
use std::fs;
use std::path::PathBuf;
use unet_core::datastore::DataStore;

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Extract configuration slices using patterns
    Slice(SliceConfigArgs),
    /// Compare two configuration files
    Diff(DiffConfigArgs),
    /// Validate configuration syntax
    Validate(ValidateConfigArgs),
    /// Get information about supported parsers and extractors
    Info(InfoConfigArgs),
}

#[derive(Args)]
pub struct SliceConfigArgs {
    /// Configuration file to process
    #[arg(short, long)]
    file: PathBuf,

    /// Pattern to match (glob, regex, or hierarchical path)
    #[arg(short, long)]
    pattern: String,

    /// Pattern type
    #[arg(short = 't', long, value_enum, default_value = "glob")]
    pattern_type: PatternType,

    /// Vendor hint for parser selection
    #[arg(long, value_enum)]
    vendor: Option<Vendor>,

    /// Output file (default: stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Case sensitive pattern matching
    #[arg(long)]
    case_sensitive: bool,

    /// Include line numbers in output
    #[arg(long)]
    line_numbers: bool,
}

#[derive(Args)]
pub struct DiffConfigArgs {
    /// First configuration file
    old_file: PathBuf,

    /// Second configuration file  
    new_file: PathBuf,

    /// Diff algorithm to use
    #[arg(short, long, value_enum, default_value = "text")]
    algorithm: DiffAlgorithm,

    /// Output file (default: stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Show colored output
    #[arg(short, long)]
    color: bool,

    /// Context lines around changes
    #[arg(short = 'U', long, default_value = "3")]
    context: usize,

    /// Use side-by-side format
    #[arg(short, long)]
    side_by_side: bool,

    /// Generate HTML output
    #[arg(long)]
    html: bool,

    /// Show statistics
    #[arg(long)]
    stats: bool,
}

#[derive(Args)]
pub struct ValidateConfigArgs {
    /// Configuration file to validate
    #[arg(short, long)]
    file: PathBuf,

    /// Vendor hint for parser selection
    #[arg(long, value_enum)]
    vendor: Option<Vendor>,

    /// Output validation report to file
    #[arg(short, long)]
    output: Option<PathBuf>,
}

#[derive(Args)]
pub struct InfoConfigArgs {
    /// Show detailed information
    #[arg(long)]
    detailed: bool,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum PatternType {
    /// Glob pattern (e.g., "interface*")
    Glob,
    /// Regular expression
    Regex,
    /// Hierarchical path (e.g., "interface/ip")
    Path,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Vendor {
    /// Cisco IOS/IOS-XE
    Cisco,
    /// Juniper JunOS
    Juniper,
    /// Arista EOS
    Arista,
    /// Generic configuration
    Generic,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum DiffAlgorithm {
    /// Text-based diff
    Text,
    /// Hierarchical diff
    Hierarchical,
    /// Semantic diff
    Semantic,
}

pub async fn execute(
    cmd: ConfigCommands,
    _datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    match cmd {
        ConfigCommands::Slice(args) => execute_slice(args, output_format).await,
        ConfigCommands::Diff(args) => execute_diff(args, output_format).await,
        ConfigCommands::Validate(args) => execute_validate(args, output_format).await,
        ConfigCommands::Info(args) => execute_info(args).await,
    }
}

async fn execute_slice(args: SliceConfigArgs, output_format: crate::OutputFormat) -> Result<()> {
    tracing::info!("Slicing configuration file: {}", args.file.display());
    tracing::debug!(
        "Pattern: '{}', Type: {:?}, Vendor: {:?}",
        args.pattern,
        args.pattern_type,
        args.vendor
    );

    // Initialize the API
    let api = ConfigSlicerApi::new();

    // Parse the configuration file
    let config_tree = api
        .parse_config_file(&args.file, args.vendor.map(convert_vendor))
        .context("Failed to parse configuration file")?;

    tracing::info!(
        "Configuration parsed successfully, found {} top-level sections",
        config_tree.children.len()
    );

    // Extract slices based on pattern type
    let slice_result = match args.pattern_type {
        PatternType::Glob => {
            tracing::debug!("Using glob pattern matching");
            api.slice_by_glob(&config_tree, &args.pattern)
        }
        PatternType::Regex => {
            tracing::debug!("Using regex pattern matching");
            api.slice_by_regex(&config_tree, &args.pattern)
        }
        PatternType::Path => {
            tracing::debug!("Using hierarchical path matching");
            api.slice_by_path(&config_tree, &args.pattern)
        }
    }
    .context("Failed to extract configuration slices")?;

    tracing::info!("Found {} matching slices", slice_result.matches.len());

    // Format and output the results
    let formatted_output = format_slice_output(&slice_result, output_format, args.line_numbers)?;

    // Write to file or stdout
    match args.output {
        Some(output_path) => {
            fs::write(&output_path, formatted_output)
                .with_context(|| format!("Failed to write output to {}", output_path.display()))?;
            tracing::info!("Output written to {}", output_path.display());
        }
        None => {
            print!("{}", formatted_output);
        }
    }

    Ok(())
}

async fn execute_diff(args: DiffConfigArgs, _output_format: crate::OutputFormat) -> Result<()> {
    tracing::info!(
        "Diffing configurations: {} vs {}",
        args.old_file.display(),
        args.new_file.display()
    );
    tracing::debug!(
        "Color: {}, Side-by-side: {}, HTML: {}, Stats: {}",
        args.color,
        args.side_by_side,
        args.html,
        args.stats
    );

    // Read the configuration files
    let old_config = fs::read_to_string(&args.old_file)
        .with_context(|| format!("Failed to read {}", args.old_file.display()))?;
    let new_config = fs::read_to_string(&args.new_file)
        .with_context(|| format!("Failed to read {}", args.new_file.display()))?;

    // Initialize diff engine
    let diff_engine = DiffEngine::new().context("Failed to create diff engine")?;

    // Perform the diff
    let diff_result = diff_engine
        .diff(&old_config, &new_config)
        .context("Failed to compute diff")?;

    tracing::info!(
        "Diff computed: {} changes found",
        diff_result.text_diff.changes.len()
    );

    // Format the diff output
    let display_options = DisplayOptions {
        use_colors: args.color,
        show_line_numbers: true,
        show_context: true,
        terminal_width: 120,
        max_lines: 0,
        compact_unchanged: true,
    };

    let diff_display = DiffDisplay::new();
    let formatted_output = if args.html {
        tracing::debug!("Generating HTML diff output");
        diff_display.format_html(&diff_result, &display_options)
    } else if args.side_by_side {
        tracing::debug!("Generating side-by-side diff output");
        diff_display.format_side_by_side(&diff_result, &display_options)
    } else if args.color {
        tracing::debug!("Generating colored terminal diff output");
        diff_display.format_colored(&diff_result, &display_options)
    } else {
        tracing::debug!("Generating unified diff output");
        diff_display.format_unified(&diff_result, &display_options)
    };

    // Add statistics if requested
    let final_output = if args.stats {
        let stats_output = format_diff_stats(&diff_result);
        format!("{}\n\n{}", formatted_output, stats_output)
    } else {
        formatted_output
    };

    // Write to file or stdout
    match args.output {
        Some(output_path) => {
            fs::write(&output_path, final_output)
                .with_context(|| format!("Failed to write output to {}", output_path.display()))?;
            tracing::info!("Diff output written to {}", output_path.display());
        }
        None => {
            print!("{}", final_output);
        }
    }

    Ok(())
}

async fn execute_validate(
    args: ValidateConfigArgs,
    output_format: crate::OutputFormat,
) -> Result<()> {
    tracing::info!("Validating configuration file: {}", args.file.display());

    // Initialize the API
    let api = ConfigSlicerApi::new();

    // Read and validate the configuration
    let config_text = fs::read_to_string(&args.file)
        .with_context(|| format!("Failed to read {}", args.file.display()))?;

    let validation_report = api
        .validate_config(&config_text, args.vendor.map(convert_vendor))
        .context("Failed to validate configuration")?;

    tracing::info!("Validation completed: {}", validation_report.summary());

    // Format the validation output
    let formatted_output = format_validation_output(&validation_report, output_format)?;

    // Write to file or stdout
    match args.output {
        Some(output_path) => {
            fs::write(&output_path, formatted_output)
                .with_context(|| format!("Failed to write output to {}", output_path.display()))?;
            tracing::info!("Validation report written to {}", output_path.display());
        }
        None => {
            print!("{}", formatted_output);
        }
    }

    // Exit with error code if validation failed
    if !validation_report.is_valid {
        tracing::warn!("Configuration validation failed");
        std::process::exit(2);
    }

    Ok(())
}

async fn execute_info(args: InfoConfigArgs) -> Result<()> {
    let api = ConfigSlicerApi::new();

    println!("μNet Configuration Slicing and Diffing Tool");
    println!("==========================================");
    println!();

    println!("Available Parsers:");
    for parser in api.available_parsers() {
        if args.detailed {
            println!("  {:?} - {}", parser, get_vendor_description(&parser));
        } else {
            println!("  {:?}", parser);
        }
    }
    println!();

    println!("Available Extractors:");
    for extractor in api.available_extractors() {
        println!("  {}", extractor);
    }
    println!();

    if args.detailed {
        println!("Supported Pattern Types:");
        println!("  glob     - Shell-style glob patterns (e.g., 'interface*', 'vlan?0?')");
        println!("  regex    - Regular expressions (e.g., '^interface\\s+GigabitEthernet')");
        println!("  path     - Hierarchical paths (e.g., 'interface/ip', 'router/bgp/neighbor')");
        println!();

        println!("Supported Diff Algorithms:");
        println!("  text         - Character/line-based text diffing");
        println!("  hierarchical - Configuration structure-aware diffing");
        println!("  semantic     - Network semantics-aware diffing");
        println!();

        println!("Output Formats:");
        println!("  table - Formatted table output (default)");
        println!("  json  - JSON structured output");
        println!("  yaml  - YAML structured output");
    }

    Ok(())
}

/// Convert CLI vendor enum to library vendor enum
fn convert_vendor(vendor: Vendor) -> config_slicer::parser::Vendor {
    match vendor {
        Vendor::Cisco => config_slicer::parser::Vendor::Cisco,
        Vendor::Juniper => config_slicer::parser::Vendor::Juniper,
        Vendor::Arista => config_slicer::parser::Vendor::Arista,
        Vendor::Generic => config_slicer::parser::Vendor::Generic,
    }
}

/// Get vendor description for detailed info output
fn get_vendor_description(vendor: &config_slicer::parser::Vendor) -> &'static str {
    match vendor {
        config_slicer::parser::Vendor::Cisco => "Cisco IOS/IOS-XE configurations",
        config_slicer::parser::Vendor::Juniper => "Juniper JunOS configurations",
        config_slicer::parser::Vendor::Arista => "Arista EOS configurations",
        config_slicer::parser::Vendor::Generic => "Generic line-based configurations",
    }
}

/// Format slice output based on output format
fn format_slice_output(
    slice_result: &config_slicer::slicer::SliceResult,
    output_format: crate::OutputFormat,
    line_numbers: bool,
) -> Result<String> {
    match output_format {
        crate::OutputFormat::Table => {
            let mut output = String::new();

            if slice_result.matches.is_empty() {
                output.push_str("No matching slices found.\n");
            } else {
                output.push_str(&format!(
                    "Found {} matching configuration sections:\n\n",
                    slice_result.matches.len()
                ));

                for (i, matched_node) in slice_result.matches.iter().enumerate() {
                    if i > 0 {
                        output.push_str("\n---\n\n");
                    }

                    output.push_str(&format!("Match {}: {}\n", i + 1, matched_node.command));

                    if line_numbers {
                        output.push_str(&format!(
                            "{:4}: {}\n",
                            matched_node.line_number, matched_node.raw_line
                        ));
                    } else {
                        output.push_str(&format!("{}\n", matched_node.raw_line));
                    }

                    // Include children if any
                    for child in &matched_node.children {
                        if line_numbers {
                            output.push_str(&format!(
                                "{:4}: {}\n",
                                child.line_number, child.raw_line
                            ));
                        } else {
                            output.push_str(&format!("{}\n", child.raw_line));
                        }
                    }
                }
            }

            Ok(output)
        }
        crate::OutputFormat::Json => {
            use serde_json;
            // For JSON output, we'll create a simplified structure
            let simplified = serde_json::json!({
                "pattern": format!("{:?}", slice_result.pattern),
                "match_count": slice_result.matches.len(),
                "matches": slice_result.matches.iter().map(|node| {
                    serde_json::json!({
                        "command": node.command,
                        "raw_line": node.raw_line,
                        "line_number": node.line_number,
                        "indent_level": node.indent_level,
                        "children_count": node.children.len()
                    })
                }).collect::<Vec<_>>(),
                "metadata": slice_result.metadata
            });

            let json_output = serde_json::to_string_pretty(&simplified)
                .context("Failed to serialize slice result to JSON")?;
            Ok(json_output)
        }
        crate::OutputFormat::Yaml => {
            use serde_json;
            use serde_yaml;
            // For YAML output, use the same simplified structure
            let simplified = serde_json::json!({
                "pattern": format!("{:?}", slice_result.pattern),
                "match_count": slice_result.matches.len(),
                "matches": slice_result.matches.iter().map(|node| {
                    serde_json::json!({
                        "command": node.command,
                        "raw_line": node.raw_line,
                        "line_number": node.line_number,
                        "indent_level": node.indent_level,
                        "children_count": node.children.len()
                    })
                }).collect::<Vec<_>>(),
                "metadata": slice_result.metadata
            });

            let yaml_output = serde_yaml::to_string(&simplified)
                .context("Failed to serialize slice result to YAML")?;
            Ok(yaml_output)
        }
    }
}

/// Format validation output based on output format
fn format_validation_output(
    validation_report: &config_slicer::ValidationReport,
    output_format: crate::OutputFormat,
) -> Result<String> {
    use serde_json;
    use serde_yaml;

    match output_format {
        crate::OutputFormat::Table => {
            let mut output = String::new();

            output.push_str(&format!(
                "Validation Summary: {}\n\n",
                validation_report.summary()
            ));

            if !validation_report.errors.is_empty() {
                output.push_str("Errors:\n");
                for error in &validation_report.errors {
                    if let Some(line_num) = error.line_number {
                        output.push_str(&format!("  Line {}: {}\n", line_num, error.message));
                    } else {
                        output.push_str(&format!("  {}\n", error.message));
                    }
                }
                output.push('\n');
            }

            if !validation_report.warnings.is_empty() {
                output.push_str("Warnings:\n");
                for warning in &validation_report.warnings {
                    if let Some(line_num) = warning.line_number {
                        output.push_str(&format!("  Line {}: {}\n", line_num, warning.message));
                    } else {
                        output.push_str(&format!("  {}\n", warning.message));
                    }
                }
                output.push('\n');
            }

            Ok(output)
        }
        crate::OutputFormat::Json => {
            // Create a simplified structure for JSON output
            let simplified = serde_json::json!({
                "is_valid": validation_report.is_valid,
                "summary": validation_report.summary(),
                "error_count": validation_report.errors.len(),
                "warning_count": validation_report.warnings.len(),
                "errors": validation_report.errors.iter().map(|err| {
                    serde_json::json!({
                        "message": err.message,
                        "line_number": err.line_number,
                        "severity": format!("{:?}", err.severity)
                    })
                }).collect::<Vec<_>>(),
                "warnings": validation_report.warnings.iter().map(|warn| {
                    serde_json::json!({
                        "message": warn.message,
                        "line_number": warn.line_number,
                        "warning_type": format!("{:?}", warn.warning_type)
                    })
                }).collect::<Vec<_>>()
            });

            let json_output = serde_json::to_string_pretty(&simplified)
                .context("Failed to serialize validation report to JSON")?;
            Ok(json_output)
        }
        crate::OutputFormat::Yaml => {
            // Create a simplified structure for YAML output
            let simplified = serde_json::json!({
                "is_valid": validation_report.is_valid,
                "summary": validation_report.summary(),
                "error_count": validation_report.errors.len(),
                "warning_count": validation_report.warnings.len(),
                "errors": validation_report.errors.iter().map(|err| {
                    serde_json::json!({
                        "message": err.message,
                        "line_number": err.line_number,
                        "severity": format!("{:?}", err.severity)
                    })
                }).collect::<Vec<_>>(),
                "warnings": validation_report.warnings.iter().map(|warn| {
                    serde_json::json!({
                        "message": warn.message,
                        "line_number": warn.line_number,
                        "warning_type": format!("{:?}", warn.warning_type)
                    })
                }).collect::<Vec<_>>()
            });

            let yaml_output = serde_yaml::to_string(&simplified)
                .context("Failed to serialize validation report to YAML")?;
            Ok(yaml_output)
        }
    }
}

/// Format diff statistics
fn format_diff_stats(diff_result: &config_slicer::DiffResult) -> String {
    let mut stats = String::new();

    stats.push_str("Diff Statistics:\n");
    stats.push_str(&format!(
        "  Total changes: {}\n",
        diff_result.text_diff.changes.len()
    ));

    let additions = diff_result
        .text_diff
        .changes
        .iter()
        .filter(|c| matches!(c.change_type, config_slicer::DiffType::Addition))
        .count();
    let deletions = diff_result
        .text_diff
        .changes
        .iter()
        .filter(|c| matches!(c.change_type, config_slicer::DiffType::Deletion))
        .count();
    let modifications = diff_result
        .text_diff
        .changes
        .iter()
        .filter(|c| matches!(c.change_type, config_slicer::DiffType::Modification))
        .count();

    stats.push_str(&format!("  Additions: {}\n", additions));
    stats.push_str(&format!("  Deletions: {}\n", deletions));
    stats.push_str(&format!("  Modifications: {}\n", modifications));

    stats
}
