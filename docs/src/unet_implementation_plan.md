# Implementation Plan for the **μNet (Unet)** Network Configuration System

> \*\*Audience \*\* Junior engineers joining a brand‑new (greenfield) project. \*\*Goal \*\* Provide a step‑by‑step roadmap, with rationale and rejected alternatives, for building Unet.

---

## 1  Project Overview

*Unet* is a lightweight, Rust‑based platform that:

- Stores **desired** configuration state for network devices (nodes, links, locations).
- Fetches **derived** state via SNMP (software version, interface info, etc.).
- Enforces rules with a **policy engine** (custom DSL inspired by Apple PKL).
- Generates vendor configs from **MiniJinja** templates and diffs them against live configs.
- Exposes an async **Axum** server API plus a **Clap**‑based CLI.
- Persists to SQLite (CSV for demo; Postgres later) using **SeaORM**.
- Is instrumented for future MCP (Model Context Protocol) integration.

> **Why Rust?** Single static binaries, memory‑safe concurrency, first‑class CLI ergonomics.

---

## 2  Milestone Roadmap

| #     | Title              | Key Deliverables                                                                                                               | Rationale                             |
| ----- | ------------------ | ------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------- |
| **1** | Core Data Layer    | `unet-core` crate, Node/Link/Location models, `DataStore` trait, CSV + SQLite back‑ends, minimal REST (create/read) + CLI CRUD | Foundation all else depends on        |
| **2** | SNMP Integration   | `snmp2` polling task, derived vs desired fields, periodic update loop, CLI `node show` w/ derived info                         | Detect drift early                    |
| **3** | Policy Engine      | Custom DSL parser (Pest), `PolicyRule` structs, in‑memory evaluation, ASSERT / SET / APPLY actions, compliance reporting       | Governs correctness & roll‑outs       |
| **4** | Template Engine    | MiniJinja environment, template‑match header, CLI render/diff, vendor‐agnostic config slicer                                   | Generates configs safely              |
| **5** | Diff & Debug Tools | `config-slicer` crate, colored diff output, `unet policy diff`, verbose tracing                                                | Operator confidence                   |
| **6** | Git Sync + Canary  | `git2` sync task, cron config, CLI push/clear canaries                                                                         | Version‑control & emergency overrides |
| **7** | Polish & Roadmap   | Docs, onboarding guide, MCP handshake stub, Postgres feature flag, Docker/Nix packaging                                        | Production‑readiness                  |

Each milestone is **merge‑ready** (tests pass, docs updated).

---

## 3  High‑Level Architecture

```ascii
           +------------------+         Git pull / cron
           | Policy / Tpl Repo|<--------------------------+
           +------------------+                           |
                        ^                                 |
                        | (sync)                          |
+--------------+   REST API    +----------------------+   | background
|  unet‑cli    |<--------------|   unet‑server        |   | tasks
|  (operator)  |  HTTPS :8080  |  Axum + SeaORM       |   |
+--------------+               |  • DataStore (SQLite)|   |
   ^    ^    ^                 |  • Policy Engine     |   |
   |    |    |                 |  • MiniJinja Engine  |   |
   |    |    +-- local mode ---+  • SNMP Poller       |   |
   |    |                      +----------------------+   |
   |    |             ^               ^                  |
   |    |             | derived state |                  |
   |    |             | (SNMP)        |                  |
   |    +----+ config‑slicer crate ----+ diff / testing  |
   |                                                    |
   +----> GitHub Actions CI/CD (build, test, release) --+
```

---

## 4  Data Models (Why, What & Alternatives)

### 4.1  Enumerations

```rust
pub enum Lifecycle { Planned, Implementing, Live, Decommissioned }
```

*Reason:* explicit state machine for site/dev roll‑outs.

### 4.2  Node

| Field                         | Type        | Reason                      |
| ----------------------------- | ----------- | --------------------------- |
| id                            | `Uuid`      | Stable PK across DBs        |
| node\_name                    | `String`    | Human hostname              |
| domain\_name                  | `String`    | Multi‑tenant support        |
| vendor / model / device\_role | `String`    | Template & policy selectors |
| mgmt\_ip                      | `IpAddr`    | SNMP target                 |
| software\_version             | `String`    | **Desired** state           |
| lifecycle                     | `Lifecycle` | Roll‑out gating             |
| location\_id                  | `Uuid`      | Site grouping               |
| custom\_data                  | `JsonValue` | Schema‑less extensions      |

> **Rejected:** Derived columns in same table – schema churn; keep separate status table instead.

### 4.3  Link & Location

*Link* includes nullable `node_z_id` for internet circuits; *Location* is hierarchical (`parent_id`). Both inherit `lifecycle` & `custom_data`.

---

## 5  Policy Engine Design

- **Grammar** `WHEN <cond> THEN <action>`
- **Parser Choice:** **Pest** → human‑readable grammar, good errors.
- **Rejected:** Nom (too low‑level), Rego/OPA (heavy), JSONLogic (verbose).
- **Actions:**
  - `ASSERT field IS value` → compliance record.
  - `SET path TO value` → writes to `custom_data`.
  - `APPLY template_path` → adds template to node.

> **Trade‑off:** Custom DSL is a learning tool for juniors; avoids sandboxing issues of embedded scripting.

---

## 6  Template + Config Diff

- MiniJinja chosen for runtime‑loadable Jinja2 syntax.
- Templates declare scope with a **template‑match** header (e.g. `interfaces ge-.*||.*`).
- Diff workflow:
  1. Render candidate snippet.
  2. `config-slicer` extracts the same hierarchy from live config.
  3. `similar` crate produces colored diff.

**Alternatives Rejected**

| Option                     | Why Not                                |
| -------------------------- | -------------------------------------- |
| Full AST parser per vendor | Huge effort, brittle to versions       |
| Plain diff of whole config | False positives; no partial templating |

---

## 7  Key Crate Selection

| Concern     | Crate                        | Alternatives        | Rationale                         |
| ----------- | ---------------------------- | ------------------- | --------------------------------- |
| CLI         | **Clap v4**                  | StructOpt (merged)  | Stable, derive macros             |
| HTTP Server | **Axum**                     | Actix‑Web, Rocket   | Tokio‑native, minimal magic       |
| ORM         | **SeaORM**                   | Diesel (sync), SQLx | Async, SQLite → Postgres path     |
| SNMP        | **snmp2**                    | net‑snmp FFI        | Pure Rust, async                  |
| Templates   | **MiniJinja**                | Tera                | Familiar syntax, runtime includes |
| Git         | **git2**                     | `git CLI` spawn     | No external binary, programmatic  |
| Slicer      | **config‑slicer** (in‑house) | Regex only          | Structured hierarchy parsing      |

---

## 8  CI/CD Pipeline (GitHub Actions)

```yaml
- fmt & clippy gates
- cargo test --workspace
- cargo audit  # security
- build release binaries
- docker build & push ghcr.io/<org>/unet-server
```

*Rejected:* CircleCI – extra SaaS cost.

---

## 9  Deployment Strategy

1. **Linux binary** + systemd (default).
2. **Docker Compose** with bind‑mounted SQLite.
3. **Nix flake** (road‑map) for reproducible builds.

> **Postgres** behind feature flag once scale demands.

---

## 10  Config‑Slicer Sub‑Project

Goal: expose `--match` hierarchy extraction as stand‑alone crate/CLI:

```bash
config-slicer --match "interfaces ge-.*||.*||.*" < junos.conf
```

Library exports `slice(text, &MatchSpec)` for integration into μNet.

*Rejected Approach:* vendor‑specific XML parsing → heavy & license‑encumbered.

---

## 11  Onboarding Tips

1. `rustup default stable` → consistent toolchain.
2. `cargo watch -x check` → fast feedback loop.
3. Good first issues labelled in GitHub.
4. Docs live in `/docs` – update in same PR as code.

Happy building! 🎉

