<!-- SPDX-License-Identifier: MIT -->

# 05 CLI Tool – `unet-cli` Command‑Line Interface Guide

> **Audience:** Engineers implementing the CLI (Clap v4) *and* operators who will invoke it daily.
>
> **Goals:**
>
> 1. Provide a **discoverable**, **script‑friendly** interface for CRUD, policy, templates, and diff operations.
> 2. Support both **client–server** mode (REST) and **offline local** mode (CSV / SQLite) with the **same** commands.
> 3. Deliver rich ergonomics: colored output, table/JSON formats, shell completion, and verbose diagnostics.

---

## Table of Contents

1. [Binary Overview](#1-binary-overview)
2. [Global Flags & Environment Variables](#2-global-flags--environment-variables)
3. [Command Tree](#3-command-tree)
4. [Detailed Command Specs](#4-detailed-command-specs)
5. [Output Formats](#5-output-formats)
6. [Configuration Files](#6-configuration-files)
7. [Offline Local‑Mode](#7-offline-local-mode)
8. [Error Codes & Exit Status](#8-error-codes--exit-status)
9. [Shell Completion & Man Pages](#9-shell-completion--man-pages)
10. [Internal Architecture](#10-internal-architecture)
11. [Testing Strategy](#11-testing-strategy)
12. [Packaging & Distribution](#12-packaging--distribution)
13. [Extending the CLI](#13-extending-the-cli)
14. [Rejected Design Choices](#14-rejected-design-choices)

---

## 1  Binary Overview

- **Name:** `unet-cli`
- **Source:** `crates/unet-cli/`
- **Framework:** **Clap v4** (*derive API*).
- **Runtime:** Single static binary (40–50 MB, stripped).

A minimal invocation (defaults):

```bash
unet --server http://localhost:8080 node list -o table
```

Alias: we expose `unet` *and* `unet-cli` (symlink) for brevity.

---

## 2  Global Flags & Environment Variables

| CLI Flag / Env Var         | Type      | Default                                      | Purpose                               |                         |                       |
| -------------------------- | --------- | -------------------------------------------- | ------------------------------------- | ----------------------- | --------------------- |
| `--server <URL>`           | flag      | `UNET_SERVER` env or `http://127.0.0.1:8080` | REST endpoint (ignored in `--local`). |                         |                       |
| `--token <string>`         | flag      | `UNET_TOKEN` env                             | Auth bearer token (future).           |                         |                       |
| `--local`                  | bool      | `false`                                      | Force local DataStore backend.        |                         |                       |
| \`--backend \<csv          | sqlite>\` | flag                                         | `sqlite`                              | Backend when `--local`. |                       |
| `--db-path <file>`         | flag      | `$PWD/unet.db`                               | SQLite file path (local).             |                         |                       |
| `--csv-path <dir>`         | flag      | `fixtures/demo`                              | CSV dir when backend=csv.             |                         |                       |
| `-o, --output <format>`    | flag      | `table` for TTY, `json` for pipe             | Output format auto‑detect (see §5).   |                         |                       |
| `-v/‑vv/‑vvv`              | count     | 0                                            | Log level: info / debug / trace.      |                         |                       |
| \`--color \<auto           | always    | never>\`                                     | flag                                  | `auto`                  | Colorization control. |
| `-h, --help` / `--version` | builtin   | –                                            | Standard Clap help/version.           |                         |                       |

> **Tip:** Operators export `UNET_SERVER` once and omit flag.

---

## 3  Command Tree

```text
unet [GLOBAL FLAGS] <COMMAND> [SUBCOMMANDS] [OPTIONS]

Commands:
  node        CRUD for devices
  link        CRUD for links
  location    CRUD for sites
  policy      Rule validation & compliance diff
  template    Render, diff, and canary push
  git         Git repository helpers (sync, status)
  config      Manage server configuration values
  import      Load fixtures into the datastore
  export      Dump data to CSV or JSON
  db          Admin actions (migrate, vacuum)
  completion  Generate shell completion script
  version     Print CLI + server versions
```

*Each CRUD command has sub‑commands*: `add`, `list`, `show`, `update`, `delete`, etc.

---

## 4  Detailed Command Specs

### 4.1 `node` Command

| Subcommand      | Flags / Args                         | Description / Behaviour                                   |                                                                           |
| --------------- | ------------------------------------ | --------------------------------------------------------- | ------------------------------------------------------------------------- |
| `add`           | `--name <str>` **(req)**             |                                                           |                                                                           |
|                 | `--vendor <str>` **(req)**           |                                                           |                                                                           |
|                 | `--model <str>`                      |                                                           |                                                                           |
|                 | `--role <str>`                       |                                                           |                                                                           |
|                 | `--ip <addr>`                        |                                                           |                                                                           |
|                 | `--version <str>`                    |                                                           |                                                                           |
|                 | \`--location \<name                  | uuid>\`                                                   | Insert Node into DataStore. Returns JSON object on success.               |
| `list`          | `-o <format>`                        |                                                           |                                                                           |
|                 | `--filter <expr>` (jq‑style)         | List nodes. Filter uses jq‑like JSON expression (future). |                                                                           |
| `show`          | \`\<name                             | uuid>\`                                                   |                                                                           |
|                 | `--eval`                             | Show single node. `--eval` merges PolicyResult fields.    |                                                                           |
| `update`        | \`\<name                             | uuid>\`                                                   |                                                                           |
|                 | `--set <field>=<value>` (repeatable) | PATCH‑like semantics. Validates field names.              |                                                                           |
| `delete`        | \`\<name                             | uuid>\`                                                   | Delete node (soft‑delete sets lifecycle=Decommissioned unless `--force`). |
| `set-lifecycle` | `<name>` `<state>`                   | Transition lifecycle (validates state machine).           |                                                                           |

**Examples**

```bash
# Add using positional JSON file
unet node add -f new_node.json

# Quick list as JSON
unet node list -o json | jq '.[].node_name'
```

### 4.2 `policy` Command

| Subcommand | Description                                      |
| ---------- | ------------------------------------------------ |
| `validate` | Parse all rules, print syntax errors with L/C.   |
| `diff`     | List violating nodes/links/locations.            |
| `push`     | Push canary `.rules` file (expires via `--ttl`). |
| `clear`    | Remove active canaries.                          |

Flags:

```text
--format table|json|yaml   (default table)
--ttl   <duration>         e.g. 2h, 1d
--filter <expr>            jq‑style filter on violations
```

### 4.3 `template` Command

| Subcommand | Key Flags               | Behaviour                                                                 |                                                      |
| ---------- | ----------------------- | ------------------------------------------------------------------------- | ---------------------------------------------------- |
| `render`   | `-t/--templates <list>` |                                                                           |                                                      |
|            | \`-o \<file             | ->\`                                                                      | Render templates for node; writes to file or stdout. |
| `diff`     | `-t <tpl>` (optional)   |                                                                           |                                                      |
|            | `-o <live.conf>`        |                                                                           |                                                      |
|            | `--native` (use lib)    | Render + diff live vs candidate. Exit code 2 on mismatch.                 |                                                      |
| `push`     | `-f <file>`             |                                                                           |                                                      |
|            | `--expire <duration>`   | Push canary template (local file) to server memory map; overrides policy. |                                                      |

### 4.4 `db` Command (Admin‑only)

```bash
unet db migrate up             # apply pending migrations
unet db vacuum                 # for SQLite
unet db export --format csv    # dump nodes, links
```

Requires `UNET_TOKEN` with `admin` scope (future). In `--local` mode no auth.

### 4.5 `git` Command

```bash
unet git sync           # pull latest policies and templates
unet git status         # show last sync time and commit ids
```

### 4.6 `config` Command

```bash
unet config show                # print current server config
unet config set polling.interval 900
```

### 4.7 `import` / `export`

```bash
unet import fixtures/demo
unet export --format json > backup.json
```

---

## 5  Output Formats

We auto‑detect TTY vs pipe using `atty::is(Stream::Stdout)`.

- **table** – borderless ASCII table (colored).
- **json**  – pretty JSON.
- **yaml**  – if `--output yaml`.
- **ndjson** – newline‑delimited JSON (`--output ndjson`), good for stream processing.

Implementation uses `` crate for tables and `serde_json` for JSON.

---

## 6  Configuration Files

`$HOME/.config/unet/config.toml` (XDG) overrides env/flags.

```toml
[defaults]
server = "https://unet.prod.corp:443"
output = "table"
color  = "always"
```

Load order precedence (highest to lowest):

1. CLI flags
2. Env variables
3. Config file
4. Built‑in defaults

---

## 7  Offline Local‑Mode

Pass `--local` **OR** set `UNET_LOCAL=1` to bypass HTTP layer.

| Backend | Flag                                | Use Case               |
| ------- | ----------------------------------- | ---------------------- |
| CSV     | `--backend csv --csv-path <dir>`    | Demo, quick edits      |
| SQLite  | `--backend sqlite --db-path <file>` | Laptop dev, unit tests |

Local mode shares the **exact** same DataStore trait as server; CLI links directly to `unet-core`.

> **Benefit:** Operators can iterate on data & templates without server connectivity (e.g., airplane).

---

## 8  Error Codes & Exit Status

| Code | Meaning                                    |
| ---- | ------------------------------------------ |
| 0    | Success                                    |
| 1    | Generic error / CLI misuse                 |
| 2    | Successful diff – differences detected     |
| 64   | Command‑line usage error (Clap validation) |
| 65   | Policy parse error                         |
| 66   | Data validation failed (e.g., bad IP)      |
| 68   | Network error (server unreachable, TLS)    |
| 69   | Auth failure / forbidden                   |

Rust type: `anyhow::Result<()>` at main + `ProcessExit` custom error converts codes.

---

## 9  Shell Completion & Man Pages

- **Clap Complete** generates Bash, Zsh, Fish, PowerShell files.
- `build.rs` creates man pages via `clap_mangen`.

```bash
unet completion bash > ~/.bash_completion.d/unet
sudo install -Dm644 <(unet completion man) /usr/share/man/man1/unet.1
```

CI artifact attaches these outputs for each release.

---

## 10  Internal Architecture

```rust
// crates/unet-cli/src/main.rs
#[derive(Parser)]
#[command(author, version, about, propagate_version = true)]
struct Cli {
    #[command(flatten)]
    global: GlobalOpts,
    #[command(subcommand)]
    cmd: Commands,
}
```

- We share `` types for REST structs via `serde` to ensure compile‑time contract.
- **REST client** uses `` (blocking feature) + `tracing` middleware for debug.
- For tables we use `` with derived `Tabled` impl.

**Layer diagram**

```text
CliFrontend ──> Dispatcher ──>   LocalStore | RestClient  ──> DataStore / Server
                          \
                           +──> OutputRenderer (table/json)
```

`Dispatcher` decides local vs REST once per invocation; shared trait `Backend` with async fns (calls are awaited via `tokio` runtime in main).

---

## 11  Testing Strategy

1. **Unit tests** – parse args (`Cli::parse_from`) assertions.
2. **Integration tests** – spin up `` in memory (SQLite) via `spawn_server()` fixture, run CLI via `assert_cmd`.
3. **Golden tests** – `cargo insta review` snapshots for `node list -o json`.
4. **Shell completion test** – generate & assert non‑empty for each shell.

CI executes with `cargo test -p unet-cli --all-features`.

---

## 12  Packaging & Distribution

| Channel        | Command / Action                                           |
| -------------- | ---------------------------------------------------------- |
| **Homebrew**   | `brew install your‑org/tap/unet` (formula taps GitHub tar) |
| **APT** (deb)  | `cargo-deb` output uploaded in Release workflow            |
| **RPM**        | `cargo-rpm` spec, CentOS/RHEL8 builders                    |
| **Chocolatey** | Windows packaging (low priority)                           |
| **Docker**     | `docker run ghcr.io/your‑org/unet-cli:latest`              |

Release workflow tags commit, builds **musl‑static** binaries for `x86_64`+`aarch64` and uploads.

---

## 13  Extending the CLI

| Task                      | How‑To                                                                                   |
| ------------------------- | ---------------------------------------------------------------------------------------- |
| Add new subcommand        | Add variant to `Commands` enum; implement `impl CommandExecutor for CmdX` trait.         |
| Support new output format | Extend `OutputFormat` enum + implement `render()` in `output.rs`.                        |
| Add interactive TUI mode  | Consider `ratatui` crate; spawn if `--tui` passed and TTY present (future milestone).    |
| Plugin system             | Plan: search `$XDG_CONFIG_HOME/unet/plugins/*.so` and load via `libloading`; define ABI. |

---

## 14  Rejected Design Choices

| Choice                             | Reason for Rejection                                                                        |
| ---------------------------------- | ------------------------------------------------------------------------------------------- |
| **Sub‑commands written in Python** | Multi‑runtime complexity; binary size; Windows packaging hurdles.                           |
| **gRPC instead of REST**           | Hard to debug via curl; JSON over HTTP is friendlier for operators and browser plugins.     |
| **Sync Reqwest client**            | Async gives better concurrency (CLI can parallel list/diff).                                |
| **Separate binaries per feature**  | Prefer one fat binary for UX; feature flags control compile‑time size if distributors care. |

---

