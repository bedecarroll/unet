# Codex Startup Script

> **Purpose:** Document the commands run when a Codex environment starts

This script installs the development and CI tooling used by μNet. It is
executed automatically for Codex containers.

```bash
rustup component add clippy rustfmt llvm-tools-preview
curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
cargo binstall --disable-telemetry --no-confirm cargo-tarpaulin cargo-audit typos-cli mise mdbook cargo-llvm-cov
mise trust
curl -LsSf https://get.nexte.st/latest/linux | tar zxf - -C ${CARGO_HOME:-~/.cargo}/bin
cargo fetch
npm install markdownlint-cli2 --global --no-progress
```

### Command Overview

- `rustup component add clippy rustfmt llvm-tools-preview` – install Rust linting, formatting, and LLVM coverage components.
- `curl ... cargo-binstall ...` – download and install the `cargo-binstall` helper.
- `cargo binstall ...` – install code coverage, security audit, formatting, and other tooling.
- `mise trust` – mark this repository's `mise` configuration as trusted.
- `curl ... nexte.st ...` – fetch the `nextest` binary used to run tests.
- `cargo fetch` – pre-download Rust dependencies.
- `npm install markdownlint-cli2 --global --no-progress` – install the Markdown linter.
