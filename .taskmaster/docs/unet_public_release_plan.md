# μNet 0.1.0 Public Release Plan  
_The repository already exists and has passing CI; ignore any tasks related to project
scaffolding, IDE setup, or “initialise repo”._

# μNet Open-Source Release — Master TODO

> **Structure:** Epics → Stories → Tasks (GitHub-ready check-boxes).  
> **Priority legend:** **P0** = critical for first public tag, **P1** = nice-to-have, **P2** = future.

---

## Epic A — Data & Persistence

### A-1 • Multi-backend datastore
- [ ] **P0** Finalise **SQLite** implementation (SeaORM migrations)  
      _Acceptance:_ `cargo test -p unet-core datastore` passes with SQLite file; all CRUD ops succeed.
- [ ] **P1** Implement **CSV demo backend** behind `--features demo-csv`  
      _Acceptance:_ `unet-cli ingest csv examples/*.csv` populates DB.
- [ ] **P1** Scaffold **Postgres** support behind `postgres` feature flag  
      _Acceptance:_ `DATABASE_URL=postgres://… cargo test` is green; migrations run.

### A-2 • Lifecycle columns
- [ ] **P0** Add `lifecycle` enum + timestamps to **node/link/location** models  
      _Acceptance:_ new rows default to `plan`; state transitions validated.

### A-3 • Custom / virtual fields
- [ ] **P0** Add `custom_data` (JSON) column & round-trip via API/CLI  
      _Acceptance:_ Policies can reference arbitrary keys.

### A-4 • Desired ↔ Derived
- [ ] **P0** SNMP collector writes **derived** rows & delta view  
      _Acceptance:_ mismatches surfaced via `/api/diffs`.

---

## Epic B — Security & Auth

- [ ] **B-1 (P0)** Require explicit `--insecure` to run with _no_ auth.  
- [ ] **B-2 (P0)** Username/password table (BCrypt hashes); CRUD routes 401 when unauthenticated.  
- [ ] **B-3 (P1)** Bearer-token header (prep for OIDC).  
- [ ] **B-4 (P1)** Route-level RBAC middleware (admin vs read-only).

---

## Epic C — Policy Engine

- [ ] **C-1 (P0)** Finalise DSL grammar (MATCH, ASSERT, SET, APPLY).  
      _Acceptance:_ `cargo test -p policy` parses sample rules.
- [ ] **C-2 (P0)** Implement evaluator core; > 90 % branch test coverage.  
- [ ] **C-3 (P0)** Background Git sync (pulls rules every _N_ min).  
- [ ] **C-4 (P0)** `unet-cli policy check nodes.csv --rules ./policies` exits 1 on failed assertions.  
- [ ] **C-5 (P1)** Cache compiled AST (10 k rules < 10 ms).

---

## Epic D — Template & Config Diff (Milestone 4)

- [ ] **D-1 (P0)** Integrate **MiniJinja** with custom filters.  
      _Acceptance:_ `unet-cli render node.json` outputs vendor config.
- [ ] **D-2 (P0)** Parse `# match:` header for subtree matching.  
- [ ] **D-3 (P0)** Implement **config-slicer** crate (Juniper & IOS parsers).  
- [ ] **D-4 (P0)** `unet-cli diff --node r1 --template interfaces` shows colour diff.  
- [ ] **D-5 (P1)** Support `{% include %}` / `{% import %}` in templates.

---

## Epic E — Client CLI Enhancements

- [ ] **E-1 (P0)** `--offline` flag (local policy/templates only).  
- [ ] **E-2 (P1)** Canary workflow: `unet-cli push-canary .`.  
- [ ] **E-3 (P1)** `--debug-policy` prints rule trace for a node.

---

## Epic F — API & MCP

- [ ] **F-1 (P0)** MCP-compliant `/mcp/context` endpoint (JSON schema).  
- [ ] **F-2 (P2)** SSE stream for live diffs; front-end PoC.

---

## Epic G — CI/CD & Packaging

- [ ] **G-1 (P0)** GitHub Actions matrix: `linux-musl`, `macOS`, `windows-gnu`.  
- [ ] **G-2 (P0)** Enforce `cargo audit`, `clippy --deny warnings`, `fmt`.  
- [ ] **G-3 (P1)** Chocolatey package script for Windows.  
- [ ] **G-4 (P1)** Auto-publish `cargo doc` + mdBook to GH Pages.

---

## Epic H — Documentation

- [ ] **H-1 (P0)** Update **docs/04_template_engine.md** with match syntax & examples.  
- [ ] **H-2 (P0)** Add **CONTRIBUTING.md** & **CODE_OF_CONDUCT.md**.  
- [ ] **H-3 (P1)** Label at least 10 good-first-issues for newcomers.

---

## Epic I — Observability & Ops

- [ ] **I-1 (P1)** Prometheus metrics (`unet_policy_eval_total` etc.).  
- [ ] **I-2 (P2)** Jaeger tracing behind feature flag.

---

### **Critical Path for First Public Release**

1. **A-1 → A-3**, **B-1 → B-2**, **C-1 → C-4**, **D-1 → D-4**, **G-1 → G-2**, **H-1 → H-2**  
2. Everything else can follow as **P1 / P2** issues.

Assign each Story to an owner, create GitHub issues referencing the Epic ID, and tag priorities for easy filtering.

