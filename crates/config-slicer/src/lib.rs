//! `config-slicer` library and CLI.

mod diff;
mod error;
mod parser;
mod slicer;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use serde::Serialize;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use tracing_subscriber::EnvFilter;

pub use diff::diff_text;
pub use error::{ConfigSlicerError, Result as ConfigResult};
pub use parser::{MatchSpec, parse_match};
pub use slicer::{Vendor, slice_text};

#[derive(Parser, Debug)]
#[command(name = "config-slicer")]
#[command(about = "Parse, slice, and diff configuration text by hierarchical match expression")]
#[command(version)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    /// Enable informational tracing output on stderr.
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Validate a match expression and print its parsed levels.
    Parse(ParseCommand),
    /// Slice configuration text from a file or stdin.
    Slice(SliceCommand),
    /// Diff two configurations after slicing them with the same match.
    Diff(DiffCommand),
}

#[derive(Args, Debug)]
pub struct ParseCommand {
    /// Match expression such as `system||ntp`.
    #[arg(short = 'm', long = "match")]
    pub match_expression: String,

    /// Emit machine-readable JSON.
    #[arg(long)]
    pub json: bool,
}

#[derive(Args, Debug)]
pub struct SliceCommand {
    /// Match expression such as `system||ntp`.
    #[arg(short = 'm', long = "match")]
    pub match_expression: String,

    /// Input format to parse.
    #[arg(long, value_enum, default_value_t = Vendor::Autodetect)]
    pub vendor: Vendor,

    /// Emit machine-readable JSON.
    #[arg(long)]
    pub json: bool,

    /// Configuration file path. Reads stdin when omitted.
    pub file: Option<std::path::PathBuf>,
}

#[derive(Args, Debug)]
pub struct DiffCommand {
    /// Match expression such as `system||ntp`.
    #[arg(short = 'm', long = "match")]
    pub match_expression: String,

    /// Source configuration file.
    #[arg(long)]
    pub source: std::path::PathBuf,

    /// Target configuration file.
    #[arg(long)]
    pub target: std::path::PathBuf,

    /// Input format to parse.
    #[arg(long, value_enum, default_value_t = Vendor::Autodetect)]
    pub vendor: Vendor,
}

#[derive(Serialize)]
struct ParsedMatch<'a> {
    expression: &'a str,
    levels: &'a [String],
}

/// Execute the CLI logic with a parsed `Cli`.
///
/// # Errors
/// Returns an error if a file cannot be read, a match expression is invalid, or
/// JSON output cannot be rendered.
pub fn run_with(cli: &Cli) -> Result<()> {
    init_tracing(cli.verbose);

    match &cli.command {
        Command::Parse(command) => run_parse(command)?,
        Command::Slice(command) => run_slice(command)?,
        Command::Diff(command) => run_diff(command)?,
    }

    Ok(())
}

/// Parse CLI args and run.
///
/// # Errors
/// Returns any execution error from `run_with`.
pub fn run<I, S>(args: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: Into<std::ffi::OsString> + Clone,
{
    let cli = Cli::parse_from(args);
    run_with(&cli)
}

fn init_tracing(verbose: bool) {
    let default_level = if verbose { "info" } else { "warn" };
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));

    let _ = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .try_init();
}

fn run_parse(command: &ParseCommand) -> Result<()> {
    let spec = parse_match(&command.match_expression)?;

    if command.json {
        println!(
            "{}",
            serde_json::to_string(&ParsedMatch {
                expression: spec.expression(),
                levels: spec.levels(),
            })?
        );
    } else {
        for (index, level) in spec.levels().iter().enumerate() {
            println!("{index}: {level}");
        }
    }

    Ok(())
}

fn run_slice(command: &SliceCommand) -> Result<()> {
    let spec = parse_match(&command.match_expression)?;
    let text = read_input(command.file.as_deref())?;
    let lines = slice_text(&text, &spec, command.vendor);

    if command.json {
        println!("{}", serde_json::to_string(&lines)?);
    } else if !lines.is_empty() {
        println!("{}", lines.join("\n"));
    }

    Ok(())
}

fn run_diff(command: &DiffCommand) -> Result<()> {
    let spec = parse_match(&command.match_expression)?;
    let source = read_input(Some(command.source.as_path()))?;
    let target = read_input(Some(command.target.as_path()))?;
    let diff = diff_text(&source, &target, &spec, command.vendor);

    if !diff.is_empty() {
        print!("{diff}");
    }

    Ok(())
}

fn read_input(path: Option<&Path>) -> io::Result<String> {
    if let Some(path) = path {
        fs::read_to_string(path)
    } else {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        Ok(buffer)
    }
}
