#!/usr/bin/env bash
set -euo pipefail

# Track total elapsed time for this status run
SECONDS=0

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
phase_start=${SECONDS}
CLIPPY_FIX_DEFAULT=1
CLIPPY_FIX=${STATUS_CLIPPY_FIX:-$CLIPPY_FIX_DEFAULT}
if [[ "$CLIPPY_FIX" == "1" ]]; then
  echo "[status] Running clippy with auto-fix (allow-dirty)"
else
  echo "[status] Running clippy (no auto-fix)"
fi
set -o pipefail
set +e
if [[ "$CLIPPY_FIX" == "1" ]]; then
  cargo clippy --workspace --fix --allow-dirty --allow-staged >"${run_dir}/clippy.log" 2>&1
else
  cargo clippy --workspace >"${run_dir}/clippy.log" 2>&1
fi
clippy_status=$?
set -e
clippy_elapsed=$(( SECONDS - phase_start ))

# 2) Coverage + tests (this runs nextest under instrumentation)
echo "[status] Running coverage+tests (llvm-cov nextest)"
: "${COVERAGE_IGNORE_PATTERN:='(tests?\.rs|_tests?\.rs|comprehensive_tests\.rs)$'}"
set +e
phase_start=${SECONDS}
cargo llvm-cov nextest --workspace --all-features --all-targets \
  --show-missing-lines --ignore-filename-regex "${COVERAGE_IGNORE_PATTERN}" \
  >"${run_dir}/coverage.log" 2>&1
coverage_status=$?
set -e
coverage_elapsed=$(( SECONDS - phase_start ))

echo "[status] --- Summary ---"
echo "[status] Logs: ${run_dir}"

# Clippy summary
clippy_errors=$(grep -c -E "^[[:space:]]*error(:|\[)" "${run_dir}/clippy.log" || true)
clippy_warnings=$(grep -c -E "^[[:space:]]*warning(:|\[)" "${run_dir}/clippy.log" || true)
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

# Coverage TOTAL row — print just the overall percent
total_line=$(grep -E "^TOTAL" "${run_dir}/coverage.log" | tail -n 1 || true)
if [[ -n "${total_line}" ]]; then
  # Extract last percentage on the line, e.g. 91.23%
  coverage_pct=$(printf '%s\n' "${total_line}" | grep -oE '[0-9]+(\.[0-9]+)?%+' | tail -n 1)
  if [[ -n "${coverage_pct}" ]]; then
    echo "[status] Coverage: ${coverage_pct}"
  else
    # Fallback to original line if extraction fails
    echo "[status] Coverage: ${total_line}"
  fi
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

# Exit codes for major phases (helpful for CI/log parsing)
echo "[status] Clippy exit: ${clippy_status}"
echo "[status] Coverage exit: ${coverage_status}"

# Print total elapsed time in a human-readable format
total_sec=${SECONDS}
mins=$(( total_sec / 60 ))
secs=$(( total_sec % 60 ))
if (( mins > 0 )); then
  echo "[status] Total time: ${mins}m${secs}s"
else
  echo "[status] Total time: ${secs}s"
fi

# Phase timings
clippy_m=$(( clippy_elapsed / 60 ))
clippy_s=$(( clippy_elapsed % 60 ))
cov_m=$(( coverage_elapsed / 60 ))
cov_s=$(( coverage_elapsed % 60 ))
if (( clippy_m > 0 )); then
  echo "[status] Clippy time: ${clippy_m}m${clippy_s}s"
else
  echo "[status] Clippy time: ${clippy_s}s"
fi
if (( cov_m > 0 )); then
  echo "[status] Coverage+tests time: ${cov_m}m${cov_s}s"
else
  echo "[status] Coverage+tests time: ${cov_s}s"
fi

exit ${overall_status}
