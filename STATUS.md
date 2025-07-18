# μNet Development Roadmap

> **Purpose:** Accurate, actionable roadmap for μNet development  
> **Status:** Current as of 2025-01-17  
> **Project Distance:** ~60% complete - solid foundation, missing key features

## Executive Summary

μNet has functional core components with 166 passing tests. The project is ~60%
complete. Key features like template engine and config-slicer exist only as
placeholder code.

**Current Reality:**

- ✅ **Implemented:** Core models, policy engine, SNMP framework, CLI, API
- ⚠️ **Missing Integration:** Background tasks, derived state tracking, some
API endpoints
- ❌ **Missing Features:** Template engine, config-slicer, Git integration (all
placeholder code)

## Current Project State

### ✅ **COMPLETED** (Production-Ready)

**Core Infrastructure:**

- Data models (Node, Location, Link) with full validation
- DataStore abstraction with SQLite & CSV implementations
- Policy engine with DSL parser and evaluation
- SNMP connection pooling and session management
- CLI interface with all major commands
- HTTP API core endpoints
- Comprehensive test suite (166 tests, 100% passing)
- Documentation system (mdBook)

**Metrics:**

- Tests: 166 passing, 0 failing
- Linting: Clippy clean, properly formatted
- Architecture: 3-crate Rust workspace
- Dependencies: Standard Rust ecosystem crates

### ⚠️ **PARTIALLY IMPLEMENTED** (Need Completion)

**Integration Gaps:**

- Location/Link API handlers (stubs exist, need implementation)
- Background SNMP polling (framework exists, needs integration)
- Derived state tracking (models exist, need database integration)
- Server background tasks (started but not connected)

**Implementation complexity:** Moderate

### ❌ **NOT STARTED** (Placeholder Code Only)

**Template Engine (Milestone 4):**

- MiniJinja integration
- Configuration rendering
- Template validation
- CLI template commands
- **Implementation complexity:** Complex

**Config-Slicer (Milestone 5):**

- Hierarchical diff engine
- Configuration parsing
- Diff workflows
- **Implementation complexity:** Complex

**Git Integration (Milestone 6):**

- Repository cloning/syncing
- Version control workflows
- Policy synchronization
- **Implementation complexity:** Moderate

## Immediate Next Steps (Priority Order)

### 1. **Complete Current Foundation** (Moderate complexity)

- [ ] Implement location/link API handlers (`/api/v1/locations/*`, `/api/v1/links/*`)
- [ ] Connect background SNMP polling to database updates
- [ ] Add derived state tracking integration
- [ ] Verify end-to-end workflows work completely

### 2. **Template Engine Implementation** (Complex)

- [ ] Integrate MiniJinja templating system
- [ ] Build configuration rendering pipeline
- [ ] Add template validation and error handling
- [ ] Create CLI commands for template operations
- [ ] Add comprehensive tests for template functionality

### 3. **Config-Slicer Implementation** (Complex)

- [ ] Build hierarchical configuration parser
- [ ] Implement intelligent diff engine
- [ ] Add configuration comparison workflows
- [ ] Create CLI commands for diffing operations
- [ ] Add visualization for configuration changes

### 4. **Git Integration** (Moderate complexity)

- [ ] Implement repository cloning and syncing
- [ ] Add policy synchronization from Git repos
- [ ] Create version control workflows
- [ ] Add change tracking and rollback capabilities

### 5. **Production Hardening** (Complex)

- [ ] Add authentication and authorization
- [ ] Implement proper logging and monitoring
- [ ] Add deployment packaging
- [ ] Create production deployment guides
- [ ] Add backup and recovery procedures

## Technical Debt & Quality Issues

### Critical Issues from FIXME.md

- **Datastore Integration:** 10 TODO items in API handlers need completion
- **Policy Evaluation:** Missing status tracking and evaluation history
- **SNMP Implementation:** Framework exists but needs real device testing

### File Organization (Completed)

- All large files successfully split into logical modules
- All files now ≤300 lines as per project standards
- Module boundaries well-defined with proper re-exports

## Distance to Production

**Current Status:** ~60% complete

**Remaining Work Breakdown:**

- Complete foundation integration: Moderate complexity
- Template engine: Complex
- Config-slicer: Complex
- Git integration: Moderate complexity
- Production hardening: Complex

## Risk Assessment

**LOW RISK:**

- Core functionality has test coverage
- Architecture follows established patterns
- All tests currently passing

**MEDIUM RISK:**

- Template engine needs careful integration with policy system
- SNMP polling needs real device testing
- Git integration requires secure credential handling

**HIGH RISK:**

- None identified - project is well-architected

## Success Metrics

**Foundation Completion:**

- All API endpoints functional
- Background tasks integrated
- End-to-end workflows verified

**Feature Completion:**

- Template rendering with configuration push
- Configuration diffing with change visualization
- Git synchronization with policy management

**Production Readiness:**

- Authentication/authorization implemented
- Monitoring and logging configured
- Deployment processes documented

## Development Guidelines

**Quality Gates:**

- All tests must pass (`mise run test`)
- Code must be lint-clean (`mise run lint`)
- Files must be ≤300 lines
- No TODO/FIXME comments in committed code

**Implementation Order:**

1. Complete existing features before starting new ones
2. Add integration tests for all new functionality
3. Verify real-world usage (SNMP polling, template rendering)
4. Focus on end-to-end workflows over individual features

## Conclusion

μNet has functional core components that support the planned feature
development. The architecture supports the remaining work without major changes
required.

**Key Insight:** Previous documentation overstated completion status. The
existing foundation supports continued development.

**Recommendation:** Focus on completing the current foundation integration
before starting new features. This will provide working end-to-end workflows
and validate the architecture before expanding functionality.
