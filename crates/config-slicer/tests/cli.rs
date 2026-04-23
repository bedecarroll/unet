use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn write_config(root: &Path, file_name: &str, contents: &str) -> String {
    let path = root.join(file_name);
    fs::write(&path, contents).unwrap();
    path.to_string_lossy().into_owned()
}

#[test]
fn help_lists_parse_slice_and_diff_workflows() {
    let mut cmd = Command::cargo_bin("config-slicer").unwrap();

    cmd.arg("--help").assert().success().stdout(
        predicate::str::contains("parse")
            .and(predicate::str::contains("slice"))
            .and(predicate::str::contains("diff")),
    );
}

#[test]
fn parse_command_supports_json_output() {
    let mut cmd = Command::cargo_bin("config-slicer").unwrap();

    cmd.args(["parse", "--match", "system||ntp", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"expression\":\"system||ntp\""))
        .stdout(predicate::str::contains("\"levels\":[\"system\",\"ntp\"]"));
}

#[test]
fn parse_command_emits_plain_text_levels() {
    let mut cmd = Command::cargo_bin("config-slicer").unwrap();

    cmd.args(["parse", "--match", "system||ntp"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0: system"))
        .stdout(predicate::str::contains("1: ntp"));
}

#[test]
fn slice_command_extracts_only_matching_lines() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = write_config(
        temp_dir.path(),
        "running.conf",
        concat!(
            "set system ntp server 192.0.2.1\n",
            "set system ntp source-address 192.0.2.10\n",
            "set interfaces ge-0/0/0 disable\n",
        ),
    );

    let mut cmd = Command::cargo_bin("config-slicer").unwrap();

    cmd.args(["slice", "--match", "system||ntp", &config_path])
        .assert()
        .success()
        .stdout(predicate::str::contains("set system ntp server 192.0.2.1"))
        .stdout(predicate::str::contains(
            "set system ntp source-address 192.0.2.10",
        ))
        .stdout(predicate::str::contains("set interfaces ge-0/0/0 disable").not());
}

#[test]
fn slice_command_supports_json_output() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = write_config(
        temp_dir.path(),
        "running.conf",
        concat!(
            "set system ntp server 192.0.2.1\n",
            "set system ntp source-address 192.0.2.10\n",
        ),
    );

    let mut cmd = Command::cargo_bin("config-slicer").unwrap();

    cmd.args(["slice", "--match", "system||ntp", "--json", &config_path])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "[\"set system ntp server 192.0.2.1\",\"set system ntp source-address 192.0.2.10\"]",
        ));
}

#[test]
fn slice_command_reads_from_stdin_when_file_is_omitted() {
    let mut cmd = Command::cargo_bin("config-slicer").unwrap();

    cmd.arg("slice")
        .arg("--match")
        .arg("system||ntp")
        .write_stdin("set system ntp server 192.0.2.1\nset interfaces ge-0/0/0 disable\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("set system ntp server 192.0.2.1"))
        .stdout(predicate::str::contains("interfaces ge-0/0/0").not());
}

#[test]
fn diff_command_reports_changes_only_within_the_selected_slice() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = write_config(
        temp_dir.path(),
        "source.conf",
        concat!(
            "set system ntp server 192.0.2.1\n",
            "set interfaces ge-0/0/0 description old\n",
        ),
    );
    let target_path = write_config(
        temp_dir.path(),
        "target.conf",
        concat!(
            "set system ntp server 192.0.2.2\n",
            "set interfaces ge-0/0/0 description new\n",
        ),
    );

    let mut cmd = Command::cargo_bin("config-slicer").unwrap();

    cmd.args([
        "diff",
        "--match",
        "system||ntp",
        "--source",
        &source_path,
        "--target",
        &target_path,
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("-set system ntp server 192.0.2.1"))
    .stdout(predicate::str::contains("+set system ntp server 192.0.2.2"))
    .stdout(predicate::str::contains("interfaces ge-0/0/0").not());
}

#[test]
fn diff_command_emits_no_output_when_selected_slice_matches() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = write_config(
        temp_dir.path(),
        "source.conf",
        concat!(
            "set system ntp server 192.0.2.1\n",
            "set interfaces ge-0/0/0 description old\n",
        ),
    );
    let target_path = write_config(
        temp_dir.path(),
        "target.conf",
        concat!(
            "set system ntp server 192.0.2.1\n",
            "set interfaces ge-0/0/0 description new\n",
        ),
    );

    let mut cmd = Command::cargo_bin("config-slicer").unwrap();

    cmd.args([
        "diff",
        "--match",
        "system||ntp",
        "--source",
        &source_path,
        "--target",
        &target_path,
    ])
    .assert()
    .success()
    .stdout(predicate::str::is_empty());
}
