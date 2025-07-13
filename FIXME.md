# Code Quality Issues and Fix Guidelines

> **IMPORTANT**: Changing clippy levels, adding `#[allow(...)]` attributes, or leaving dead code/unused "placeholder" functions/unused variables is **EXPRESSLY FORBIDDEN**. We must fix all underlying issues properly.

This document outlines code quality issues found during analysis and provides detailed steps for fixing each category of problems.

## Task Complexity Rankings (Least to Most Complex)

### 🟢 **SIMPLE** (1-2 hours each)
1. **Hardcoded Values - Network Configuration** - Replace magic numbers with constants
2. **Test Code Improvements** - Add descriptive `expect()` messages  
3. **SNMP Address Parsing** - Create parsing helper function
4. **Error Conversion Pattern** - Create `From` trait implementations

### 🟡 **MODERATE** (3-8 hours each)
5. **Configuration Parsing Issues** - Replace `unwrap()` with proper error handling
6. **Database Transaction Patterns** - Create transaction helper functions
7. **Error Codes and Messages** - Centralize error handling system
8. **Database and Performance Tuning** - Create configuration constants
9. **Split: `/crates/unet-core/src/policy/tests.rs` (847 lines)** - Simple test organization
10. **Split: `/crates/unet-cli/src/commands/policy.rs` (531 lines)** - CLI command splitting
11. **Split: `/crates/unet-core/src/config.rs` (349 lines)** - Configuration module
12. **Split: `/crates/unet-cli/src/commands/links.rs` (323 lines)** - CLI command splitting
13. **Split: `/crates/unet-cli/src/commands/locations.rs` (317 lines)** - CLI command splitting

### 🟠 **COMPLEX** (1-3 days each)
14. **Split: `/crates/unet-core/src/models/derived.rs` (804 lines)** - Well-defined boundaries
15. **Split: `/crates/unet-core/src/snmp.rs` (639 lines)** - Core functionality with clear separation
16. **Split: `/crates/unet-core/src/snmp/poller.rs` (622 lines)** - Polling logic separation
17. **Split: `/crates/unet-core/src/policy/loader.rs` (616 lines)** - Policy loading logic
18. **Split: `/crates/unet-core/src/error.rs` (505 lines)** - Error type organization
19. **Split: `/crates/unet-core/src/snmp/oids.rs` (443 lines)** - OID management
20. **Split: `/crates/unet-core/src/policy/parser.rs` (434 lines)** - Parser logic
21. **Split: `/crates/unet-server/src/handlers/policies.rs` (393 lines)** - API handlers
22. **Split: `/crates/unet-core/src/policy_integration.rs` (391 lines)** - Integration logic
23. **Split: `/crates/unet-server/src/handlers/nodes.rs` (320 lines)** - API handlers
24. **Split: `/crates/unet-cli/src/commands/nodes.rs` (1,051 lines)** - Large CLI refactoring
25. **SNMP Session Management** - Connection pooling and lifecycle
26. **Complex Function Signatures** - API redesign for parameter objects

### 🔴 **VERY COMPLEX** (3-7 days each)
27. **Datastore Integration Gaps** - Complete API implementations (10 TODO items)
28. **Missing API Features** - Status tracking and evaluation systems
29. **Split: `/crates/unet-core/src/models.rs` (2,689 lines)** - Core model refactoring
30. **Split: `/crates/unet-core/src/policy/evaluator.rs` (2,251 lines)** - Complex policy engine
31. **Large Trait Interfaces** - DataStore trait decomposition
32. **Split: `/crates/unet-core/src/datastore.rs` (2,740 lines)** - Massive trait refactoring

### 🟣 **EXTREMELY COMPLEX** (1-4 weeks each)
33. **SNMP Implementation Stubs** - Complete SNMP protocol implementation
34. **Advanced Features - Environmental Metrics** - New feature development
35. **Advanced Features - Git Repository Integration** - External service integration  
36. **Advanced Features - Policy Orchestration** - Complex business logic

---

## Recommended Implementation Order

### **Week 1: Quick Wins (Items 1-8)**
Focus on simple constant replacements and basic error handling improvements. These provide immediate code quality improvements with minimal risk.

### **Week 2: Module Organization (Items 9-13)**  
Tackle smaller file splits and configuration improvements. Build confidence with file splitting process.

### **Week 3-4: Medium Complexity (Items 14-26)**
Work on larger file splits and system improvements. Establish patterns for complex refactoring.

### **Month 2: Core Refactoring (Items 27-32)**
Address the major architectural changes and large file splits. These require careful planning and testing.

### **Month 3+: New Feature Development (Items 33-36)**
Implement missing core functionality and advanced features. These are new development rather than refactoring.

---

## Complexity Analysis Rationale

**Simple tasks** involve straightforward code changes with minimal dependencies and clear solutions.

**Moderate tasks** require more planning but have well-defined boundaries and existing patterns to follow.

**Complex tasks** involve significant refactoring with multiple dependencies and potential breaking changes.

**Very Complex tasks** require architectural changes, multiple file coordination, and extensive testing.

**Extremely Complex tasks** involve new feature development, external integrations, or fundamental system changes.

---

## 1. TODO/FIXME Comments (21 items found)

### Priority: HIGH - Core Functionality Missing

#### SNMP Implementation Stubs
**Files affected:**
- `/crates/unet-core/src/snmp.rs:259` - SNMP GET operation
- `/crates/unet-core/src/snmp.rs:305` - SNMP GETNEXT operation

**Current state:** Placeholder functions returning mock data
**Fix steps:**
1. Replace TODO comments with actual `snmp2` crate integration
2. Implement proper SNMP GET operations using the established session management
3. Add comprehensive error handling for network timeouts, authentication failures
4. Add unit tests for SNMP operations
5. Validate against real SNMP devices in integration tests

#### Datastore Integration Gaps
**Files affected:**
- `/crates/unet-server/src/handlers/links.rs` (5 TODOs)
- `/crates/unet-server/src/handlers/locations.rs` (5 TODOs)

**Current state:** All CRUD operations return placeholder responses
**Fix steps:**
1. Remove all TODO comments and placeholder return statements
2. Integrate with actual DataStore trait methods
3. Implement proper error handling and validation
4. Add database constraints and foreign key relationships
5. Create comprehensive API tests for all endpoints

#### Missing API Features
**Files affected:**
- `/crates/unet-cli/src/commands/nodes.rs:685,909` - Polling task status
- `/crates/unet-server/src/handlers/nodes.rs:94` - Node status fetching
- `/crates/unet-server/src/handlers/policies.rs:390,391` - Policy evaluation tracking

**Fix steps:**
1. Implement polling task status tracking in DataStore trait
2. Add database tables for tracking policy evaluation history
3. Create background tasks for scheduled policy evaluations
4. Add CLI commands for viewing evaluation results
5. Implement proper status aggregation and reporting

#### Advanced Features
**Files affected:**
- `/crates/unet-core/src/models/derived.rs:529` - Environmental metrics
- `/crates/unet-core/src/policy/loader.rs:281` - Git repository integration
- `/crates/unet-core/src/policy_integration.rs:305` - Policy orchestration

**Fix steps:**
1. Design environmental metrics schema (temperature, humidity, power)
2. Implement Git repository cloning and syncing for policy files
3. Add proper policy orchestration with conflict resolution
4. Create monitoring and alerting for environmental thresholds
5. Add comprehensive documentation for new features

---

## 2. Panic-Prone Code (~30 unwrap() calls)

### Priority: MEDIUM - Improve Error Handling

#### Configuration Parsing Issues
**Files affected:**
- `/crates/unet-core/src/config.rs:320,323,325,327`

**Current issues:** Using `unwrap()` for file operations and path conversion
**Fix steps:**
1. Replace all `unwrap()` calls with proper `Result` propagation
2. Create custom error types for configuration parsing failures
3. Add validation for file paths and permissions
4. Implement graceful fallbacks for missing configuration files
5. Add comprehensive error messages for debugging

#### Test Code Improvements
**Files affected:** Multiple test files using `unwrap()` and `expect()`

**Fix steps:**
1. Replace `unwrap()` with descriptive `expect()` messages in tests
2. Use `assert!` macros with custom messages for better test failures
3. Create test helper functions for common setup patterns
4. Add proper cleanup for test resources (temp files, databases)

#### SNMP Address Parsing
**Files affected:**
- `/crates/unet-core/src/snmp.rs:194,628`
- `/crates/unet-core/src/policy_integration.rs:352`

**Fix steps:**
1. Create a `SocketAddr` parsing helper function with proper error handling
2. Validate IP addresses and port ranges
3. Add configuration options for default SNMP ports
4. Implement address resolution for hostnames
5. Add comprehensive input validation

---

## 3. Large Files/Functions

### Priority: MEDIUM - Improve Maintainability

#### Datastore.rs (2,740 lines)
**Issue:** Single trait with too many methods
**Fix steps:**
1. Split into multiple trait files:
   - `core_datastore.rs` - Basic CRUD operations
   - `policy_datastore.rs` - Policy-related operations  
   - `derived_datastore.rs` - Derived state operations
   - `admin_datastore.rs` - Administrative operations
2. Use trait composition to maintain single interface
3. Move implementation details to separate files
4. Add comprehensive documentation for each trait
5. Create integration tests for trait boundaries

#### Models.rs (2,689 lines)
**Issue:** All data models in single file
**Fix steps:**
1. Split into logical modules:
   - `models/core.rs` - Node, Location, Link
   - `models/policy.rs` - Policy-related models
   - `models/derived.rs` - Keep existing derived state models
   - `models/config.rs` - Configuration models
2. Use `mod.rs` to re-export public interfaces
3. Group related models and their implementations
4. Add module-level documentation
5. Ensure proper dependency ordering

#### Policy Evaluator (2,251 lines)
**Issue:** Complex policy evaluation logic in single file
**Fix steps:**
1. Extract action executors to separate files:
   - `policy/actions/assert.rs`
   - `policy/actions/set.rs` 
   - `policy/actions/template.rs`
2. Move orchestration logic to `policy/orchestrator.rs`
3. Split evaluation logic from execution logic
4. Create policy execution pipeline with clear stages
5. Add comprehensive unit tests for each component

#### CLI Nodes Commands (1,051 lines)
**Issue:** All node commands in single file
**Fix steps:**
1. Split into command-specific files:
   - `commands/nodes/create.rs`
   - `commands/nodes/list.rs`
   - `commands/nodes/show.rs`
   - `commands/nodes/update.rs`
   - `commands/nodes/delete.rs`
2. Create shared utilities in `commands/nodes/common.rs`
3. Use consistent error handling patterns across commands
4. Add command-specific validation and help text
5. Create integration tests for each command

---

## 4. Duplicate Code Patterns

### Priority: MEDIUM - Reduce Maintenance Burden

#### Error Conversion Pattern
**Issue:** Repeated `.map_err(|e| ServerError::Internal(e.to_string()))` (6+ occurrences)
**Fix steps:**
1. Create error conversion helper function:
   ```rust
   impl From<DataStoreError> for ServerError {
       fn from(err: DataStoreError) -> Self {
           match err {
               DataStoreError::NotFound { .. } => ServerError::NotFound(err.to_string()),
               DataStoreError::ValidationError { .. } => ServerError::Validation(err.to_string()),
               _ => ServerError::Internal(err.to_string()),
           }
       }
   }
   ```
2. Replace all manual error conversion calls
3. Add specific error mapping for common cases
4. Create unit tests for error conversion logic
5. Document error handling patterns in AGENTS.md

#### Database Transaction Patterns
**Files affected:** Multiple handlers using similar transaction patterns
**Fix steps:**
1. Create transaction helper macros or functions
2. Implement standard rollback and commit patterns
3. Add automatic retry logic for transient failures
4. Create integration tests for transaction behavior
5. Document transaction best practices

#### SNMP Session Management
**Files affected:** Repeated session creation and management code
**Fix steps:**
1. Create SNMP session pool with connection reuse
2. Implement automatic session cleanup and timeout handling
3. Add session health checking and recovery
4. Create comprehensive SNMP integration tests
5. Add monitoring for SNMP connection metrics

---

## 5. Hardcoded Values/Magic Numbers

### Priority: MEDIUM - Improve Configuration Management

#### Network Configuration
**Issues found:**
- `127.0.0.1:161` - SNMP test addresses (2 occurrences)
- `8080` - Default server port (2 occurrences)  
- `161` - SNMP port (3 occurrences)

**Fix steps:**
1. Create configuration constants file:
   ```rust
   pub mod defaults {
       pub const DEFAULT_SERVER_PORT: u16 = 8080;
       pub const SNMP_DEFAULT_PORT: u16 = 161;
       pub const SNMP_TRAP_PORT: u16 = 162;
       pub const LOCALHOST: &str = "127.0.0.1";
   }
   ```
2. Replace all hardcoded values with named constants
3. Make values configurable through environment variables
4. Add validation for port ranges and IP addresses
5. Document configuration options in README

#### Database and Performance Tuning
**Issues found:** Various timeout and limit values scattered throughout code
**Fix steps:**
1. Centralize all performance-related constants
2. Create tuning profiles (development, production, testing)
3. Add runtime configuration for critical performance parameters
4. Implement configuration validation and bounds checking
5. Add performance monitoring and alerting

#### Error Codes and Messages
**Issues found:** Hardcoded error codes and messages in multiple files
**Fix steps:**
1. Create error code constants with descriptive names
2. Implement internationalization-ready error message system
3. Add structured error responses with error codes
4. Create error message testing and validation
5. Document all error codes and their meanings

---

## 6. File Size Management (Target: 250-300 Lines Per File)

### Priority: HIGH - Code Organization and Maintainability

Files exceeding 300 lines significantly impact code readability, maintainability, and collaboration. The following files require immediate splitting:

#### Critical Files (>1000 lines) - Split Immediately

##### `/crates/unet-core/src/datastore.rs` (2,740 lines)
**Split into:**
```
datastore/
├── mod.rs (200 lines) - Main trait definition, error types, query types
├── types.rs (250 lines) - Filter, Sort, Pagination types and implementations  
├── helpers.rs (150 lines) - Helper functions for creating filters/sorts
├── csv.rs (750 lines) - Complete CSV implementation (move existing)
└── sqlite/
    ├── mod.rs (300 lines) - Main SQLite implementation
    ├── migrations.rs (250 lines) - Database schema migrations
    ├── queries.rs (400 lines) - Query building logic
    └── transactions.rs (350 lines) - Transaction handling
```

**Implementation steps:**
1. Create `datastore/` directory structure
2. Move trait definition to `mod.rs` with `pub use` re-exports
3. Extract type definitions to `types.rs`
4. Split SQLite implementation into logical modules
5. Update all import statements across the codebase
6. Ensure all tests pass after refactoring

##### `/crates/unet-core/src/models.rs` (2,689 lines)
**Split into:**
```
models/
├── mod.rs (150 lines) - Re-exports, basic enums (Lifecycle, DeviceRole, Vendor)
├── node.rs (400 lines) - Node struct and implementation
├── node_builder.rs (300 lines) - NodeBuilder implementation
├── location.rs (250 lines) - Location struct and implementation
├── link.rs (300 lines) - Link struct and implementation
├── validation.rs (400 lines) - Validation logic for all models
├── conversions.rs (300 lines) - From/Into implementations
├── derived.rs (804 lines) - Keep existing file (will be split separately)
└── tests.rs (585 lines) - All model tests
```

**Implementation steps:**
1. Extract basic enums to `mod.rs` first (other modules depend on these)
2. Move each struct and its implementations to separate files
3. Create shared validation module for common validation logic
4. Group conversion implementations in `conversions.rs`
5. Move all tests to `tests.rs` with proper module organization

##### `/crates/unet-core/src/policy/evaluator.rs` (2,251 lines)
**Split into:**
```
policy/evaluator/
├── mod.rs (250 lines) - Main PolicyEvaluator with basic evaluation
├── context.rs (200 lines) - EvaluationContext and related types
├── actions.rs (400 lines) - Action execution (SET, ASSERT, APPLY)
├── rollback.rs (350 lines) - Transaction and rollback logic
├── orchestration.rs (600 lines) - PolicyOrchestrator and batch processing
├── results.rs (250 lines) - Result types and aggregation
└── tests.rs (200 lines) - Core evaluator tests
```

**Implementation steps:**
1. Extract context types first (needed by other modules)
2. Split action execution into separate module
3. Move orchestration logic to dedicated file
4. Create results module for result aggregation
5. Preserve all public APIs through `mod.rs` re-exports

#### Large Files (500-1000 lines) - Split Next

##### `/crates/unet-cli/src/commands/nodes.rs` (1,051 lines)
**Split into:**
```
commands/nodes/
├── mod.rs (200 lines) - Command definitions and main execute function
├── args.rs (300 lines) - All argument structs and enums
├── crud.rs (350 lines) - Basic CRUD operations (add, list, show, update, delete)
└── status.rs (200 lines) - Status, monitoring, and metrics commands
```

##### `/crates/unet-core/src/policy/tests.rs` (847 lines)
**Split into:**
```
policy/tests/
├── mod.rs (50 lines) - Common test utilities
├── performance.rs (200 lines) - Performance tests
├── error_handling.rs (300 lines) - Error handling tests
└── grammar.rs (300 lines) - Grammar construct tests
```

##### `/crates/unet-core/src/models/derived.rs` (804 lines)
**Split into:**
```
models/derived/
├── mod.rs (150 lines) - NodeStatus and main types
├── system.rs (200 lines) - SystemInfo and related types
├── interfaces.rs (250 lines) - Interface status and statistics
└── metrics.rs (200 lines) - Performance and environmental metrics
```

#### Medium Files (300-700 lines) - Split When Time Permits

##### `/crates/unet-core/src/snmp.rs` (639 lines)
**Split into:**
```
snmp/
├── mod.rs (200 lines) - Main types and SnmpSession
├── client.rs (250 lines) - SnmpClient with connection pooling
├── values.rs (100 lines) - SnmpValue enum and conversions
└── config.rs (90 lines) - Configuration types
```

##### `/crates/unet-core/src/snmp/poller.rs` (622 lines)
**Split into:**
```
snmp/poller/
├── mod.rs (200 lines) - PollingTask and basic types
├── scheduler.rs (250 lines) - PollingScheduler implementation
└── handle.rs (172 lines) - PollingHandle and message handling
```

##### `/crates/unet-core/src/policy/loader.rs` (616 lines)
**Split into:**
```
policy/loader/
├── mod.rs (250 lines) - PolicyLoader main implementation
├── cache.rs (150 lines) - Caching logic and types
├── validation.rs (150 lines) - File validation logic
└── git.rs (66 lines) - Git integration (placeholder for future)
```

##### Additional Files Over 300 Lines:
- `/crates/unet-cli/src/commands/policy.rs` (531 lines)
- `/crates/unet-core/src/error.rs` (505 lines)
- `/crates/unet-core/src/snmp/oids.rs` (443 lines)
- `/crates/unet-core/src/policy/parser.rs` (434 lines)
- `/crates/unet-server/src/handlers/policies.rs` (393 lines)
- `/crates/unet-core/src/policy_integration.rs` (391 lines)
- `/crates/unet-core/src/config.rs` (349 lines)
- `/crates/unet-cli/src/commands/links.rs` (323 lines)
- `/crates/unet-server/src/handlers/nodes.rs` (320 lines)
- `/crates/unet-cli/src/commands/locations.rs` (317 lines)

### Implementation Priority and Timeline

#### Phase 1: Foundation Files (Week 1)
1. **models.rs** - Core types used everywhere, split first
2. **datastore.rs** - Core trait definitions and implementations

#### Phase 2: Complex Logic (Week 2)  
3. **policy/evaluator.rs** - Complex evaluation engine
4. **snmp.rs** - Core SNMP functionality

#### Phase 3: Specialized Modules (Week 3)
5. **models/derived.rs** - Derived state models
6. **snmp/poller.rs** - Polling implementation
7. **policy/loader.rs** - Policy loading logic

#### Phase 4: Supporting Files (Week 4)
8. **commands/nodes.rs** - CLI commands
9. **policy/tests.rs** - Test organization
10. **Additional files over 300 lines** - Based on priority and usage

### File Splitting Guidelines

#### Mandatory Requirements:
1. **Preserve Public APIs** - All `pub` items must remain accessible at same import paths
2. **Use Re-exports** - `pub use` statements in `mod.rs` to maintain compatibility
3. **Logical Boundaries** - Split along natural functional boundaries (structs, impls, features)
4. **Test Organization** - Related tests should be in same module as implementation
5. **Documentation** - Update module documentation and add navigation comments

#### Implementation Steps for Each File:
1. **Analyze Dependencies** - Identify what other modules import from this file
2. **Create Module Structure** - Design directory structure and `mod.rs`
3. **Extract Core Types** - Move fundamental types to `mod.rs` first
4. **Split Implementation** - Move implementations to logical modules
5. **Update Imports** - Fix all import statements across codebase
6. **Verify Tests** - Ensure all tests pass after refactoring
7. **Update Documentation** - Add module docs and update references

#### Quality Gates for File Splitting:
- All files must be ≤300 lines after splitting
- No breaking changes to public APIs
- All tests must pass
- `cargo clippy` must pass without warnings
- Documentation must be updated
- No unused imports or dead code

---

## 7. Architecture and Design Issues

### Priority: LOW-MEDIUM - Long-term Improvements

#### Large Trait Interfaces
**Issue:** DataStore trait has too many methods (40+ methods)
**Fix steps:**
1. Apply Interface Segregation Principle
2. Create focused traits for specific responsibilities
3. Use trait composition for complex operations
4. Add default implementations where appropriate
5. Create trait documentation with usage examples

#### Complex Function Signatures
**Issue:** Some functions have many parameters
**Fix steps:**
1. Create parameter objects for complex operations
2. Use builder patterns for optional parameters
3. Implement fluent APIs for common operations
4. Add parameter validation and documentation
5. Create helper functions for common parameter combinations

---

## Implementation Priority

### Phase 1 (Critical - Fix Immediately)
1. Implement actual SNMP operations (remove TODO stubs)
2. Complete datastore integration for links/locations handlers
3. Replace dangerous `unwrap()` calls in production code paths

### Phase 2 (Important - Next Sprint)
1. Create error conversion helpers to reduce duplication
2. Split large files into logical modules
3. Centralize configuration constants

### Phase 3 (Improvement - Following Sprint)
1. Add comprehensive error handling and validation
2. Create performance tuning configuration
3. Implement missing advanced features (Git integration, environmental metrics)

### Phase 4 (Polish - Future Releases)
1. Refactor large functions and complex interfaces
2. Add comprehensive monitoring and alerting
3. Optimize performance and resource usage

---

## Testing Requirements

Every fix must include:
1. **Unit tests** for new functionality
2. **Integration tests** for API endpoints
3. **Error case testing** for all error paths
4. **Performance tests** for critical operations
5. **Documentation updates** in relevant files

## Quality Gates

Before marking any issue as "fixed":
1. All tests must pass
2. `cargo clippy` must pass without warnings
3. `cargo fmt` must pass
4. No unused code or variables remain
5. All TODO comments addressed or moved to GitHub issues
6. Documentation updated to reflect changes

---

**Remember**: The goal is to eliminate all code smells permanently, not to suppress warnings. Each fix should improve code quality, maintainability, and reliability.