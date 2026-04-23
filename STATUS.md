# μNet Status Snapshot

> **Purpose:** Current repo snapshot for contributors and maintainers
> **Audited on:** 2026-04-07
> **Evidence sources:** `Cargo.toml`, `docs/src/`, `bash scripts/check-large-files.sh`, `rg -n --glob '!target/**' --glob '!**/.git/**' 'TODO|FIXME'`

## Workspace Shape

The current Rust workspace has six members:

- `crates/unet-core`: shared library for models, `DataStore`, policy evaluation, SNMP, and configuration
- `crates/unet-server`: `Axum` API server and background task runtime
- `crates/unet-cli`: operator CLI for datastore- and API-backed workflows
- `crates/config-slicer`: configuration slicing and diffing library plus `config-slicer` CLI
- `crates/migrations`: `SeaORM` migrations library and `migration` binary
- `crates/test-support`: shared test utilities

## Maintained Documentation

- `docs/src/developer_guide.md` now points at the live SNMP and policy parser paths:
  `crates/unet-core/src/snmp/{client,config,oids,poller,session,types,values}` and
  `crates/unet-core/src/policy/{grammar.rs,policy.pest,parser/,evaluator/,loader/}`.
- `docs/src/architecture.md` documents the current workspace members in addition to the main CLI/server/core runtime flow.
- `docs/src/roadmap.md` remains the place for planned feature work. This file records measured current-state facts instead of completion percentages.

## Measured Quality Snapshot

- `bash scripts/check-large-files.sh` passes the hard-limit baseline.
- The same file-size check still reports advisory offenders above 300 lines, and the recorded legacy baseline still includes files above 500 lines. The file-splitting effort is ongoing rather than complete.
- `rg -n --glob '!target/**' --glob '!**/.git/**' 'TODO|FIXME'` still finds committed `TODO` and `FIXME` comments in maintained code and docs. Cleanup is not complete.
- Earlier status statements about workspace size, universal 300-line compliance, and TODO/FIXME cleanup are no longer accurate for the current tree.

## Current Constraints

- Treat the 300-line Rust file target as an active cleanup goal, not as a description of the current tree.
- Treat `TODO` and `FIXME` removal as incomplete work.
- Refresh claims about passing tests, coverage, lint status, or release readiness from current command output before documenting them here.

## Recommended Verification

- `mise run status`
- `mise run lint`
- `bash scripts/check-large-files.sh`
- `rg -n --glob '!target/**' --glob '!**/.git/**' 'TODO|FIXME'`
