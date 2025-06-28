# 07 CI/CD – Building, Testing & Releasing μNet with GitHub Actions

> **Audience:** Engineers responsible for automation and release engineering.
>
> **Goals:**
>
> 1. Provide a **repeatable, deterministic** pipeline from PR to production container.
> 2. Deliver **fast feedback** to contributors (≤5 min) while keeping resource usage reasonable.
> 3. Enforce **quality gates** (format, lint, tests, security) before code merges.
> 4. Publish **multi‑platform artifacts** (Linux static binaries, Docker images, Homebrew & Debian/RPM packages).

---

## Table of Contents

1. [Repository Layout](#1-repository-layout)
2. [Git Branch Strategy](#2-git-branch-strategy)
3. [Workflow File Structure](#3-workflow-file-structure)
4. [Core Jobs Explained](#4-core-jobs-explained)
5. [Reusable Workflow Templates](#5-reusable-workflow-templates)
6. [Secrets & Permissions](#6-secrets--permissions)
7. [Caching & Speed Optimisations](#7-caching--speed-optimisations)
8. [Release Process](#8-release-process)
9. [Matrix Builds & Platforms](#9-matrix-builds--platforms)
10. [Self‑Hosted Runners](#10-self-hosted-runners)
11. [Monitoring Failed Runs](#11-monitoring-failed-runs)
12. [Rejected Alternatives](#12-rejected-alternatives)

---

## 1  Repository Layout

```
.github/
├── workflows/
│   ├── check.yml              # PR lint + test gate
│   ├── release.yml            # Tag‑triggered build & publish
│   ├── docker.yml             # Nightly image (main branch)
│   └── reusable/
│       ├── setup-rust.yml     # Composite action
│       └── cargo-cache.yml    # Reusable cache step
└── dependabot.yml             # Dependency updates
```

> **Tip:** Keep workflow files short – move repeated logic into `reusable/`.

---

## 2  Git Branch Strategy

| Branch                  | Protection Rules                        | Who pushes | What builds run               |
| ----------------------- | --------------------------------------- | ---------- | ----------------------------- |
| `main`                  | Require PR, 2 reviews, status checks OK | Humans/CI  | `check.yml` then `docker.yml` |
| `release/*`             | Same as `main`                          | Maintainer | `check.yml`                   |
| version tags (`vX.Y.Z`) | Manual via PR merge + GH Release        | Maintainer | `release.yml`                 |

Feature branches live on forks; CI still runs but with reduced permissions (no image push).

---

## 3  Workflow File Structure

### 3.1 `check.yml` (Pull Request Gate)

```yaml
name: PR Checks
on:
  pull_request:
    branches: [main, 'release/**']
permissions:
  contents: read
  pull-requests: write
jobs:
  lint-test:
    uses: ./.github/workflows/reusable/setup-rust.yml
    with:
      rust-version: stable
    secrets: inherit
    steps:
      - name: Install Linux deps
        run: sudo apt-get update && sudo apt-get install -y libsnmp-dev
      - name: Cache cargo
        uses: ./.github/workflows/reusable/cargo-cache.yml
      - name: Clippy
        run: cargo clippy --workspace --all-targets -- -D warnings
      - name: Format
        run: cargo fmt --check
      - name: Tests
        run: cargo test --workspace --all-features -- --nocapture
      - name: Security Audit
        run: cargo audit --deny warnings
```

### 3.2 `docker.yml` (Nightly Image)

```yaml
name: Nightly Docker Build
on:
  push:
    branches: [main]
  workflow_dispatch: {}
permissions:
  packages: write
  contents: read
jobs:
  build-push:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Set up QEMU for multi‑arch
        uses: docker/setup-qemu-action@v3
      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3
      - name: Login to GHCR
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}
      - name: Build & Push
        uses: docker/build-push-action@v5
        with:
          context: .
          file: docker/Dockerfile.server
          platforms: linux/amd64,linux/arm64
          push: true
          tags: |
            ghcr.io/${{ github.repository }}/unet-server:nightly
            ghcr.io/${{ github.repository }}/unet-server:sha-${{ github.sha }}
```

### 3.3 `release.yml` (SemVer Tag)

```yaml
name: Release
on:
  push:
    tags: ['v*.*.*']
permissions:
  contents: write       # upload artifacts to GitHub Release
  packages: write       # push Docker + Homebrew bottle
jobs:
  build-artifacts:
    strategy:
      matrix:
        target: [x86_64-unknown-linux-musl, aarch64-unknown-linux-musl]
    uses: ./.github/workflows/reusable/setup-rust.yml
    with:
      rust-version: stable
    steps:
      - uses: taiki-e/upload-rust-binary-action@v1
        with:
          bin: unet-server,unet-cli
          target: ${{ matrix.target }}
          tar: gz

  create-release:
    needs: build-artifacts
    runs-on: ubuntu-latest
    steps:
      - uses: softprops/action-gh-release@v2
        with:
          generate_release_notes: true
          files: target/release/artifacts/**
```

> **Why separate jobs?** Matrix build can run in parallel; release job waits and uploads.

---

## 4  Core Jobs Explained

| Job                 | Triggers              | Key Steps / Tools                              | Approx Time |
| ------------------- | --------------------- | ---------------------------------------------- | ----------- |
| **lint-test**       | PRs, `main` pushes    | `cargo clippy`, `fmt`, `test`, `audit`         | 3–5 min     |
| **coverage**        | Nightly (cron)        | `cargo tarpaulin --out Xml`, upload to Codecov | 6 min       |
| **docker‑build**    | `main` & Release tags | Buildx multi‑arch, push to GHCR                | 4 min       |
| **package‑deb/rpm** | Release tags          | `cargo-deb` / `cargo‑rpm`, upload              | 2 min       |

---

## 5  Reusable Workflow Templates

### 5.1 `setup-rust.yml`

```yaml
name: Setup Rust
on:
  workflow_call:
    inputs:
      rust-version:
        required: false
        type: string
        default: stable
jobs:
  init:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: ${{ inputs.rust-version }}
          components: clippy, rustfmt
      - name: Install cargo tools
        run: |
          cargo install cargo-audit --locked
          cargo install cargo-tarpaulin --locked
```

### 5.2 `cargo-cache.yml`

```yaml
- uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
      target
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    restore-keys: ${{ runner.os }}-cargo-
```

---

## 6  Secrets & Permissions

| Secret Name                        | Purpose                           | Scope (env)             |
| ---------------------------------- | --------------------------------- | ----------------------- |
| `GHCR_PAT` (optional)              | Push to GitHub Container Registry | `docker.yml`, `release` |
| `CARGO_REGISTRIES_CRATES_IO_TOKEN` | Publish crates (future)           | `publish.yml`           |
| `SLACK_WEBHOOK`                    | Failure notifications             | `notify` reusable step  |

Use `` blocks to follow **least privilege**; avoid `contents: write` except in release.

---

## 7  Caching & Speed Optimisations

- **Rust dependency cache** – `actions/cache` keyed on `Cargo.lock`.
- **Docker layer cache** – Buildx automatically utilises GHCR `--cache-from` (use nightly tag).
- **Parallel matrix** – set `max-parallel` to CPU count; default runner concurrency = 2.
- **Incremental tests** – leverage `cargo test --doc --no-default-features` in a fast pre‑job (TODO).

> **Metrics:** Track average workflow duration in GitHub Insights; aim < 7 min per PR.

---

## 8  Release Process

1. **Bump version** via PR (`cargo set-version x.y.z`).
2. Merge to `main` – CI passes.
3. Maintainer creates tag `vX.Y.Z` & GitHub Release draft.
4. `release.yml` builds binaries, checksum, SBOM, Docker images, Homebrew bottles.
5. Verify artifacts, press **Publish**.
6. Homebrew tap auto‑PR created by `homebrew-bump-formula-pr`.
7. Docker `latest` tag updated to new version.

> **Semantic Versioning:** MAJOR – breaking CLI/API, MINOR – new features, PATCH – bug fix.

---

## 9  Matrix Builds & Platforms

| Target Triple                    | Use Case                 | Notes                              |
| -------------------------------- | ------------------------ | ---------------------------------- |
| `x86_64-unknown-linux-musl`      | Most servers & Alpine    | Static binary, no GLIBC.           |
| `aarch64-unknown-linux-musl`     | ARM servers & Apple M‑1  | QEMU cross‑compilation via buildx. |
| `x86_64-apple-darwin` (optional) | Operator laptops (macOS) | Requires macOS runner (costly).    |

MUSL targets ensure Docker image size ≈ 15 MB.

---

## 10  Self‑Hosted Runners

*Use only if OSS minutes are exhausted or hardware‑specific tests (e.g., SNMP devices) needed.*

| Hostname     | Labels                 | Specs         | Jobs routed              |
| ------------ | ---------------------- | ------------- | ------------------------ |
| `runner‑amd` | `self-hosted`, `amd64` | 8 vCPU, 16 GB | lint‑test, coverage      |
| `runner‑arm` | `self-hosted`, `arm64` | 4 vCPU, 8 GB  | docker build arm64 image |

Register via `actions-runner` service; ensure firewall outbound `github.com`.

---

## 11  Monitoring Failed Runs

- **Slack Notification** – `post‑if` step on failure (`SLACK_WEBHOOK`).
- **GitHub Code Owners** – auto‑request review from `@net‑platform‑team`.
- **Retry Policy** – enable `max‑attempts: 2` on flaky tests.
- **Long‑term Metrics** – integrate with `opsgenie‑actions` or GH Insights.

---

## 12  Rejected Alternatives

| Option                                        | Reason for Rejection                                             |
| --------------------------------------------- | ---------------------------------------------------------------- |
| **CircleCI / Travis**                         | Extra SaaS cost; OSS minutes limited; less tight GH integration. |
| **Jenkinsfile**                               | Requires maintaining server; plugin sprawl; slower UI.           |
| **Docker‑in‑Docker tests**                    | Security & performance issues; network complexity.               |
| **GitHub Actions monolithic** single workflow | Hard to read; split for clarity & reuse.                         |

---

### Next Steps

1. Implement `setup-rust.yml` & `cargo-cache.yml`.
2. Commit `check.yml` and ensure PRs pass.
3. Draft `docker.yml`; push to `main`; verify images on GHCR.
4. Document release checklist in `docs/RELEASING.md`.

*Proceed to*[*08\_deployment.md*](08_deployment.md)
