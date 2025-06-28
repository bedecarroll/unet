# 04 Template Engine – MiniJinja Rendering, Match Headers & Diff Workflow

> **Audience:** Engineers wiring MiniJinja into `unet-core`, and operators writing templates.\
> **Goals:**
>
> - Render vendor‑specific configuration snippets from Node JSON.
> - Allow **partial templating** via a *template‑match* header, enabling safe progressive adoption.
> - Offer a **diff tool** that compares rendered output with live device config using the stand‑alone `config-slicer` crate.

---

## Table of Contents

1. [Conceptual Overview](#1-conceptual-overview)
2. [MiniJinja Fundamentals](#2-minijinja-fundamentals)
3. [Template Directory Layout](#3-template-directory-layout)
4. [Template‑Match Header Spec](#4-template-match-header-spec)
5. [Rendering Pipeline](#5-rendering-pipeline)
6. [Diff Workflow](#6-diff-workflow)
7. [Config‑Slicer Integration](#7-config-slicer-integration)
8. [CLI Commands & Examples](#8-cli-commands--examples)
9. [Rust API (Library Side)](#9-rust-api-library-side)
10. [Template Environment Loader](#10-template-environment-loader)
11. [Custom Filters & Globals](#11-custom-filters--globals)
12. [Error Handling & Diagnostics](#12-error-handling--diagnostics)
13. [Testing Strategy](#13-testing-strategy)
14. [Extending the System](#14-extending-the-system)
15. [Performance Notes](#15-performance-notes)
16. [Rejected Alternatives](#16-rejected-alternatives)
17. [FAQ for Template Authors](#17-faq-for-template-authors)

---

## 1  Conceptual Overview

- Templates are **plain text** files using **Jinja2 syntax** (e.g. `{{ variable }}`, `{% for ... %}`) rendered by **MiniJinja** at runtime.
- A template **owns** a *slice* of a device configuration, declared via a **template‑match** header comment (`{# template-match: … #}`).
- Multiple templates may apply to a single Node (selected by the policy engine’s `APPLY` action). They are rendered **independently** and concatenated in CLI output (ordering follows policy evaluation order).
- The **config‑slicer** CLI/library uses the same match header to cut the equivalent slice from the *live* config, enabling an atomic diff.

> **Key benefit:** Operators can roll out templating gradually – one stanza at a time – without rewriting entire config files.

---

## 2  MiniJinja Fundamentals

MiniJinja is a *pure‑Rust* re‑implementation of Jinja2, supporting nearly all syntax operators plus runtime template includes.\
At runtime we create a `minijinja::Environment` and load templates from strings (git‑synced files).\
**Features supported out‑of‑the‑box:**

- `{% include "foo.jinja" %}` – file reuse.
- `{% import "macros.jinja" as macros %}` – macro libraries.
- Filters (`| upper`, `| default('x')`).
- Global functions (we add network helpers later).

### 2.1 Why MiniJinja vs Askama / Tera

| Option        | Pros                                   | Cons                                        | Verdict      |
| ------------- | -------------------------------------- | ------------------------------------------- | ------------ |
| **MiniJinja** | Runtime load, Jinja2 parity, sandboxed | Slightly slower than compile‑time templates | **✔ chosen** |
| Askama        | Compile‑time speed, Rust‑typed context | Cannot load user‑provided templates         | ✖            |
| Tera          | Similar to Jinja but not identical     | Larger dep tree, slower compile times       | ✖            |

---

## 3  Template Directory Layout

Templates live in a **separate Git repository** (path configured in `config.toml`).\
Operators organise arbitrarily, but we *recommend* vendor‑then‑purpose hierarchy:

```
templates/
├── juniper/
│   ├── _macros.jinja           # helper macros shared by all Juniper templates
│   ├── qfx/
│   │   ├── system.jinja
│   │   ├── interfaces.jinja
│   │   └── routing.jinja
│   └── ex/
│       └── system.jinja
└── cisco/
    └── interface.jinja
```

> **Naming convention:** snake\_case or kebab‑case; `.jinja` extension mandatory.

---

## 4  Template‑Match Header Spec

A *template‑match* tells μNet and `config‑slicer` which lines in the vendor config the template owns.

```jinja
{# template-match: interfaces ge-.*||.*||.* #}
set interfaces ge-0/0/0 unit 0 family inet address {{ iface_ip }}
```

### 4.1 Grammar

```
<root>          ::= <token> ( '||' <token> )*
<token>         ::= <regex> | '*'
<regex>         ::= Regular expression (PCRE) **without** delimiter slashes
```

- `*` acts as a catch‑all “any token at this level”.
- Regex anchors (`^`, `$`) **not** required; implicit `^token$`.
- Case sensitive unless the vendor config is naturally case‑insensitive (e.g. Cisco IOS).
- Depth is vendor‑specific – for JunOS hierarchical configs depth = nesting braces.

### 4.2 Examples

| Match String                    | Applies to…                                       |     |   |       |                                                    |
| ------------------------------- | ------------------------------------------------- | --- | - | ----- | -------------------------------------------------- |
| \`system                        |                                                   | ntp |   | .\*\` | JunOS `system { ntp { … } }` block (all children)  |
| \`interfaces ge-.\*             |                                                   | .\* |   | .\*\` | All `ge-*` interfaces and their sub‑stanzas        |
| \`.\*                           |                                                   | .\* |   | .\*\` | **Anti‑pattern** – captures *entire* configuration |
| `^interface GigabitEthernet\d+` | Cisco `interface GigabitEthernetX` section (flat) |     |   |       |                                                    |

---

## 5  Rendering Pipeline

### 5.1 Data Context

The **policy‑evaluated** Node JSON is serialised to a `serde_json::Value` and passed as the sole variable `node` into MiniJinja.

```jinja
hostname {{ node.node_name }}.{{ node.domain_name or 'local' }}
```

Operators may define `set` statements within templates for readability:

```jinja
{% set is_lab = node.location.name == 'LAB' %}
{% if is_lab %}
set system services telnet
{% endif %}
```

### 5.2 Render Function Signature

```rust
pub fn render_templates(node: &NodeWithPolicy, env: &Environment) -> RenderedConfig {
    let mut output = String::new();
    for tpl_name in &node.templates {          // order from policy engine
        let tpl = env.get_template(tpl_name)?;
        output.push_str(&tpl.render(node)?);
        output.push('\n');
    }
    Ok(RenderedConfig { text: output })
}
```

*`NodeWithPolicy`**= Node + **`custom_data`** mutations + template list.*

---

## 6  Diff Workflow

```mermaid
graph TD;
A[Rendered candidate<br>(MiniJinja)] -- match header --> B[Slice spec];
C[Live device config<br>(file or SSH)] -- config‑slicer -- B --> D[Relevant slice];
A --> E(diff similar crate) --> F[Colored diff output];
D --> E;
```

Steps executed by `unet template diff` CLI:

1. Fetch Node+Policy JSON from server (or local).
2. Render candidate snippet(s).
3. For each template:
   1. Parse match header → `MatchSpec`.
   2. Call `config-slicer --match <spec>` on live config (stdin/file).
   3. Run line diff (`similar::TextDiff`).
4. Aggregate diffs; exit code **2** when differences found (Unix diff convention).

### 6.1 Live Config Acquisition

| Scenario        | CLI flag                   | Behaviour                               |
| --------------- | -------------------------- | --------------------------------------- |
| Local text file | `-o live.txt`              | Reads from file path                    |
| Standard input  | `-o -`                     | Reads from `stdin`                      |
| SSH fetch       | `--ssh user@host show ...` | **TODO** Milestone 5 – plugin execution |

---

## 7  Config‑Slicer Integration

`config-slicer` is consumed **two ways**:

1. **CLI path** – `Command::new("config-slicer") …` to avoid re‑parsing configs in Rust memory twice (keep binary dependency sharp).
2. **Library path** – `config_slicer::slice(text, &MatchSpec)` used in tests & when CLI flag `--native` is passed (no external binary).

> **Guideline:** For production CLI we shell‑out; for unit tests we link as a lib for speed.

---

## 8  CLI Commands & Examples

### 8.1 Render

```bash
unet template render core‑1 \
    --templates juniper/qfx/system.jinja juniper/qfx/interfaces.jinja
```

- `--templates` optional – default is all assigned.
- `-o candidate.conf` writes file.

### 8.2 Diff

```bash
# diff the "system" template only
unet template diff core‑1 -t system.jinja -o live.conf
```

Exit codes:

| Code | Meaning              |
| ---- | -------------------- |
| 0    | No differences       |
| 2    | Differences detected |
| 1    | Error                |

### 8.3 Canary Push

```bash
unet template push -f ~/scratch/lab_banner.jinja --expire 2h
```

Server stores file in `canary/` memory map; override precedence until expiry.

---

## 9  Rust API (Library Side)

```rust
pub struct TemplateEngine {
    env:   minijinja::Environment<'static>,
    cache: dashmap::DashMap<String, Arc<ParsedMatchSpec>>, // header cache
}

impl TemplateEngine {
    pub fn new(repo_dir: &Path) -> Result<Self> { ... }
    pub fn reload(&self) -> Result<()> { ... }           // called after git sync
    pub fn render(&self, node: &NodeWithPolicy) -> Result<Vec<RenderedSnippet>> { ... }
}

pub struct RenderedSnippet {
    pub template_name: String,
    pub match_spec: Option<MatchSpec>,
    pub text: String,
}
```

*`MatchSpec`**imported from **`config_slicer::spec::MatchSpec`**.*

---

## 10  Template Environment Loader

### 10.1 Implementation Sketch

```rust
fn build_env(repo_dir: &Path) -> Result<Environment<'static>> {
    let mut env = Environment::new();
    for path in walkdir::WalkDir::new(repo_dir).into_iter().filter_map(Result::ok) {
        if path.path().extension() == Some("jinja".as_ref()) {
            let name = path.path().strip_prefix(repo_dir)?.to_string_lossy();
            let src  = std::fs::read_to_string(path.path())?;
            env.add_template(&name, src)?;           // compiles & caches bytecode
        }
    }
    Ok(env)
}
```

*We store the env inside **`Arc<Environment>`** for cheap cloning across threads.*

### 10.2 Hot Reload Strategy

1. Git sync writes new files into `repo_dir` atomically (`git fetch && git reset --hard`).
2. Call `TemplateEngine::reload()` which rebuilds the env in a background thread, then swaps `Arc` pointer (lock‑free RCU pattern).
3. Old renders finish using old env; new requests use new env.

---

## 11  Custom Filters & Globals

### 11.1 Filter: `cidr_mask`

```rust
fn cidr_mask<'a>(value: i64, _args: &mut mini jinja::value::Rest<'_>) -> MiniResult<Value> {
    let mask = ipnet::Ipv4Net::new(Ipv4Addr::UNSPECIFIED, value as u8)
        .map_err(|_| MiniError::new("invalid mask"))?
        .mask()
        .into();
    Ok(mask.into())
}

env.add_filter("cidr_mask", cidr_mask);
```

Usage:

```jinja
set interfaces ge-0/0/0 unit 0 family inet address {{ node.custom_data.ip }}/{{ 30 | cidr_mask }}
```

> **Rule:** Keep filter library *tiny*; prefer pure Jinja expressions when possible.

---

## 12  Error Handling & Diagnostics

| Layer     | Failure Example                 | Handling                                 |
| --------- | ------------------------------- | ---------------------------------------- |
| Loader    | Template syntax error           | Skip template, log `error!`, return 206  |
| Rendering | Undefined variable (`node.foo`) | MiniJinja error bubbled; CLI shows line  |
| MatchSpec | Invalid regex in header         | Fail parse at Git sync (policy abort)    |
| Diff step | Slicer returns non‑zero exit    | CLI warning, diff skipped for that slice |

CLI `-v` prints per‑template render time; `-vv` dumps the AST tokens from config‑slicer for debugging.

---

## 13  Testing Strategy

1. **Golden tests**: in `tests/template_render.rs` load sample Node JSON, render `system.jinja`, snapshot output with `insta`.
2. **Header parsing**: regex compile check in `tests/template_header.rs`.
3. **End‑to‑end diff**: run `cargo test --features native_slicer` which calls slicer lib directly.

CI runs all with `cargo test --workspace`.

---

## 14  Extending the System

| Extension           | How‑to                                                                                      |
| ------------------- | ------------------------------------------------------------------------------------------- |
| **Vendor parser**   | Implement `config_slicer::vendor::junos::TokenizerV2` etc. for smarter brace handling.      |
| **Binary output**   | MiniJinja supports non‑UTF8 via `Value::Binary`; could render compiled configs (e.g., F5).  |
| **Template groups** | Allow `{% extends "base_router.jinja" %}` – just works via MiniJinja’s inheritance feature. |

---

## 15  Performance Notes

- MiniJinja compiles templates to bytecode on load – runtime render is fast (<200 µs for 300‑line template).
- Environment rebuild after git sync takes O(#files) but is async; does not block API.
- Regex header parse cached in `DashMap` keyed by template path.

---

## 16  Rejected Alternatives

| Option                                      | Reason for Rejection                                             |
| ------------------------------------------- | ---------------------------------------------------------------- |
| **String replace (no templating)**          | Unmaintainable for multi‑vendor logic, no loops/conditionals.    |
| **Mustache**                                | Lack of control structures, no filters.                          |
| **Pushing full configs via Ansible/Jinja2** | Requires Python runtime, operators need agent, heavy diff logic. |
| **Vendor AST diff (e.g., junos EZ‑NC)**     | Large parsing libs, brittle to version changes, longer delivery. |

---

## 17  FAQ for Template Authors

**Q:** *Can I include other templates?*\
**A:** Yes, `{% include "_macros.jinja" %}` or `{% import … %}` works.

**Q:** *How do I access custom\_data?*\
**A:** `node.custom_data.my_key` – keys with dashes use `['my-key']` syntax.

**Q:** *What if two templates try to configure same interface stanza?*\
**A:** Avoid duplication; policy should ensure unique match scopes. Duplicate lines will appear in diff and fail CI.

**Q:** *Can template‑match regex be vendor specific?*\
**A:** Yes. For flat configs (Cisco) omit `||` and write a plain regex.

---

### Next Steps

- Implement `template::engine` module (Milestone 4).
- Add `_macros.jinja` with common helpers for Juniper.
- Provide sample live configs in `tests/fixtures` for diff testing.

*Proceed to *[*05\_cli\_tool.md*](05_cli_tool.md)*.*
