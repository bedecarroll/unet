# Code Quality Issues and Fix Guidelines

> **IMPORTANT**: Changing clippy levels, adding `#[allow(...)]` attributes, or leaving dead code/unused "placeholder" functions/unused variables is **EXPRESSLY FORBIDDEN**. We must fix all underlying issues properly.

> **✅ SIMPLE TASKS COMPLETED**: All 4 SIMPLE priority tasks have been completed:
> - ✅ Network constants centralized (`config::defaults`)
> - ✅ Test error messages improved (descriptive `expect()`)  
> - ✅ SNMP address parsing helpers added (`config::network`)
> - ✅ Error conversion patterns simplified (using `From` traits)

This document outlines code quality issues found during analysis and provides detailed steps for fixing each category of problems.

## Task Complexity Rankings (Least to Most Complex)

### 🟢 **SIMPLE** (1-2 hours each)
✅ **ALL SIMPLE TASKS COMPLETED**

### 🟡 **MODERATE** (3-8 hours each)
✅ **ALL MODERATE TASKS COMPLETED**

### 🟠 **COMPLEX** (1-3 days each)
✅ **ALL FILE SPLITTING COMPLEX TASKS COMPLETED:**
- ✅ Split: `/crates/unet-core/src/models/derived.rs` (804 lines) - Well-defined boundaries
- ✅ Split: `/crates/unet-core/src/snmp.rs` (642 lines) - Core functionality with clear separation
- ✅ Split: `/crates/unet-core/src/snmp/poller.rs` (623 lines) - Polling logic separation
- ✅ Split: `/crates/unet-core/src/policy/loader.rs` (616 lines) - Policy loading logic
- ✅ Split: `/crates/unet-core/src/error.rs` (505 lines) - Error type organization
- ✅ Split: `/crates/unet-core/src/snmp/oids.rs` (443 lines) - OID management
- ✅ Split: `/crates/unet-core/src/policy/parser.rs` (434 lines) - Parser logic
- ✅ Split: `/crates/unet-server/src/handlers/policies.rs` (393 lines) - API handlers
- ✅ Split: `/crates/unet-core/src/policy_integration.rs` (391 lines) - Integration logic
- ✅ Split: `/crates/unet-server/src/handlers/nodes.rs` (318 lines) - API handlers
- ✅ Split: `/crates/unet-cli/src/commands/nodes.rs` (1,051 lines) - Large CLI refactoring

**✅ ALL COMPLEX TASKS COMPLETED:**
- ✅ SNMP Session Management - Connection pooling and lifecycle implemented with real snmp2 integration
- ✅ Complex Function Signatures - API redesign completed with PolicyExecutionContext parameter objects

### 🔴 **VERY COMPLEX** (3-7 days each)
14. **Datastore Integration Gaps** - Complete API implementations (10 TODO items)
15. **Missing API Features** - Status tracking and evaluation systems
16. ✅ **Split: `/crates/unet-core/src/models.rs` (2,689 lines)** - Core model refactoring **COMPLETED**
17. ✅ **Split: `/crates/unet-core/src/policy/evaluator.rs` (2,251 lines)** - Complex policy engine **COMPLETED**
18. **Large Trait Interfaces** - DataStore trait decomposition
19. ✅ **Split: `/crates/unet-core/src/datastore.rs` (2,740 lines)** - Massive trait refactoring **COMPLETED**

### 🟣 **EXTREMELY COMPLEX** (1-4 weeks each)
20. ✅ **SNMP Implementation Stubs** - Complete SNMP protocol implementation **COMPLETED**
21. **Advanced Features - Environmental Metrics** - New feature development
22. **Advanced Features - Git Repository Integration** - External service integration  
23. **Advanced Features - Policy Orchestration** - Complex business logic

---

## Recommended Implementation Order

### **✅ Week 1: Quick Wins - COMPLETED**
~~Focus on simple constant replacements and basic error handling improvements. These provide immediate code quality improvements with minimal risk.~~

**COMPLETED TASKS:**
- ✅ Hardcoded Values - Network Configuration
- ✅ Test Code Improvements - Descriptive expect() messages  
- ✅ SNMP Address Parsing - Helper functions with validation
- ✅ Error Conversion Pattern - From trait implementations

### **✅ Week 2: Module Organization - COMPLETED**
~~Tackle smaller file splits and configuration improvements. Build confidence with file splitting process.~~

**COMPLETED TASKS:**
- ✅ Configuration Parsing Issues - Proper error handling implemented
- ✅ Database Transaction Patterns - Helper functions created
- ✅ Error Codes and Messages - Centralized error handling system
- ✅ Database and Performance Tuning - Configuration constants created
- ✅ Policy tests split into organized modules
- ✅ CLI policy commands split and organized
- ✅ Config.rs modularized into logical components

### **✅ Week 3-4: Complex File Splits (Items 1-11) - COMPLETED**
~~Work on larger file splits and system improvements. Establish patterns for complex refactoring.~~

**COMPLETED TASKS:**
- ✅ All 11 file splitting tasks completed with logical module boundaries
- ✅ All files now ≤300 lines as recommended
- ✅ Backward compatibility maintained through proper re-exports
- ✅ Build verification: `cargo check --all-targets --all-features` passes
- ✅ Linting verification: `cargo clippy` passes without warnings

**✅ ALL NON-FILE-SPLITTING COMPLEX TASKS COMPLETED:**
- ✅ SNMP Session Management - Connection pooling and lifecycle
- ✅ Complex Function Signatures - API redesign for parameter objects

### **Month 2: Core Refactoring (Items 14-19)**
Address the major architectural changes and large file splits. These require careful planning and testing.

### **Month 3+: New Feature Development (Items 20-23)**
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

#### ✅ SNMP Implementation Stubs - **COMPLETED**
**Files affected:**
- `/crates/unet-core/src/snmp/session.rs` - SNMP GET and GETNEXT operations

**✅ Completed implementations:**
1. ✅ Replaced TODO comments with actual `snmp2` crate integration using AsyncSession
2. ✅ Implemented proper SNMP GET operations with real network calls
3. ✅ Added comprehensive error handling for network timeouts, authentication failures
4. ✅ Implemented SNMP GETNEXT for table walking operations
5. ✅ Added proper value conversion between snmp2 and internal types

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

## 3. Large Files/Functions

### Priority: MEDIUM - Improve Maintainability

#### ✅ Datastore.rs (2,740 lines) - **COMPLETED**
**Issue:** Single trait with too many methods
**✅ Completed:**
1. Split into multiple logical modules in `datastore/` directory
2. Maintained single interface through trait re-exports  
3. Moved implementation details to separate modules (CSV, SQLite)
4. Added comprehensive documentation for each module
5. All tests pass with new structure

#### ✅ Models.rs (2,689 lines) - **COMPLETED**  
**Issue:** All data models in single file
**✅ Completed:**
1. Split into logical modules:
   - `models/mod.rs` - Main exports and basic types
   - `models/node.rs` - Node model and implementations
   - `models/location.rs` - Location model and implementations  
   - `models/link.rs` - Link model and implementations
   - `models/validation.rs` - Validation logic
   - `models/conversions.rs` - Type conversions
   - `models/derived/` - Derived state models (modularized)
2. Used `mod.rs` to re-export public interfaces
3. Grouped related models and their implementations
4. Added module-level documentation
5. Ensured proper dependency ordering

#### ✅ Policy Evaluator (2,251 lines) - **COMPLETED**
**Issue:** Complex policy evaluation logic in single file  
**✅ Completed:**
1. Extracted action executors to separate files:
   - `policy/evaluator/actions.rs` - Action execution logic
   - `policy/evaluator/context.rs` - Evaluation context
   - `policy/evaluator/orchestration.rs` - Policy orchestration
   - `policy/evaluator/results.rs` - Result handling
   - `policy/evaluator/rollback.rs` - Transaction and rollback
2. Split evaluation logic from execution logic
3. Created clear policy execution pipeline
4. Added comprehensive unit tests for each component

#### ✅ CLI Nodes Commands (1,051 lines) - **COMPLETED**
**Issue:** All node commands in single file
**✅ Completed:**
1. Split into command-specific modules in `commands/nodes/` directory
2. Created shared utilities and common patterns
3. Used consistent error handling patterns across commands
4. Added command-specific validation and help text
5. Maintained integration tests for each command

---

## 4. File Size Management (Target: 250-300 Lines Per File)

### Priority: HIGH - Code Organization and Maintainability

Files exceeding 300 lines significantly impact code readability, maintainability, and collaboration. The following files require immediate splitting:

#### Critical Files (>1000 lines) - Split Immediately

##### ✅ `/crates/unet-core/src/datastore.rs` (2,740 lines) - **COMPLETED**
**Split into:**
```
datastore/
├── mod.rs - Main trait definition and re-exports
├── types.rs - Filter, Sort, Pagination types and implementations  
├── helpers.rs - Helper functions for creating filters/sorts
├── csv/ - Complete CSV implementation (modularized)
│   ├── mod.rs - Main CSV store implementation
│   ├── nodes.rs - Node operations
│   ├── links.rs - Link operations  
│   ├── locations.rs - Location operations
│   └── utils.rs - Utility functions
└── sqlite/ - SQLite implementation (modularized)
    ├── mod.rs - Main SQLite module
    ├── store.rs - Core SQLite store implementation
    ├── nodes.rs - Node operations
    ├── links.rs - Link operations
    ├── locations.rs - Location operations
    ├── conversions.rs - Type conversions
    ├── filters.rs - Query filtering
    └── transaction.rs - Transaction handling
```

##### ✅ `/crates/unet-core/src/models.rs` (2,689 lines) - **COMPLETED**
**Split into:**
```
models/
├── mod.rs - Re-exports, basic enums (Lifecycle, DeviceRole, Vendor)
├── node.rs - Node struct and implementation
├── node_builder.rs - NodeBuilder implementation
├── location.rs - Location struct and implementation
├── link.rs - Link struct and implementation
├── validation.rs - Validation logic for all models
├── conversions.rs - From/Into implementations
├── derived/ - Derived state models (modularized)
│   ├── mod.rs - Main derived types
│   ├── system.rs - System information
│   ├── interfaces.rs - Interface status
│   └── metrics.rs - Performance metrics
└── tests/ - All model tests (modularized)
    ├── mod.rs - Test utilities
    ├── enums.rs - Enum tests
    ├── node.rs - Node tests
    ├── link.rs - Link tests
    └── location.rs - Location tests
```

##### ✅ `/crates/unet-core/src/policy/evaluator.rs` (2,251 lines) - **COMPLETED**
**Split into:**
```
policy/evaluator/
├── mod.rs - Main PolicyEvaluator with basic evaluation
├── context.rs - EvaluationContext and related types
├── actions.rs - Action execution (SET, ASSERT, APPLY)
├── rollback.rs - Transaction and rollback logic
├── orchestration.rs - PolicyOrchestrator and batch processing
├── results.rs - Result types and aggregation
└── tests.rs - Core evaluator tests
```

#### Large Files (500-1000 lines) - Split Next

##### ✅ `/crates/unet-cli/src/commands/nodes.rs` (1,051 lines) - **COMPLETED**
**Split into:**
```
commands/nodes/ - Modularized CLI node commands
├── mod.rs - Command definitions and main execute function
├── [various modules] - Command-specific implementations
```

##### ✅ `/crates/unet-core/src/policy/tests.rs` (845 lines) - **COMPLETED**
**Split into:**
```
policy/tests/ - Modularized policy tests
├── mod.rs - Common test utilities
├── [various modules] - Test-specific implementations
```

##### ✅ `/crates/unet-core/src/models/derived.rs` (804 lines) - **COMPLETED**
**Split into:**
```
models/derived/
├── mod.rs (150 lines) - NodeStatus and main types
├── system.rs (200 lines) - SystemInfo and related types
├── interfaces.rs (250 lines) - Interface status and statistics
└── metrics.rs (200 lines) - Performance and environmental metrics
```

#### Additional Files Recently Split

##### ✅ `/crates/unet-core/src/datastore/csv.rs` (765 lines) - **COMPLETED**
**Split into:**
```
datastore/csv/
├── mod.rs - Main CSV store implementation and trait delegation
├── nodes.rs - Node operations for CSV datastore
├── links.rs - Link operations for CSV datastore  
├── locations.rs - Location operations for CSV datastore
└── utils.rs - Utility functions for filtering and sorting
```

##### ✅ `/crates/unet-core/src/datastore/sqlite/store.rs` (689 lines) - **COMPLETED**
**Split into:**
```
datastore/sqlite/
├── store.rs - Main SQLite store implementation and trait delegation
├── nodes.rs - Node operations for SQLite datastore
├── links.rs - Link operations for SQLite datastore
└── locations.rs - Location operations for SQLite datastore
```

##### ✅ `/crates/unet-core/src/config/mod.rs` (441 lines) - **COMPLETED**
**Split into:**
```
config/
├── mod.rs - Main module with re-exports
├── core.rs - Core Config struct and loading/saving functionality
├── validation.rs - Configuration validation and adjustment logic
└── utils.rs - Utility functions (database_url, is_development, etc.)
```

#### Medium Files (300-700 lines) - Split When Time Permits

##### ✅ `/crates/unet-core/src/snmp.rs` (642 lines) - **COMPLETED**
**Split into:**
```
snmp/
├── mod.rs (200 lines) - Main types and SnmpSession
├── client.rs (250 lines) - SnmpClient with connection pooling
├── values.rs (100 lines) - SnmpValue enum and conversions
└── config.rs (90 lines) - Configuration types
```

##### ✅ `/crates/unet-core/src/snmp/poller.rs` (623 lines) - **COMPLETED**
**Split into:**
```
snmp/poller/
├── mod.rs (200 lines) - PollingTask and basic types
├── scheduler.rs (250 lines) - PollingScheduler implementation
└── handle.rs (172 lines) - PollingHandle and message handling
```

##### ✅ `/crates/unet-core/src/policy/loader.rs` (616 lines) - **COMPLETED**
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
- ✅ `/crates/unet-core/src/error.rs` (505 lines) - **COMPLETED**
- `/crates/unet-core/src/snmp/oids.rs` (443 lines)
- `/crates/unet-core/src/policy/parser.rs` (434 lines)
- `/crates/unet-server/src/handlers/policies.rs` (393 lines)
- `/crates/unet-core/src/policy_integration.rs` (391 lines)
- `/crates/unet-core/src/config.rs` (515 lines)
- `/crates/unet-cli/src/commands/links.rs` (323 lines)
- `/crates/unet-server/src/handlers/nodes.rs` (318 lines)
- `/crates/unet-cli/src/commands/locations.rs` (317 lines)

### Implementation Priority and Timeline

#### ✅ Phase 1: Foundation Files (Week 2) - **COMPLETED**
1. ✅ **models.rs** - Core types used everywhere, split first
2. ✅ **datastore.rs** - Core trait definitions and implementations

#### ✅ Phase 2: Complex Logic (Week 3) - **COMPLETED**  
3. ✅ **policy/evaluator.rs** - Complex evaluation engine
4. ✅ **snmp.rs** - Core SNMP functionality

#### ✅ Phase 3: Specialized Modules (Week 4) - **COMPLETED**
5. ✅ **models/derived.rs** - Derived state models
6. ✅ **snmp/poller.rs** - Polling implementation
7. ✅ **policy/loader.rs** - Policy loading logic
8. ✅ **error.rs** - Error type organization

#### ✅ Phase 4: Supporting Files (Week 5) - **COMPLETED**
9. ✅ **commands/nodes.rs** - CLI commands
10. ✅ **policy/tests.rs** - Test organization
11. ✅ **Additional files over 300 lines** - All remaining large files split

#### ✅ Phase 5: Additional File Splits - **COMPLETED**
12. ✅ **datastore/csv.rs** - CSV datastore modularization
13. ✅ **datastore/sqlite/store.rs** - SQLite store modularization  
14. ✅ **config/mod.rs** - Configuration module organization

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

## 5. Architecture and Design Issues

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
3. Replace remaining `unwrap()` calls in production code paths (~158 calls remaining)

### Phase 2 (Important - Next Sprint)
1. ~~Create error conversion helpers to reduce duplication~~ ✅ **COMPLETED**
2. Split large files into logical modules
3. ~~Centralize configuration constants~~ ✅ **COMPLETED**

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