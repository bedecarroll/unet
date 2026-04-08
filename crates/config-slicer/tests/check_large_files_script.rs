use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn script_path() -> PathBuf {
    workspace_root().join("scripts/check-large-files.sh")
}

fn write_rust_file(root: &Path, relative_path: &str, line_count: usize) {
    let file_path = root.join(relative_path);
    fs::create_dir_all(file_path.parent().unwrap()).unwrap();

    let contents = (1..=line_count)
        .map(|line_number| format!("// line {line_number}\n"))
        .collect::<String>();

    fs::write(file_path, contents).unwrap();
}

fn write_baseline(root: &Path, entries: &[(&str, usize)]) {
    let baseline_path = root.join("scripts/check-large-files.baseline");
    fs::create_dir_all(baseline_path.parent().unwrap()).unwrap();

    let mut contents = String::from("# path\tmax_lines\n");
    for (relative_path, line_count) in entries {
        contents.push_str(&format!("{relative_path}\t{line_count}\n"));
    }

    fs::write(baseline_path, contents).unwrap();
}

fn run_large_file_check(repo_root: &Path) -> assert_cmd::assert::Assert {
    let mut command = Command::new("bash");
    command.arg(script_path());
    command.env("CHECK_LARGE_FILES_ROOT", repo_root);
    command.assert()
}

#[test]
fn passes_when_hard_limit_exception_matches_recorded_baseline() {
    let temp_dir = TempDir::new().unwrap();
    write_rust_file(temp_dir.path(), "crates/example/src/advisory.rs", 301);
    write_rust_file(temp_dir.path(), "crates/example/src/legacy.rs", 520);
    write_baseline(temp_dir.path(), &[("crates/example/src/legacy.rs", 520)]);

    run_large_file_check(temp_dir.path())
        .success()
        .stdout(predicate::str::contains("Advisory offenders (>300 lines)"))
        .stdout(predicate::str::contains(
            "301 crates/example/src/advisory.rs",
        ))
        .stdout(predicate::str::contains("520 crates/example/src/legacy.rs"))
        .stdout(predicate::str::contains("Legacy hard-limit exceptions"));
}

#[test]
fn fails_when_new_file_exceeds_hard_limit() {
    let temp_dir = TempDir::new().unwrap();
    write_rust_file(temp_dir.path(), "crates/example/src/new_violation.rs", 501);
    write_baseline(temp_dir.path(), &[]);

    run_large_file_check(temp_dir.path())
        .failure()
        .stdout(predicate::str::contains(
            "Hard failures (>500 lines outside the recorded baseline)",
        ))
        .stdout(predicate::str::contains(
            "501 crates/example/src/new_violation.rs",
        ));
}

#[test]
fn fails_when_baselined_file_grows_beyond_recorded_limit() {
    let temp_dir = TempDir::new().unwrap();
    write_rust_file(temp_dir.path(), "crates/example/src/legacy.rs", 521);
    write_baseline(temp_dir.path(), &[("crates/example/src/legacy.rs", 520)]);

    run_large_file_check(temp_dir.path())
        .failure()
        .stdout(predicate::str::contains(
            "Hard failures (>500 lines outside the recorded baseline)",
        ))
        .stdout(predicate::str::contains("521 crates/example/src/legacy.rs"));
}
