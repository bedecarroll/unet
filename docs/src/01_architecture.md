# 01 Architecture – μNet (Unet)

> **Audience:** New engineers joining the project (0–2 yrs experience).\
> **Objective:** Give you a deep, end‑to‑end mental model of how every μNet component fits together, why we chose each technology, and what we intentionally left out.

---

## Table of Contents

1. [Goals & Non‑Goals](#1-goals--non-goals)
2. [High‑Level System Diagram](#2-high-level-system-diagram)
3. [Component Responsibilities](#3-component-responsibilities)
4. [Data Flow Walk‑Through](#4-data-flow-walk-through)
5. [Sequence Diagrams](#5-sequence-diagrams)
6. [Technology Decision Matrix](#6-technology-decision-matrix)
7. [Extensibility Points](#7-extensibility-points)
8. [Operational Considerations](#8-operational-considerations)
9. [Rejected Architectures](#9-rejected-architectures)
---

## 1  Goals & Non‑Goals

| #      | Goal                                                                                             | Why it matters                                    |
| ------ | ------------------------------------------------------------------------------------------------ | ------------------------------------------------- |
| **G1** | **Single‑binary deploy** for both server & CLI                                                   | Eases ops; juniors avoid package hell             |
| **G2** | **Separation of concerns** – data, policy, template engines are libraries reused by CLI & server | DRY & testable                                    |
| **G3** | **Runtime‑configurable** system (policies, templates via Git, domains, auth)                   | Network engineers can iterate without recompiling |
| **G4** | **Support partial adoption** – users can template 5 % of config and diff that slice only         | Low friction entry                                |
| **G5** | Be **MCP‑ready** (Model Context Protocol)                                                        | Future AI tooling alignment                       |

**Non‑Goals (for v0):**

- Real‑time config push/rollback (👀 future).
- Full‑fidelity vendor AST parsing.
- Horizontal scaling across >10k devices (SQLite OK initially).

---

## 2  High‑Level System Diagram

```ascii
          ┌────────────────────────────── Git (pull) ───────────────────────────────┐
          │                                                                         │
┌─────────▼──────────┐                  cron / webhook                 ┌───────────▼───────────┐
│ Policy Repository  │                                                 │ Template Repository   │
│  *.rules, README   │                                                 │  *.jinja, helpers/    │
└─────────▲──────────┘                                                 └───────────▲───────────┘
          │                                                                         │
          │   1. Clone/Fetch                                                        │
          │                                                                         │
          │                                                                         │
          │                                                                         │
          │   2. Load into memory                                                   │
          │                                                                         │
          ▼                                                                         ▼
┌──────────────────────────────────────────────────────────────────────────────────────────┐
│                                  unet‑server (Axum)                                     │
│ ───────────────────────────────────────────────────────────────────────────────────────── │
│  • DataStore (SQLite via SeaORM)        • SNMP Poller (Tokio task)                      │
│  • Policy Engine (Pest DSL)             • Template Engine (MiniJinja)                   │
│  • Git Sync Scheduler (git2)            • REST API (OpenAPI)                            │
└──────────────────────────────────────────────────────────────────────────────────────────┘
          ▲                               ▲                                   ▲
          │                               │ derived state                     │ rendered config / diff
          │                               │                                   │
 REST/JSON│                               │                                   │ REST/JSON
          │                               │                                   │
┌─────────┴──────────┐        Web‑Socket (future)                ┌────────────┴───────────┐
│      unet‑cli      │──────────────────────────────────────────▶│    (Optional UI)      │
│  (operator laptop) │                                           │  Rust + Tauri (roadmap)│
└────────────────────┘                                           └────────────────────────┘
```

**Legend**

- **Solid arrows →** synchronous REST/JSON.
- **Dashed arrows →** background/scheduled tasks.
- **Grey boxes →** external repos under version control.

---

## 3  Component Responsibilities

### 3.1 `unet-core` (Library Crate)

| Module      | What it owns                                                                   | Dependencies           |
| ----------- | ------------------------------------------------------------------------------ | ---------------------- |
| `models`    | `Node`, `Link`, `Location`, `Lifecycle` enums                                  | `serde`, `uuid`        |
| `datastore` | `trait DataStore` + impls:• `CsvStore` (demo)• `SqliteStore` (SeaORM)          | `csv`, `sea-orm`       |
| `policy`    | DSL grammar (Pest), AST, evaluator, actions (`Assert`, `Set`, `ApplyTemplate`) | `pest`, `serde_json`   |
| `template`  | MiniJinja environment loader, template‑match header parser, diff helpers       | `minijinja`, `similar` |
| `snmp`      | Async bulk GET for `sysDescr`, `sysObjectID`, etc.                             | `snmp2`, `tokio`       |

> **Rule of thumb:** Anything that can be unit‑tested without IO lives in `unet-core`.

### 3.2 `unet-server` (Binary Crate)

| Area                 | Responsibility                                             |
| -------------------- | ---------------------------------------------------------- |
| **HTTP API**         | CRUD endpoints (`/nodes`, `/links`, …) using Axum routers. |
| **Background Tasks** | Tokio‐spawned: `git_sync`, `snmp_poll`, `policy_eval`. |
| **Config Loader**    | Parses `config.toml` into strongly-typed struct. |
| **Authentication**   | Basic and JWT middleware backed by `users` table. |
| **Change Management** | Tracks configuration changes & approvals. |
| **Metrics**          | Exposes Prometheus `/metrics` endpoint. |
| **Network Access**   | SSH command plugins with secrets stored securely. |

### 3.3 `unet-cli` (Binary Crate)

| Command Group              | Purpose                                                    |
| -------------------------- | ---------------------------------------------------------- |
| `node`, `link`, `location` | Operator CRUD from shell.                                  |
| `policy`                   | Validate, diff, canary push.                               |
| `template`                 | Render, diff, canary push.                                 |
| `--local` flag             | Bypass server, embed `CsvStore` or `SqliteStore` for demo. |

---

## 4  Data Flow Walk‑Through

### 4.1 “Happy Path” – Add Node & Enforce Version Policy

```text
operator$ unet node add --name dist‑01 --vendor juniper --model qfx5120 --software-version 17.2R1
   │
   │   POST /nodes  {json}
   ▼
[server]  INSERT INTO nodes …                         (DataStore)
   │
   │   (later) SNMP poller fires @15‑min interval
   ▼
[server]  snmp2 bulk‑get("sysDescr")  => "JUNOS 17.1R3-S2"
   ▼
[server]  policy_engine eval: rule "juniper‑qfx‑version" ASSERT 17.2R1 → ❌ violated
   ▼
[server]  stores violation in memory (PolicyResult)   GET /nodes?eval=true
   │
   │   operator$ unet policy diff
   ▼
   CLI fetches JSON → prints table:
   +---------+--------------+-------------------+
   | Node    | Rule         | Status            |
   +---------+--------------+-------------------+
   | dist‑01 | jun‑qfx‑ver  | ✖ wanted 17.2R1   |
```

### 4.2 Template Render & Diff

1. Operator writes new `interfaces.qfx.jinja` → commits to template repo.
2. Git sync cron pulls repo; server reloads MiniJinja env.
3. Operator: `unet template diff dist‑01 -t interfaces.qfx.jinja -o live.conf`.
4. CLI:
   - GET node JSON + assigned template list.
   - Renders candidate config section.
   - Runs `config-slicer` with header string to slice `live.conf`.
   - Runs line diff → colored output.
5. Operator reviews diff in CLI or attaches to a PR.

---

## 5  Sequence Diagrams

### 5.1 Git Sync Scheduler

```plaintext
cron (tokio-cron)          git2              policy loader              template loader
     |                       |                       |                        |
     |---- trigger() ------->|                       |                        |
     |                       |-- fetch --+           |                        |
     |                       |           |           |                        |
     |                       |<- refs ---+           |                        |
     |                       |                       |                        |
     |                       |                       |-- parse *.rules ------>|
     |                       |                       |                        |-- compile jinja --|
```

### 5.2 SNMP Poll + Policy Eval (per node)

```plaintext
SNMP Poller ─┐          snmp2 lib          Policy Engine
             │               │                   │
   loop(nodes)|-- async get -->│               ┌──┴───┐
             │               │               │Rules  │
             │<-- result ----│               └──┬───┘
             │               │                   │evaluate
             │               │------------------>│
             │               │<-- compliance ----│
             └─ update derived state + cache PolicyResult
```

---

## 6  Technology Decision Matrix

| Concern         | Choice                         | Pros                               | Cons                                       | Key References                                                                                 |
| --------------- | ------------------------------ | ---------------------------------- | ------------------------------------------ | ---------------------------------------------------------------------------------------------- |
| HTTP Framework  | **Axum**                       | Tower ecosystem, async, no macros  | Slightly verbose                           | [https://docs.rs/axum](https://docs.rs/axum)                                                   |
| ORM             | **SeaORM**                     | Async, cross‑DB, migrations        | Younger than Diesel                        | [https://www.sea-ql.org/SeaORM/](https://www.sea-ql.org/SeaORM/)                               |
| SNMP            | **snmp2**                      | Pure Rust, no C deps               | Smaller community                          | [https://crates.io/crates/snmp2](https://crates.io/crates/snmp2)                               |
| Git             | **git2**                       | Programmatic, link against libgit2 | libgit2 static build adds MBs              | [https://docs.rs/git2](https://docs.rs/git2)                                                   |
| Templating      | **MiniJinja**                  | Runtime load, Jinja2 syntax        | Slightly slower than Askama (compile‑time) | [https://docs.rs/minijinja](https://docs.rs/minijinja)                                         |
| Config Diff     | **similar** crate              | Colored diff, low dep              | Not structural diff                        | [https://crates.io/crates/similar](https://crates.io/crates/similar)                           |
| Background Jobs | Tokio + `tokio-cron-scheduler` | One runtime, cron syntax           | Cron crate maturity                        | [https://crates.io/crates/tokio-cron-scheduler](https://crates.io/crates/tokio-cron-scheduler) |

---

## 7  Extensibility Points

| Point                | How to extend                                                   | Example                               |                     |
| -------------------- | --------------------------------------------------------------- | ------------------------------------- | ------------------- |
| **Database**         | Implement `DataStore` for Postgres; feature flag `--backend pg` | `PgStore` using SeaORM `PostgresConn` |                     |
| **Policy verbs**     | Add new `Action` enum variant + match arm in evaluator          | `RAISE alert("pagerduty")`            |                     |
| **Template filters** | `template::env.add_filter("cidr_mask", mask_fn)`                | \`{{ ip                               | cidr\_mask(30) }}\` |
| **SNMP OIDs**        | Update `snmp::OIDS` map; rule can SET custom\_data              | Track temperature sensor OID          |                     |
| **MCP**              | Create new crate `unet-mcp-adapter`; expose JSON‑RPC functions  | `getNodeStatus(nodeId)`               |                     |

---

## 8  Operational Considerations

| Topic        | Guidance                                                                  |
| ------------ | ------------------------------------------------------------------------- |
| **Logging**  | `RUST_LOG=info unet-server …`; logs to stdout (systemd/journal friendly). |
| **Metrics**  | Prometheus metrics via `axum-prometheus`; `/metrics` endpoint for Grafana. |
| **Backups**  | If using SQLite – copy db file; `VACUUM` monthly.                         |
| **Scaling**  | Single instance handles \~10k nodes polling every 15 min (Tokio async).   |
| **Security** | Run server behind VPN; enable TLS via Nginx or Axum’s TLS feature.        |

---

## 9  Rejected Architectures

| Idea                                                           | Why We Said **No**                                                                       |
| -------------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| **Microservices** (poller, policy, API in separate containers) | Over‑engineered for MVP; complicates local dev; cross‑service schema coordination.       |
| **Monolithic YAML Config** (no DB)                             | Impossible to query at scale; error‑prone merges; poor concurrency.                      |
| **Full Rego/OPA for policy**                                   | Large binary, steep learning curve; network engineers prefer simple boolean rules.       |
| **NETCONF/RESTCONF instead of SNMP**                           | Not universally enabled; increases dependency footprint; SNMP gives 80 % of needed data. |

---

