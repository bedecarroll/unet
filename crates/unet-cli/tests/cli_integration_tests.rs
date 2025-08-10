//! CLI integration tests
//!
//! These tests verify the CLI commands work end-to-end. Most run in-process
//! against an in-memory `SQLite` schema; a few smoke tests invoke the binary.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use unet_cli::{Cli, AppContext, Db};
use std::future::Future;
use std::pin::Pin;
use clap::Parser;
use test_support::sqlite::entity_db;

/// Create a test command with in-memory database
fn create_test_command() -> (Command, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let mut cmd = Command::cargo_bin("unet").expect("Failed to find unet binary");
    // Use in-memory SQLite database for testing
    cmd.arg("--database-url").arg("sqlite::memory:");

    (cmd, temp_dir)
}

/// Run the CLI in-process with an entity-schema in-memory DB.
async fn run_in_process<I, S>(args: I) -> anyhow::Result<()>
where
    I: IntoIterator<Item = S>,
    S: Into<std::ffi::OsString> + Clone,
{
    // Build a context that ignores the URL and uses shared entity DB
    let connect = Box::new(|_url: &str| {
        Box::pin(async move {
            let conn = entity_db().await;
            Ok::<Db, anyhow::Error>(Db(conn))
        }) as Pin<Box<dyn Future<Output = anyhow::Result<Db>> + Send>>
    });
    let migrate = Box::new(|_db: &Db| {
        // Schema is already applied by entity_db(); no-op here.
        Box::pin(async { Ok::<(), anyhow::Error>(()) })
            as Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>
    });
    let ctx = AppContext { connect, migrate };

    let cli = Cli::parse_from(args);
    unet_cli::run_with(ctx, cli).await
}

#[test]
fn test_cli_help_command() {
    let mut cmd = Command::cargo_bin("unet").expect("Failed to find unet binary");

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "μNet network configuration management",
        ));
}

#[test]
fn test_cli_version_command() {
    let mut cmd = Command::cargo_bin("unet").expect("Failed to find unet binary");

    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("unet"));
}

#[test]
fn test_invalid_output_format() {
    let (mut cmd, _temp_dir) = create_test_command();

    cmd.args(["--output", "invalid", "nodes", "list"])
        .assert()
        .failure();
}

#[tokio::test]
async fn test_output_format_json() {
    let res = run_in_process(["unet", "--output", "json", "nodes", "list"]).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_output_format_yaml() {
    let res = run_in_process(["unet", "--output", "yaml", "nodes", "list"]).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_output_format_table() {
    let res = run_in_process(["unet", "--output", "table", "nodes", "list"]).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_nodes_list_empty() {
    let res = run_in_process(["unet", "nodes", "list"]).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_locations_list_empty() {
    let res = run_in_process(["unet", "locations", "list"]).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_links_list_empty() {
    let res = run_in_process(["unet", "links", "list"]).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_vendors_list_empty() {
    let res = run_in_process(["unet", "vendors", "list"]).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_policy_list_empty() {
    let temp_dir = TempDir::new().expect("tempdir");
    let policy_dir = temp_dir.path().join("policies");
    std::fs::create_dir(&policy_dir).expect("mkdir policy dir");
    let res = run_in_process([
        "unet",
        "policy",
        "list",
        "--path",
        &policy_dir.to_string_lossy(),
    ])
    .await;
    assert!(res.is_ok());
}

#[test]
fn test_invalid_database_url() {
    let mut cmd = Command::cargo_bin("unet").expect("Failed to find unet binary");

    cmd.args(["--database-url", "invalid://url", "nodes", "list"])
        .assert()
        .failure();
}

#[tokio::test]
async fn test_verbose_flag() {
    let res = run_in_process(["unet", "--verbose", "nodes", "list"]).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_verbose_with_config_file() {
    use tempfile::NamedTempFile;

    // Run in-process

    // Create a temporary config file
    let temp_config =
        NamedTempFile::with_suffix(".toml").expect("Failed to create temp config file");
    let toml_content = r#"
[database]
url = "sqlite://test.db"

[logging]
level = "debug"
format = "text"

[server]
host = "127.0.0.1"
port = 8080
max_request_size = 1048576

[snmp]
community = "public"
timeout = 10
retries = 3

[git]
branch = "main"
sync_interval = 300

[domain]
search_domains = []

[auth]
enabled = false
token_expiry = 3600
"#;
    std::fs::write(temp_config.path(), toml_content).expect("Failed to write to temp config");

    // Test CLI with both --verbose and --config flags
    // This should exercise line 115: info!("Using configuration from: {}", config_path.display());
    let res = run_in_process([
        "unet",
        "--verbose",
        "--config",
        &temp_config.path().to_string_lossy(),
        "nodes",
        "list",
    ])
    .await;
    assert!(res.is_ok());
}

#[test]
fn test_unknown_subcommand() {
    let (mut cmd, _temp_dir) = create_test_command();
    cmd.args(["unknown", "command"]).assert().failure();
}

#[test]
fn test_nodes_subcommand_help() {
    let mut cmd = Command::cargo_bin("unet").expect("Failed to find unet binary");

    cmd.args(["nodes", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Node management commands"));
}

#[test]
fn test_locations_subcommand_help() {
    let mut cmd = Command::cargo_bin("unet").expect("Failed to find unet binary");

    cmd.args(["locations", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Location management commands"));
}

#[test]
fn test_links_subcommand_help() {
    let mut cmd = Command::cargo_bin("unet").expect("Failed to find unet binary");

    cmd.args(["links", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Link management commands"));
}

#[test]
fn test_policy_subcommand_help() {
    let mut cmd = Command::cargo_bin("unet").expect("Failed to find unet binary");

    cmd.args(["policy", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Policy management commands"));
}
