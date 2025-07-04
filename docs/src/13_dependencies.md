<!-- SPDX-License-Identifier: MIT -->

# 13 Dependencies & Coding Conventions – Crate‐by‐Crate Rationale

> **Audience:** All contributors (especially juniors) touching Rust code, Cargo manifests, or CI workflows.\
> **Purpose:** Explain *why* each external crate was picked, **how** to use it correctly, and **when** to avoid it.\
> **Scope:** Runtime + build‑time crates across **unet‑core**, **unet‑server**, **unet‑cli**, **config‑slicer**, and test/CI tooling.

---

## Table of Contents

1. [Dependency Tiers](#1-dependency-tiers)
2. [Core Runtime Crates](#2-core-runtime-crates)
3. [Supporting & Utility Crates](#3-supporting--utility-crates)
4. [Developer & CI Tooling](#4-developer--ci-tooling)
5. [Feature‑Gated / Optional Crates](#5-feature‑gated--optional-crates)
6. [Version Policy & Update Cadence](#6-version-policy--update-cadence)
7. [Coding Conventions & Best Practices](#7-coding-conventions--best-practices)
8. [Rejected or Replaced Dependencies](#8-rejected-or-replaced-dependencies)
9. [FAQ](#9-faq)

---

## 1  Dependency Tiers

| Tier                    | Definition                                                              | Examples                           |
| ----------------------- | ----------------------------------------------------------------------- | ---------------------------------- |
| **Tier 0 – Foundation** | Required in *every* binary at run‑time. Removing them breaks the build. | `tokio`, `anyhow`, `tracing`       |
| **Tier 1 – Domain**     | Needed for μNet’s unique functionality.                                 | `sea-orm`, `axum`, `minijinja`     |
| **Tier 2 – Utilities**  | Quality‑of‑life helpers; easy to swap.                                  | `dashmap`, `regex`, `uuid`         |
| **Tier 3 – Dev/Test**   | Compile only under `#[cfg(test)]` or in CI.                             | `criterion`, `insta`, `assert_cmd` |
| **Tier 4 – Optional**   | Behind Cargo *feature flags*.                                           | `postgres`, `otel`, `simd`         |

**Why tiers?** Helps juniors understand impact when adding/updating crates and keeps the attack surface predictable.

---

## 2  Core Runtime Crates

| Crate                            | Version Pin | Tier | Purpose / Usage Pattern                                                        | Alternatives Considered & Why Rejected                                   |
| -------------------------------- | ----------- | ---- | ------------------------------------------------------------------------------ | ------------------------------------------------------------------------ |
| **tokio**                        | `1.*`       | 0    | Async runtime for *all* IO (HTTP, SNMP, DB). Use `#[tokio::main]` and `spawn`. | `async‑std` (smaller eco‑system); ruled out due to Axum/Tower alignment. |
| **anyhow**                       | `1.*`       | 0    | *Error context & propagation*. Wrap with `?` and enrich via `.context("…")`.   | `failure` (deprecated), `eyre` (no std feature sets).                    |
| **thiserror**                    | `1.*`       | 2    | Derive helper for typed error enums (`ServerError`, `SliceError`).             | Manual `std::fmt` boilerplate – verbose.                                 |
| **tracing / tracing‑subscriber** | `0.1.*`     | 0    | Structured logging (`info!`, `error!`) and JSON formatting in production.      | `log` crate (no span metadata).                                          |
| **serde / serde\_json**          | `1.*`       | 0    | De/serialize REST payloads, DB JSON columns, `custom_data`.                    | `rkyv` (zero copy, but nightly) – overkill for operator text files.      |
| **sea‑orm**                      | `0.12.*`    | 1    | Async ORM + migrations for SQLite/Postgres.                                    | `Diesel` (sync), `SQLx` (no high‑level model code).                      |
| **axum**                         | `0.7.*`     | 1    | HTTP API server; integrates with Tower middleware ecosystem.                   | `Actix‑Web` (macros heavy, less Tower interop).                          |
| **clap** (v4)                    | `4.*`       | 1    | CLI arg parsing via derive macros (`#[derive(Parser)]`).                       | `structopt` (merged), `argh` (less mature).                              |
| **minijinja**                    | `1.*`       | 1    | Jinja‑style template rendering at runtime.                                     | `tera` (larger dep tree), `askama` (compile‑time only).                  |
| **pest**                         | `2.*`       | 1    | PEG parser for Policy DSL grammar (`grammar.pest`).                            | `nom` (verbose), `combine` (abandoned).                                  |
| **snmp2**                        | `0.4.*`     | 1    | Pure‑Rust SNMP v1/v2c client for polling derived state.                        | C FFI to Net‑SNMP (unsafe pointers, build pain).                         |
| **git2**                         | `0.18.*`    | 2    | Programmatic Git clone/fetch for policy/template repos.                        | Shelling to `git` CLI (parsing, env dependence).                         |
| **similar**                      | `2.*`       | 2    | Colorized line/word diff for configs.                                          | `diffy` (no color), `difference` (slower).                               |
| **regex**                        | `1.*`       | 2    | Compile template‑match patterns & policy `=~` ops.                             | PCRE2‑sys (heavy C dep).                                                 |
| **dashmap**                      | `5.*`       | 2    | Concurrent map cache (PolicyResult, template regex).                           | `chashmap` (abandoned), `std::sync::Mutex<HashMap>` (contention).        |
| **uuid**                         | `1.*`       | 2    | Primary keys; supports v4 random + parsing from strings.                       | `ulid` (chronological sort not needed yet).                              |
| **chrono**                       | `0.4.*`     | 2    | Timestamp conversions & RFC3339 in REST API.                                   | `time` (1.3) – fine but chrono more widely known.                        |
| **tokio‑cron‑scheduler**         | `0.10.*`    | 2    | Cron‑like background task runner (git sync).                                   | `cron` crate (sync), home‑grown loops – less accurate.                   |

> \*\*ℹ  Note on "\**Version Pin"* – we use `~` (patch‑level) pin in Cargo.toml to avoid breaking changes, but display caret‑compatible ranges here for readability.

---

## 3  Supporting & Utility Crates

| Crate                  | Domain  | Why We Use It / Example Code                                   |
| ---------------------- | ------- | -------------------------------------------------------------- |
| `ipnet`, `ipaddress`   | IP math | CIDR mask calculations in templates & validation tooling.      |
| `walkdir`              | FS      | Recursively load templates during env rebuild.                 |
| `serde_yaml`           | Docs    | Parse YAML front‑matter on `.rules` files (ignored if absent). |
| `strum / strum_macros` | Enum    | Auto‑derive `Display`, `EnumIter` for `Lifecycle`, `LinkRole`. |
| `bytes`                | Perf    | SNMP PDU buffers without reallocs.                             |
| `futures-util`         | Async   | Stream combinators for concurrent SNMP polling.                |
| `tower`                | HTTP    | Middleware (`TraceLayer`, future `AuthLayer`).                 |

---

## 4  Developer & CI Tooling

| Crate/Tool                   | Purpose                                   | CI Job / Script              |
| ---------------------------- | ----------------------------------------- | ---------------------------- |
| **cargo‑audit**              | Report insecure dependencies              | `check.yml` – security audit |
| **cargo‑tarpaulin**          | Code‑coverage generation (`lcov`, XML)    | Nightly `coverage` job       |
| **criterion**                | Micro‑benchmarks (config‑slicer perf)     | Optional `bench` workflow    |
| **insta**                    | Snapshot tests for policy/template output | `cargo insta review` in PRs  |
| **assert\_cmd + predicates** | Black‑box CLI tests                       | `tests/cli_diff.rs`          |
| **proptest / quickcheck**    | Property tests for DSL equivalences       | Backlog Milestone 5          |

These dev‑only crates are declared under `[dev-dependencies]` **only** to keep release binaries slim.

---

## 5  Feature‑Gated / Optional Crates

| Feature Flag           | Adds Crate(s)                            | When to Enable                                    | Notes & Caveats                                     |
| ---------------------- | ---------------------------------------- | ------------------------------------------------- | --------------------------------------------------- |
| `pg`                   | `sea-orm` (Postgres)                     | Production scale >10 k nodes                      | Requires Postgres ≥ 16; migrations differ slightly. |
| `otel`                 | `opentelemetry`, `tracing‑opentelemetry` | Env with Grafana Tempo / Jaeger                   | Extra 2–3 MB binary, network traffic.               |
| `simd` (config‑slicer) | `memchr`, nightly feature flags          | Benchmark shows >15 % speed gain on >1 MB configs | Compile only on x86\_64/aarch64.                    |
| `bin` (config‑slicer)  | `clap`, `color‑eyre`                     | Needed for CLI build                              | Library users can disable to save size.             |

**Guideline:** Default build = **minimal**; CI `release.yml` builds *both* default and `--all-features` to catch breakage.

---

## 6  Version Policy & Update Cadence

| Rule                                   | Rationale                                     |
| -------------------------------------- | --------------------------------------------- |
| Use **caret** `^` for Tier 0/1 crates. | Get patch releases automatically (security).  |
| Pin **exact** (`=`) in CI Dockerfiles. | Reproducible builds for official images.      |
| **Dependabot weekly** for runtime deps | Prevent stale security fixes.                 |
| `cargo upgrade --workspace` monthly    | Refresh Tier 2/3 crates; run full test suite. |
| Breaking major upgrades → RFC process  | e.g., Tokio 2, Axum 0.8 – need design review. |

---

## 7  Coding Conventions & Best Practices

### 7.1 Error Handling (`anyhow` + `thiserror`)

```rust
use anyhow::{Context, Result};

fn load_cfg(path: &Path) -> Result<Config> {
    let data = std::fs::read_to_string(path)
        .with_context(|| format!("reading config {:?}", path))?;
    toml::from_str(&data).context("parsing TOML")
}
```

*Principles*

- Library crates (`unet-core`) expose **typed** errors via `thiserror` (`ErrorKind`).
- Binary crates (`unet-server`, `unet-cli`) use `anyhow::Result` at *outer* boundary and map to exit codes / HTTP status.
- Never `unwrap()` outside tests; use `.expect("…")` only when logic invariant *guarantees* success.

### 7.2 Logging (`tracing`)

- Structured: `info!(node_id=%id, "poll success")` – searchable by field.
- Add `instrument` macros to async fns for per‑span timing.

### 7.3 Async & Concurrency

- Use `tokio::spawn` **only** inside binaries; keep core library non‑Tokio for testability.
- Shared mutable caches → `DashMap`; large read‑mostly data → `Arc<Vec<_>>`.

### 7.4 Regex Usage

- Pre‑compile at load time (`lazy_static!` or once‑cell) – never build in hot loop.
- Validate user‑supplied regex in policy/template header at Git sync stage.

---

## 8  Rejected or Replaced Dependencies

| Crate / Tool                      | Reason Dropped                                           | Replacement                      |
| --------------------------------- | -------------------------------------------------------- | -------------------------------- |
| `failure`                         | Abandoned, conflicts with `std::error::Error` revisions. | `anyhow` + `thiserror`           |
| `simplelog`                       | Plain‑text log strings, no spans/fields.                 | `tracing`                        |
| `diesel`                          | Sync‑only; heavy macros; no SQLite UPSERT.               | `sea-orm`                        |
| `tera`                            | Larger dep tree, syntax divergence from Jinja2.          | `minijinja`                      |
| `openssl` sys dep via reqwest TLS | Cross‑compilation headaches (musl).                      | `rustls` default TLS in reqwest. |

*Lessons Learned:* Keep binary pure‑Rust + *musl* ‑ link for smooth Docker/Nix builds.

---

## 9  FAQ

**Q:** *When should I add a new crate?*\
**A:** Check if existing utility can solve it first. For Tier 0/1 additions open an **RFC** (see 10\_future\_work.md §5). For Tier 2 utilities, small PR + justification section is okay.

**Q:** *What about unsafe code?*\
**A:** Only transitive – μNet crates must have `#![forbid(unsafe_code)]` except for FFI needed in `git2`, which is audited upstream.

**Q:** *Why do our binaries remain < 20 MB even with so many crates?*\
**A:** MUSL static linking w/ `strip` and `cargo build --release` plus `opt-level = "z"` in `[profile.release]` keep size minimal.

**Q:** *How do I troubleshoot a version conflict?*\
**A:** `cargo tree -d` shows duplicates; prefer newest. Add `patch.crates-io` or `cargo update -p crate@version`.

---

*End of 13\_dependencies.md – keep this document aligned whenever you add/upgrade a crate. PRs touching **``** should link back here in the description.*
