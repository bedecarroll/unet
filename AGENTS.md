# AGENTS.md – LLM Guidelines for μNet Development

> **Audience:** AI/LLM agents assisting with μNet development and maintenance  
> **Purpose:** Establish guidelines for code generation, system modification, and architectural decisions to maintain system integrity and code quality  
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
- **ALWAYS** maintain separation of concerns between `unet-core`, `unet-server`, and `unet-cli`
- **NEVER** bypass the `DataStore` trait - all data access must go through the abstraction layer
- **ALWAYS** understand the difference between desired state (user input) and derived state (SNMP polling)

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
- **ALWAYS** write comprehensive tests for new functionality
- **NEVER** commit code that doesn't compile or pass tests
- **ALWAYS** keep Rust files under 300 lines (absolute maximum 500 lines)
- **ALWAYS** split large files into smaller, focused modules

---

## Project Structure

### Cargo Workspace Structure

```
unet/
├── crates/
│   ├── unet-core/     # SHARED LIBRARY - models, datastore, policy, SNMP
│   ├── unet-server/   # BINARY - Axum API + background tasks
│   ├── unet-cli/      # BINARY - Clap CLI interface
│   ├── migrations/    # LIBRARY - SeaORM database migrations
│   └── config-slicer/ # LIBRARY - hierarchical config diffing
```

### Development Tools (mise.toml)

- **linting:** `mise run lint` - runs typos, clippy, and fmt checks
- **fixing:** `mise run lint-fix` - auto-fixes linting issues (typos, clippy, formatting)
- **testing:** `mise run test` - runs unit tests with coverage

### Example Development Workflow

```bash
# 1. Make your changes
vim crates/unet-core/src/models/node.rs

# 2. Run tests to ensure functionality works
mise run test

# 3. Auto-fix linting issues
mise run lint-fix

# 4. Check if any manual fixes are needed
mise run lint

# 5. Commit clean code
git add -A && git commit -m "feat: add node validation logic"
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

## Testing Requirements

### Test Coverage

- **ALL** new functionality MUST have unit tests
- **ALL** new API endpoints MUST have integration tests
- **ALL** new CLI commands MUST have E2E tests
- **ALL** policy rules MUST have evaluation tests

### Test Patterns

```rust
// CORRECT: Proper async test setup
#[tokio::test]
async fn test_node_creation() {
    let store = setup_test_datastore().await;
    let node = create_test_node(&store).await.unwrap();
    assert_eq!(node.name, "test-node");
    cleanup_test_data(&store).await;
}
```

---

## Documentation Standards

### Code Documentation

- **ALWAYS** document public APIs with rustdoc
- **NEVER** document obvious getter/setter methods
- **ALWAYS** include examples in complex function documentation
- **NEVER** use outdated or misleading comments

### Project Documentation

Documentation is maintained in `docs/src/` using mdBook:

- **ALWAYS** update relevant documentation when changing behavior
- **NEVER** leave documentation stale after code changes
- **ALWAYS** maintain the established documentation structure

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

- [ ] All tests pass (`mise run test`)
- [ ] Code is properly formatted and lint-free (`mise run lint`)
- [ ] No dead code or unused variables
- [ ] Documentation is updated if behavior changed
- [ ] Security implications are considered
- [ ] Integration with existing systems is verified

---

## Forbidden Operations

### Absolutely Never Do These

- **NEVER** add clippy allows or change clippy lint levels
- **NEVER** leave dead code or unused variables in the codebase
- **NEVER** expose internal database structure to API consumers
- **NEVER** bypass the policy engine for configuration changes
- **NEVER** store credentials in version control
- **NEVER** ignore SeaORM entity validation
- **NEVER** implement custom crypto - use established crates
- **NEVER** create SQL injection vulnerabilities
- **NEVER** ignore network timeouts in production code

---

## When In Doubt

1. **Read the Architecture Document** - `docs/src/architecture.md`
2. **Check Existing Patterns** - Look for similar implementations in the codebase
3. **Run the Full Test Suite** - `mise run test`
4. **Review Security Implications** - Network automation requires extra care
5. **Ask for Human Review** - Complex changes need human oversight

---

## Emergency Procedures

If you encounter:

- **Test failures**: Stop immediately and investigate root cause
- **Security concerns**: Halt development and flag for human review
- **Performance degradation**: Profile and optimize before proceeding
- **Breaking changes**: Ensure proper migration path exists
- **Dependency conflicts**: Resolve through established dependency matrix

Remember: **μNet manages critical network infrastructure. Code quality and security are non-negotiable.**

---

*This document is living and should be updated as the codebase evolves. All changes to AGENTS.md require review by senior maintainers.*