# test-support (μNet)

Shared test utilities for the μNet workspace:

- In-memory SQLite connection with entity-based schema.
- Global connection reuse for faster tests.
- SAVEPOINT helpers for per-test rollback.
- One-time tracing initialization for tests.

This crate is for internal testing only; it contains no production code.

