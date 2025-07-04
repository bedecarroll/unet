<!-- SPDX-License-Identifier: MIT -->

# 12 Onboarding – Getting Started With μNet as a Junior Engineer

> **Audience:** Brand‑new engineers (intern → 2 yrs) joining the Unet team.\
> **Scope:** Covers *everything* needed in the first **5 days**—from laptop prep to your first merged PR—plus FAQs, best practices, and a 30‑day skills roadmap.
>
> **Assumptions:** You know basic Git, Linux shell, and have used an IDE before, but **no prior Rust or networking automation experience is required**.

---

## Table of Contents

1. [High‑Level Onboarding Timeline](#1-high-level-onboarding-timeline)
2. [Hardware & OS Requirements](#2-hardware--os-requirements)
3. [Essential Accounts & Access](#3-essential-accounts--access)
4. [Local Environment Setup](#4-local-environment-setup)
5. [Cloning & Building μNet](#5-cloning--building-μnet)
6. [Running Unit & Integration Tests](#6-running-unit--integration-tests)
7. [Daily Development Workflow](#7-daily-development-workflow)
8. [Coding Standards & Tools](#8-coding-standards--tools)
9. [Branching, Commits & PR Etiquette](#9-branching-commits--pr-etiquette)
10. [Debugging & Troubleshooting](#10-debugging--troubleshooting)
11. [Documentation Contribution](#11-documentation-contribution)
12. [Knowledge Base & Learning Resources](#12-knowledge-base--learning-resources)
13. [30‑Day Skills Growth Plan](#13-30-day-skills-growth-plan)
14. [FAQ](#14-faq)

---

## 1  High‑Level Onboarding Timeline

| Day | Milestone                                      | Outcome / Artifact                          |
| --- | ---------------------------------------------- | ------------------------------------------- |
| 1   | Laptop ready, SSH key uploaded, repo cloned    | `git clone` succeeds, `cargo check` passes. |
| 2   | Dev env verified                               | Run `unet-server --help`, `unet --help`.    |
| 3   | First small PR (docs or clippy fix)            | CI green; PR merged by reviewer.            |
| 4   | Play with demo dataset & CLI                   | Can add node, run `policy diff`.            |
| 5   | Pick “Good First Issue” & start implementation | Local tests added, WIP branch pushed.       |

> **Tip:** Track progress in the shared **Onboarding Kanban** board on GitHub Projects.

---

## 2  Hardware & OS Requirements

| Resource     | Minimum                          | Recommended                  | Notes                                |
| ------------ | -------------------------------- | ---------------------------- | ------------------------------------ |
| CPU          | 2 vCPU                           | 4 vCPU                       | `cargo check` parallelism            |
| RAM          | 4 GB                             | 8 GB                         | Tests + IDE                          |
| Disk         | 5 GB free                        | 20 GB free                   | Rust target dir grows (\~2 GB)       |
| OS           | Linux (Ubuntu 20.04+/Fedora 38+) | macOS 13+ (Apple Silicon OK) | Windows WSL 2 also works (slower FS) |
| Connectivity | GitHub, crates.io, Docker Hub    | VPN (for staging lab access) | Port 8080/8443 free locally          |

---

## 3  Essential Accounts & Access

1. **GitHub Enterprise** – request `@corp-net` org access.
2. **VPN creds** – for staging lab SSH later.
3. **1Password** – store GitHub PAT & VPN profile.

*DM @engineering‑ops on Slack if anything is missing.*

---

## 4  Local Environment Setup

### 4.1 Install Toolchain (Linux/macOS)

```bash
# 1 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
rustup default stable

# 2 Required components
rustup component add clippy rustfmt

# 3 Optional (for release size checks)
rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl

# 4 Git & build deps (Debian example)
sudo apt-get update && sudo apt-get install -y \
  git cmake pkg-config libssl-dev build-essential libsnmp-dev jq
```

> **macOS:** Install Homebrew + `brew install git jq snmp rustup-init` then run rustup.

### 4.2 Install Docker (Optional for Compose tests)

Follow [docs.docker.com/engine/install](https://docs.docker.com/engine/install). Ensure `docker info` shows `Version >= 25`.

### 4.3 Node.js (Docs Only)

`brew install node` or `apt install nodejs` – required for Mermaid live diagrams in VS Code.

---

## 5  Cloning & Building μNet

```bash
mkdir -p ~/src && cd ~/src
# Fork then clone your fork (replace YOURUSER)
git clone git@github.com:YOURUSER/unet.git
cd unet

# Upstream remote for eventual PR sync
git remote add upstream https://github.com/corp-net/unet.git

# First compile (debug)
cargo check --workspace --all-targets

# Run clippy (lint)
cargo clippy --workspace --all-targets -- -D warnings
```

### 5.1 Running the Server Locally

```bash
# Create demo SQLite file + sample data
cp fixtures/demo/unet.db .

# Start server (port 8080)
cargo run -p unet-server -- \
  --config fixtures/demo/config.toml
```

Open another shell:

```bash
curl http://127.0.0.1:8080/health/live       # → {"status":"live"}
./target/debug/unet --server http://localhost:8080 node list -o table
```

---

## 6  Running Unit & Integration Tests

```bash
# Unit + integration
cargo test --workspace

# Code coverage (Linux only)
cargo tarpaulin -o Xml
```

`tests/fixtures/` contains mock configs, CSVs, rules, templates for deterministic tests.

---

## 7  Daily Development Workflow

```text
        ┌──── pull upstream/main ───┐
        │                           │
        ▼                           │
   feature/#123‑my‑fix  (local)     │
        │                           │
  cargo check + tests               │
        │                           │
   git commit -S -m "feat: …"       │
        │                           │
   git push --set‑upstream origin   │
        │                           │
   Draft PR → CI runs               │
        │                           │
   Address review, squash‑merge ⇦───┘
```

1. Keep branches **small (≤300 LOC)**; vertical slices over horizontal.
2. Always link GitHub Issue (`fixes #123`) in PR description.
3. Run `./scripts/pre‑commit.sh` (format + clippy + tests) before push.

---

## 8  Coding Standards & Tools

| Concern        | Enforcement                              | Command                                  |
| -------------- | ---------------------------------------- | ---------------------------------------- |
| **Formatting** | `rustfmt.toml`                           | `cargo fmt`                              |
| **Linting**    | `clippy.toml` (`-D warnings`)            | `cargo clippy --workspace --all-targets` |
| **Commit msg** | Conventional Commits + Jira ID           | Verified in PR template                  |
| **Secrets**    | `git secrets` pre‑commit hook (optional) | `./scripts/install_git_hooks.sh`         |
| **Editor**     | VS Code + `rust‑analyzer` extension      | Auto‑configured on open (devcontainer)   |

> **Auto‑fix:** Run `just fmt` or `cargo make format` if you install **just** or **cargo‑make**.

---

## 9  Branching, Commits & PR Etiquette

1. **Branch name format:** `feat/<brief‑slug>` or `fix/<bug‑slug>`.
2. **Commit sign‑off:** `git commit -S` (GPG key set in GitHub).
3. \*\*PR template fields: \*\*Description, Checklist, Screenshots (if UI), Linked Issue.
4. **Labels:** Add `area:policy`, `area:server`, etc. Use existing labels.
5. **Reviews:** At least **1 approval** from senior or peer.
6. **Squash vs Merge:** Squash‑merge preferred; rebase if >3 commits.

---

## 10  Debugging & Troubleshooting

| Symptom                          | Quick Checks & Fixes                                                  |
| -------------------------------- | --------------------------------------------------------------------- |
| `cargo build` slow (>3 min)      | Check antivirus (Windows), use `sccache`, update Rust.                |
| `EACCES: port 8080 in use`       | Another process (VS Code Live Share?) – kill or change port.          |
| `database locked` when tests run | Use tmp dir for SQLite; ensure tests run serial (`--test-threads=1`). |
| `SNMP poll timeout` in demo      | Demo uses `127.0.0.1`; set `snmp.enabled=false` in config.            |
| CI fails on clippy               | Run `cargo clippy --fix -Z unstable-options` then review diffs.       |

### Log Levels

```bash
RUST_LOG=debug cargo run -p unet-server -- --config …
```

Add `trace` for HTTP wire dumps (large!).

---

## 11  Documentation Contribution

- **Markdown files** live under `docs/` (same repo).
- Each major feature has its own `XX_<topic>.md` (see `docs/manifest.md`).
- Use absolute links `[02_data_models.md](02_data_models.md)` within docs.
- Include **decision records** under `docs/rfcs/` (`YYYY‑NN‑slug.md`).
- Run `markdownlint` (`npm i -g markdownlint-cli`) before committing.

> Graphs: embed Mermaid fenced blocks; GitHub renders automatically.

---

## 12  Knowledge Base & Learning Resources

| Topic          | Resource                                                         | Why pick this | Est. time |
| -------------- | ---------------------------------------------------------------- | ------------- | --------- |
| Rust beginners | [rust‑lings](https://github.com/rust-lang/rustlings)             | Interactive   | 4–6 hrs   |
| Async Rust     | [Zero To Production](https://zero2prod.com) Chapters 4‑6         | HTTP + tokio  | 3 hrs     |
| SNMP basics    | NetMan “SNMP 101” slides (internal)                              | Terminology   | 1 hr      |
| MiniJinja docs | [docs.rs/minijinja](https://docs.rs/minijinja/latest/minijinja/) | Template API  | 30 min    |
| Pest grammar   | [pest.rs/book](https://pest.rs/book/)                            | DSL parser    | 45 min    |
| GitHub Actions | [docs.github.com/actions](https://docs.github.com/actions)       | CI workflow   | 1 hr      |

Bookmark **#unet‑dev** Slack channel for Q&A.

---

## 13  30‑Day Skills Growth Plan

| Week | Focus Area           | Suggested Tasks                                            | Mentor Check‑in |
| ---- | -------------------- | ---------------------------------------------------------- | --------------- |
| 1    | Rust basics          | Complete rust‑lings, fix a clippy „warn“ in codebase       | 1:1 with buddy  |
| 2    | Data models + SeaORM | Add field to `custom_data` via migration; write test       | PR review       |
| 3    | Policy engine        | Write new rule enforcing hostnames; add unit test          | Pair‑prog       |
| 4    | Templates & Diff     | Create JunOS banner template, demo diff against lab switch | Demo Friday     |

> Graduation: ability to deliver • PR with code + tests + docs, self‑driven.

---

## 14  FAQ

**Q:** *I get **`linker cc not found`** on cargo build.*\
**A:** Install build tools (`sudo apt install build-essential`) or XCode Command Line Tools.

**Q:** *Do I need to know networking to start?*\
**A:** Not on day 1. You’ll learn vendor lingo gradually; start with data & policy modules.

**Q:** *Can I use Windows?*\
**A:** Yes—use WSL 2; see `docs/windows_setup.md` (future). Native Windows builds are lower priority.

**Q:** *Who reviews my PRs?*\
**A:** Tag `@net-platform‑reviewers`. Use CODEOWNERS for auto‑request.

---

### 🎉 Welcome to μNet

Ping your mentor once you’ve reached “Day 2 – dev env verified.” Happy hacking & don’t hesitate to ask questions—no matter how small. Our mantra: **“Docs are never done.”** If something was confusing during setup, open an issue or PR to improve this file on your very first week!
