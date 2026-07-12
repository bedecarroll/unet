#!/usr/bin/env bash
set -euo pipefail

repo_root="${CHECK_LARGE_FILES_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
advisory_limit="${CHECK_LARGE_FILES_ADVISORY_LIMIT:-300}"
hard_limit="${CHECK_LARGE_FILES_HARD_LIMIT:-500}"
baseline_file="${CHECK_LARGE_FILES_BASELINE:-$repo_root/scripts/check-large-files.baseline}"

if [[ ! -d "$repo_root/crates" ]]; then
  printf 'Large file check could not find %s\n' "$repo_root/crates" >&2
  exit 1
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

all_offenders_file="$tmp_dir/all-offenders.txt"
legacy_exceptions_file="$tmp_dir/legacy-exceptions.txt"
hard_failures_file="$tmp_dir/hard-failures.txt"
baseline_input="$baseline_file"

if [[ ! -f "$baseline_file" ]]; then
  baseline_input="$tmp_dir/empty-baseline.txt"
  : > "$baseline_input"
fi

find "$repo_root/crates" -type f -name '*.rs' -exec wc -l {} + \
  | sed "s#${repo_root}/##" \
  | sort -nr \
  | awk \
      -v advisory_limit="$advisory_limit" \
      -v hard_limit="$hard_limit" \
      -v baseline_file="$baseline_input" \
      -v all_offenders_file="$all_offenders_file" \
      -v legacy_exceptions_file="$legacy_exceptions_file" \
      -v hard_failures_file="$hard_failures_file" '
BEGIN {
  while ((getline line < baseline_file) > 0) {
    if (line ~ /^#/ || line ~ /^[[:space:]]*$/) {
      continue
    }

    split(line, fields, "\t")
    baseline[fields[1]] = fields[2] + 0
  }
}
$2 == "total" {
  next
}
{
  count = $1 + 0
  path = $2

  if (count > advisory_limit) {
    print count " " path >> all_offenders_file
  }

  if (count > hard_limit) {
    if ((path in baseline) && baseline[path] >= count) {
      print count " " path >> legacy_exceptions_file
      next
    }

    print count " " path >> hard_failures_file
  }
}'

relative_baseline_path="${baseline_file#$repo_root/}"

printf 'Rust file size policy\n'
printf '  Advisory threshold: >%s lines\n' "$advisory_limit"
printf '  Hard threshold: >%s lines\n' "$hard_limit"
printf '  Recorded legacy baseline: %s\n\n' "$relative_baseline_path"

if [[ -s "$all_offenders_file" ]]; then
  printf 'Advisory offenders (>%s lines)\n' "$advisory_limit"
  sed 's/^/  /' "$all_offenders_file"
  printf '\n'
else
  printf 'Advisory offenders (>%s lines)\n' "$advisory_limit"
  printf '  None\n\n'
fi

if [[ -s "$legacy_exceptions_file" ]]; then
  printf 'Legacy hard-limit exceptions (grandfathered, must not grow)\n'
  sed 's/^/  /' "$legacy_exceptions_file"
  printf '\n'
else
  printf 'Legacy hard-limit exceptions (grandfathered, must not grow)\n'
  printf '  None\n\n'
fi

if [[ -s "$hard_failures_file" ]]; then
  printf 'Hard failures (>%s lines outside the recorded baseline)\n' "$hard_limit"
  sed 's/^/  /' "$hard_failures_file"
  printf '\nResult: FAIL\n'
  exit 1
fi

printf 'Hard failures (>%s lines outside the recorded baseline)\n' "$hard_limit"
printf '  None\n\n'
printf 'Result: PASS\n'
