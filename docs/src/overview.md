<!-- SPDX-License-Identifier: MIT -->

# Introduction to μNet

> **Audience:** Network engineers evaluating or adopting μNet.
> **Objective:** Provide a quick overview of what the project offers and how the rest of the documentation is organized.

μNet is an open‑source network configuration system written in Rust. It stores your **desired configuration** in a database, polls devices to collect **actual state**, and highlights drift through a powerful policy engine. Configurations are rendered from reusable templates, giving you consistent, vendor‑specific output while keeping everything under version control for easy auditing and rollback.

Some highlights:

- **Single‑binary** server and CLI – deploy one Rust executable for both API and command line interactions.
- **SQLite** backend managed via SeaORM for reliable ACID transactions.
- **Custom policy DSL** to enforce standards and detect configuration drift.
- **MiniJinja** templates for fast, deterministic config generation.
- **Hierarchical config diffing** provided by the standalone `config-slicer` crate.
- **Prometheus** metrics and structured logging for observability.
- **Example fixtures** so you can try μNet without writing your own dataset.

You can spin up a demo in minutes: clone the repo, run `cargo run -p unet-server`, then visit [`http://localhost:3000`](http://localhost:3000) for the API and documentation. The CLI (`unet`) lets you add nodes, run policy checks, and preview rendered configs.

The rest of this documentation is organized as follows:

| Order  | File name                      | Purpose (headline)                                                                         |
| ------ | ------------------------------ | ------------------------------------------------------------------------------------------ |
| **1**  | **01\_architecture.md**        | System overview, flow diagram, component responsibilities, and tech-stack rationale.       |
| **2**  | **02\_data\_models.md**        | Exact Rust struct & DB schema, lifecycle fields, and why we chose JSON custom\_data.       |
| **3**  | **03\_policy\_engine.md**      | DSL grammar, parser design, evaluation flow, sample rules, and rejected options.           |
| **4**  | **04\_template\_engine.md**    | MiniJinja usage, template-match header spec, diff workflow, and config-slicer tie-in.      |
| **5**  | **05\_cli\_tool.md**           | Command reference, global flags, daily workflows, and troubleshooting tips.                |
| **6**  | **06\_server\_backend.md**     | Axum route table, background tasks (SNMP, Git sync), config file examples, error handling. |
| **7**  | **07\_ci\_cd.md**              | GitHub Actions pipeline, quality gates (fmt, clippy, tests, audit), release artifacts.     |
| **8**  | **08\_deployment.md**          | Linux + systemd guide, Docker Compose recipe, future Nix flake outline.                    |
| **9**  | **09\_config\_match\_tool.md** | Stand-alone “config-slicer” crate/CLI spec, library API, publishing steps.                 |
| **10** | **10\_onboarding.md**          | New-hire checklist, dev-box setup, coding conventions, good-first-issue guide.             |
| **11** | **glossary.md**                | One-liner definitions of all domain terms and acronyms.                                    |
