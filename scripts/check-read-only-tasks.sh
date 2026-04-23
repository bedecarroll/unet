#!/usr/bin/env bash
set -euo pipefail

extract_task_block() {
  local task_name=$1

  awk -v task_name="$task_name" '
    $0 == "[tasks." task_name "]" { in_block = 1; next }
    /^\[tasks\./ && in_block { exit }
    in_block { print }
  ' mise.toml
}

assert_block_contains() {
  local block_name=$1
  local expected_line=$2
  local block_content=$3

  if ! grep -Fqx "$expected_line" <<<"$block_content"; then
    printf 'expected %s to contain exact line: %s\n' "$block_name" "$expected_line" >&2
    exit 1
  fi
}

assert_file_contains() {
  local file_path=$1
  local expected_text=$2

  if command -v rg >/dev/null 2>&1; then
    if rg -Fq -- "$expected_text" "$file_path"; then
      return 0
    fi
  elif grep -Fq -- "$expected_text" "$file_path"; then
    return 0
  fi

  {
    printf 'expected %s to contain: %s\n' "$file_path" "$expected_text" >&2
    exit 1
  }
}

lint_block=$(extract_task_block "lint")
assert_block_contains "tasks.lint" 'description = "Run read-only linting and formatting checks"' "$lint_block"
assert_block_contains "tasks.lint" 'run = ["typos", "cargo fmt --check", "cargo clippy --workspace"]' "$lint_block"

lint_fix_block=$(extract_task_block "lint-fix")
assert_block_contains "tasks.lint-fix" 'description = "Auto-fix typos and Rust formatting/lint issues"' "$lint_fix_block"
assert_block_contains "tasks.lint-fix" 'run = ["typos -w", "cargo fmt", "cargo clippy --workspace --allow-dirty --fix"]' "$lint_fix_block"

ci_lint_block=$(extract_task_block "ci-lint")
assert_block_contains "tasks.ci-lint" 'depends = ["check-read-only-tasks", "lint"]' "$ci_lint_block"

pre_commit_block=$(extract_task_block "pre-commit")
assert_block_contains "tasks.pre-commit" 'depends = ["lint-fix"]' "$pre_commit_block"

assert_file_contains "scripts/status-llm.sh" 'CLIPPY_FIX_DEFAULT=0'
assert_file_contains "scripts/status-llm.sh" 'STATUS_CLIPPY_FIX'
