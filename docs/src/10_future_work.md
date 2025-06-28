# 10 Future Work – Roadmap, Research & Stretch Goals

> **Audience:** Core maintainers and senior juniors tasked with shaping μNet **after** the MVP ships (Milestone 7).\
> **Purpose:** Capture *everything* we already know we want **but consciously deferred**, plus speculative R&D ideas. Use this as a living backlog—each bullet should eventually become an RFC or GitHub Issue.

---

## Table of Contents

1. [Guiding Principles](#1-guiding-principles)
2. [Short‑Term Enhancements (0.5 → 0.9)](#2-short-term-enhancements-05--09)
3. [Medium‑Term Ambitions (1.x)](#3-medium-term-ambitions-1x)
4. [Long‑Term Research (2.0+)](#4-long-term-research-20)
5. [Architectural Spikes & RFC Process](#5-architectural-spikes--rfc-process)
6. [Resourcing & Skills Matrix](#6-resourcing--skills-matrix)
7. [Risk Register](#7-risk-register)
8. [Nice‑to‑Haves Backlog](#8-nice-to-haves-backlog)
9. [Parking Lot / Rejected for Now](#9-parking-lot--rejected-for-now)
10. [Timeline Gantt (Draft)](#10-timeline-gantt-draft)

---

## 1  Guiding Principles

| #  | Principle                         | Rationale                                                  |
| -- | --------------------------------- | ---------------------------------------------------------- |
| P1 | **Minimise operator toil**        | Every feature must save at least 10× the time it costs.    |
| P2 | **Invisible scalability**         | Users should not *feel* the switch from SQLite → Postgres. |
| P3 | **Batteries‑included ≠ monolith** | Add plugins/APIs **before** forking the code base.         |
| P4 | **Document everything**           | Future contributors are junior; cargo‑cult is dangerous.   |
| P5 | **Prefer opt‑in over defaults**   | Experimental modules ship disabled behind feature flags.   |

---

## 2  Short‑Term Enhancements (0.5 → 0.9)

> **Goal:** Ship within 3 months post‑MVP; low architectural risk.

### 2.1 RBAC & Auth Middleware

| Task                    | Detail                                                      | Estimated Effort |
| ----------------------- | ----------------------------------------------------------- | ---------------- |
| **Basic Auth**          | Axum `AuthLayer` checking `users` table (bcrypt hash).      | 3 PD             |
| **JWT**                 | `jsonwebtoken` crate; HS256 signing key in config.          | 5 PD             |
| **Role table & policy** | Roles: `viewer`, `operator`, `admin`; middleware on routes. | 2 PD             |

*Why first?* Users cannot expose server on prod network without auth.

### 2.2 Prometheus + OpenTelemetry

1. Add `axum-prometheus` for `/metrics`.
2. Expose custom counters: `snmp_poll_failure_total`, `policy_violation_total`.
3. Compile‑time `otel` feature: spans around SNMP poll & policy eval.

### 2.3 Postgres Feature Flag

- Add `--features pg` to `unet-core`.
- Provide docker‑compose override with Postgres 16.
- Migration: `sea_orm_cli migrate refresh -u pg://` in CI matrix.

### 2.4 Historical Metrics MVP

- Create `poll_history` (node\_id, ts, oids JSONB).
- Background task rotates partition table weekly (Postgres only).
- CLI `node history --metric actual_sw_version --last 30d` -> CSV/plot.

### 2.5 UI Prototype (Tauri)

- Scaffold Rust + Tauri desktop app consuming REST.
- Views: Node table, live diff viewer (monaco‑diff), policy violations red badge.
- CI job builds dmg/AppImage.

---

## 3  Medium‑Term Ambitions (1.x)

> **Timeline:** 6–12 months; requires design docs & small spikes.

### 3.1 Config Push & Rollback Engine

| Sub‑Component         | Design Notes                                                | Dependencies                   |
| --------------------- | ----------------------------------------------------------- | ------------------------------ |
| **Vendor drivers**    | Start with JunOS NETCONF, IOS‑XE SSH expect‑script.         | `russh`, `netconf-rs`          |
| **Transaction model** | Staging config, commit‑confirm with 5 min timer.            |                                |
| **Diff‑to‑patch**     | Translate snippet diff → vendor CLI set operations.         | `juniper‑conf‑patch` crate TBD |
| **Rollback DB**       | `deployment` table storing commit ID, status, diff summary. | Postgres                       |

### 3.2 Distributed Poller Microservice

- Binary `unet-poller` runs on jump hosts, authenticates to server, pulls poll job queue (RabbitMQ or Postgres LISTEN/NOTIFY).
- Publishes SNMP results back via gRPC stream.
- Horizontal scaling to 100 k nodes.

### 3.3 Plugin System (Dynamic SO)

- Search `~/.config/unet/plugins/*.so`.
- ABI: `extern "C" fn register(engine: &mut PluginRegistry)`.
- Use `abi_stable` crate for semver‑safe interface.

### 3.4 Multi‑Tenant Support

- Add `tenant_id` UUID to core tables.
- Row‑level security in Postgres; CLI flag `--tenant`.
- UI host header mapping.

### 3.5 Full MCP Adapter

- Implement gRPC service per MCP spec (`GetNode`, `SetDesired`).
- Code‑gen via `prost`.
- Conformance tests suite (Google open‑source).

---

## 4  Long‑Term Research (2.0+)

| Idea                           | Potential Benefit                                        | Unknowns / Risks                                        |
| ------------------------------ | -------------------------------------------------------- | ------------------------------------------------------- |
| **Streaming Telemetry (gNMI)** | Real‑time interface counters into timeseries             | High ingest cost; vendor support patchy.                |
| **Intent‑based DSL**           | Operators express topology intents, engine plans changes | Complexity vs ROI; verification algorithms.             |
| **AI diff summariser**         | LLM summarises diff & policy impact in English           | Token cost, hallucinations; requires guardrails.        |
| **WebAssembly policy plugins** | Sandboxed, multi‑lang policy extension                   | Runtime latency; compile targets.                       |
| **Data‑plane testing harness** | Generate traffic, verify ACLs automatically              | Lab gear availability; maybe use containerised net‑lab. |
| **Formal verification**        | Use SMT (Z3) to prove template idempotence               | Staff expertise, long compute times.                    |

---

## 5  Architectural Spikes & RFC Process

1. **Spike template:** 2‑page Markdown (Problem, Approach Options, PoC diff, Perf numbers).
2. Max spike time **5 PD**; target branch `spike/<topic>`; demo on Friday.
3. Promote to **RFC** in `docs/rfcs/YYYY‑NN‑slug.md`; require two reviewer LGTMs.
4. Merge gated by CI, PoC behind `cfg(feature="experimental")` or hidden CLI flag.

---

## 6  Resourcing & Skills Matrix

| Skill             | Owner (today) | Backup / Mentor | Hiring Need?  | Notes                          |
| ----------------- | ------------- | --------------- | ------------- | ------------------------------ |
| Rust async/Axum   | @alice        | @mentor\_dev    | Yes (mid)     | Critical path for poller split |
| SNMP/Vendor RPC   | @bob          | –               | No            | Could outsource driver stubs   |
| Postgres tuning   | –             | external SME    | Yes (consult) | For RLS & partitioning         |
| Front‑end (Tauri) | @carol        | –               | No            | Junior can shadow              |
| DevOps/Nix        | –             | @mentor\_dev    | Maybe         | Flake packaging upkeep         |

---

## 7  Risk Register

| Risk ID | Description                          | Probability | Impact | Mitigation                              |
| ------- | ------------------------------------ | ----------- | ------ | --------------------------------------- |
| R‑01    | SNMP library unsound bug → mem leak  | Low         | High   | Stress‑test + valgrind CI               |
| R‑02    | Policy DSL complexity creep          | Medium      | Med    | RFC gate + linter                       |
| R‑03    | SQLite file corruption on power loss | Low         | Med    | WAL+sync, encourage Postgres for prod   |
| R‑04    | Git sync DOS via huge repos          | Low         | Low    | Set repo size limit, shallow clone      |
| R‑05    | Talent churn (Rust skills rare)      | Medium      | High   | Pair programming, documented code style |

---

## 8  Nice‑to‑Haves Backlog

- `unet doctor` CLI diag command.
- `unet db export --diagram` → Mermaid ERD.
- Bash/Zsh prompt integration showing policy violations count.
- VS Code extension for `.rules` syntax highlighting & linting.
- `config-slicer --html` coloured syntax output for Docs.

---

## 9  Parking Lot / Rejected for Now

| Idea                            | Reason Parked / Rejected                |
| ------------------------------- | --------------------------------------- |
| Full Kubernetes Helm deployment | Overkill until HA requirements emerge.  |
| IPv6 only mode                  | Wait for real user demand; SNMP libs ok |
| gRPC first before REST          | Harder for curl & browsers; revisit.    |
| GraphQL API                     | Extra complexity, marginal gain.        |

---

## 10  Timeline Gantt (Draft)

```mermaid
gantt
title μNet Post‑MVP Roadmap (quarters)
    dateFormat  YYYY‑MM
    section Short‑Term (0.5‑0.9)
    Auth & RBAC               :a1, 2025‑07, 15d
    Prometheus Metrics        :a2, 2025‑07, 10d
    Postgres Feature Flag     :a3, after a2, 12d
    UI Prototype (Tauri)      :a4, after a2, 20d
    section Medium‑Term (1.x)
    Config Push Engine        :b1, 2025‑09, 30d
    Distributed Poller        :b2, after b1, 25d
    Plugin System             :b3, parallel b2, 20d
    section Long‑Term (R&D)
    Streaming Telemetry PoC   :c1, 2026‑Q1, 30d
```

> **Note:** Adjust durations after spikes complete; Gantt is guidance, not contract.

---

### Next Action Items

- Convert **Short‑Term** bullets to GitHub Milestone `0.8` issues.
- Recruit one intermediate Rust dev (Job req #NET‑2025‑07‑01).
- Draft RFC–001: *Config Push & Rollback* before 2025‑08‑15.

*End of 10\_future\_work.md.*
