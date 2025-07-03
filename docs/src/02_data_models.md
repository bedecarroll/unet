# 02 Data Models – Schema & Rust Structs

> **Audience:** Engineers implementing the storage layer and writing migrations.\
> **Prerequisites:** Skim **01\_architecture.md** for context.

---

## Table of Contents

1. [Philosophy](#1-philosophy)
2. [Entity Relationship Diagram](#2-entity-relationship-diagram)
3. [Core Enumerations](#3-core-enumerations)
4. [Primary Tables & Structs](#4-primary-tables--structs)
5. [Derived‑State Strategy](#5-derived‑state-strategy)
6. [SeaORM Notes & Migrations](#6-seaorm-notes--migrations)
7. [Validation Rules](#7-validation-rules)
8. [Custom Data Field](#8-custom data-field)
9. [Future Schema Evolution](#9-future-schema-evolution)
10. [Rejected Designs](#10-rejected-designs)

---

## 1  Philosophy

- **Start simple, evolve with migrations.** SQLite is fine for 10–100 k rows; design so Postgres can drop‑in later.
- **Desired vs Derived separation.** Never overwrite operator intent when pulling SNMP — keep a separate status table.
- **Schema‑less experimentation.** `custom_data` JSON column captures "unknown yet" attributes until promoted.
- **Declarative constraints.** Use DB‑level INDEX + UNIQUE constraints *and* Rust‑side validation.

---

## 2  Entity Relationship Diagram

```ascii
┌────────────┐ 1        * ┌─────────────┐
│  Location  │───────────│    Node     │
└────────────┘            └─────────────┘
      ▲                         ▲
      │ 0..*               1    │ 0..*
┌────────────┐           ┌─────────────┐
│  LinkSide  │◄──────────│    Link     │
└────────────┘  (view)   └─────────────┘
```

*`LinkSide`**is a SQL VIEW giving a flat row per node‑interface side. Useful for JOIN‑heavy queries; optional for v0.*

---

## 3  Core Enumerations

### 3.1 `Lifecycle`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, sea_orm::EnumIter, strum::Display)]
#[sea_orm(rs_type = "String", db_type = "Enum(String)")]
pub enum Lifecycle {
    Planned,
    Implementing,
    Live,
    Decommissioned,
}
```

*Stored as string; easier to read in DB browser.*

### 3.2 `LinkRole`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, sea_orm::EnumIter, strum::Display)]
pub enum LinkRole {
    Backbone,
    Access,
    Internet,
    Peering,
    Unknown,
}
```

*`Unknown`**default when CSV import leaves field empty.*

---

## 4  Primary Tables & Structs

### 4.1 `location` table

| Column        | Type               | Notes                         |
| ------------- | ------------------ | ----------------------------- |
| `id`          | `UUID PRIMARY KEY` | `Uuid::new_v4()`              |
| `name`        | `TEXT NOT NULL`    | Unique *within sibling scope* |
| `parent_id`   | `UUID NULL`        | FK → `location.id`            |
| `lifecycle`   | `TEXT NOT NULL`    | Enum string                   |
| `custom_data` | `JSON NOT NULL`    | `'{ }'` default               |
| `created_at`  | `INTEGER`          | unix epoch (ms)               |
| `updated_at`  | `INTEGER`          | auto‑update via trigger       |

Rust struct (SeaORM entity):

```rust
#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "location")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub lifecycle: Lifecycle,
    pub custom_data: JsonValue,
    pub created_at: i64,
    pub updated_at: i64,
}
```

*Trigger functions update **`updated_at`** on row change.*

### 4.2 `node` table

| Column                    | Type      | Constraint / FK                       |
| ------------------------- | --------- | ------------------------------------- |
| `id`                      | `UUID`    | PK                                    |
| `node_name`               | `TEXT`    | **UNIQUE** w/ `domain_name`           |
| `domain_name`             | `TEXT`    | NULL allowed (single‑tenant)          |
| `vendor`                  | `TEXT`    | index for policy queries              |
| `model`                   | `TEXT`    | index                                 |
| `device_role`             | `TEXT`    | optional, index                       |
| `mgmt_ip`                 | `TEXT`    | stored as string, validated by Rust   |
| `software_version`        | `TEXT`    | desired                               |
| `location_id`             | `UUID`    | FK → `location.id ON DELETE SET NULL` |
| `lifecycle`               | `TEXT`    | cannot be NULL                        |
| `custom_data`             | `JSON`    | default `'{}'`                        |
| `created_at`/`updated_at` | `INTEGER` | triggers                              |

*Compound UNIQUE*: `(node_name, domain_name)` ensures duplicates don’t sneak in.

### 4.3 `node_status` table (derived)

| Column              | Type      | Notes                         |
| ------------------- | --------- | ----------------------------- |
| `node_id`           | `UUID`    | PK + FK → `node.id`           |
| `last_polled_at`    | `INTEGER` | epoch ms                      |
| `snmp_reachable`    | `BOOLEAN` | true on success               |
| `actual_sw_version` | `TEXT`    | null if not polled            |
| `raw_kv`            | `JSON`    | arbitrary SNMP OID→value hash |

*Why not embed in Node?* avoids row‑lock contention; SNMP writes can be frequent.

### 4.4 `link` table

| Column                    | Type      | Detail                            |
| ------------------------- | --------- | --------------------------------- |
| `id`                      | `UUID`    | PK                                |
| `node_a_id`               | `UUID`    | FK → `node.id`                    |
| `interface_a`             | `TEXT`    | e.g. "ge-0/0/1"                   |
| `node_z_id`               | `UUID`    | FK → `node.id` (NULL if external) |
| `interface_z`             | `TEXT`    | may be NULL when `node_z_id` NULL |
| `link_role`               | `TEXT`    | Enum string                       |
| `bandwidth_mbps`          | `INTEGER` | optional integer                  |
| `lifecycle`               | `TEXT`    | Enum                              |
| `custom_data`             | `JSON`    | for provider circuit id, etc.     |
| `created_at`/`updated_at` | INTEGER   | triggers                          |

*Composite UNIQUE* on `(node_a_id, interface_a)` prevents duplicate rows.

### 4.5 `template_assignment` table (optional future)

If we later decide to make template‑to‑node mapping persistent (instead of only policy in memory), we can materialize to this narrow table:

| Column     | Type | Detail      |
| ---------- | ---- | ----------- |
| `node_id`  | UUID | FK          |
| `template` | TEXT | path string |

Policy engine would `UPSERT` rows each run; CLI can query quickly.

### 4.6 Authentication & RBAC tables

μNet now ships with user and role management. These SeaORM entities back the
authentication module:

| Table        | Purpose                                                      |
| ------------ | ------------------------------------------------------------ |
| `users`      | Login accounts containing bcrypt password hashes            |
| `roles`      | Named roles such as `viewer`, `operator` and `admin`        |
| `user_roles` | Join table mapping users to roles (many‑to‑many)            |
| `api_keys`   | Bearer tokens with expiry and optional scope                |

### 4.7 Change management tables

Configuration changes are tracked and audited in the database:

| Table                      | Purpose                                         |
| -------------------------- | ----------------------------------------------- |
| `configuration_changes`    | Staged config diffs awaiting approval           |
| `change_approval_workflow` | Required approvers and status for each change   |
| `change_audit_log`         | Immutable record of approvals and rejections    |
| `change_rollback_snapshot` | Backup of previous config before apply          |

### 4.8 Operational metrics tables

Additional entities persist runtime information:

| Table               | Purpose                                        |
| ------------------- | ---------------------------------------------- |
| `polling_tasks`     | Tracks last SNMP poll attempt per node         |
| `interface_status`  | Per‑interface counters and operational state   |
| `template_usage`    | Records which nodes rendered a template        |
| `template_versions` | Stores versioned templates fetched from Git    |

---

## 5  Derived‑State Strategy

- Derived data (poll results) is **write‑heavy**, so isolate it.
- `node_status` is small (1 row / device) ➜ cheap updates.
- We keep **raw\_kv JSON** for vendor‑specific OIDs without altering schema.

### Snapshot vs History

For v0 we **only store last snapshot**. If historical trend is required, add `node_status_history` with `(node_id, ts)` PK later.

---

## 6  SeaORM Notes & Migrations

### 6.1 Enabling JSON in SQLite

SQLite default builds *do* include JSON1 extension. SeaORM maps `JsonValue` ↔ `TEXT` column; we store stringified JSON.\
For Postgres future → `jsonb` column automatically.

### 6.2 Migration Example (Create `node`)

```rust
pub struct M2025_06_21_create_node;

#[async_trait::async_trait]
impl MigrationTrait for M2025_06_21_create_node {
    async fn up(&self, mgr: &SchemaManager) -> Result<()> {
        mgr.create_table(
            Table::create()
                .table(Node::Table)
                .if_not_exists()
                .col(ColumnDef::new(Node::Id).uuid().not_null().primary_key())
                .col(ColumnDef::new(Node::NodeName).string().not_null())
                .col(ColumnDef::new(Node::DomainName).string())
                .col(ColumnDef::new(Node::Vendor).string())
                .col(ColumnDef::new(Node::Model).string())
                .col(ColumnDef::new(Node::DeviceRole).string())
                .col(ColumnDef::new(Node::MgmtIp).string())
                .col(ColumnDef::new(Node::SoftwareVersion).string())
                .col(ColumnDef::new(Node::Lifecycle).string().not_null())
                .col(ColumnDef::new(Node::LocationId).uuid())
                .col(ColumnDef::new(Node::CustomData).json_binary().not_null())
                .col(ColumnDef::new(Node::CreatedAt).big_integer().not_null())
                .col(ColumnDef::new(Node::UpdatedAt).big_integer().not_null())
                .to_owned(),
        ).await?;

        mgr.create_index(
            Index::create()
                .name("idx_node_vendor_model")
                .table(Node::Table)
                .col(Node::Vendor)
                .col(Node::Model)
                .to_owned(),
        ).await?;
        Ok(())
    }

    async fn down(&self, mgr: &SchemaManager) -> Result<()> {
        mgr.drop_table(Table::drop().table(Node::Table).to_owned()).await
    }
}
```

*Run with* `sea-orm-cli migrate up` (*CI runs this automatically*).

### 6.3 Triggers for `updated_at`

SQLite trigger example (raw SQL executed via migration):

```sql
CREATE TRIGGER node_updated
AFTER UPDATE ON node
BEGIN
  UPDATE node SET updated_at = strftime('%s','now')*1000 WHERE id = NEW.id;
END;
```

SeaORM’s migration can embed this SQL.

---

## 7  Validation Rules

| Validation                                         | Implementation                                        | Failure Behaviour        |
| -------------------------------------------------- | ----------------------------------------------------- | ------------------------ |
| `mgmt_ip` is valid IPv4/v6                         | `ipaddress::ip_net::IpAddr::from_str` in CLI & server | 400 Bad Request          |
| `software_version` non‑empty for `lifecycle=Live`  | pre‑save check                                        | 422 UnprocessableEntity  |
| Node belongs to an *existing* Location if not NULL | FK ensures                                            | SQL error bubbled to 400 |
| JSON in `custom_data` must be an object            | Serde check                                           | 400                      |

*Lightweight – not every constraint enforced; rely on policy engine for business rules.*

---

## 8  Custom Data Field

- JSON object, shallow by convention.
- Policy `SET path TO value` uses dot‑notation path (`maintenance.window`) to insert nested keys.
- CLI `--json` flag pretty‑prints this column.

### Promotion Workflow

1. Field proves useful (appears in many nodes).
2. DBA adds real column via migration; CLI migrates value out of `custom_data` (one‑off script).
3. Policy updated to stop writing JSON key.

---

## 9  Future Schema Evolution

| Feature              | Action                                                                                                    |
| -------------------- | --------------------------------------------------------------------------------------------------------- |
| **Postgres**         | Add `store_pg.rs`; compile under `--features pg`; ensure UUID & JSONB columns.                            |
| **Interface table**  | If we need per‑interface desired state (speed, vlan) create `interface` table keyed by `(node_id, name)`. |
| **History**          | Append‑only tables with `ts` PK; use PARTITION BY month in Postgres.                                      |
| **Full‑text search** | Enable FTS5 on `node.custom_data` for tags.                                                               |

We keep migrations *idempotent*; use semver tags to mark breaking changes.

---

## 10  Rejected Designs

| Design Idea                                 | Rejection Reason                                                                                    |
| ------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| **Single “everything” JSON blob per node**  | Hard to query (e.g., `vendor='juniper'` becomes JSON\_SEARCH), no FK integrity, indexes impossible. |
| **Storing derived data in same row**        | SNMP poll rate causes write‑amplification; read/write contention with operator edits.               |
| **Multiple DBs – split reads/writes**       | Overkill for v0; network automation teams typically small; focus on correctness first.              |
| **Using Diesel (sync) ORM**                 | Blocks async; would need thread‑pool; SeaORM gives compile‑time safety close to Diesel now.         |
| **Integer surrogate keys (auto increment)** | UUID easier for multi‑source imports (CSV) and safer for offline editing.                           |
| **Separate table per vendor type**          | Schema explosion, duplication. Keep generic + vendor in `vendor` field.                             |

---

## Next Steps

🔹 Implement migrations (`unet-core/migrations`).\
🔹 Write unit tests: `model::tests::roundtrip_serialization()`.\
🔹 Update **05\_cli\_tool.md** once CRUD commands compiled.

Back to [01\_architecture.md](01_architecture.md) • Forward to **03\_policy\_engine.md**
