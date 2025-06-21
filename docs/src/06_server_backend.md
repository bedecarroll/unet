# 06 Server Backend – Axum API, Tasks & Infrastructure

> **Audience:** Engineers implementing the `unet-server` binary crate.
>
> **Mission:** Expose a robust HTTP API, run background tasks (SNMP polling, git sync, policy eval), and persist data via SeaORM.
>
> **Prereqs:** Read **01\_architecture.md** for the big picture and **02\_data\_models.md** for schema details.

---

## Table of Contents

1. [Crate Layout](#1-crate-layout)
2. [Configuration File (](#2-configuration-file-configtoml)[`config.toml`](#2-configuration-file-configtoml)[)](#2-configuration-file-configtoml)
3. [`main.rs`](#3-mainrs-boot-sequence)[ Boot Sequence](#3-mainrs-boot-sequence)
4. [HTTP Layer (Axum)](#4-http-layer-axum)
5. [Data Access & Connection Pools](#5-data-access--connection-pools)
6. [Background Tasks](#6-background-tasks)
7. [Error Handling Model](#7-error-handling-model)
8. [Logging, Tracing & Metrics](#8-logging-tracing--metrics)
9. [Graceful Shutdown](#9-graceful-shutdown)
10. [Security & Auth](#10-security--auth)
11. [Testing Strategy](#11-testing-strategy)
12. [Performance Notes](#12-performance-notes)
13. [Extending the Backend](#13-extending-the-backend)
14. [Rejected Designs](#14-rejected-designs)

---

## 1  Crate Layout

```text
crates/
└── unet-server/
    ├── src/
    │   ├── main.rs            # entrypoint
    │   ├── api/               # route handlers (per resource)
    │   │   ├── mod.rs
    │   │   ├── nodes.rs
    │   │   ├── links.rs
    │   │   ├── locations.rs
    │   │   └── health.rs
    │   ├── tasks/             # background jobs
    │   │   ├── mod.rs
    │   │   ├── git_sync.rs
    │   │   ├── snmp_poll.rs
    │   │   └── policy_eval.rs
    │   ├── state.rs           # AppState struct (Arc)
    │   ├── error.rs           # `ServerError` enum -> IntoResponse
    │   └── config.rs          # Config structs + loader
    └── Cargo.toml
```

> **Rule:** Keep each handler ≤200 LOC. Move reusable logic into `unet-core` when possible.

---

## 2  Configuration File (`config.toml`)

### 2.1 Minimal Example

```toml
[server]
bind = "0.0.0.0:8080"
workers = 0                  # 0 = Tokio picks #CPUs

[database]
type = "sqlite"             # "sqlite" | "postgres"
path = "/var/lib/unet/unet.db"  # Ignored for postgres

[git]
policies = "https://github.com/org/unet-policies.git"
templates = "https://github.com/org/unet-templates.git"
sync_cron = "*/15 * * * *"        # every 15 min

[snmp]
poll_interval_secs = 900
community = "public"
retry = 1

auth = { mode = "none" }          # future: "basic", "jwt"
```

### 2.2 Typed Structs

```rust
#[derive(Deserialize)]
pub struct Config {
    pub server: ServerCfg,
    pub database: DbCfg,
    pub git: GitCfg,
    pub snmp: SnmpCfg,
    pub auth: AuthCfg,
}
```

Load via `` crate (layer = defaults → toml → env → cmd flags).

---

## 3  `main.rs` Boot Sequence

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Config + tracing
    let cfg = Config::load()?;
    tracing_subscriber::fmt::init();

    // 2. DB Connection
    let db = cfg.database.connect().await?;          // returns SeaORM Connection

    // 3. Build Template & Policy engines (initial load)
    let tpl_engine = TemplateEngine::new(cfg.git.templates.clone())?;
    let rules      = PolicyLoader::load_dir(cfg.git.policies.clone())?;

    // 4. Shared state
    let state = AppState::new(db, tpl_engine, rules, cfg.clone());

    // 5. Spawn background tasks (non‑blocking)
    tasks::spawn_all(&state).await;

    // 6. Build router
    let app = api::build_router(state.clone());

    // 7. Start server
    axum::Server::bind(&cfg.server.bind.parse()?)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}
```

*`shutdown_signal()`** listens for SIGINT/SIGTERM & triggers Task cancellation (see §9).*

---

## 4  HTTP Layer (Axum)

### 4.1 Router Hierarchy

```rust
pub fn build_router(state: AppState) -> Router {
    Router::new()
        // health
        .route("/health/live", get(health::live))
        .route("/health/ready", get(health::ready))

        // API v1 group
        .nest(
            "/api/v1",
            Router::new()
                .merge(nodes::routes())
                .merge(links::routes())
                .merge(locations::routes()),
        )
        .layer(TraceLayer::new_for_http())
        .layer(AddExtensionLayer::new(state))
}
```

### 4.2 Nodes Endpoint Table

| Method   | Path                       | Handler       | Description             |
| -------- | -------------------------- | ------------- | ----------------------- |
| `GET`    | `/api/v1/nodes`            | `list_nodes`  | List / filter           |
| `POST`   | `/api/v1/nodes`            | `create_node` | Create node             |
| `GET`    | `/api/v1/nodes/:id`        | `get_node`    | Get by UUID             |
| `PATCH`  | `/api/v1/nodes/:id`        | `update_node` | Partial update          |
| `DELETE` | `/api/v1/nodes/:id`        | `delete_node` | Soft‑delete (lifecycle) |
| `GET`    | `/api/v1/nodes/:id/render` | `render_node` | Return rendered config  |

> **OpenAPI generation:** Use `` derive on handlers; exposed at `/api/v1/openapi.json`.

### 4.3 Middleware

| Layer                 | Purpose                                       |
| --------------------- | --------------------------------------------- |
| `TraceLayer`          | request/response logs (URI, status, latency). |
| `CorsLayer` (feature) | allow JS UI to call API.                      |
| `AuthLayer` (future)  | Basic / JWT auth; sets `Extension<User>`.     |

---

## 5  Data Access & Connection Pools

- **SeaORM** connection is *async‑pg* or *sqlx‑sqlite* behind `DatabaseConnection`.
- `` shared in `AppState`; SeaORM uses internal pool.
- For heavy SELECT loops (policy eval) use `` to exploit pool concurrency.

### 5.1 Transaction Example

```rust
pub async fn create_node(state: Extension<AppState>, Json(body): Json<NodeCreate>) -> Result<Json<Node>, ServerError> {
    let txn = state.db.begin().await?;
    let node = node::ActiveModel { /* set columns */ }.insert(&txn).await?;
    txn.commit().await?;
    Ok(Json(node.into()))
}
```

Error converted via `From<sea_orm::DbErr> for ServerError`.

---

## 6  Background Tasks

### 6.1 Git Sync (`git_sync.rs`)

| Step | Detail                                                     | Crates                      |
| ---- | ---------------------------------------------------------- | --------------------------- |
| 1    | Sleep until next `cron` tick (`tokio_cron_scheduler`).     | `tokio_cron_scheduler`      |
| 2    | `git2::Repository::open_or_clone(url, path)`.              | `git2`                      |
| 3    | `repo.find_remote("origin").fetch(&["main"], None, None)`  |                             |
| 4    | If HEAD changed: reload **policy** & **template** engines. | reload functions in `state` |

Reloads are **atomic** by swapping `Arc`s inside `AppState`.

### 6.2 SNMP Poll (`snmp_poll.rs`)

| Parameter              | Source (`config.toml`)      | Default  |
| ---------------------- | --------------------------- | -------- |
| `poll_interval_secs`   | `[snmp] poll_interval_secs` | `900`    |
| `community`            | `[snmp] community`          | `public` |
| `retry` + `timeout_ms` | TBD                         |          |

Algorithm per interval:

1. Fetch list of **Live** nodes where `mgmt_ip` IS NOT NULL.
2. Use `tokio::task::spawn` per node (`limit_concurrency` via semaphore).
3. For each node:
   - Bulk‑GET OIDs `sysDescr`, `sysObjectID`, etc.
   - Write into `node_status` table.
4. Send metrics (`poll_success_total`, `poll_duration_ms`).

### 6.3 Policy Eval (`policy_eval.rs`)

Runs **after** SNMP poll completes or on `git_sync` rule reload.

1. Query all nodes, links, locations into in‑mem structs (`Vec`).
2. Iterate objects × rules; produce `PolicyResult` (see §03).
3. Store in `DashMap<Uuid, Arc<PolicyResult>>` inside `AppState`.
4. Emit Prometheus gauges (`violations_total{rule="ver"}`).

---

## 7  Error Handling Model

- Unified `` enum with variants:
  - `Db(sea_orm::DbErr)` → 500
  - `NotFound` → 404
  - `Validation(String)` → 422
  - `BadRequest(String)` → 400
  - `Conflict(String)` → 409
  - `Internal(anyhow::Error)` → 500
- Implements `IntoResponse` to map to JSON body:

```jsonc
{
  "error": "Validation",
  "detail": "node_name must be unique"
}
```

- In debug build, internal errors include backtrace; in release, omit detail.

---

## 8  Logging, Tracing & Metrics

| Tool          | Endpoint   | Notes                                    |
| ------------- | ---------- | ---------------------------------------- |
| `tracing`     | stdout     | JSON logs behind `--log-format json`.    |
| Prometheus    | `/metrics` | Expose via `axum-prometheus` middleware. |
| OpenTelemetry | feature    | Optional compile‑time feature `otel`.    |

Configure via env vars (`RUST_LOG`, `OTEL_EXPORTER_OTLP_ENDPOINT`).

---

## 9  Graceful Shutdown

```rust
async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.expect("failed to install ctrl_c");
}
```

- Shutdown hook triggers `notify` to background tasks via `broadcast::Sender<()>`; each tokio task listens and breaks loop.
- Axum `with_graceful_shutdown` drains in‑flight requests (30 s timeout).

---

## 10  Security & Auth

| Mode    | Description                     | Status |
| ------- | ------------------------------- | ------ |
| `none`  | No auth; for PoC / lab          | READY  |
| `basic` | HTTP Basic (users table in DB)  | TODO   |
| `jwt`   | Bearer JWT, public key rotation | FUTURE |

### 10.1 TLS Termination

- Recommended: front with **Nginx** / **Traefik** TLS.
- For standalone: enable `rustls` feature; server binds `0.0.0.0:8443` with cert/key paths in config.

---

## 11  Testing Strategy

1. **Unit** – handlers with Axum `Router::new()` + `tower::ServiceExt::oneshot`.
2. **Integration** – spin up SQLite temp DB, run background tasks in memory using `#[tokio::test(flavor = "multi_thread")]`.
3. **API Contract** – generate OpenAPI and validate against Postman collection (CI step).
4. **Load test** – k6 script hitting `GET /nodes` 1 k RPS (GitHub Actions self‑hosted runner).

CI job `cargo test -p unet-server --all-features` plus `cargo tarpaulin` for coverage.

---

## 12  Performance Notes

| Aspect           | Baseline (8‑core VM)            | Notes                         |
| ---------------- | ------------------------------- | ----------------------------- |
| HTTP throughput  | 4 k req/s `GET /nodes`          | Axum + JSON serde             |
| SNMP poll window | 10 k nodes / 900 s              | 3 concurrent sockets × 200 ms |
| Policy eval      | 100 rules × 10 k nodes ≈ 200 ms | Single thread; Rayon TBD      |
| Memory footprint | \~110 MB RSS (templates cached) | Use `Arc<str>` in AST         |

Optimization backlog: prepared statement cache, predicate push‑down (SeaORM → raw SQL), Rayon in policy loop.

---

## 13  Extending the Backend

| Feature               | Implementation Sketch                                      |
| --------------------- | ---------------------------------------------------------- |
| **Postgres support**  | `cfg(feature = "pg")` compile; SeaORM `Database::connect`. |
| **WebSocket push**    | Add `ws` route; use `axum_ws`; stream PolicyResult deltas. |
| **Scheduled reports** | New task that generates CSV and emails via SMTP crate.     |
| **gRPC API**          | Add tonic‑build; share Protobuf with other systems.        |

---

## 14  Rejected Designs

| Design Idea                      | Rejection Reason                                                   |
| -------------------------------- | ------------------------------------------------------------------ |
| **Actix‑Web** runtime            | Slightly faster but heavier macros, non‑Tower middleware.          |
| **Diesel sync pool**             | Blocking executor complicates Tokio; prefer SeaORM async.          |
| **Separate poller microservice** | Adds deploy unit, duplicates models; single process simpler.       |
| **Cron via systemd‑timer**       | Requires host root; fewer cross‑platform guarantees inside Docker. |

---

### Next Steps (Milestone 2 & 3)

1. Implement `` & `` routes (return hard‑coded data).
2. Wire **SQLite** connection; run migrations on startup.
3. Build **git sync** task (log diff only).
4. Add **SNMP poller** (mock implementation returning random version).
5. Integrate **Policy Engine** (see 03\_policy\_engine.md) and expose `?eval=true` query param.

*Proceed to *[*07\_ci\_cd.md*](07_ci_cd.md)*.*

