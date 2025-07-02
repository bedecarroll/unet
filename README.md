# μNet (Unet) – Simple Network Configuration System

> **Status:** Green‑field (Milestone 1).  Everything is subject to change—embrace PRs!\
> **Docs:** Rendered with **mdBook** in [`docs/`](docs/) → [http://localhost:3000](http://localhost:3000) when running `mdbook serve`.

---

## What is μNet?

μNet is a Rust‑based platform that helps network operators **store desired state**, **pull actual state**, **enforce policy** and **generate vendor configs**—all from a single set of Git‑version‑controlled files.

Key features:

- **Rust single‑binary** server & CLI—no Python/JDK runtime surprises.
- **SQLite database** with full ACID transactions via SeaORM and complete CRUD operations.
- **Example data fixtures** for quick onboarding and testing with realistic network topologies.
- **Custom DSL** policy engine, familiar **MiniJinja** templates.
- **Hierarchical config diff** via the stand‑alone `config‑slicer` crate.

*(Full architectural deep‑dive in *[*`docs/01_architecture.md`*](docs/src/01_architecture.md)*.)*

---

## Repository Layout

```text
unet/                       # ← you are here
├── Cargo.toml              # Cargo workspace manifest
├── README.md               # <— this file
├── LICENSE
├── .gitignore
├── docs/                   # mdBook sources (Markdown)
│   └── src/
│       ├── SUMMARY.md      # mdBook table of contents
│       ├── 01_architecture.md
│       ├── 02_data_models.md
│       ├── ...
├── crates/                 # All Rust crates (pure Cargo workspace)
│   ├── unet-core/          # shared library: models, datastore, policy, template
│   ├── unet-server/        # binary: Axum API + background tasks
│   ├── unet-cli/           # binary: Clap command‑line interface
│   └── config-slicer/      # library + CLI for cfg hierarchy slicing
├── migrations/             # SeaORM migration files (timestamped)
├── fixtures/               # Example data for quick onboarding
│   ├── examples/           # Network topology examples
│   │   ├── small-office/   # Small business network (10-50 users)
│   │   ├── datacenter/     # Enterprise datacenter topology
│   │   └── campus/         # Multi-building campus network
│   └── schemas/            # JSON schemas for validation
├── policies/               # Sample *.rules checked into Git (optional)
├── templates/              # Sample *.jinja templates (optional)
├── deploy/                 # Deployment assets and container configs
│   ├── ansible/            # Ansible playbooks for bare‑metal installs
│   ├── helm/               # Helm charts for Kubernetes
│   ├── k8s/                # Raw Kubernetes manifests
│   ├── packaging/          # DEB/RPM packaging scripts
│   ├── systemd/            # Service unit files
│   ├── docker-compose.yml  # PoC stack (server + Caddy TLS)
│   └── docker-compose.prod.yml
├── configs/                # Runtime configuration templates and examples
│   ├── examples/           # Sample config.toml variants
│   ├── templates/          # Environment templates
│   └── environments/       # Per-environment configs
├── tests/                  # Integration helpers and smoke tests
├── docker/                 # Container build context
│   ├── Dockerfile.server
│   └── Dockerfile.cli
├── scripts/                # Helper scripts (pre‑commit, release)
└── .github/
    └── workflows/          # GitHub Actions CI/CD
```

- Every Rust crate lives under `crates/` → one `cargo check` covers all.
- Docs are **outside** source tree so we can publish via GitHub Pages or Netlify with mdBook.
- Sample `policies/`, `templates/` and `fixtures/` directories provide demo data so you can try μNet without authoring anything from scratch.

These sample directories are referenced throughout the docs and tests. They allow you to spin up a demo environment or run the integration helpers in `tests/` without building your own dataset first.

---

## Quick Start (Developer Laptop)

```bash
# 0. Prerequisites
#    – Rust 1.77+  – Git  – (optional) Docker 25+  – mdBook 0.4+

# 1. Clone & enter workspace
$ git clone https://github.com/<your‑org>/unet.git && cd unet

# 2. Check the toolchain & build all crates (debug)
$ rustup default stable
$ cargo check --workspace --all-targets

# 3. Run unit/integration tests
$ cargo test --workspace

# 4. Start the demo server (SQLite, fixtures)
$ cargo run -p unet-cli -- init --database ./unet.db
$ cargo run -p unet-cli -- import --from fixtures/examples/small-office/
$ cargo run -p unet-server -- --database-url sqlite:./unet.db

# 5. Open a new shell – list demo nodes via CLI
$ cargo run -p unet-cli -- nodes list
$ cargo run -p unet-cli -- locations list

# 6. View docs (mdBook)
$ mdbook serve docs --open   # http://localhost:3000
```

---

## Building the Documentation

We ship **mdBook** sources so juniors can edit docs next to code.

```bash
cargo install mdbook --locked   # once
mdbook serve docs               # auto‑reload on save
```

CI runs `mdbook build docs/` to ensure no broken links.

---

## Continuous Integration / Delivery

GitHub Actions workflows live under `.github/workflows/` (see detailed spec in [`docs/07_ci_cd.md`](docs/src/07_ci_cd.md)).

- **PR Gate** → format, clippy, tests, audit.
- **Nightly** → Docker multi‑arch images (`ghcr.io/<org>/unet-server:nightly`).
- **Release Tag** (`vX.Y.Z`) → static binaries, Docker, Homebrew/RPM/DEB artifacts.

---

## Contributing

1. Fork → feature branch `feat/<slug>` → PR.
2. Run `./scripts/pre-commit.sh` (fmt + clippy + tests).
3. Update docs **in the same PR** if behaviour or API changes.
4. Tag with appropriate `area:` label (`area:server`, `area:policy`, etc.).

New?  Start with “Good First Issue” in GitHub or follow the 30‑day plan in [`docs/12_onboarding.md`](docs/src/12_onboarding.md).

---

## License

MIT OR Apache‑2.0 – choose whichever suits your project, or dual‑license as we do.

---

### At a Glance (One‑Liner)

```bash
cargo run -p unet-cli -- --server http://localhost:8080 node diff core‑01 -o live.conf
```

This renders policy‑assigned templates for *core‑01*, slices the live config, and shows a colorised diff—in **one command**. Welcome to μNet 🚀
