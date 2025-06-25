//! Configuration Slicer CLI Tool
//!
//! Command-line tool for slicing and diffing network device configurations.

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use config_slicer::{
    ApprovalPriority, ConfigSlicerApi, DiffDisplay, DiffEngine, DiffOptions,
    DiffWorkflowOrchestrator, DisplayOptions,
};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use tracing::{debug, error, info, warn};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "config-slicer")]
#[command(about = "Network configuration slicing and diffing tool")]
#[command(version)]
#[command(long_about = r#"
config-slicer is a command-line tool for parsing, slicing, and diffing network device configurations.

It supports multiple vendor formats including Cisco IOS/IOS-XE, Juniper JunOS, 
Arista EOS, and generic configurations. Use it to extract specific configuration 
sections, compare configurations, and analyze configuration changes.

Examples:
  # Parse and slice interfaces from a Cisco config
  config-slicer slice -f config.txt -p "interface*" -v cisco

  # Diff two configurations with colored output
  config-slicer diff old-config.txt new-config.txt --color

  # Extract BGP configuration to a file
  config-slicer slice -f config.txt -p "router bgp*" -o bgp-config.txt

  # Generate shell completions
  config-slicer completions bash > config-slicer-completions.bash
"#)]
struct Cli {
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Enable debug logging (implies --verbose)
    #[arg(short, long, global = true)]
    debug: bool,

    /// Output format
    #[arg(short = 'F', long, value_enum, default_value = "text", global = true)]
    output_format: OutputFormat,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse and slice configuration sections
    Slice {
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
    },

    /// Compare two configurations
    Diff {
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
    },

    /// Validate configuration syntax
    Validate {
        /// Configuration file to validate
        #[arg(short, long)]
        file: PathBuf,

        /// Vendor hint for parser selection
        #[arg(long, value_enum)]
        vendor: Option<Vendor>,

        /// Output validation report to file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Batch processing for multiple files
    Batch {
        #[command(subcommand)]
        batch_command: BatchCommands,
    },

    /// List available parsers and extractors
    Info {
        /// Show detailed information
        #[arg(long)]
        detailed: bool,
    },

    /// Workflow management for diff operations
    Workflow {
        #[command(subcommand)]
        workflow_command: WorkflowCommands,
    },

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: clap_complete::Shell,
    },
}

#[derive(Subcommand)]
enum BatchCommands {
    /// Process multiple configuration files for slicing
    Slice {
        /// Input directory or glob pattern for config files
        #[arg(short, long)]
        input: String,

        /// Pattern to match (glob, regex, or hierarchical path)
        #[arg(short, long)]
        pattern: String,

        /// Pattern type
        #[arg(short = 't', long, value_enum, default_value = "glob")]
        pattern_type: PatternType,

        /// Vendor hint for parser selection
        #[arg(long, value_enum)]
        vendor: Option<Vendor>,

        /// Output directory for results
        #[arg(short, long)]
        output_dir: PathBuf,

        /// Number of parallel workers (default: CPU cores)
        #[arg(short = 'j', long)]
        jobs: Option<usize>,

        /// Show progress bar
        #[arg(long)]
        progress: bool,

        /// Continue processing on errors
        #[arg(long)]
        continue_on_error: bool,
    },

    /// Batch diff operations for multiple file pairs
    Diff {
        /// Old configurations directory or glob pattern
        #[arg(long)]
        old_dir: String,

        /// New configurations directory or glob pattern
        #[arg(long)]
        new_dir: String,

        /// Output directory for diff results
        #[arg(short, long)]
        output_dir: PathBuf,

        /// Diff algorithm to use
        #[arg(short, long, value_enum, default_value = "text")]
        algorithm: DiffAlgorithm,

        /// Generate HTML output
        #[arg(long)]
        html: bool,

        /// Show statistics in output
        #[arg(long)]
        stats: bool,

        /// Number of parallel workers (default: CPU cores)
        #[arg(short = 'j', long)]
        jobs: Option<usize>,

        /// Show progress bar
        #[arg(long)]
        progress: bool,

        /// Continue processing on errors
        #[arg(long)]
        continue_on_error: bool,
    },

    /// Batch validate multiple configuration files
    Validate {
        /// Input directory or glob pattern for config files
        #[arg(short, long)]
        input: String,

        /// Vendor hint for parser selection
        #[arg(long, value_enum)]
        vendor: Option<Vendor>,

        /// Output directory for validation reports
        #[arg(short, long)]
        output_dir: PathBuf,

        /// Number of parallel workers (default: CPU cores)
        #[arg(short = 'j', long)]
        jobs: Option<usize>,

        /// Show progress bar
        #[arg(long)]
        progress: bool,

        /// Continue processing on errors
        #[arg(long)]
        continue_on_error: bool,

        /// Generate summary report
        #[arg(long)]
        summary: bool,
    },
}

#[derive(Subcommand)]
enum WorkflowCommands {
    /// Execute a diff workflow with caching and approval
    Execute {
        /// Source identifier (file path or node ID)
        #[arg(short, long)]
        source: String,

        /// Target identifier (file path or node ID)
        #[arg(short, long)]
        target: String,

        /// Old configuration file
        #[arg(long)]
        old_config: PathBuf,

        /// New configuration file
        #[arg(long)]
        new_config: PathBuf,

        /// Require approval for this workflow
        #[arg(long)]
        require_approval: bool,

        /// Approval requester username
        #[arg(long)]
        requester: Option<String>,

        /// Approvers (comma-separated list)
        #[arg(long)]
        approvers: Option<String>,

        /// Approval priority
        #[arg(long, value_enum, default_value = "normal")]
        priority: ApprovalPriorityArg,
    },

    /// List active workflows
    List {
        /// Filter by status
        #[arg(long, value_enum)]
        status: Option<WorkflowStatusArg>,

        /// Show detailed information
        #[arg(long)]
        detailed: bool,
    },

    /// Show workflow details
    Show {
        /// Workflow execution ID
        workflow_id: String,

        /// Show history
        #[arg(long)]
        history: bool,
    },

    /// Approve a workflow
    Approve {
        /// Workflow execution ID
        workflow_id: String,

        /// Reviewer username
        #[arg(long)]
        reviewer: String,

        /// Approval reason
        #[arg(long)]
        reason: Option<String>,
    },

    /// Reject a workflow
    Reject {
        /// Workflow execution ID
        workflow_id: String,

        /// Reviewer username
        #[arg(long)]
        reviewer: String,

        /// Rejection reason
        #[arg(long)]
        reason: Option<String>,
    },

    /// Archive a completed workflow
    Archive {
        /// Workflow execution ID
        workflow_id: String,
    },

    /// Show workflow history
    History {
        /// Workflow execution ID (optional, shows all if not provided)
        workflow_id: Option<String>,

        /// Limit number of entries
        #[arg(long, default_value = "50")]
        limit: usize,
    },

    /// Show cache statistics
    Cache {
        /// Clean up expired entries
        #[arg(long)]
        cleanup: bool,
    },
}

#[derive(Clone, ValueEnum)]
enum WorkflowStatusArg {
    Computing,
    Completed,
    Failed,
    PendingApproval,
    Approved,
    Rejected,
    Archived,
}

#[derive(Clone, ValueEnum)]
enum ApprovalPriorityArg {
    Low,
    Normal,
    High,
    Emergency,
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    /// Plain text output
    Text,
    /// JSON output
    Json,
    /// YAML output  
    Yaml,
}

#[derive(Debug, Clone, ValueEnum)]
enum PatternType {
    /// Glob pattern (e.g., "interface*")
    Glob,
    /// Regular expression
    Regex,
    /// Hierarchical path (e.g., "interface/ip")
    Path,
}

#[derive(Debug, Clone, ValueEnum)]
enum Vendor {
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
enum DiffAlgorithm {
    /// Text-based diff
    Text,
    /// Hierarchical diff
    Hierarchical,
    /// Semantic diff
    Semantic,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    initialize_logging(cli.verbose || cli.debug, cli.debug)?;

    debug!(
        "Starting config-slicer CLI with args: {:?}",
        std::env::args().collect::<Vec<_>>()
    );

    let result = match cli.command {
        Commands::Slice {
            file,
            pattern,
            pattern_type,
            vendor,
            output,
            case_sensitive,
            line_numbers,
        } => handle_slice_command(
            file,
            pattern,
            pattern_type,
            vendor,
            output,
            case_sensitive,
            line_numbers,
            cli.output_format,
        ),
        Commands::Diff {
            old_file,
            new_file,
            algorithm,
            output,
            color,
            context,
            side_by_side,
            html,
            stats,
        } => handle_diff_command(
            old_file,
            new_file,
            algorithm,
            output,
            color,
            context,
            side_by_side,
            html,
            stats,
            cli.output_format,
        ),
        Commands::Validate {
            file,
            vendor,
            output,
        } => handle_validate_command(file, vendor, output, cli.output_format),
        Commands::Batch { batch_command } => handle_batch_command(batch_command, cli.output_format),
        Commands::Info { detailed } => handle_info_command(detailed),
        Commands::Workflow { workflow_command } => {
            handle_workflow_command(workflow_command, cli.output_format)
        }
        Commands::Completions { shell } => handle_completions_command(shell),
    };

    if let Err(e) = result {
        error!("Command failed: {}", e);
        std::process::exit(1);
    }

    Ok(())
}

/// Initialize logging based on verbosity level
fn initialize_logging(verbose: bool, debug: bool) -> Result<()> {
    let filter = if debug {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"))
    } else if verbose {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"))
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_level(true)
        .init();

    Ok(())
}

/// Handle the slice command
fn handle_slice_command(
    file: PathBuf,
    pattern: String,
    pattern_type: PatternType,
    vendor: Option<Vendor>,
    output: Option<PathBuf>,
    _case_sensitive: bool, // Currently unused - would need pattern builder support
    line_numbers: bool,
    output_format: OutputFormat,
) -> Result<()> {
    info!("Slicing configuration file: {}", file.display());
    debug!(
        "Pattern: '{}', Type: {:?}, Vendor: {:?}",
        pattern, pattern_type, vendor
    );

    // Initialize the API
    let api = ConfigSlicerApi::new();

    // Parse the configuration file
    let config_tree = api
        .parse_config_file(&file, vendor.map(convert_vendor))
        .context("Failed to parse configuration file")?;

    info!(
        "Configuration parsed successfully, found {} top-level sections",
        config_tree.children.len()
    );

    // Extract slices based on pattern type
    let slice_result = match pattern_type {
        PatternType::Glob => {
            debug!("Using glob pattern matching");
            api.slice_by_glob(&config_tree, &pattern)
        }
        PatternType::Regex => {
            debug!("Using regex pattern matching");
            api.slice_by_regex(&config_tree, &pattern)
        }
        PatternType::Path => {
            debug!("Using hierarchical path matching");
            api.slice_by_path(&config_tree, &pattern)
        }
    }
    .context("Failed to extract configuration slices")?;

    info!("Found {} matching slices", slice_result.matches.len());

    // Format and output the results
    let formatted_output = format_slice_output(&slice_result, output_format, line_numbers)?;

    // Write to file or stdout
    match output {
        Some(output_path) => {
            fs::write(&output_path, formatted_output)
                .with_context(|| format!("Failed to write output to {}", output_path.display()))?;
            info!("Output written to {}", output_path.display());
        }
        None => {
            print!("{}", formatted_output);
        }
    }

    Ok(())
}

/// Handle the diff command
fn handle_diff_command(
    old_file: PathBuf,
    new_file: PathBuf,
    _algorithm: DiffAlgorithm, // Currently unused - engine provides comprehensive diff
    output: Option<PathBuf>,
    color: bool,
    _context: usize, // Context is handled by display options
    side_by_side: bool,
    html: bool,
    stats: bool,
    _output_format: OutputFormat, // Currently unused for diff output
) -> Result<()> {
    info!(
        "Diffing configurations: {} vs {}",
        old_file.display(),
        new_file.display()
    );
    debug!(
        "Color: {}, Side-by-side: {}, HTML: {}, Stats: {}",
        color, side_by_side, html, stats
    );

    // Read the configuration files
    let old_config = fs::read_to_string(&old_file)
        .with_context(|| format!("Failed to read {}", old_file.display()))?;
    let new_config = fs::read_to_string(&new_file)
        .with_context(|| format!("Failed to read {}", new_file.display()))?;

    // Initialize diff engine
    let diff_engine = DiffEngine::new().context("Failed to create diff engine")?;

    // Perform the diff - the engine provides a comprehensive diff
    let diff_result = diff_engine
        .diff(&old_config, &new_config)
        .context("Failed to compute diff")?;

    info!(
        "Diff computed: {} changes found",
        diff_result.text_diff.changes.len()
    );

    // Format the diff output
    let display_options = DisplayOptions {
        use_colors: color,
        show_line_numbers: true,
        show_context: true,
        terminal_width: 120,
        max_lines: 0,
        compact_unchanged: true,
    };

    let diff_display = DiffDisplay::new();
    let formatted_output = if html {
        debug!("Generating HTML diff output");
        diff_display.format_html(&diff_result, &display_options)
    } else if side_by_side {
        debug!("Generating side-by-side diff output");
        diff_display.format_side_by_side(&diff_result, &display_options)
    } else if color {
        debug!("Generating colored terminal diff output");
        diff_display.format_colored(&diff_result, &display_options)
    } else {
        debug!("Generating unified diff output");
        diff_display.format_unified(&diff_result, &display_options)
    };

    // Add statistics if requested
    let final_output = if stats {
        let stats_output = format_diff_stats(&diff_result);
        format!("{}\n\n{}", formatted_output, stats_output)
    } else {
        formatted_output
    };

    // Write to file or stdout
    match output {
        Some(output_path) => {
            fs::write(&output_path, final_output)
                .with_context(|| format!("Failed to write output to {}", output_path.display()))?;
            info!("Diff output written to {}", output_path.display());
        }
        None => {
            print!("{}", final_output);
        }
    }

    Ok(())
}

/// Handle the validate command
fn handle_validate_command(
    file: PathBuf,
    vendor: Option<Vendor>,
    output: Option<PathBuf>,
    output_format: OutputFormat,
) -> Result<()> {
    info!("Validating configuration file: {}", file.display());

    // Initialize the API
    let api = ConfigSlicerApi::new();

    // Read and validate the configuration
    let config_text =
        fs::read_to_string(&file).with_context(|| format!("Failed to read {}", file.display()))?;

    let validation_report = api
        .validate_config(&config_text, vendor.map(convert_vendor))
        .context("Failed to validate configuration")?;

    info!("Validation completed: {}", validation_report.summary());

    // Format the validation output
    let formatted_output = format_validation_output(&validation_report, output_format.clone())?;

    // Write to file or stdout
    match output {
        Some(output_path) => {
            fs::write(&output_path, formatted_output)
                .with_context(|| format!("Failed to write output to {}", output_path.display()))?;
            info!("Validation report written to {}", output_path.display());
        }
        None => {
            print!("{}", formatted_output);
        }
    }

    // Set exit code based on validation result
    if !validation_report.is_valid {
        warn!("Configuration validation failed");
        std::process::exit(2);
    }

    Ok(())
}

/// Handle the info command
fn handle_info_command(detailed: bool) -> Result<()> {
    let api = ConfigSlicerApi::new();

    println!("config-slicer - Network Configuration Slicing and Diffing Tool");
    println!("================================================================");
    println!();

    println!("Available Parsers:");
    for parser in api.available_parsers() {
        if detailed {
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

    if detailed {
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
        println!("  text - Plain text output (default)");
        println!("  json - JSON structured output");
        println!("  yaml - YAML structured output");
    }

    Ok(())
}

/// Handle the completions command
fn handle_completions_command(shell: clap_complete::Shell) -> Result<()> {
    use clap_complete::generate;
    use std::io;

    let mut app = Cli::command();
    generate(shell, &mut app, "config-slicer", &mut io::stdout());

    Ok(())
}

/// Handle batch processing commands
fn handle_batch_command(batch_command: BatchCommands, output_format: OutputFormat) -> Result<()> {
    match batch_command {
        BatchCommands::Slice {
            input,
            pattern,
            pattern_type,
            vendor,
            output_dir,
            jobs,
            progress,
            continue_on_error,
        } => handle_batch_slice(
            input,
            pattern,
            pattern_type,
            vendor,
            output_dir,
            jobs,
            progress,
            continue_on_error,
            output_format,
        ),
        BatchCommands::Diff {
            old_dir,
            new_dir,
            output_dir,
            algorithm,
            html,
            stats,
            jobs,
            progress,
            continue_on_error,
        } => handle_batch_diff(
            old_dir,
            new_dir,
            output_dir,
            algorithm,
            html,
            stats,
            jobs,
            progress,
            continue_on_error,
        ),
        BatchCommands::Validate {
            input,
            vendor,
            output_dir,
            jobs,
            progress,
            continue_on_error,
            summary,
        } => handle_batch_validate(
            input,
            vendor,
            output_dir,
            jobs,
            progress,
            continue_on_error,
            summary,
            output_format,
        ),
    }
}

/// Handle batch slice operations
fn handle_batch_slice(
    input: String,
    pattern: String,
    pattern_type: PatternType,
    vendor: Option<Vendor>,
    output_dir: PathBuf,
    jobs: Option<usize>,
    progress: bool,
    continue_on_error: bool,
    output_format: OutputFormat,
) -> Result<()> {
    info!("Starting batch slice operation");
    info!(
        "Input: {}, Pattern: {}, Output: {}",
        input,
        pattern,
        output_dir.display()
    );

    // Create output directory if it doesn't exist
    fs::create_dir_all(&output_dir).with_context(|| {
        format!(
            "Failed to create output directory: {}",
            output_dir.display()
        )
    })?;

    // Find input files using glob or directory walking
    let input_files = find_input_files(&input)?;
    info!("Found {} files to process", input_files.len());

    if input_files.is_empty() {
        warn!("No input files found matching pattern: {}", input);
        return Ok(());
    }

    // Set up parallelism
    if let Some(num_jobs) = jobs {
        rayon::ThreadPoolBuilder::new()
            .num_threads(num_jobs)
            .build_global()
            .context("Failed to configure thread pool")?;
    }

    // Set up progress bar
    let progress_bar = if progress {
        let pb = ProgressBar::new(input_files.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
                .unwrap()
                .progress_chars("##-"),
        );
        pb.set_message("Processing files");
        Some(pb)
    } else {
        None
    };

    let start_time = Instant::now();
    let error_count = Arc::new(AtomicUsize::new(0));
    let success_count = Arc::new(AtomicUsize::new(0));

    // Process files in parallel
    let _results: Vec<Result<()>> = input_files
        .par_iter()
        .map(|input_file| {
            let result = process_slice_file(
                input_file,
                &pattern,
                pattern_type.clone(),
                vendor.clone(),
                &output_dir,
                output_format.clone(),
            );

            match &result {
                Ok(_) => {
                    success_count.fetch_add(1, Ordering::SeqCst);
                    debug!("Successfully processed: {}", input_file.display());
                }
                Err(e) => {
                    error_count.fetch_add(1, Ordering::SeqCst);
                    if continue_on_error {
                        warn!("Error processing {}: {}", input_file.display(), e);
                    } else {
                        error!("Error processing {}: {}", input_file.display(), e);
                    }
                }
            }

            if let Some(ref pb) = progress_bar {
                pb.inc(1);
            }

            result
        })
        .collect();

    if let Some(ref pb) = progress_bar {
        pb.finish_with_message("Batch slice complete");
    }

    let elapsed = start_time.elapsed();
    let success = success_count.load(Ordering::SeqCst);
    let errors = error_count.load(Ordering::SeqCst);

    info!(
        "Batch slice completed in {:.2}s: {} successful, {} errors",
        elapsed.as_secs_f64(),
        success,
        errors
    );

    // Handle errors
    if !continue_on_error && errors > 0 {
        return Err(anyhow::anyhow!(
            "Batch operation failed with {} errors",
            errors
        ));
    }

    Ok(())
}

/// Handle batch diff operations
fn handle_batch_diff(
    old_dir: String,
    new_dir: String,
    output_dir: PathBuf,
    _algorithm: DiffAlgorithm,
    html: bool,
    stats: bool,
    jobs: Option<usize>,
    progress: bool,
    continue_on_error: bool,
) -> Result<()> {
    info!("Starting batch diff operation");
    info!(
        "Old: {}, New: {}, Output: {}",
        old_dir,
        new_dir,
        output_dir.display()
    );

    // Create output directory if it doesn't exist
    fs::create_dir_all(&output_dir).with_context(|| {
        format!(
            "Failed to create output directory: {}",
            output_dir.display()
        )
    })?;

    // Find matching file pairs
    let old_files = find_input_files(&old_dir)?;
    let new_files = find_input_files(&new_dir)?;

    let file_pairs = match_file_pairs(&old_files, &new_files)?;
    info!("Found {} file pairs to diff", file_pairs.len());

    if file_pairs.is_empty() {
        warn!("No matching file pairs found for diffing");
        return Ok(());
    }

    // Set up parallelism
    if let Some(num_jobs) = jobs {
        rayon::ThreadPoolBuilder::new()
            .num_threads(num_jobs)
            .build_global()
            .context("Failed to configure thread pool")?;
    }

    // Set up progress bar
    let progress_bar = if progress {
        let pb = ProgressBar::new(file_pairs.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
                .unwrap()
                .progress_chars("##-"),
        );
        pb.set_message("Diffing files");
        Some(pb)
    } else {
        None
    };

    let start_time = Instant::now();
    let error_count = Arc::new(AtomicUsize::new(0));
    let success_count = Arc::new(AtomicUsize::new(0));

    // Process file pairs in parallel
    let _results: Vec<Result<()>> = file_pairs
        .par_iter()
        .map(|(old_file, new_file)| {
            let result = process_diff_pair(old_file, new_file, &output_dir, html, stats);

            match &result {
                Ok(_) => {
                    success_count.fetch_add(1, Ordering::SeqCst);
                    debug!(
                        "Successfully diffed: {} vs {}",
                        old_file.display(),
                        new_file.display()
                    );
                }
                Err(e) => {
                    error_count.fetch_add(1, Ordering::SeqCst);
                    if continue_on_error {
                        warn!(
                            "Error diffing {} vs {}: {}",
                            old_file.display(),
                            new_file.display(),
                            e
                        );
                    } else {
                        error!(
                            "Error diffing {} vs {}: {}",
                            old_file.display(),
                            new_file.display(),
                            e
                        );
                    }
                }
            }

            if let Some(ref pb) = progress_bar {
                pb.inc(1);
            }

            result
        })
        .collect();

    if let Some(ref pb) = progress_bar {
        pb.finish_with_message("Batch diff complete");
    }

    let elapsed = start_time.elapsed();
    let success = success_count.load(Ordering::SeqCst);
    let errors = error_count.load(Ordering::SeqCst);

    info!(
        "Batch diff completed in {:.2}s: {} successful, {} errors",
        elapsed.as_secs_f64(),
        success,
        errors
    );

    // Handle errors
    if !continue_on_error && errors > 0 {
        return Err(anyhow::anyhow!(
            "Batch operation failed with {} errors",
            errors
        ));
    }

    Ok(())
}

/// Handle batch validate operations
fn handle_batch_validate(
    input: String,
    vendor: Option<Vendor>,
    output_dir: PathBuf,
    jobs: Option<usize>,
    progress: bool,
    continue_on_error: bool,
    summary: bool,
    output_format: OutputFormat,
) -> Result<()> {
    info!("Starting batch validate operation");
    info!("Input: {}, Output: {}", input, output_dir.display());

    // Create output directory if it doesn't exist
    fs::create_dir_all(&output_dir).with_context(|| {
        format!(
            "Failed to create output directory: {}",
            output_dir.display()
        )
    })?;

    // Find input files
    let input_files = find_input_files(&input)?;
    info!("Found {} files to validate", input_files.len());

    if input_files.is_empty() {
        warn!("No input files found matching pattern: {}", input);
        return Ok(());
    }

    // Set up parallelism
    if let Some(num_jobs) = jobs {
        rayon::ThreadPoolBuilder::new()
            .num_threads(num_jobs)
            .build_global()
            .context("Failed to configure thread pool")?;
    }

    // Set up progress bar
    let progress_bar = if progress {
        let pb = ProgressBar::new(input_files.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
                .unwrap()
                .progress_chars("##-"),
        );
        pb.set_message("Validating files");
        Some(pb)
    } else {
        None
    };

    let start_time = Instant::now();
    let error_count = Arc::new(AtomicUsize::new(0));
    let success_count = Arc::new(AtomicUsize::new(0));
    let validation_error_count = Arc::new(AtomicUsize::new(0));

    // Process files in parallel
    let results: Vec<Result<bool>> = input_files
        .par_iter()
        .map(|input_file| {
            let result = process_validate_file(
                input_file,
                vendor.clone(),
                &output_dir,
                output_format.clone(),
            );

            match &result {
                Ok(is_valid) => {
                    success_count.fetch_add(1, Ordering::SeqCst);
                    if !is_valid {
                        validation_error_count.fetch_add(1, Ordering::SeqCst);
                    }
                    debug!(
                        "Successfully validated: {} (valid: {})",
                        input_file.display(),
                        is_valid
                    );
                }
                Err(e) => {
                    error_count.fetch_add(1, Ordering::SeqCst);
                    if continue_on_error {
                        warn!("Error validating {}: {}", input_file.display(), e);
                    } else {
                        error!("Error validating {}: {}", input_file.display(), e);
                    }
                }
            }

            if let Some(ref pb) = progress_bar {
                pb.inc(1);
            }

            result
        })
        .collect();

    if let Some(ref pb) = progress_bar {
        pb.finish_with_message("Batch validation complete");
    }

    let elapsed = start_time.elapsed();
    let success = success_count.load(Ordering::SeqCst);
    let errors = error_count.load(Ordering::SeqCst);
    let validation_errors = validation_error_count.load(Ordering::SeqCst);

    info!(
        "Batch validation completed in {:.2}s: {} processed, {} errors, {} validation failures",
        elapsed.as_secs_f64(),
        success,
        errors,
        validation_errors
    );

    // Generate summary report if requested
    if summary {
        generate_validation_summary(&output_dir, &results, output_format)?;
    }

    // Handle errors
    if !continue_on_error && errors > 0 {
        return Err(anyhow::anyhow!(
            "Batch operation failed with {} errors",
            errors
        ));
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
    output_format: OutputFormat,
    line_numbers: bool,
) -> Result<String> {
    match output_format {
        OutputFormat::Text => {
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
        OutputFormat::Json => {
            // For JSON output, we'll create a simplified structure since SliceResult might not be Serialize
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
        OutputFormat::Yaml => {
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
    output_format: OutputFormat,
) -> Result<String> {
    use serde_json;
    use serde_yaml;

    match output_format {
        OutputFormat::Text => {
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
        OutputFormat::Json => {
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
        OutputFormat::Yaml => {
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

/// Find input files based on glob pattern or directory path
fn find_input_files(input: &str) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    // Check if input looks like a glob pattern
    if input.contains('*') || input.contains('?') || input.contains('[') {
        // Use glob pattern matching
        for entry in glob::glob(input).context("Failed to parse glob pattern")? {
            match entry {
                Ok(path) => {
                    if path.is_file() {
                        files.push(path);
                    }
                }
                Err(e) => warn!("Error processing glob entry: {}", e),
            }
        }
    } else {
        // Treat as directory or single file
        let path = Path::new(input);
        if path.is_file() {
            files.push(path.to_path_buf());
        } else if path.is_dir() {
            // Walk directory recursively
            for entry in walkdir::WalkDir::new(path) {
                match entry {
                    Ok(entry) => {
                        if entry.file_type().is_file() {
                            // Filter for likely config files
                            if let Some(ext) = entry.path().extension() {
                                if matches!(
                                    ext.to_str(),
                                    Some("conf") | Some("cfg") | Some("txt") | Some("config")
                                ) {
                                    files.push(entry.path().to_path_buf());
                                }
                            } else {
                                // Include files without extensions
                                files.push(entry.path().to_path_buf());
                            }
                        }
                    }
                    Err(e) => warn!("Error walking directory: {}", e),
                }
            }
        } else {
            return Err(anyhow::anyhow!("Input path does not exist: {}", input));
        }
    }

    files.sort();
    Ok(files)
}

/// Match file pairs for diffing based on filename
fn match_file_pairs(
    old_files: &[PathBuf],
    new_files: &[PathBuf],
) -> Result<Vec<(PathBuf, PathBuf)>> {
    let mut pairs = Vec::new();

    // Create a map of filenames to paths for quick lookup
    let new_file_map: std::collections::HashMap<_, _> = new_files
        .iter()
        .filter_map(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| (name, path.clone()))
        })
        .collect();

    for old_file in old_files {
        if let Some(filename) = old_file.file_name().and_then(|name| name.to_str()) {
            if let Some(new_file) = new_file_map.get(filename) {
                pairs.push((old_file.clone(), new_file.clone()));
            } else {
                debug!("No matching new file for: {}", old_file.display());
            }
        }
    }

    Ok(pairs)
}

/// Process a single file for slicing
fn process_slice_file(
    input_file: &Path,
    pattern: &str,
    pattern_type: PatternType,
    vendor: Option<Vendor>,
    output_dir: &Path,
    output_format: OutputFormat,
) -> Result<()> {
    // Initialize the API
    let api = ConfigSlicerApi::new();

    // Parse the configuration file
    let config_tree = api
        .parse_config_file(input_file, vendor.map(convert_vendor))
        .with_context(|| {
            format!(
                "Failed to parse configuration file: {}",
                input_file.display()
            )
        })?;

    // Extract slices based on pattern type
    let slice_result = match pattern_type {
        PatternType::Glob => api.slice_by_glob(&config_tree, pattern),
        PatternType::Regex => api.slice_by_regex(&config_tree, pattern),
        PatternType::Path => api.slice_by_path(&config_tree, pattern),
    }
    .with_context(|| format!("Failed to extract slices from {}", input_file.display()))?;

    // Format and write output
    let formatted_output = format_slice_output(&slice_result, output_format.clone(), false)?;

    // Generate output filename
    let output_filename = input_file
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("slice");
    let output_extension = match output_format {
        OutputFormat::Text => "txt",
        OutputFormat::Json => "json",
        OutputFormat::Yaml => "yaml",
    };
    let output_path = output_dir.join(format!("{}_slice.{}", output_filename, output_extension));

    fs::write(&output_path, formatted_output)
        .with_context(|| format!("Failed to write slice output to {}", output_path.display()))?;

    Ok(())
}

/// Process a file pair for diffing
fn process_diff_pair(
    old_file: &Path,
    new_file: &Path,
    output_dir: &Path,
    html: bool,
    stats: bool,
) -> Result<()> {
    // Read the configuration files
    let old_config = fs::read_to_string(old_file)
        .with_context(|| format!("Failed to read {}", old_file.display()))?;
    let new_config = fs::read_to_string(new_file)
        .with_context(|| format!("Failed to read {}", new_file.display()))?;

    // Initialize diff engine
    let diff_engine = DiffEngine::new().with_context(|| {
        format!(
            "Failed to create diff engine for {} vs {}",
            old_file.display(),
            new_file.display()
        )
    })?;

    // Perform the diff
    let diff_result = diff_engine
        .diff(&old_config, &new_config)
        .with_context(|| {
            format!(
                "Failed to compute diff for {} vs {}",
                old_file.display(),
                new_file.display()
            )
        })?;

    // Format the diff output
    let display_options = DisplayOptions {
        use_colors: false, // No colors in file output
        show_line_numbers: true,
        show_context: true,
        terminal_width: 120,
        max_lines: 0,
        compact_unchanged: true,
    };

    let diff_display = DiffDisplay::new();
    let formatted_output = if html {
        diff_display.format_html(&diff_result, &display_options)
    } else {
        diff_display.format_unified(&diff_result, &display_options)
    };

    // Add statistics if requested
    let final_output = if stats {
        let stats_output = format_diff_stats(&diff_result);
        format!("{}\n\n{}", formatted_output, stats_output)
    } else {
        formatted_output
    };

    // Generate output filename
    let old_filename = old_file
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("old");
    let new_filename = new_file
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("new");
    let output_extension = if html { "html" } else { "diff" };
    let output_path = output_dir.join(format!(
        "{}_vs_{}.{}",
        old_filename, new_filename, output_extension
    ));

    fs::write(&output_path, final_output)
        .with_context(|| format!("Failed to write diff output to {}", output_path.display()))?;

    Ok(())
}

/// Process a single file for validation
fn process_validate_file(
    input_file: &Path,
    vendor: Option<Vendor>,
    output_dir: &Path,
    output_format: OutputFormat,
) -> Result<bool> {
    // Initialize the API
    let api = ConfigSlicerApi::new();

    // Read and validate the configuration
    let config_text = fs::read_to_string(input_file)
        .with_context(|| format!("Failed to read {}", input_file.display()))?;

    let validation_report = api
        .validate_config(&config_text, vendor.map(convert_vendor))
        .with_context(|| format!("Failed to validate configuration: {}", input_file.display()))?;

    // Format the validation output
    let formatted_output = format_validation_output(&validation_report, output_format.clone())?;

    // Generate output filename
    let output_filename = input_file
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("validation");
    let output_extension = match output_format {
        OutputFormat::Text => "txt",
        OutputFormat::Json => "json",
        OutputFormat::Yaml => "yaml",
    };
    let output_path = output_dir.join(format!(
        "{}_validation.{}",
        output_filename, output_extension
    ));

    fs::write(&output_path, formatted_output).with_context(|| {
        format!(
            "Failed to write validation output to {}",
            output_path.display()
        )
    })?;

    Ok(validation_report.is_valid)
}

/// Generate validation summary report
fn generate_validation_summary(
    output_dir: &Path,
    results: &[Result<bool>],
    output_format: OutputFormat,
) -> Result<()> {
    let total_files = results.len();
    let successful = results.iter().filter(|r| r.is_ok()).count();
    let errors = results.iter().filter(|r| r.is_err()).count();
    let valid_files = results
        .iter()
        .filter_map(|r| r.as_ref().ok())
        .filter(|&&is_valid| is_valid)
        .count();
    let invalid_files = results
        .iter()
        .filter_map(|r| r.as_ref().ok())
        .filter(|&&is_valid| !is_valid)
        .count();

    let summary = match output_format {
        OutputFormat::Text => {
            format!(
                "Batch Validation Summary\n\
                 ========================\n\
                 Total files processed: {}\n\
                 Successful validations: {}\n\
                 Processing errors: {}\n\
                 Valid configurations: {}\n\
                 Invalid configurations: {}\n\
                 \n\
                 Success rate: {:.1}%\n\
                 Validity rate: {:.1}%\n",
                total_files,
                successful,
                errors,
                valid_files,
                invalid_files,
                if total_files > 0 {
                    (successful as f64 / total_files as f64) * 100.0
                } else {
                    0.0
                },
                if successful > 0 {
                    (valid_files as f64 / successful as f64) * 100.0
                } else {
                    0.0
                }
            )
        }
        OutputFormat::Json => {
            let summary_data = serde_json::json!({
                "total_files": total_files,
                "successful_validations": successful,
                "processing_errors": errors,
                "valid_configurations": valid_files,
                "invalid_configurations": invalid_files,
                "success_rate_percent": if total_files > 0 { (successful as f64 / total_files as f64) * 100.0 } else { 0.0 },
                "validity_rate_percent": if successful > 0 { (valid_files as f64 / successful as f64) * 100.0 } else { 0.0 }
            });
            serde_json::to_string_pretty(&summary_data)
                .context("Failed to serialize validation summary to JSON")?
        }
        OutputFormat::Yaml => {
            let summary_data = serde_json::json!({
                "total_files": total_files,
                "successful_validations": successful,
                "processing_errors": errors,
                "valid_configurations": valid_files,
                "invalid_configurations": invalid_files,
                "success_rate_percent": if total_files > 0 { (successful as f64 / total_files as f64) * 100.0 } else { 0.0 },
                "validity_rate_percent": if successful > 0 { (valid_files as f64 / successful as f64) * 100.0 } else { 0.0 }
            });
            serde_yaml::to_string(&summary_data)
                .context("Failed to serialize validation summary to YAML")?
        }
    };

    let output_extension = match output_format {
        OutputFormat::Text => "txt",
        OutputFormat::Json => "json",
        OutputFormat::Yaml => "yaml",
    };
    let summary_path = output_dir.join(format!("validation_summary.{}", output_extension));

    fs::write(&summary_path, summary).with_context(|| {
        format!(
            "Failed to write validation summary to {}",
            summary_path.display()
        )
    })?;

    info!("Validation summary written to {}", summary_path.display());
    Ok(())
}

/// Handle workflow management commands
fn handle_workflow_command(command: WorkflowCommands, output_format: OutputFormat) -> Result<()> {
    let rt = tokio::runtime::Runtime::new().context("Failed to create async runtime")?;

    match command {
        WorkflowCommands::Execute {
            source,
            target,
            old_config,
            new_config,
            require_approval,
            requester,
            approvers,
            priority,
        } => rt.block_on(async {
            handle_workflow_execute(
                source,
                target,
                old_config,
                new_config,
                require_approval,
                requester,
                approvers,
                priority,
                output_format,
            )
            .await
        }),
        WorkflowCommands::List { status, detailed } => {
            handle_workflow_list(status, detailed, output_format)
        }
        WorkflowCommands::Show {
            workflow_id,
            history,
        } => handle_workflow_show(workflow_id, history, output_format),
        WorkflowCommands::Approve {
            workflow_id,
            reviewer,
            reason,
        } => handle_workflow_approve(workflow_id, reviewer, reason, output_format),
        WorkflowCommands::Reject {
            workflow_id,
            reviewer,
            reason,
        } => handle_workflow_reject(workflow_id, reviewer, reason, output_format),
        WorkflowCommands::Archive { workflow_id } => {
            handle_workflow_archive(workflow_id, output_format)
        }
        WorkflowCommands::History { workflow_id, limit } => {
            handle_workflow_history(workflow_id, limit, output_format)
        }
        WorkflowCommands::Cache { cleanup } => handle_workflow_cache(cleanup, output_format),
    }
}

/// Execute a diff workflow
async fn handle_workflow_execute(
    source: String,
    target: String,
    old_config: PathBuf,
    new_config: PathBuf,
    require_approval: bool,
    requester: Option<String>,
    approvers: Option<String>,
    priority: ApprovalPriorityArg,
    output_format: OutputFormat,
) -> Result<()> {
    let old_content = fs::read_to_string(&old_config)
        .with_context(|| format!("Failed to read old config file: {}", old_config.display()))?;
    let new_content = fs::read_to_string(&new_config)
        .with_context(|| format!("Failed to read new config file: {}", new_config.display()))?;

    let diff_engine = DiffEngine::new().context("Failed to create diff engine")?;
    let orchestrator = DiffWorkflowOrchestrator::new(diff_engine);

    let workflow_id = orchestrator
        .execute_workflow(
            &source,
            &target,
            &old_content,
            &new_content,
            DiffOptions::default(),
            require_approval,
        )
        .await
        .context("Failed to execute workflow")?;

    // Request approval if needed
    if require_approval {
        if let (Some(requester), Some(approvers_str)) = (requester, approvers) {
            let approvers_list: Vec<String> = approvers_str
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();

            let priority_enum = match priority {
                ApprovalPriorityArg::Low => ApprovalPriority::Low,
                ApprovalPriorityArg::Normal => ApprovalPriority::Normal,
                ApprovalPriorityArg::High => ApprovalPriority::High,
                ApprovalPriorityArg::Emergency => ApprovalPriority::Emergency,
            };

            orchestrator
                .request_approval(workflow_id, &requester, approvers_list, priority_enum)
                .context("Failed to request approval")?;
        }
    }

    let output = match output_format {
        OutputFormat::Text => format!("Workflow executed successfully: {}", workflow_id),
        OutputFormat::Json => serde_json::to_string_pretty(&serde_json::json!({
            "workflow_id": workflow_id.to_string(),
            "status": "executed",
            "source": source,
            "target": target,
            "approval_required": require_approval
        }))?,
        OutputFormat::Yaml => serde_yaml::to_string(&serde_json::json!({
            "workflow_id": workflow_id.to_string(),
            "status": "executed",
            "source": source,
            "target": target,
            "approval_required": require_approval
        }))?,
    };

    println!("{}", output);
    Ok(())
}

/// List active workflows
fn handle_workflow_list(
    _status_filter: Option<WorkflowStatusArg>,
    _detailed: bool,
    output_format: OutputFormat,
) -> Result<()> {
    // For now, return a placeholder since we'd need persistent storage
    // In a real implementation, this would connect to a persistent orchestrator
    let workflows: Vec<String> = Vec::new(); // Placeholder

    let output = match output_format {
        OutputFormat::Text => {
            if workflows.is_empty() {
                "No active workflows found".to_string()
            } else {
                "Active workflows would be listed here".to_string()
            }
        }
        OutputFormat::Json => serde_json::to_string_pretty(&workflows)?,
        OutputFormat::Yaml => serde_yaml::to_string(&workflows)?,
    };

    println!("{}", output);
    Ok(())
}

/// Show workflow details
fn handle_workflow_show(
    workflow_id: String,
    _show_history: bool,
    output_format: OutputFormat,
) -> Result<()> {
    // Placeholder implementation
    let output = match output_format {
        OutputFormat::Text => format!("Workflow details for {} would be shown here", workflow_id),
        OutputFormat::Json => serde_json::to_string_pretty(&serde_json::json!({
            "workflow_id": workflow_id,
            "message": "Workflow details would be shown here"
        }))?,
        OutputFormat::Yaml => serde_yaml::to_string(&serde_json::json!({
            "workflow_id": workflow_id,
            "message": "Workflow details would be shown here"
        }))?,
    };

    println!("{}", output);
    Ok(())
}

/// Approve a workflow
fn handle_workflow_approve(
    workflow_id: String,
    reviewer: String,
    reason: Option<String>,
    output_format: OutputFormat,
) -> Result<()> {
    // Placeholder implementation
    let output = match output_format {
        OutputFormat::Text => format!("Workflow {} would be approved by {}", workflow_id, reviewer),
        OutputFormat::Json => serde_json::to_string_pretty(&serde_json::json!({
            "workflow_id": workflow_id,
            "reviewer": reviewer,
            "action": "approve",
            "reason": reason
        }))?,
        OutputFormat::Yaml => serde_yaml::to_string(&serde_json::json!({
            "workflow_id": workflow_id,
            "reviewer": reviewer,
            "action": "approve",
            "reason": reason
        }))?,
    };

    println!("{}", output);
    Ok(())
}

/// Reject a workflow
fn handle_workflow_reject(
    workflow_id: String,
    reviewer: String,
    reason: Option<String>,
    output_format: OutputFormat,
) -> Result<()> {
    // Placeholder implementation
    let output = match output_format {
        OutputFormat::Text => format!("Workflow {} would be rejected by {}", workflow_id, reviewer),
        OutputFormat::Json => serde_json::to_string_pretty(&serde_json::json!({
            "workflow_id": workflow_id,
            "reviewer": reviewer,
            "action": "reject",
            "reason": reason
        }))?,
        OutputFormat::Yaml => serde_yaml::to_string(&serde_json::json!({
            "workflow_id": workflow_id,
            "reviewer": reviewer,
            "action": "reject",
            "reason": reason
        }))?,
    };

    println!("{}", output);
    Ok(())
}

/// Archive a workflow
fn handle_workflow_archive(workflow_id: String, output_format: OutputFormat) -> Result<()> {
    // Placeholder implementation
    let output = match output_format {
        OutputFormat::Text => format!("Workflow {} would be archived", workflow_id),
        OutputFormat::Json => serde_json::to_string_pretty(&serde_json::json!({
            "workflow_id": workflow_id,
            "action": "archive"
        }))?,
        OutputFormat::Yaml => serde_yaml::to_string(&serde_json::json!({
            "workflow_id": workflow_id,
            "action": "archive"
        }))?,
    };

    println!("{}", output);
    Ok(())
}

/// Show workflow history
fn handle_workflow_history(
    workflow_id: Option<String>,
    limit: usize,
    output_format: OutputFormat,
) -> Result<()> {
    // Placeholder implementation
    let output = match output_format {
        OutputFormat::Text => {
            if let Some(id) = workflow_id {
                format!(
                    "History for workflow {} (limit: {}) would be shown here",
                    id, limit
                )
            } else {
                format!(
                    "All workflow history (limit: {}) would be shown here",
                    limit
                )
            }
        }
        OutputFormat::Json => serde_json::to_string_pretty(&serde_json::json!({
            "workflow_id": workflow_id,
            "limit": limit,
            "history": []
        }))?,
        OutputFormat::Yaml => serde_yaml::to_string(&serde_json::json!({
            "workflow_id": workflow_id,
            "limit": limit,
            "history": []
        }))?,
    };

    println!("{}", output);
    Ok(())
}

/// Handle cache operations
fn handle_workflow_cache(cleanup: bool, output_format: OutputFormat) -> Result<()> {
    // Placeholder implementation
    let output = match output_format {
        OutputFormat::Text => {
            if cleanup {
                "Cache cleanup would be performed".to_string()
            } else {
                "Cache statistics would be shown here".to_string()
            }
        }
        OutputFormat::Json => serde_json::to_string_pretty(&serde_json::json!({
            "action": if cleanup { "cleanup" } else { "stats" },
            "message": "Cache operation would be performed"
        }))?,
        OutputFormat::Yaml => serde_yaml::to_string(&serde_json::json!({
            "action": if cleanup { "cleanup" } else { "stats" },
            "message": "Cache operation would be performed"
        }))?,
    };

    println!("{}", output);
    Ok(())
}
