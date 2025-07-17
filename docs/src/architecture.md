# Architecture Overview – μNet Network Management System

> **Audience:** Engineers and operators working with μNet  
> **Status:** Documents current implementation (v0.1.0)

---

## Overview

μNet is a network configuration management system built in Rust, designed around three core principles:

1. **Single Binary Deployment** - Both CLI and server are self-contained executables
2. **Modular Architecture** - Clear separation between data, policy, and presentation layers  
3. **Progressive Adoption** - Start small, grow capabilities over time

## System Architecture

### Current Implementation

```ascii
┌─────────────────────────────────────────────────────────────────────────┐
│                          μNet System (v0.1.0)                          │
│ ─────────────────────────────────────────────────────────────────────── │
│                                                                         │
│  ┌─────────────────┐           ┌─────────────────┐                     │
│  │   unet-cli      │    HTTP   │  unet-server    │                     │
│  │                 │◄──────────┤                 │                     │
│  │ • Commands      │    JSON   │ • REST API      │                     │
│  │ • Output Format │           │ • Background    │                     │
│  │ • Local/Remote  │           │   Tasks         │                     │
│  └─────────────────┘           └─────────────────┘                     │
│           │                             │                               │
│           │                             │                               │
│           ▼                             ▼                               │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                    unet-core Library                           │   │
│  │ ─────────────────────────────────────────────────────────────── │   │
│  │ • Data Models    • DataStore Trait   • Policy Engine          │   │
│  │ • SNMP Client    • Error Handling    • Configuration          │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                              │                                         │
│                              ▼                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                    Data Storage                                │   │
│  │ ─────────────────────────────────────────────────────────────── │   │
│  │ SQLite Database              CSV Files (Development)           │   │
│  │ • SeaORM Entities           • File-based Storage              │   │
│  │ • Migrations                • In-memory Caching               │   │
│  │ • ACID Transactions         • Simple Testing                  │   │
│  └─────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘

External Network Devices
├── SNMP Polling ──────► Node Status, Interface Stats, System Info
├── Policy Evaluation ──► Compliance Checking
└── Configuration Management (Planned for v0.2.0)
```

---

## Component Architecture

### Core Library (`unet-core`)

The heart of μNet, providing all business logic as a reusable library.

#### Data Models

- **Node**: Network devices with lifecycle management
- **Link**: Connections between nodes (point-to-point and circuits)
- **Location**: Hierarchical physical locations (campus → building → rack)
- **Derived State**: SNMP-collected data (status, interfaces, metrics)

#### DataStore Abstraction

```rust
trait DataStore {
    async fn get_node(&self, id: &str) -> Result<Option<Node>>;
    async fn create_node(&self, node: &Node) -> Result<()>;
    async fn list_nodes(&self, options: QueryOptions) -> Result<PagedResult<Node>>;
    // ... comprehensive CRUD operations
}
```

**Implementations:**

- **SQLiteStore**: Production backend using SeaORM
- **CsvStore**: Development/testing backend with file storage

#### Policy Engine

- **DSL Parser**: Pest-based grammar for WHEN/THEN rules
- **Evaluator**: Context-based rule execution against JSON data
- **Actions**: Assert (compliance), Set (data modification)

#### SNMP Integration

- **Client**: Async SNMP operations with connection pooling
- **Polling**: Background tasks for device data collection
- **OID Mapping**: Standard and vendor-specific MIB support

### Server Binary (`unet-server`)

HTTP API server built with Axum framework.

#### API Features

- **REST Endpoints**: Full CRUD operations for all data types
- **Pagination**: Built-in support for large datasets
- **Filtering**: Query nodes by vendor, role, lifecycle
- **Error Handling**: Consistent JSON error responses
- **Derived State**: SNMP data accessible via dedicated endpoints

#### Background Services

- **SNMP Polling**: Continuous device monitoring
- **Policy Evaluation**: Periodic compliance checking
- **Health Monitoring**: Component status tracking

### CLI Binary (`unet-cli`)

Command-line interface for operators and automation.

#### Command Structure

```bash
unet [global-options] <command> <subcommand> [args]

Commands:
├── nodes     - Device CRUD operations
├── links     - Connection management  
├── locations - Site management
├── policy    - Compliance operations
├── import    - Data import from JSON/YAML
└── export    - Data export to JSON/YAML
```

#### Features

- **Multiple Output Formats**: Table, JSON, YAML
- **Local & Remote Operation**: Direct database or HTTP API
- **Comprehensive Validation**: Input checking and error reporting
- **Pagination Support**: Handle large datasets efficiently

---

## Data Flow

### 1. Device Management Flow

```ascii
Operator ──► CLI Command ──► DataStore ──► Database
   │              │             │            │
   │              │             │            │
   ▼              ▼             ▼            ▼
Input          Validation   Business     Persistence
Parsing        & Building   Logic        & Integrity
```

### 2. SNMP Monitoring Flow

```ascii
Device ──► SNMP Client ──► DataStore ──► Derived State
   │           │              │             │
   │           │              │             │
   ▼           ▼              ▼             ▼
Status      OID Mapping   Storage        API Access
Data        & Parsing     Update         & Queries
```

### 3. Policy Evaluation Flow

```ascii
Policy Files ──► Parser ──► Evaluator ──► Results
     │             │           │            │
     │             │           │            │
     ▼             ▼           ▼            ▼
   Rules         AST       Context        Compliance
  Syntax      Building    Evaluation      Reports
```

---

## Technology Stack

### Core Technologies

| Component | Technology | Rationale |
|-----------|------------|-----------|
| **Language** | Rust | Memory safety, performance, excellent async support |
| **Database** | SQLite + SeaORM | Zero-ops deployment, type-safe queries |
| **HTTP Server** | Axum | Modern async framework, excellent performance |
| **CLI Framework** | Clap v4 | Rich features, derive macros, shell completion |
| **SNMP Client** | csnmp | Pure Rust, async-first SNMP implementation |
| **Policy Parser** | Pest | PEG parser with excellent error reporting |

### Data Formats

| Use Case | Format | Details |
|----------|--------|---------|
| **API Requests/Responses** | JSON | Standard REST API format |
| **Configuration** | TOML | Human-readable, well-structured |
| **Data Import/Export** | JSON/YAML | Flexible bulk operations |
| **Error Logging** | Structured JSON | Machine-parsable logs |

### Async Architecture

All I/O operations are fully asynchronous using Tokio:

- **Database Operations**: Non-blocking SeaORM queries
- **HTTP Server**: Concurrent request handling
- **SNMP Polling**: Parallel device monitoring
- **CLI Operations**: Async throughout, no blocking calls

---

## Current Capabilities

### ✅ Implemented Features

#### Data Management

- **Full CRUD Operations**: Create, read, update, delete for all entities
- **Hierarchical Locations**: Campus → Building → Floor → Rack structure
- **Lifecycle Management**: Device states from planning to decommissioned
- **Custom Data Support**: JSON fields for extensibility

#### Network Monitoring

- **SNMP Polling**: Background collection of device status
- **Interface Statistics**: Port-level traffic and error counters
- **System Information**: Device description, uptime, contact info
- **Connection Pooling**: Efficient SNMP session management

#### Policy & Compliance

- **DSL Language**: WHEN/THEN rule syntax for network policies
- **Rule Evaluation**: Real-time compliance checking
- **Bulk Operations**: Evaluate policies across entire network
- **Result Storage**: Historical compliance tracking

#### API & CLI

- **Comprehensive REST API**: Full HTTP interface for automation
- **Rich CLI**: Interactive commands with multiple output formats
- **Data Import/Export**: Bulk operations for network topology
- **Error Handling**: Detailed error messages and recovery guidance

### 🚧 Planned Features

- **Template Engine**: Configuration generation (v0.2.0)
- **Advanced SNMP CLI**: Historical metrics, polling controls
- **Configuration Push**: Safe deployment to devices
- **Git Integration**: Version control for policies and templates
- **Web UI**: Browser-based management interface

---

## Database Schema

### Core Entities

```sql
-- Locations (hierarchical)
locations
├── id (UUID)
├── name (TEXT)
├── location_type (TEXT)
├── parent_id (UUID, FK)
├── address (TEXT)
└── custom_data (JSON)

-- Nodes (network devices)  
nodes
├── id (UUID)
├── name (TEXT)
├── domain (TEXT)
├── fqdn (TEXT, computed)
├── vendor (TEXT)
├── model (TEXT)
├── role (TEXT)
├── lifecycle (TEXT)
├── management_ip (TEXT)
├── location_id (UUID, FK)
└── custom_data (JSON)

-- Links (connections)
links
├── id (UUID)
├── node_a_id (UUID, FK)
├── interface_a (TEXT)
├── node_z_id (UUID, FK, nullable)
├── interface_z (TEXT, nullable)
├── circuit_id (TEXT, nullable)
├── bandwidth (BIGINT)
└── custom_data (JSON)
```

### Derived State Tables

```sql
-- SNMP-collected data
node_status
├── node_id (UUID, FK)
├── last_updated (TIMESTAMP)
├── reachable (BOOLEAN)
├── system_info (JSON)
├── raw_snmp_data (JSON)
└── last_error (TEXT)

interface_status
├── node_id (UUID, FK)
├── interface_index (INTEGER)
├── name (TEXT)
├── admin_status (TEXT)
├── oper_status (TEXT)
├── statistics (JSON)
└── last_updated (TIMESTAMP)
```

---

## Security Considerations

### Current State

- **No Authentication**: All endpoints publicly accessible
- **Input Validation**: Comprehensive validation of all user inputs
- **SQL Injection Protection**: SeaORM provides safe query building
- **Error Information**: Sanitized error responses, no sensitive data exposure

### Planned Security Features

- **Token-based Authentication**: JWT or similar for API access
- **Role-based Authorization**: Different permission levels
- **Audit Logging**: Comprehensive change tracking
- **TLS/HTTPS**: Encrypted communication

---

## Performance Characteristics

### Current Performance

- **Database**: SQLite performs well for <10K devices
- **API Response Times**: Sub-100ms for typical operations
- **SNMP Polling**: Configurable intervals, parallel execution
- **Memory Usage**: ~50MB base, scales with dataset size

### Scalability Considerations

- **Horizontal Scaling**: Not currently supported (SQLite limitation)
- **Vertical Scaling**: Excellent single-machine performance
- **Caching**: Minimal caching, relies on SQLite performance
- **Connection Pooling**: SNMP connections reused efficiently

---

## Development Philosophy

### Code Quality

- **Type Safety**: Rust's type system prevents many runtime errors
- **Error Handling**: Comprehensive error types with context
- **Testing**: Unit tests for business logic, integration tests for APIs
- **Documentation**: Inline docs and comprehensive guides

### Operational Excellence

- **Single Binary**: No external dependencies for deployment
- **Configuration**: Environment variables, files, or CLI arguments
- **Logging**: Structured logging with multiple levels
- **Metrics**: Built-in health checks and status endpoints

---

## Future Architecture Plans

### Template Engine (v0.2.0)

- **MiniJinja Integration**: Rust-native templating
- **Configuration Generation**: Device-specific config templates
- **Diff Engine**: Compare generated vs. actual configurations
- **Partial Templates**: Template only specific config sections

### Enhanced Storage (v0.3.0)

- **PostgreSQL Support**: Production-scale database backend
- **Time-Series Data**: Historical metrics and performance data
- **Backup/Restore**: Automated data protection
- **Multi-tenancy**: Support for multiple organizations

### Advanced Features (v1.0.0)

- **Configuration Push**: Safe deployment to network devices
- **Rollback Mechanisms**: Automatic recovery from failed changes
- **Change Management**: Approval workflows and audit trails
- **High Availability**: Clustering and replication support

The architecture is designed to grow incrementally while maintaining backward compatibility and operational simplicity.
