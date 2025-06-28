# 03 Policy Engine – Design, DSL & Implementation Guide

> **Audience:** Engineers building the rule parser/evaluator, and operators authoring policies.
>
> **Goals:**
>
> 1. Enable **declarative assertions** & **mutations** on device data.
> 2. Run identically in **CLI local mode** *and* **server** background tasks.
> 3. Remain approachable for network engineers with minimal coding experience.

---

## Table of Contents

1. [Conceptual Overview](#1-conceptual-overview)
2. [Policy File Layout](#2-policy-file-layout)
3. [DSL Grammar Reference](#3-dsl-grammar-reference)
4. [Parser Architecture (Pest)](#4-parser-architecture-pest)
5. [AST & Data Structures](#5-ast--data-structures)
6. [Evaluation Engine](#6-evaluation-engine)
7. [Runtime Integration](#7-runtime-integration)
8. [Error Handling & Diagnostics](#8-error-handling--diagnostics)
9. [Testing Strategy](#9-testing-strategy)
10. [Extending the DSL](#10-extending-the-dsl)
11. [Performance Considerations](#11-performance-considerations)
12. [Rejected Alternatives](#12-rejected-alternatives)
13. [FAQ for Operators](#13-faq-for-operators)

---

## 1  Conceptual Overview

A **policy** is a *rule* evaluated against a **target object** (Node, Link or Location). If the rule’s **condition** is `true`, its **action** executes.

```text
WHEN <condition> THEN <action>
```

- **Conditions** are pure boolean expressions referencing object fields.
- **Actions** are side‑effect expressions that either **assert** compliance or **mutate** fields (including `custom_data`) or **assign** templates.

> **Think of a policy file** as a set of *lint rules* for your inventory.

### 1.1 Why Not Just SQL?

- SQL lacks easy access to JSON sub‑paths (`custom_data.maintenance.window`).
- DSL decouples rule authorship from DB schema and supports **client‑side offline eval**.

---

## 2  Policy File Layout

*Files live in a dedicated Git repo (see **`config.toml`**).*

```text
policies/
├── 10_version.rules      # naming convention: 2‑digit order + slug
├── 20_templates.rules
└── README.md
```

Rules are evaluated **in lexical order** (00 → 99). Later rules may *override* earlier actions; this is intentional for layering (e.g. site‑specific overrides after global rules).

### 2.1 File Header (optional metadata)

Each file can start with a **YAML front‑matter** block used only for documentation & future tooling:

```yaml
---
id: version-compliance
owner: netops@corp
summary: >
  Enforce vendor‑approved software versions on production devices.
---
WHEN node.vendor == "juniper" ...
```

Front‑matter is ignored by the parser and stripped before tokenisation.

---

## 3  DSL Grammar Reference

### 3.1 Lexical Tokens

| Token          | Example                                                       |
| -------------- | ------------------------------------------------------------- |
| **Identifier** | `node.vendor`                                                 |
| **String**     | `'QFX'` or `"QFX"`                                            |
| **Regex**      | `/^qfx\d+/i`                                                  |
| **Number**     | `42`, `3.14`                                                  |
| **Boolean**    | `true`, `false`                                               |
| **Operators**  | `==`, `!=`, `=~` (regex contains), `!~`, `>`, `<`, `>=`, `<=` |
| **Logical**    | `and`, `or`, `not`                                            |

### 3.2 BNF (Simplified)

```bnf
rule        ::= "WHEN" expr "THEN" action
expr        ::= term ( ("and" | "or") term )*
term        ::= factor | "not" factor
factor      ::= comparison | '(' expr ')'
comparison  ::= ident op value
op          ::= '==' | '!=' | '=~' | '!~' | '>' | '<' | '>=' | '<='
value       ::= STRING | NUMBER | BOOLEAN | REGEX
action      ::= assert_action | set_action | apply_action | block_action
assert_action ::= 'ASSERT' ident 'IS' value
set_action     ::= 'SET' json_path 'TO' value
apply_action   ::= 'APPLY' STRING            # template path
block_action   ::= '{' (action ';')+ '}'     # future: multiple actions
```

> *Case‑insensitive keywords* (`WHEN`, `THEN`, `ASSERT`, …) for operator friendliness.

### 3.3 Examples

```text
WHEN node.vendor == 'juniper' and node.model =~ /^(qfx|ex)/i
THEN ASSERT software_version IS '17.2R3'
```

```text
WHEN location.name == 'LAB'              # disable strict versioning in lab
THEN SET custom_data.ignore_compliance TO true
```

```text
WHEN node.device_role == 'core'
THEN APPLY 'templates/juniper/core_system.jinja'
```

---

## 4  Parser Architecture (Pest)

### 4.1 File: `unet-core/src/policy/grammar.pest`

```pest
WHITESPACE       = _{ " " | "\t" | "\r" | "\n" }
comment          = _{ "#" ~ (!"\n" ~ ANY)* ~ "\n" }
ident            = @{ (ASCII_ALPHANUMERIC | "_" | "." )+ }
string_literal   = @{ "'" ~ ("\\'" | !"'" ~ ANY)* ~ "'" |
                    "\"" ~ ("\\\"" | !"\"" ~ ANY)* ~ "\"" }
regex_literal    = @{ '/' ~ (!'/' ~ ANY)* ~ '/' ~ ("i" | "m" | "s")* }
number           = @{ ASCII_DIGIT+ ~ ("." ~ ASCII_DIGIT+)? }
boolean          = { "true" | "false" }
value            = _{ string_literal | regex_literal | number | boolean }
op               = { "==" | "!=" | "=~" | "!~" | ">=" | "<=" | ">" | "<" }
comparison       = { ident ~ WHITESPACE* ~ op ~ WHITESPACE* ~ value }
factor           = { comparison | "(" ~ expr ~ ")" | "not" ~ WHITESPACE* ~ comparison }
term             = { factor ~ (WHITESPACE* ~ ("and" | "or") ~ WHITESPACE* ~ factor)* }
expr             = _{ term }
action_assert    = { "ASSERT" ~ WHITESPACE+ ~ ident ~ WHITESPACE+ ~ "IS" ~ WHITESPACE+ ~ value }
action_set       = { "SET" ~ WHITESPACE+ ~ ident ~ WHITESPACE+ ~ "TO" ~ WHITESPACE+ ~ value }
action_apply     = { "APPLY" ~ WHITESPACE+ ~ string_literal }
action           = _{ action_assert | action_set | action_apply }
rule             = { "WHEN" ~ WHITESPACE+ ~ expr ~ WHITESPACE+ ~ "THEN" ~ WHITESPACE+ ~ action }
file             = { SOI ~ (comment | rule)* ~ EOI }
```

> **Tip:** Keep grammar small; Pest’s error messages map line/col to offending rule for nice CLI output.

### 4.2 Compile Grammar & Generate Parser

```rust
// build.rs
fn main() {
    println!("cargo:rerun-if-changed=src/policy/grammar.pest");
}
```

`cargo build` auto‑generates `policy::parser` module.

---

## 5  AST & Data Structures

```rust
#[derive(Debug, Clone)]
pub enum Value {
    Str(String),
    Number(f64),
    Bool(bool),
    Regex(regex::Regex),
}

#[derive(Debug, Clone)]
pub enum CmpOp { Eq, Ne, Contains, NotContains, Gt, Ge, Lt, Le }

#[derive(Debug, Clone)]
pub struct Comparison { pub left: String, pub op: CmpOp, pub right: Value }

#[derive(Debug, Clone)]
pub enum Expr {
    Comp(Comparison),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Action {
    Assert { field: String, expected: Value },
    Set    { json_path: String, value: Value },
    Apply  { template: String },
}

#[derive(Debug, Clone)]
pub struct PolicyRule { pub condition: Expr, pub action: Action, pub line_no: usize }
```

*Why boxes?* Avoid deep recursive enum size explosion.

---

## 6  Evaluation Engine

### 6.1 Evaluation Context

```rust
pub struct EvalCtx<'a> {
    pub node:   Option<&'a NodeModel>,
    pub link:   Option<&'a LinkModel>,
    pub loc:    Option<&'a LocationModel>,
}
```

*Only one variant is non‑None per call.*

### 6.2 Algorithm

1. **walk objects** – iterate over all Nodes then Links then Locations.
2. For each policy rule:\
   a. `condition.eval(&ctx)` → bool\
   b. `if true` → run action:
   - **Assert** → if `ctx.get(field) != expected`, record `PolicyViolation`.
   - **Set** → mutate copy of object JSON (do **not** write DB yet).
   - **Apply** → push template string onto `ctx.assigned_templates` Vec.
3. Emit `PolicyResult` struct:

```rust
pub struct PolicyResult {
    pub object_id: Uuid,
    pub violations: Vec<Violation>,
    pub custom_data_delta: Option<JsonValue>,
    pub templates: Vec<String>,
}
```

### 6.3 Field Access Helper

```rust
impl EvalCtx<'_> {
    fn get(&self, ident: &str) -> Option<Value> {
        match ident.split_once('.') {
            Some((prefix, rest)) if prefix == "node" => {
                self.node.and_then(|n| fetch_node_value(n, rest))
            }
            Some(("location", rest)) => { /* … */ }
            _ => None,
        }
    }
}
```

*`fetch_node_value`**uses **`serde_json::value::to_value`** for dynamic attrs.*

### 6.4 Operator Semantics

| Op                   | Behaviour                                       |
| -------------------- | ----------------------------------------------- |
| `==`                 | strict equality (`Value::Number` uses EPS 1e‑9) |
| `=~`                 | if `right` Regex → `regex.is_match(left_str)`   |
| `>`, `>=`, `<`, `<=` | numeric compare (string to f64 fails rule)      |

If type mismatch, evaluation returns **false** (rule doesn’t match).

---

## 7  Runtime Integration

### 7.1 Server Flow

*Task order (in **`tasks.rs`**):*

1. `git_sync` pulls files, reloads `Vec<PolicyRule>` (with `Arc`).
2. `snmp_poll` updates `node_status` (derived).
3. `policy_eval` iterates DB objects – every rule eval.
4. Cache result in `RwLock<HashMap<Uuid, PolicyResult>>` for fast API.
5. API `GET /nodes?eval=true` merges `Node` + `PolicyResult` in JSON.

### 7.2 CLI Local Mode

```bash
unet policy test -n core‑1 -f policies/override.rules --json
```

- Loads node JSON via DataStore or `--json-file`.
- Parses rules from `-f` (overrides).
- Prints JSON diff or table summary.

---

## 8  Error Handling & Diagnostics

| Error Class | Example Cause                            | HTTP Status / CLI Exit | Message Pattern              |
| ----------- | ---------------------------------------- | ---------------------- | ---------------------------- |
| **Parse**   | `token 'THEN' found where 'IS' expected` | 400 / exit 65          | `PolicyParse( line:col )`    |
| **Eval**    | Unknown field `node.firmware`            | 422 / exit 78          | `PolicyEval(UndefinedField)` |
| **Action**  | Assert mismatch                          | 200 + JSON             | Report under `violations[]`  |

CLI flag `--verbose` prints the rule text before evaluation and timings per rule.

---

## 9  Testing Strategy

1. **Unit tests** – `tests/parser.rs` round‑trip (source → AST → string).
2. **Golden tests** – `tests/fixtures/*.rules` run against sample node JSON; output `violations.json` captured via `insta` snapshots.
3. **Property tests** (future) – quick‑check that `not (A and B)` ↔ `!A or !B` equivalence.

Run in CI (`cargo test --workspace`).

---

## 10  Extending the DSL

| Extension                       | Steps                                                                                              |
| ------------------------------- | -------------------------------------------------------------------------------------------------- |
| **New Operator** (`startsWith`) | 1. Add enum variant `CmpOp::StartsWith`.2. Update grammar rule `op`.3. Extend evaluator match arm. |
| **Composite Actions**           | Enable `block_action` in grammar; parse `{ … }` into `Vec<Action>`.                                |
| **Math Functions**              | Add `FunctionCall` node in AST; register in evaluator (e.g., `len(node.custom_data.tags)` )        |

*Maintain ****semver****: bump minor when adding non‑breaking grammar, major on breaking.*

---

## 11  Performance Considerations

| Dimension | Baseline (10k nodes)                         | Bottleneck            | Mitigation                                        |
| --------- | -------------------------------------------- | --------------------- | ------------------------------------------------- |
| Parsing   | On git sync only                             | O(N rules) PEST parse | <5 ms per 100 rules – negligible                  |
| Eval      | Rules × Objects (100 rules × 10k) = 1 M eval | Regex matching        | Pre‑compile `Value::Regex` + use `rayon` (future) |
| Memory    | AST + Results                                | JSON clones           | Use `Arc<str>` & smallvec where helpful           |

---

## 12  Rejected Alternatives

| Option                | Why Not                                                                         |
| --------------------- | ------------------------------------------------------------------------------- |
| **Rego/OPA**          | 40 MB binary, JSON‑only input, steep syntax; local CLI embedding painful.       |
| **Lua scripts**       | Need sandboxing, harder to statically analyse, performance jitter.              |
| **Nom parser**        | Great performance but verbose for juniors; Pest’s grammar files double as docs. |
| **Inline YAML rules** | Indentation errors cause silent mis‑parses; less expressive for regex.          |

---

## 13  FAQ for Operators

**Q:** *How do I disable a rule temporarily?*\
**A:** Prefix each line with `#` or rename the file to `99_disabled.rules`.

**Q:** *Can a rule target both Node and Link?*\
**A:** Write two separate rules with different identifiers (`node.` vs `link.` prefixes).

**Q:** *What happens if two rules **`SET`** the same key?*\
**A:** Last rule wins (lexical order). Use file numbering to control precedence.

**Q:** *Does **`ASSERT`** stop other actions?*\
**A:** No. `ASSERT` records violation but evaluation continues so you can gather all issues in one pass.

---

### Next Steps

1. Implement grammar & AST (`policy/grammar.pest`, `ast.rs`).
2. Write unit tests in `tests/policy_parser.rs`.
3. Hook evaluator into server’s `policy_eval` task (Milestone 3).
4. Document two real rules in `policies/10_version.rules` as examples.

*Proceed to *[*04\_template\_engine.md*](04_template_engine.md)*.*
