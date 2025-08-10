#!/usr/bin/env bash
set -euo pipefail

# LLM-friendly project status: runs clippy and coverage+tests, summarizes results,
# and stores full logs under target/mise-logs/run-<timestamp>/.

run_dir="target/mise-logs/latest"
# Clean previous run to avoid disk growth for LLM workflows.
rm -rf "${run_dir}"
mkdir -p "${run_dir}"

echo "[status] Logs in ${run_dir} (overwrites)"
echo "[status] LLM quick tips:"
echo "[status]  - Test summary:   rg -n '\\A\\s*Summary' ${run_dir}/coverage.log"
echo "[status]  - Failures quick: grep -i 'fail' ${run_dir}/coverage.log"
echo "[status]  - Failures exact: rg -n '(?i)^(failures:|.*\\bFAILED\\b)|panicked at' ${run_dir}/coverage.log"
echo "[status]  - Coverage total: rg -n '^TOTAL' ${run_dir}/coverage.log"
echo "[status]  - Missing lines:  rg -n '^[^ ]+\\.rs:' ${run_dir}/coverage.log"

# 1) Clippy (non-invasive: no --fix, no formatting)
echo "[status] Running clippy (non-fixing)"
set -o pipefail
set +e
cargo clippy --workspace >"${run_dir}/clippy.log" 2>&1
clippy_status=$?
set -e

# 2) Coverage + tests (this runs nextest under instrumentation)
echo "[status] Running coverage+tests (llvm-cov nextest)"
: "${COVERAGE_IGNORE_PATTERN:='(tests?\.rs|_tests?\.rs|comprehensive_tests\.rs)$'}"
set +e
cargo llvm-cov nextest --workspace --all-features --all-targets \
  --show-missing-lines --ignore-filename-regex "${COVERAGE_IGNORE_PATTERN}" \
  >"${run_dir}/coverage.log" 2>&1
coverage_status=$?
set -e

echo "[status] --- Summary ---"
echo "[status] Logs: ${run_dir}"

# Clippy summary
clippy_errors=$(grep -E "^[[:space:]]*error(:|\[)" -n "${run_dir}/clippy.log" | wc -l | tr -d ' ')
clippy_warnings=$(grep -E "^[[:space:]]*warning(:|\[)" -n "${run_dir}/clippy.log" | wc -l | tr -d ' ')
echo "[status] Clippy: ${clippy_errors} errors, ${clippy_warnings} warnings"
if [[ "${clippy_errors}" != "0" ]]; then
  echo "[status] Clippy errors (first 40 lines):"
  grep -nE "^[[:space:]]*error(:|\[)" "${run_dir}/clippy.log" | head -n 40 || true
fi

# Tests + coverage summary
summary_line=$(grep -E "^[[:space:]]*Summary[[:space:]]" "${run_dir}/coverage.log" | tail -n 1 || true)
if [[ -n "${summary_line}" ]]; then
  echo "[status] Tests: ${summary_line#*[ ]}"
else
  echo "[status] Tests: summary not found"
fi

# If failures occurred, list them (best-effort nextest parse)
if grep -Ei "[[:space:]]([0-9]+)[[:space:]]+failed" "${run_dir}/coverage.log" >/dev/null 2>&1; then
  echo "[status] Failed tests (top 20):"
  # Prefer the 'failures:' section if present.
  if sed -n '/^failures:/,/^$/p' "${run_dir}/coverage.log" | sed '1d;/^$/q' | head -n 1 | grep -q .; then
    sed -n '/^failures:/,/^$/p' "${run_dir}/coverage.log" | sed '1d;/^$/q' | head -n 20 || true
  else
    # Fallback: capture panic lines with test names.
    grep -E "^\s*thread '[^']+' panicked at" "${run_dir}/coverage.log" | \
      sed -E "s/^\s*thread '([^']+)'.*/\1/" | head -n 20 || true
  fi
fi

# Coverage TOTAL row
total_line=$(grep -E "^TOTAL" "${run_dir}/coverage.log" | tail -n 1 || true)
if [[ -n "${total_line}" ]]; then
  echo "[status] Coverage: ${total_line}"
else
  echo "[status] Coverage: TOTAL row not found"
fi

# Exit non-zero if clippy or tests failed
overall_status=0
if [[ ${clippy_status} -ne 0 ]]; then overall_status=${clippy_status}; fi
if [[ ${coverage_status} -ne 0 ]]; then overall_status=${coverage_status}; fi

if [[ ${overall_status} -ne 0 ]]; then
  echo "[status] Result: FAILED — see logs in ${run_dir}"
else
  echo "[status] Result: OK — full details in ${run_dir}"
fi

exit ${overall_status}
