use assert_cmd::prelude::*;
use std::process::Command;

// Arrange-Act-Assert: run the binary with and without verbose to exercise main
#[test]
fn runs_without_args() {
    let mut cmd = Command::cargo_bin("config-slicer").unwrap();
    cmd.assert().success();
}

#[test]
fn runs_with_verbose() {
    let mut cmd = Command::cargo_bin("config-slicer").unwrap();
    cmd.arg("--verbose");
    cmd.assert().success();
}
