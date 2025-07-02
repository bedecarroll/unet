# Repository Structure Review - 2025-07-02

**Report Time (UTC):** 05:50:28

The top-level layout of the project was examined for clarity and maintainability. Numerous artifacts from earlier development phases are mixed with production assets. This makes the repository difficult to navigate for new contributors and obscures which files are actually required.

## Observations

- Multiple configuration files (`config*.toml`) live alongside source code. Most are example configs and are duplicated under `configs/`.
- Deployment tooling is scattered across `ansible/`, `k8s/`, `helm/`, `packaging/`, `docker-compose*.yml`, and `systemd/`.
- Legacy TODO files (`TODO-OLD.md`, `TODO-BACKUP.md`) remain at the project root.
- Test utilities such as `test_sqlite_datastore.rs` sit at the top level instead of under `tests/`.
- Sample templates, policies and fixtures each have their own directories but lack explanation of how they relate to the code.

## Recommendations

1. **Group deployment assets** under a unified `deploy/` directory (`ansible`, `helm`, `k8s`, `packaging`, systemd units, docker-compose files).
2. **Move example configuration** files into `configs/` and remove duplicates like `config-postgres.toml` from the root.
3. **Archive or delete legacy TODO files** leaving only `TODO.md` plus `TODO-ARCHIVE.md`.
4. **Create a `tests/` directory** for integration helpers such as `test_sqlite_datastore.rs`.
5. **Document sample data** (templates, policies, fixtures) in the README and clarify their role.

A cleanup pass addressing the above will help present a professional, easy-to-understand structure for potential open source contributors.
