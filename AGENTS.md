# AGENTS.md – LLM Guidelines for μNet Development

> **Audience:** AI/LLM agents assisting with μNet development and maintenance.  
> **Purpose:** Establish strict guidelines for code generation, system modification, and architectural decisions to maintain system integrity and code quality.  
> **Prerequisites:** Read `docs/src/01_architecture.md` and `docs/src/12_onboarding.md` before making any changes.

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
- **Benefits:**
  - Single source of truth for all LLM development guidelines
  - Automatic synchronization without manual copying
  - Version control treats symlink as a single entity
- **Cross-Platform Note:** Symlinks work on Linux/macOS/WSL and are supported by Git/jj

---

## Core Principles

### 0. PROJECT INTEGRITY (HIGHEST PRIORITY)

- **NEVER** claim work is complete unless it demonstrably functions
- **NEVER** mark milestones complete with incomplete tasks
- **ALWAYS** verify implementations exist before marking tasks complete
- **ALWAYS** require human approval before milestone advancement

### 1. Architecture Respect

- **NEVER** violate the established component boundaries defined in `docs/src/01_architecture.md`
- **ALWAYS** maintain separation of concerns between `unet-core`, `unet-server`, and `unet-cli`
- **NEVER** bypass the `DataStore` trait - all data access must go through the abstraction layer
- **ALWAYS** understand the difference between desired state (user input) and derived state (SNMP polling)

### 2. Rust Ecosystem Compliance

- **ALWAYS** use the established dependencies listed in the technology decision matrix
- **NEVER** introduce new major dependencies without explicit approval
- **ALWAYS** maintain async/await patterns consistently throughout the codebase
- **NEVER** use blocking operations in async contexts

### 3. Incremental Development & Task Management

- **ALWAYS** complete tasks one at a time and wait for instructions to continue
- **ALWAYS** update TODO.md file as tasks are completed with granular deliverable tracking
- **ALWAYS** check off individual deliverables as they are implemented and verified
- **NEVER** mark high-level tasks complete without completing ALL sub-deliverables
- **ALWAYS** make the smallest possible change to achieve the desired outcome
- **NEVER** rewrite large portions of code without explicit permission
- **ALWAYS** maintain backward compatibility unless explicitly instructed otherwise
- **NEVER** modify unrelated code during focused tasks
- **ALWAYS** stop after completing each major task to allow for review and direction
- **NEVER** proceed to the next milestone without explicit approval

---

## Cargo Workspace Structure

### Crate Boundaries

```
unet/
├── crates/
│   ├── unet-core/     # SHARED LIBRARY - models, datastore, policy, template
│   ├── unet-server/   # BINARY - Axum API + background tasks
│   ├── unet-cli/      # BINARY - Clap CLI interface
│   └── config-slicer/ # LIBRARY - hierarchical config diffing
```

### Modification Rules

- **unet-core changes**: Require unit tests for all new functionality
- **unet-server changes**: Require integration tests for new endpoints
- **unet-cli changes**: Require E2E tests for new commands
- **config-slicer changes**: Require both unit tests and CLI integration tests

---

## Database & Data Layer

### SeaORM Usage

- **ALWAYS** create migrations for schema changes using `sea-orm-cli migrate generate`
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

### Custom Data Field Usage

- **ALWAYS** use `custom_data: Value` for experimental attributes
- **NEVER** add new columns for one-off use cases
- **ALWAYS** plan promotion path from `custom_data` to proper schema

---

## Policy Engine Guidelines

### DSL Safety Rules

- **NEVER** allow arbitrary code execution in policy rules
- **ALWAYS** validate DSL syntax before storing policies
- **NEVER** modify the Pest grammar without comprehensive testing
- **ALWAYS** maintain backward compatibility in policy evaluation

### Policy Actions

```rust
// CORRECT: Extend existing Action enum
pub enum Action {
    Assert(String),
    Set(String, Value),
    ApplyTemplate(String),
    // Add new actions here
}

// INCORRECT: Create parallel action systems
```

---

## Template Engine Guidelines

### MiniJinja Integration

- **ALWAYS** use the centralized template environment loader
- **NEVER** create separate MiniJinja environments
- **ALWAYS** validate template syntax before storage
- **NEVER** allow template access to system functions

### Template Security

- **NEVER** expose sensitive data in template context
- **ALWAYS** sanitize user input passed to templates
- **NEVER** allow templates to execute arbitrary code
- **ALWAYS** use the established filter system for custom functions

---

## HTTP API Standards

### Axum Route Patterns

```rust
// CORRECT: Follow established patterns
async fn create_node(
    State(app_state): State<AppState>,
    Json(payload): Json<CreateNodeRequest>,
) -> Result<Json<Node>, ApiError> {
    // Implementation
}

// INCORRECT: Different parameter patterns
async fn create_node(req: Json<CreateNodeRequest>) -> Json<Node> {
    // Missing error handling and state
}
```

### Error Handling

- **ALWAYS** use the established `ApiError` type
- **NEVER** return raw error strings to clients
- **ALWAYS** log errors appropriately (info/warn/error levels)
- **NEVER** expose internal implementation details in error messages

---

## Background Tasks & Async Patterns

### Tokio Task Management

- **ALWAYS** use structured concurrency patterns
- **NEVER** spawn unconstrained tasks without cancellation
- **ALWAYS** handle task cancellation gracefully
- **NEVER** use blocking operations in async tasks

### SNMP Polling Guidelines

- **ALWAYS** use the established `snmp2` crate patterns
- **NEVER** add synchronous SNMP calls
- **ALWAYS** handle SNMP timeouts and retries
- **NEVER** poll faster than configured intervals

---

## Testing Requirements

### Test Coverage Mandate

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

// INCORRECT: Missing async setup
#[test]
fn test_node_creation() {
    // Synchronous test for async code
}
```

### Test Data Management

- **ALWAYS** use isolated test databases
- **NEVER** rely on shared test state
- **ALWAYS** clean up test data after tests
- **NEVER** use production data in tests

---

## TODO.md Management

### Progress Tracking Requirements

- **ALWAYS** update TODO.md immediately after completing any task
- **NEVER** mark a task as complete unless it fully meets acceptance criteria
- **ALWAYS** update task status when starting work (mark as "in progress")
- **NEVER** proceed to subsequent tasks without completing dependencies
- **ALWAYS** document any blockers or issues encountered in TODO.md
- **NEVER** skip TODO.md updates even for small changes

### CRITICAL: Task Completion Verification Protocol

Before marking ANY task as complete, you MUST verify:

1. **Functional verification:** The described functionality actually works
2. **Code verification:** Implementation exists and compiles
3. **Test verification:** Required tests pass
4. **Documentation verification:** Changes are documented
5. **TODO.md granular tracking:** ALL sub-tasks within the task are marked complete

### MANDATORY: Granular Task Tracking in TODO.md

EVERY task has detailed sub-tasks (deliverables) that MUST be individually tracked:

**CORRECT approach:**

```markdown
- [x] **M1.4.2** Node CRUD commands ✅ **COMPLETED**
  - **Deliverables:**
    - [x] Implement `unet nodes add` with full validation ✅
    - [x] Implement `unet nodes list` with filtering ✅
    - [x] Implement `unet nodes show` with detailed output ✅
    - [x] Implement `unet nodes update` with partial updates ✅
    - [x] Implement `unet nodes delete` with confirmation ✅
    - [x] Add comprehensive error handling for all commands ✅
    - [x] Add integration tests for all CRUD operations ✅
```

**INCORRECT approach (NEVER DO THIS):**

```markdown
- [x] **M1.4.2** Node CRUD commands ✅ **COMPLETED**
  - **Deliverables:**
    - [ ] Implement `unet nodes add` with full validation
    - [ ] Implement `unet nodes list` with filtering
    - [x] Implement `unet nodes show` with detailed output ✅
    - [ ] Implement `unet nodes update` with partial updates
    - [ ] Implement `unet nodes delete` with confirmation
    - [ ] Add comprehensive error handling for all commands
    - [ ] Add integration tests for all CRUD operations
```

**VIOLATION EXAMPLES TO NEVER REPEAT:**

- ❌ Marking "CLI CRUD operations" complete when only stubs exist
- ❌ Marking "CI/CD pipeline" complete when workflows aren't implemented
- ❌ Marking "API endpoints" complete when handlers return empty responses
- ❌ Claiming milestone completion with multiple incomplete tasks
- ❌ Marking high-level task complete when sub-tasks remain incomplete
- ❌ Skipping granular TODO.md updates for individual deliverables

### Task Completion Protocol

```markdown
- [x] **M0.1.1** Initialize Cargo workspace in `/unet` ✅ COMPLETED
  - **Status:** All workspace-level dependencies configured
  - **Validation:** `cargo check --workspace` succeeds
  - **Notes:** Simplified dependencies to avoid OpenSSL issues initially
```

### Milestone Tracking

- **ALWAYS** mark milestones as complete only when ALL acceptance criteria are met
- **NEVER** advance to next milestone without explicit human approval
- **ALWAYS** include completion timestamps and validation notes
- **NEVER** leave partially completed milestones unmarked

### MANDATORY: Milestone Audit Protocol

Before claiming milestone completion, you MUST:

1. **Audit ALL tasks:** Review every task in the milestone section
2. **Verify granular completeness:** Confirm ALL sub-tasks/deliverables are checked ✅
3. **Verify functional completion:** Test that all functionality actually works
4. **Check acceptance criteria:** Confirm ALL criteria are genuinely met
5. **Document gaps:** If any tasks are incomplete, they MUST be moved to future milestones
6. **Create completion report:** Required summary report with validation evidence

**MILESTONE COMPLETION CHECKLIST:**

- [ ] ALL tasks AND their sub-deliverables are marked complete in TODO.md
- [ ] Every deliverable line item has been individually verified and checked ✅
- [ ] All tasks marked complete have been functionally verified
- [ ] All acceptance criteria have been tested and confirmed
- [ ] No placeholder implementations remain
- [ ] No incomplete sub-tasks exist under completed high-level tasks
- [ ] Completion report created in docs/src/reports/
- [ ] Human approval obtained before proceeding to next milestone

### TODO.md Granular Tracking Rules

**RULE 1:** No high-level task can be marked complete unless ALL its deliverables are complete
**RULE 2:** Each deliverable must be individually checked ✅ when implemented
**RULE 3:** Partial completion must be clearly visible (mix of [ ] and [x])
**RULE 4:** Milestone sections must show 100% deliverable completion before milestone is marked complete
**RULE 5:** Any incomplete deliverables must be moved to appropriate future milestones

### Milestone Completion Reporting

- **ALWAYS** create a summary report in `docs/src/reports/` when milestones are completed
- **ALWAYS** use the filename format: `$(TZ='America/Los_Angeles' date +"%Y-%m-%d_%H-%M-%S").md`
- **ALWAYS** use America/Los_Angeles timezone for all dates and times in reports
- **ALWAYS** include stakeholder-friendly language and technical summaries
- **NEVER** skip report creation for completed milestones
- **ALWAYS** update the documentation SUMMARY.md to include new reports
- **NEVER** use technical jargon without explanation in stakeholder reports
- **ALWAYS** format times as "YYYY-MM-DD HH:MM:SS PDT/PST" in report headers

---

## Documentation Standards

### mdBook Integration

- **ALWAYS** update relevant documentation when changing behavior
- **NEVER** leave documentation stale after code changes
- **ALWAYS** maintain the established documentation structure
- **NEVER** create duplicate documentation

### Code Documentation

- **ALWAYS** document public APIs with rustdoc
- **NEVER** document obvious getter/setter methods
- **ALWAYS** include examples in complex function documentation
- **NEVER** use outdated or misleading comments

---

## Git Workflow & CI/CD

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

### CI/CD Pipeline Respect

- **ALWAYS** ensure all CI checks pass before merging
- **NEVER** bypass CI/CD gates
- **ALWAYS** fix clippy warnings and formatting issues
- **NEVER** ignore test failures

---

## Security Guidelines

### Network Automation Security

- **NEVER** log or expose credentials in any form
- **ALWAYS** use secure credential storage mechanisms
- **NEVER** hard-code network device credentials
- **ALWAYS** validate all network device inputs

### SNMP Security

- **ALWAYS** use SNMPv3 when possible
- **NEVER** transmit community strings in plain text logs
- **ALWAYS** validate SNMP response data
- **NEVER** trust SNMP data without bounds checking

---

## Extension Points

### Adding New DataStore Implementations

```rust
// CORRECT: Implement the trait properly
impl DataStore for MyNewStore {
    async fn get_node(&self, id: &str) -> Result<Option<Node>, DataStoreError> {
        // Implementation
    }
    // ... other required methods
}

// INCORRECT: Partial implementation
impl DataStore for MyNewStore {
    async fn get_node(&self, id: &str) -> Result<Option<Node>, DataStoreError> {
        todo!("Implement later")
    }
}
```

### Adding New Policy Actions

- **ALWAYS** update the DSL grammar in `policy.pest`
- **NEVER** add actions without corresponding tests
- **ALWAYS** maintain evaluation performance
- **NEVER** break existing policy files

### Adding New Template Filters

- **ALWAYS** register filters in the centralized environment
- **NEVER** create filters with side effects
- **ALWAYS** document filter behavior
- **NEVER** create filters that access external resources

---

## Performance Guidelines

### Database Queries

- **ALWAYS** use appropriate indexes for queries
- **NEVER** use N+1 query patterns
- **ALWAYS** batch database operations when possible
- **NEVER** fetch unnecessary data from database

### Memory Management

- **ALWAYS** use streaming for large datasets
- **NEVER** load entire datasets into memory unnecessarily
- **ALWAYS** implement proper pagination for APIs
- **NEVER** ignore memory usage in background tasks

---

## Error Handling Patterns

### Result Types

```rust
// CORRECT: Proper error propagation
pub async fn process_node(node_id: &str) -> Result<ProcessedNode, ProcessingError> {
    let node = datastore.get_node(node_id).await?;
    let processed = apply_policies(&node).await?;
    Ok(processed)
}

// INCORRECT: Panic on errors
pub async fn process_node(node_id: &str) -> ProcessedNode {
    let node = datastore.get_node(node_id).await.unwrap();
    apply_policies(&node).await.unwrap()
}
```

### Logging Patterns

- **ALWAYS** use structured logging with appropriate levels
- **NEVER** log sensitive information
- **ALWAYS** include context in error logs
- **NEVER** spam logs with debug information in production

---

## Code Quality Checklist

Before submitting any code changes, ensure:

- [ ] All tests pass (`cargo test --workspace`)
- [ ] Code is properly formatted (`cargo fmt`)
- [ ] Clippy warnings are addressed (`cargo clippy`)
- [ ] Documentation is updated if behavior changed
- [ ] Security implications are considered
- [ ] Performance impact is evaluated
- [ ] Backward compatibility is maintained
- [ ] Error handling is comprehensive
- [ ] Logging is appropriate and secure
- [ ] Integration with existing systems is verified

---

## Forbidden Operations

### Absolutely Never Do These

- **NEVER** expose internal database structure to API consumers
- **NEVER** modify migration files after they've been applied
- **NEVER** bypass the policy engine for configuration changes
- **NEVER** store credentials in version control
- **NEVER** create endpoints without authentication considerations
- **NEVER** ignore SeaORM entity validation
- **NEVER** implement custom crypto - use established crates
- **NEVER** create SQL injection vulnerabilities
- **NEVER** ignore network timeouts in production code
- **NEVER** implement custom serialization for security-critical data

---

## When In Doubt

1. **Read the Architecture Document** - `docs/src/01_architecture.md`
2. **Check Existing Patterns** - Look for similar implementations in the codebase
3. **Run the Full Test Suite** - Ensure no regressions
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
