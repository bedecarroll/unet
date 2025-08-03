//! CLI integration tests
//!
//! These tests verify the CLI commands work end-to-end using `assert_cmd`.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// Create a test command with in-memory database
fn create_test_command() -> (Command, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let mut cmd = Command::cargo_bin("unet").expect("Failed to find unet binary");
    // Use in-memory SQLite database for testing
    cmd.arg("--database-url").arg("sqlite::memory:");

    (cmd, temp_dir)
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

#[test]
fn test_output_format_json() {
    let (mut cmd, _temp_dir) = create_test_command();

    cmd.args(["--output", "json", "nodes", "list"])
        .assert()
        .success();
}

#[test]
fn test_output_format_yaml() {
    let (mut cmd, _temp_dir) = create_test_command();

    cmd.args(["--output", "yaml", "nodes", "list"])
        .assert()
        .success();
}

#[test]
fn test_output_format_table() {
    let (mut cmd, _temp_dir) = create_test_command();

    cmd.args(["--output", "table", "nodes", "list"])
        .assert()
        .success();
}

#[test]
fn test_nodes_list_empty() {
    let (mut cmd, _temp_dir) = create_test_command();

    cmd.args(["nodes", "list"]).assert().success();
}

#[test]
fn test_locations_list_empty() {
    let (mut cmd, _temp_dir) = create_test_command();

    cmd.args(["locations", "list"]).assert().success();
}

#[test]
fn test_links_list_empty() {
    let (mut cmd, _temp_dir) = create_test_command();

    cmd.args(["links", "list"]).assert().success();
}

#[test]
fn test_vendors_list_empty() {
    let (mut cmd, _temp_dir) = create_test_command();

    cmd.args(["vendors", "list"]).assert().success();
}

#[test]
fn test_policy_list_empty() {
    let (mut cmd, temp_dir) = create_test_command();

    // Create a temporary policy directory
    let policy_dir = temp_dir.path().join("policies");
    std::fs::create_dir(&policy_dir).expect("Failed to create policy directory");

    cmd.args(["policy", "list", "--path", &policy_dir.to_string_lossy()])
        .assert()
        .success();
}

#[test]
fn test_invalid_database_url() {
    let mut cmd = Command::cargo_bin("unet").expect("Failed to find unet binary");

    cmd.args(["--database-url", "invalid://url", "nodes", "list"])
        .assert()
        .failure();
}

#[test]
fn test_verbose_flag() {
    let (mut cmd, _temp_dir) = create_test_command();

    cmd.args(["--verbose", "nodes", "list"]).assert().success();
}

#[test]
fn test_verbose_with_config_file() {
    use tempfile::NamedTempFile;

    let (mut cmd, _temp_dir) = create_test_command();

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
    cmd.args([
        "--verbose",
        "--config",
        &temp_config.path().to_string_lossy(),
        "nodes",
        "list",
    ])
    .assert()
    .success();
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
