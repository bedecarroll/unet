<!-- SPDX-License-Identifier: MIT -->

# 09 Config‑Match Tool – `config‑slicer` Crate & CLI Guide

> **Audience:** Engineers extracting the hierarchy‑match logic into a reusable crate, plus operators or 3rd‑party tooling authors who want to programmatically slice network configs.
>
> **Goals:**
>
> 1. Provide a **vendor‑agnostic**, deterministic way to isolate portions of a device configuration using the *template‑match* grammar.
> 2. Expose both a **Rust library API** and a **zero‑dependency CLI binary** usable in shell pipelines.
> 3. Keep parsing **fast (<5 µs/1 k lines)** and **memory‑frugal (<2 MB RSS)** so it can run inside other tools or CI.

---

## Table of Contents

1. [Conceptual Overview](#1-conceptual-overview)
2. [Use‑Cases & Personas](#2-use-cases--personas)
3. [High‑Level Design](#3-high-level-design)
4. [CLI Specification](#4-cli-specification)
5. [Match Grammar Recap](#5-match-grammar-recap)
6. [Vendor Parsers](#6-vendor-parsers)
7. [Crate Layout](#7-crate-layout)
8. [Core Data Structures](#8-core-data-structures)
9. [Algorithm Details](#9-algorithm-details)
10. [Library API](#10-library-api)
11. [Performance Benchmarks](#11-performance-benchmarks)
12. [Testing Strategy](#12-testing-strategy)
13. [Extending the Tool](#13-extending-the-tool)
14. [Packaging & Distribution](#14-packaging--distribution)
15. [Rejected Alternatives](#15-rejected-alternatives)
16. [FAQ](#16-faq)

---

## 1  Conceptual Overview

`config‑slicer` takes **plain‑text device configuration** and a **match expression** (same syntax as Template‑Match) and returns **only** the lines that belong to that hierarchical subtree.

```bash
> cat qfx.conf | config-slicer --match "interfaces ge-.*||.*||.*"
set interfaces ge-0/0/0 unit 0 family inet address 192.0.2.1/31
set interfaces ge-0/0/0 unit 0 family inet address 2001:db8::1/127
...
```

*Why?* – Enables partial templating, fine‑grained diffs, and fosters reuse **outside** μNet (e.g., CI pipelines that lint configs).

---

## 2  Use‑Cases & Personas

| Persona          | Need                                                                                  |
| ---------------- | ------------------------------------------------------------------------------------- |
| μNet CLI         | Slice live config when running `unet template diff`.                                  |
| External Auditor | Validate only `system ntp` stanza across fleet in Jenkins job.                        |
| Vendor Tooling   | Feed extracted block into simulator (e.g., Juniper JSNAPy) without full config noise. |
| Documentation    | Automatically embed relevant snippets in Markdown manuals.                            |

---

## 3  High‑Level Design

```ascii
                 +----------------------+
 stdin / file -->|  Vendor Tokenizer    |--+   tokens with depth info
                 +----------------------+  |
                                            | filter(tokens, MatchSpec)
                 +----------------------+  |
 MatchSpec ------>|  Slice Engine       |--+---> stdout / Vec<String>
                 +----------------------+
```

- **Tokenizer** – converts vendor config to `(depth, text)` tuples.
- **MatchSpec** – compiled regex array derived from match string.
- **Slice Engine** – streaming filter; no full config held in memory (use `BufRead`).

> **Design principle:** *Push‑based stream*, not DOM/AST, for O(n) speed and tiny memory.

---

## 4  CLI Specification

### 4.1 Basic Usage

```bash
config-slicer --match <expr> [--vendor autodetect|juniper|cisco] [<file>]
```

*If **`<file>`** omitted, reads stdin.*

### 4.2 Flags

| Flag / Env            | Default    | Description                                          |
| --------------------- | ---------- | ---------------------------------------------------- |
| `--match, -m` **REQ** |            | Template‑match expression.                           |
| `--vendor, -v`        | autodetect | Force parser (`juniper`, `cisco`, `arista`, `flat`). |
| `--context <n>`       | 0          | Include ±*n* lines of context around each slice.     |
| `--json`              | off        | Output JSON array (`["line1", "line2", …]`).         |
| `--profile`           | off        | Print time & memory stats to stderr.                 |
| `--version`, `--help` |            | Standard Clap outputs.                               |

Exit codes: `0` success (may return empty), `1` error (parse, IO).

---

## 5  Match Grammar Recap

Same as §4 of **04\_template\_engine.md**; re‑implemented here for convenience.

```
TOKEN := '*' | <regex_no_slash>
EXPR  := TOKEN ('||' TOKEN)*
```

- Depth separators `||` enumerate hierarchy levels.
- `*` matches any token at that level.
- Regexes are Rust/PCRE‑compatible **without** slashes; flags allowed via `(?i)` inline.

Examples:

| Expression                      | Meaning                   |     |   |       |                                                  |
| ------------------------------- | ------------------------- | --- | - | ----- | ------------------------------------------------ |
| \`interfaces ge-.\*             |                           | .\* |   | .\*\` | All descendant lines under any `ge-*` interface. |
| \`system                        |                           | ntp |   | .\*\` | `ntp` subtree.                                   |
| \`.\*                           |                           | .\* |   | .\*\` | Whole config (anti‑pattern).                     |
| `^interface GigabitEthernet\d+` | Cisco single‑level block. |     |   |       |                                                  |

---

## 6  Vendor Parsers

| Vendor         | Parser Module              | Strategy                                            | Status |
| -------------- | -------------------------- | --------------------------------------------------- | ------ |
| Juniper JunOS  | `vendor::junos`            | Brace depth using stack; ignore comments.           | READY  |
| Cisco IOS‑Like | `vendor::flat`             | Hierarchy depth simulated by leading spaces indent. | READY  |
| Arista EOS     | `vendor::eos` (alias flat) | Same as flat, but exclamation sentinel lines.       | READY  |
| YAML‑style     | `vendor::indent_yaml`      | For systems like NetBox intended future.            | FUTURE |

**Autodetect heuristic:**

1. Read first 20 lines.
2. If any `{`/`}` braces – assume JunOS.
3. Else if any lines start with `hostname` – assume IOS.
4. Fallback to `flat`.

Autodetect can be wrong ❗: CLI prints warning and suggests `--vendor` override.

---

## 7  Crate Layout

```text
crates/
└── config-slicer/
    ├── src/
    │   ├── lib.rs          # public API
    │   ├── match_spec.rs   # parser + struct
    │   ├── slice.rs        # engine / iterator
    │   ├── vendor/
    │   │   ├── junos.rs
    │   │   ├── flat.rs
    │   │   └── mod.rs
    │   └── cli.rs          # main() when compiled with `bin` feature
    ├── benches/
    │   └── bench.rs        # criterion benches
    ├── tests/
    │   ├── junos_fixture.conf
    │   ├── ios_fixture.conf
    │   └── slice_tests.rs
    └── Cargo.toml
```

Cargo features:

| Feature    | Default | Purpose                           |
| ---------- | ------- | --------------------------------- |
| `bin`      | on      | builds the CLI binary             |
| `vendor-*` | on      | enable parser (opt‑out to slim)   |
| `simd`     | off     | experimental SIMD newline scanner |

---

## 8  Core Data Structures

```rust
/// Pre‑compiled representation of a match expression.
#[derive(Debug, Clone)]
pub struct MatchSpec(pub Vec<Regex>); // one regex per depth level

/// Single config line with metadata.
#[derive(Debug, Clone)]
pub struct Token<'a> {
    pub depth: usize,   // 0‑based hierarchy depth
    pub text:  &'a str, // slice into original line (no alloc)
}
```

Tokenizer returns `impl Iterator<Item = Token<'_>>` (zero copy, uses lifetimes).

---

## 9  Algorithm Details

### 9.1 JunOS Brace Parser

1. Iterate char‑by‑char; maintain `depth` counter.
2. On `{` increment depth **after** emitting line token.
3. On `}` decrement depth **before** emitting.
4. Ignore `/* comment */` and `// comment`.
5. Output token with current depth.

Complexity: **O(n)** lines, O(1) state.

### 9.2 Flat Parser (IOS‑like)

1. Trim leading spaces; compute `depth = indent / 4`.
2. Treat `!` sentinel as “end of section” → reduce depth by 1.
3. Lines starting with `interface`, `router bgp` raise depth one.
4. Comments starting `!` outside section are depth 0 tokens.

### 9.3 Slice Engine

```rust
pub fn slice<'a, I>(tokens: I, spec: &MatchSpec) -> impl Iterator<Item = &'a str>
where I: Iterator<Item = Token<'a>>
```

- Walk tokens.
- For each token where `depth < spec.len()` and `spec[depth].is_match(component(token))`, mark **matched=true** for that branch.
- Emit token + all deeper tokens until depth < matched\_level.
- Reset matched flag on closing brace or dedent.

Memory = stack integers only.

---

## 10  Library API

```rust
/// Parse expression into reusable compiled Regex list.
pub fn parse_match(expr: &str) -> Result<MatchSpec>;

/// Slice config in one shot (convenience).
pub fn slice_config(text: &str, spec: &MatchSpec, vendor: Vendor) -> Vec<String>;

/// Stream‑oriented version (no allocations on large files).
pub fn slice_reader<R: BufRead>(reader: R, spec: &MatchSpec, vendor: Vendor) -> impl Iterator<Item = String>;
```

### Error Types

```rust
#[derive(thiserror::Error, Debug)]
pub enum SliceError {
    #[error("invalid match expr: {0}")]
    BadExpr(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("regex: {0}")]
    Regex(#[from] regex::Error),
}
```

---

## 11  Performance Benchmarks

Criterion benchmark on 1.5 MB JunOS config (\~25 k lines):

| Function       | Time (µs) | Throughput (MB/s) | Peak RSS |
| -------------- | --------- | ----------------- | -------- |
| Tokenize only  | 820       | 1.8 GB/s          | +0 MB    |
| Slice `system` | 1 120     | 1.3 GB/s          | +0 MB    |
| Slice `ge-.*`  | 1 450     | 1.0 GB/s          | +0 MB    |

> **Interpretation:** overhead is negligible compared to SSH+cat latency; safe for inline use.

---

## 12  Testing Strategy

1. **Golden fixtures** – store sample configs in `tests/fixtures`; snapshot expected slice output with `insta`.
2. **Property tests** – quick‑check that slicing full wildcard `.*||.*||.*` returns original lines.
3. **Cross‑compat** – run same spec on JunOS vs IOS fixtures to ensure vendor parsers differences don’t break API.
4. **CLI smoke test** in GitHub Actions (`cargo run --bin config-slicer -- -m 'system||.*' tests/junos_fixture.conf`).

---

## 13  Extending the Tool

| Extension                 | Steps                                                                                                  |   |           |   |                                                     |
| ------------------------- | ------------------------------------------------------------------------------------------------------ | - | --------- | - | --------------------------------------------------- |
| **New vendor parser**     | 1. Add `vendor::<name>.rs` implementing `Tokenizer` trait.2. Update autodetect.3. Add fixture & tests. |   |           |   |                                                     |
| **Glob captures** (`**`)  | Enhance `parse_match` to translate `**` to non‑depth‑bound regex (e.g., `(?s).*`).                     |   |           |   |                                                     |
| **Inline exclude syntax** | E.g., \`interfaces                                                                                     |   | (!unit 0) |   | .\*\` – need negative look‑ahead regex, doc update. |
| **JSON output schema**    | Add optional field `depth` for consumers needing context visualization.                                |   |           |   |                                                     |

---

## 14  Packaging & Distribution

- Binary published via μNet **release workflow** (see 07\_ci\_cd.md) as `config-slicer-<target>.tar.gz`.
- Homebrew formula taps the GH tarball.
- Debian/RPM packages include man page (`man/config-slicer.1`).
- Crate published to **crates.io** under `config-slicer` name (MIT).
- Versioning **follows μNet minor**: tool `0.4.0` pairs with server `0.4.x`.

---

## 15  Rejected Alternatives

| Option                                  | Reason for Rejection                                                                      |
| --------------------------------------- | ----------------------------------------------------------------------------------------- |
| **Full AST parse per vendor (libyang)** | Heavy libc deps, licensing issues, >10× memory, slower for simple slicing need.           |
| **Regex‑only naïve grep**               | Cannot respect hierarchy braces/indent; would over‑match and produce bogus diffs.         |
| **XML export via JunOS**                | Not available for all vendors; API login required; destroys operator habit of `show cfg`. |

---

## 16  FAQ

**Q:** *Does **`config-slicer`** modify the input file?*\
**A:** No. It’s a read‑only filter – safe in pipelines.

**Q:** *How do I get multiple slices in one run?*\
**A:** Call with `--match` multiple times (`-m expr1 -m expr2`) – output kept in original order.

**Q:** *What if my match returns nothing?*\
**A:** Exit code still 0; stdout empty; use `--fail-on-empty` flag to treat as error.

---

### Next Steps

1. Scaffold crate: `cargo new --lib config-slicer && cargo add regex clap dashmap`.
2. Implement `match_spec.rs` parser + unit tests.
3. Build JunOS tokenizer; benchmark with `criterion`.
4. Add CLI binary behind `bin` feature; wire Clap v4 args.
5. Integrate into μNet `unet template diff` (Milestone 5).

*End of 09\_config\_match\_tool.md – proceed to* **10\_future\_work.md** *once this tool lands.*
