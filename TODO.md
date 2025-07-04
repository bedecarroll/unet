# TODO.md - Documentation Cleanup Tasks

These tasks track issues identified during a review of the `docs/` directory. They focus on making the project ready for open source release.

## High Priority

 - [x] **Update Data Models:** `docs/src/02_data_models.md` is far behind the code. Rewrite tables and diagrams to match the current SeaORM entities and migrations.
 - [x] **Revise Architecture Docs:** `docs/src/01_architecture.md` and `docs/src/06_server_backend.md` omit major subsystems (authentication, change management, metrics, network access, secrets). Expand diagrams and descriptions.
 - [x] **Synchronize API Reference:** Many implemented endpoints are missing from the documentation. Generate an accurate API reference from `unet-server` routes.
 - [x] **Refresh CLI Documentation:** `docs/src/05_cli_tool.md` does not match the commands in `unet-cli`. Document all commands and flags.
 - [x] **Remove Deprecated Content:** Several sections describe early prototypes or planned features that were removed. Identify and delete outdated paragraphs.

## Medium Priority

 - [x] **Resolve TODO Placeholders:** Search the documentation for `TODO` markers and either implement the missing content or remove the placeholders.
  - [x] Replaced metrics placeholder in `01_architecture.md` and updated auth table in `06_server_backend.md`.
- [x] **Add Missing Feature Guides:** Write new docs for authentication, change management workflows, metrics/monitoring, and secrets management.
- [x] **Improve Cross-Linking:** Ensure related topics link to each other (e.g., policy docs linking to template docs and CLI examples).
- [x] **Spell Check and Style Pass:** Run a spell checker and ensure consistent heading levels and code block formatting.
- [x] **Review Examples:** Validate that configuration snippets and CLI samples actually work with the current codebase.

## Low Priority

- [x] **Update Future Work Section:** Trim unrealistic items and align with current roadmap.
- [x] **Add Contribution Guidelines:** Provide clear instructions for external contributors, including coding standards and the CLA process.
- [x] **Consider Diagrams:** Evaluate whether architecture diagrams should be regenerated with a consistent tool.
- [x] **License Audit:** Confirm all documentation files contain the correct license headers.


## Repository Cleanup

 - [x] **Consolidate Deployment Assets:** Move `ansible/`, `helm/`, `k8s/`, `packaging/`, `systemd/` and `docker-compose*.yml` into a single `deploy/` directory.
 - [x] **Centralize Example Configs:** Keep example configuration files under `configs/` and delete root-level duplicates (`config-*.toml`).
 - [x] **Archive Legacy TODO Files:** Remove `TODO-BACKUP.md` and `TODO-OLD.md` after migrating any useful notes to `TODO.md` or `TODO-ARCHIVE.md`.
 - [x] **Create `tests/` Directory:** Move `test_sqlite_datastore.rs` and similar helpers out of the root.
 - [x] **Document Sample Data:** Explain the purpose of `templates/`, `policies/` and `fixtures/` in the README.
