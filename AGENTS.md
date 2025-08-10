# AGENTS.md – LLM Guidelines for μNet Development

> **Audience:** AI/LLM agents assisting with μNet development and maintenance
> **Purpose:** Establish guidelines for code generation, system modification,
> and architectural decisions to maintain system integrity and code quality
> **Prerequisites:** Read `docs/src/architecture.md` before making any changes

---

## File Synchronization

### CLAUDE.md ↔ AGENTS.md Synchronization

- **Implementation:** `CLAUDE.md` is a filesystem symlink to `AGENTS.md`
- **Purpose:** Multiple LLM systems require identical guidance documents
- **Maintenance:**
  - **ALWAYS** edit `AGENTS.md` as the canonical source
  - **NEVER** edit `CLAUDE.md` directly (it's a symlink)
  - **VERIFY** symlink integrity: `ls -la CLAUDE.md` should show `CLAUDE.md -> AGENTS.md`
  - **RESTORE** if broken: `ln -sf AGENTS.md CLAUDE.md`

---

## Core Principles

### 1. Architecture Respect

- **NEVER** violate the established component boundaries defined in `docs/src/architecture.md`
- **ALWAYS** maintain separation of concerns between `unet-core`,
`unet-server`, and `unet-cli`
- **NEVER** bypass the `DataStore` trait - all data access must go through the
abstraction layer
- **ALWAYS** understand the difference between desired state (user input) and
derived state (SNMP polling)

### 2. Rust Ecosystem Compliance

- **ALWAYS** use the established dependencies from existing `Cargo.toml` files
- **NEVER** introduce new major dependencies without explicit approval
- **ALWAYS** maintain async/await patterns consistently throughout the codebase
- **NEVER** use blocking operations in async contexts

### 3. Code Quality Standards

- **ALWAYS** follow existing code style and patterns in the codebase
- **NEVER** add clippy allows or change clippy lint levels
- **ALWAYS** remove all dead code and unused variables
- **NEVER** leave placeholder implementations (`todo!()`, `unimplemented!()`)
- **NEVER** commit unused functions, commented code blocks, or stub implementations
- **NEVER** leave "example code for future development" in the codebase
- **ALWAYS** follow Test Driven Development (TDD) practices - see TDD section below
- **NEVER** commit code that doesn't compile or pass tests
- **ALWAYS** keep Rust files under 300 lines (absolute maximum 500 lines)
- **ALWAYS** split large files into smaller, focused modules
- **ALWAYS** run `mise run check-large-files` before commits to verify file sizes

### 4. Documentation Standards

- **NEVER** make qualitative judgments about code quality in documentation
- **ALWAYS** let code quality speak for itself through metrics and tests
- **NEVER** use subjective terms like "excellent", "solid", "clean", "high-quality"
- **ALWAYS** use objective statements with measurable criteria
- **NEVER** make claims about production-readiness without evidence
- **ALWAYS** focus on factual status, features, and completion percentages
- **NEVER** provide time estimates as they depend on external factors
- **ALWAYS** use implementation complexity estimates (simple, moderate, complex) if estimates are required
- **NEVER** promise delivery timelines or completion dates
- **ALWAYS** document public APIs with rustdoc
- **NEVER** document obvious getter/setter methods
- **ALWAYS** include examples in complex function documentation
- **NEVER** use outdated or misleading comments
- **ALWAYS** update relevant documentation when changing behavior
- **NEVER** leave documentation stale after code changes
- **ALWAYS** maintain the established documentation structure in `docs/src/` using mdBook
- **ALWAYS** use backticks around technical terms and library names

### 5. Tool Usage Patterns

- **ALWAYS** use Grep/Glob for code searches before making changes
- **NEVER** assume file locations - search first
- **ALWAYS** use Read tool before Edit/MultiEdit operations
- **ALWAYS** run `mise run lint` after code changes
- **NEVER** skip the TodoWrite tool for multi-step tasks
- **REFERENCE** `docs/src/developer_guide.md` for detailed implementation patterns and decision trees

---

## Test Driven Development (TDD)

μNet follows **strict Test Driven Development** practices. All code development MUST follow the Red-Green-Refactor cycle.

### TDD Principles

- **ALWAYS** write tests BEFORE writing any implementation code
- **NEVER** write implementation code without failing tests first
- **ALWAYS** follow the Red-Green-Refactor cycle:
  1. **Red**: Write a failing test that defines the desired behavior
  2. **Green**: Write the minimal code to make the test pass
  3. **Refactor**: Improve code quality while keeping tests passing

### TDD Workflow Requirements

- **ALWAYS** start development by writing a failing test
- **VERIFY** the test fails for the right reason before implementing
- **NEVER** skip the Red phase - confirm test failure first
- **ALWAYS** run tests after each micro-change during implementation
- **NEVER** implement more than needed to make tests pass
- **ALWAYS** refactor with confidence once tests are green

### Test Design Guidelines

- **ALWAYS** write clear, descriptive test names that explain the behavior
- **NEVER** write vague test names like `test_function()` or `it_works()`
- **ALWAYS** structure tests with Arrange-Act-Assert pattern
- **NEVER** test multiple behaviors in a single test function
- **ALWAYS** test edge cases and error conditions
- **NEVER** write tests that depend on external state or other tests

### TDD Coverage Requirements

- **ALL** new functionality MUST have tests written FIRST (before implementation)
- **ALL** tests MUST fail initially (Red phase verification required)
- **ALL** new API endpoints MUST have integration tests written before implementation
- **ALL** new CLI commands MUST have E2E tests written before implementation
- **ALL** policy rules MUST have evaluation tests written before implementation
- **NEVER** write implementation code without a failing test

### TDD Test Patterns

```rust
// CORRECT: TDD test written BEFORE implementation
#[tokio::test]
async fn test_node_calculates_uptime_correctly() {
    // Arrange
    let store = setup_test_datastore().await;
    let node = create_test_node_with_start_time(&store, "2024-01-01T00:00:00Z").await.unwrap();
    
    // Act
    let uptime = node.calculate_uptime().await.unwrap(); // This method doesn't exist yet!
    
    // Assert
    assert!(uptime.as_secs() > 0);
    cleanup_test_data(&store).await;
}

// CORRECT: Test edge cases first
#[tokio::test]
async fn test_node_uptime_handles_invalid_start_time() {
    // Write this test BEFORE implementing error handling
    let store = setup_test_datastore().await;
    let node = create_test_node_with_invalid_time(&store).await.unwrap();
    
    let result = node.calculate_uptime().await;
    assert!(result.is_err());
    cleanup_test_data(&store).await;
}
```

### Test-First Development Rules

- **ALWAYS** write the test name and structure first
- **ALWAYS** verify the test fails for the expected reason
- **NEVER** proceed to implementation until test failure is confirmed
- **ALWAYS** write minimal code to make tests pass
- **NEVER** over-implement beyond what tests require

---

## Before Making Changes

### Required Context Gathering

1. **Write failing tests first**: Define expected behavior through tests
2. **Read architecture document**: `docs/src/architecture.md`
3. **Search for existing patterns**: Use Grep to find similar implementations
4. **Check test patterns**: Look at existing tests for structure and naming
5. **Verify test failure**: Ensure tests fail for the right reason before implementing
6. **Verify dependencies**: Check `Cargo.toml` for available crates

---

## Decision Framework

When implementing features:

1. **Write failing tests first** to define expected behavior (TDD Red phase)
2. **Search existing codebase** for similar patterns and test structures
3. **Follow established conventions** over creating new ones  
4. **Prefer composition** over inheritance
5. **Default to explicit** over implicit behavior
6. **Favor readability** over cleverness
7. **Implement minimally** to make tests pass (TDD Green phase)
8. **Refactor confidently** with test coverage (TDD Refactor phase)

---

## Project Structure

### Cargo Workspace Structure

```text
unet/
├── crates/
│   ├── unet-core/     # SHARED LIBRARY - models, datastore, policy, SNMP
│   ├── unet-server/   # BINARY - Axum API + background tasks
│   ├── unet-cli/      # BINARY - Clap CLI interface
│   ├── migrations/    # LIBRARY - SeaORM database migrations
│   └── config-slicer/ # LIBRARY - hierarchical config diffing
```

### Development Tools (mise.toml)

- **linting:** `mise run lint` - lints and auto-fixes linting issues (typos, clippy, formatting)
- **testing:** `mise run test` - runs unit tests with nextest
- **coverage:** `mise run coverage` - generates code coverage reports with `llvm-cov`
- **file size check:** `mise run check-large-files` - identifies files exceeding size guidelines
- **LLM status (recommended):** `mise run status` - runs `clippy` and `coverage+tests`, writes full logs to `target/mise-logs/latest/` (overwrites on each run), and prints a concise summary with:
  - clippy error count (and first few error lines)
  - test summary (and top failing tests if any)
  - coverage TOTAL line

Deprecated tasks removed: `test-quiet`, `coverage-quiet`, and the `overview` alias — use `status`.

Notes:
- `status` uses the coverage run which executes the full test suite under instrumentation; you usually do not need to run `test` afterwards.
- All detailed stdout/stderr for clippy and coverage are saved under `target/mise-logs/latest/` and overwrite on each run to avoid disk growth.
 - The `status` header prints quick LLM tips with common `rg`/`grep` commands to extract failures, summaries, and coverage lines from the logs.

Note: `mise run test` and `mise run coverage` can be long-running and produce large amounts of output. For LLM workflows, prefer `mise run status`, which captures raw logs and prints a concise overview suitable for model contexts.

### Preferred Workflow

- **LLM overview:** Run `mise run status` to check clippy, run tests with coverage, and get a concise summary. Inspect `target/mise-logs/latest/` for full logs.
- **TDD loops:** Use `mise run test -- <filters>` (package and/or name patterns) to iterate quickly; confirm failures first, then implement minimal code.
- **Before commit:** Run `mise run lint` to auto-fix formatting/typos and surface clippy issues; run `mise run check-large-files` to enforce size guidelines.
- **CI-only tasks:** `ci-*` tasks are invoked by GitHub Actions; do not modify without coordinating with CI config.

### Nextest Usage Patterns

μNet uses `cargo-nextest` for faster test execution. **ALWAYS** use `mise run test` (which runs `cargo nextest run`) instead of `cargo test`. For LLM usage, prefer `mise run status` which executes coverage+tests once and summarizes output.

#### Nextest Syntax Differences

- **CORRECT nextest syntax:** `mise run test -- -p package_name test_filter`
- **INCORRECT:** `mise run test -- --exact test_name` (--exact doesn't exist in nextest)
- **CORRECT:** `mise run test -- -p unet-server test_evaluate_policies_all_nodes`
- **CORRECT:** `mise run test -- test_pattern` (runs all tests matching pattern across workspace)

#### Common Testing Commands

```bash
# Run all tests
mise run test

# LLM-friendly project status overview (concise console output)
mise run status

# Run tests for specific package
mise run test -- -p unet-server

# Run specific test by name pattern
mise run test -- -p unet-server test_process_node_evaluation

# Run tests matching pattern across workspace
mise run test -- policy_execution

# Run tests with verbose output
mise run test -- -v test_pattern

# Grepping logs from the latest status run
rg -n "^TOTAL" target/mise-logs/latest/coverage.log       # coverage summary
rg -n "^\s*Summary" target/mise-logs/latest/coverage.log # test summary
rg -n "^error" target/mise-logs/latest/clippy.log         # clippy errors
```

#### Key Nextest Features

- **Faster execution:** Parallel test execution by default
- **Better filtering:** Use test name patterns as positional arguments
- **No --exact flag:** Test filters are substring matches by default
- **Package filtering:** Use `-p package_name` to limit scope
- **Consistent syntax:** Same filtering works across packages and workspace

### Example TDD Development Workflow

```bash
# 1. ALWAYS start by writing a failing test first (RED phase)
vim crates/unet-core/src/models/tests/node.rs
# Write test: test_node_calculates_uptime_correctly()

# 2. Verify the test fails for the right reason
mise run test
# Expected: test should fail because function doesn't exist

# 3. Write minimal implementation to make test pass (GREEN phase)
vim crates/unet-core/src/models/node.rs
# Add just enough code to make the test pass

# 4. Run tests to confirm they pass
mise run test

# 5. Refactor while keeping tests green (REFACTOR phase)
vim crates/unet-core/src/models/node.rs
# Improve code quality, extract methods, etc.

# 6. Run tests after each refactoring step
mise run test

# 7. Check if any manual fixes are needed, will format code
mise run lint

# 8. Generate coverage report to ensure adequate test coverage
mise run coverage

# 9. Verify file sizes are within guidelines
mise run check-large-files
```

---

## Database & Data Layer

### SeaORM Usage

- **ALWAYS** create migrations for schema changes using SeaORM migration tools
- **NEVER** modify existing migration files - create new ones for changes
- **ALWAYS** implement both `up` and `down` migrations
- **NEVER** bypass SeaORM entities - use the generated models

### DataStore Trait Implementation

```rust
// CORRECT: Implement through DataStore trait
let node = datastore.get_node(&node_id).await?;

// INCORRECT: Direct database access
let node = Node::find_by_id(node_id).one(&db).await?;
```

---

## API Standards

### Axum Route Patterns

Follow established patterns in `unet-server/src/handlers/`:

```rust
// CORRECT: Follow established patterns
async fn create_node(
    State(app_state): State<AppState>,
    Json(payload): Json<CreateNodeRequest>,
) -> Result<Json<Node>, ApiError> {
    // Implementation
}
```

### Error Handling

- **ALWAYS** use the established `ApiError` type
- **NEVER** return raw error strings to clients
- **ALWAYS** log errors appropriately (info/warn/error levels)
- **NEVER** expose internal implementation details in error messages

---

## SNMP Integration

### Current Implementation

- **ALWAYS** use the established `csnmp` patterns in `unet-core/src/snmp/`
- **NEVER** add synchronous SNMP calls
- **ALWAYS** handle SNMP timeouts and retries
- **NEVER** poll faster than configured intervals

### Security Considerations

- **ALWAYS** use SNMPv3 when possible
- **NEVER** transmit community strings in plain text logs
- **ALWAYS** validate SNMP response data
- **NEVER** trust SNMP data without bounds checking

---

## Policy Engine

### DSL Implementation

- **NEVER** allow arbitrary code execution in policy rules
- **ALWAYS** validate DSL syntax before storing policies
- **NEVER** modify the Pest grammar without comprehensive testing
- **ALWAYS** maintain backward compatibility in policy evaluation

---

## Code Quality Patterns

### Numeric Literals

- **ALWAYS** add underscores to large numeric literals for readability
- **CORRECT:** `1_000_000_000` instead of `1000000000`
- **CORRECT:** `100_000` instead of `100000`

### Variable Naming

- **NEVER** use similar variable names in the same scope
- **CORRECT:** `node_a_relation` and `node_b_relation` instead of `node_a_rel` and `node_b_rel`
- **ALWAYS** use descriptive names that clearly distinguish purpose

### Function Signatures

- **NEVER** mark functions as `async` unless they use `await`
- **ALWAYS** make functions `const` when they don't mutate state
- **CORRECT:** `pub const fn update_stats(&self, _success: bool, _duration: Duration)`

### Resource Management

- **ALWAYS** drop significant resources explicitly when done
- **CORRECT:** Add `drop(data);` after using mutex guards in tests
- **NEVER** hold locks longer than necessary

### Test Assertions

- **NEVER** use `assert_eq!` with boolean literals
- **CORRECT:** `assert!(value)` instead of `assert_eq!(value, true)`
- **CORRECT:** `assert!(!value)` instead of `assert_eq!(value, false)`

### Float Comparisons

- **NEVER** use direct equality for floating-point comparisons
- **CORRECT:** `assert!((value - expected).abs() < f64::EPSILON)`
- **ALWAYS** use appropriate epsilon for the float type (f32::EPSILON vs f64::EPSILON)

### String Formatting

- **ALWAYS** use inline format arguments for readability
- **CORRECT:** `format!("node-{i}")` instead of `format!("node-{}", i)`
- **NEVER** borrow format results unnecessarily

### Pattern Matching

- **NEVER** use wildcard matches when only one variant remains
- **CORRECT:** `SnmpCredentials::UserBased { .. } =>` instead of `_ =>`
- **ALWAYS** be explicit about what you're matching

### Clone Usage

- **NEVER** clone values that can be moved
- **NEVER** write tests that clone without testing clone behavior meaningfully
- **ALWAYS** prefer moving values when the original isn't needed again

### Documentation Comments

- **ALWAYS** use backticks around technical terms and library names
- **CORRECT:** `//! Tests for \`SeaORM\` entities` instead of `//! Tests for SeaORM entities`

---

## Git Workflow

### Branch Naming

```bash
# CORRECT: Feature branches
feat/add-node-location-support
fix/snmp-timeout-handling
docs/update-architecture-guide

# INCORRECT: Vague branch names
feature
bugfix
update
```

### Commit Messages

- **ALWAYS** use conventional commit format
- **NEVER** make commits without clear descriptions
- **ALWAYS** include context in commit messages
- **NEVER** commit broken or untested code

---

## Code Quality Checklist

Before submitting any code changes, ensure:

- [ ] **TDD compliance**: All functionality was implemented test-first
- [ ] **Red phase verified**: Initial tests failed for the expected reasons
- [ ] All tests pass (`mise run test`)
- [ ] Code coverage is adequate (`mise run coverage` - maintain 80%+ coverage)
- [ ] Code is properly formatted and lint-free (`mise run lint`)
- [ ] No dead code, unused variables, or placeholder implementations
- [ ] All files are under 300 lines (`mise run check-large-files` shows no results)
- [ ] No commented-out code blocks or stub functions
- [ ] **Test coverage**: Every new function/method has corresponding tests
- [ ] **Test quality**: Tests have descriptive names and clear Arrange-Act-Assert structure
- [ ] Documentation is updated if behavior changed
- [ ] Security implications are considered
- [ ] Integration with existing systems is verified

---

## Forbidden Operations

### Absolutely Never Do These

- **NEVER** add clippy allows or change clippy lint levels
- **NEVER** leave dead code or unused variables in the codebase
- **NEVER** commit unused functions, even as "examples" or "stubs for future use"
- **NEVER** leave commented-out code blocks in commits
- **NEVER** commit placeholder implementations (`todo!()`, `unimplemented!()`, `panic!()`)
- **NEVER** violate TDD practices - see TDD section for complete requirements
- **NEVER** expose internal database structure to API consumers
- **NEVER** bypass the policy engine for configuration changes
- **NEVER** store credentials in version control
- **NEVER** ignore SeaORM entity validation
- **NEVER** implement custom crypto - use established crates
- **NEVER** create SQL injection vulnerabilities
- **NEVER** ignore network timeouts in production code
- **NEVER** commit files exceeding 500 lines (target: under 300 lines)

---

## When In Doubt

1. **STOP** - Don't guess or assume
2. **SEARCH** - Use Grep/Glob to find existing patterns
3. **READ** - Check related code and tests
4. **ASK** - Request clarification rather than proceeding
5. **TEST** - Verify assumptions with small experiments

---

## Emergency Procedures

If you encounter:

- **Test failures**: Stop immediately and investigate root cause
- **Security concerns**: Halt development and flag for human review
- **Performance degradation**: Profile and optimize before proceeding
- **Breaking changes**: Ensure proper migration path exists
- **Dependency conflicts**: Resolve through established dependency matrix

Remember: **μNet manages critical network infrastructure. Code quality and
security are non-negotiable.**

---

*This document is living and should be updated as the codebase evolves. All
changes to AGENTS.md require review by senior maintainers.*
