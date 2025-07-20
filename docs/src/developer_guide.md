# Developer Guide

This guide provides comprehensive patterns, best practices, and decision trees for developing μNet. It complements the [Architecture](architecture.md) document with practical implementation guidance.

## Quick Reference

### Finding Implementation Patterns

- **New entities**: Copy pattern from `crates/unet-core/src/entities/nodes.rs`
- **New API endpoints**: Copy pattern from `crates/unet-server/src/handlers/nodes/crud.rs`
- **New CLI commands**: Copy pattern from `crates/unet-cli/src/commands/nodes/crud.rs`
- **Database migrations**: Copy pattern from `crates/migrations/src/m20241221_000002_create_nodes_table.rs`
- **SNMP integration**: Study patterns in `crates/unet-core/src/snmp/`

### Essential Search Terms

```bash
# Find similar API implementations
grep -r "async fn.*Result<Json" crates/unet-server/src/handlers/

# Find entity patterns
grep -r "DeriveEntityModel" crates/unet-core/src/entities/

# Find test patterns
grep -r "#\[tokio::test\]" crates/*/src/

# Find error handling patterns
grep -r "ApiError" crates/unet-server/src/

# Find SNMP patterns
grep -r "SnmpSession" crates/unet-core/src/snmp/
```

## Decision Trees

### Adding New Data Fields

```
Is this field user-configurable?
├─ Yes → Desired state (nodes, locations, links tables)
│   ├─ Required for basic functionality? → NOT NULL
│   └─ Optional enhancement? → NULL allowed
└─ No → Is it derived from SNMP/monitoring?
    ├─ Yes → Derived state (node_status, interface_status tables)
    │   ├─ Always available from SNMP? → NOT NULL
    │   └─ Device-dependent? → NULL allowed
    └─ No → Consider if this belongs in custom_data JSON field
```

### Choosing Error Handling Approach

```
Where does the error originate?
├─ API endpoint → Use ApiError with appropriate HTTP status
│   ├─ User input validation → 400 Bad Request
│   ├─ Resource not found → 404 Not Found
│   ├─ Database constraint → 409 Conflict
│   └─ Internal system error → 500 Internal Server Error
├─ SNMP operation → Use csnmp error types
│   ├─ Network timeout → Retry with exponential backoff
│   ├─ Authentication failure → Log and mark node unreachable
│   └─ Parse error → Log raw data and continue
└─ Database operation → Use sea_orm::DbErr
    ├─ Transaction conflict → Retry operation
    └─ Schema mismatch → Return structured error
```

### Database Query Optimization

```
What type of query?
├─ Single record by ID → Use find_by_id() - most efficient
├─ List with filters → Use find() with where clauses
│   ├─ Common filter? → Ensure index exists
│   ├─ Large result set? → Add pagination
│   └─ Complex joins? → Consider separate queries
└─ Aggregation → Use select() with group_by()
    ├─ Real-time display? → Cache results
    └─ Background reporting? → Direct SQL may be faster
```

## Code Patterns

### Standard Entity Implementation

When creating a new entity, follow this pattern:

```rust
// crates/unet-core/src/entities/example.rs
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Description of what this entity represents
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "example")]
pub struct Model {
    /// Primary key - always use String for UUID
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    
    /// Required fields come first
    pub name: String,
    pub example_type: String,
    
    /// Optional fields
    pub description: Option<String>,
    pub custom_data: Option<String>,
    
    /// Timestamps - always include these
    pub created_at: String,
    pub updated_at: String,
}

/// Database relations
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    // Define relationships to other entities
}

impl ActiveModelBehavior for ActiveModel {}
```

### Standard API Endpoint Implementation

```rust
// crates/unet-server/src/handlers/example/crud.rs
use axum::{extract::State, http::StatusCode, Json};
use crate::{ApiError, AppState};

/// Create a new example
pub async fn create_example(
    State(app_state): State<AppState>,
    Json(payload): Json<CreateExampleRequest>,
) -> Result<Json<Example>, ApiError> {
    // 1. Validate input
    payload.validate()?;
    
    // 2. Check permissions/authorization if needed
    
    // 3. Transform to domain model
    let example = Example::from(payload);
    
    // 4. Store via DataStore trait
    let created = app_state.datastore.create_example(example).await
        .map_err(|e| ApiError::internal_error("Failed to create example", e))?;
    
    // 5. Return response
    Ok(Json(created))
}

/// Request/response types
#[derive(Debug, Deserialize)]
pub struct CreateExampleRequest {
    pub name: String,
    pub example_type: String,
    pub description: Option<String>,
}

impl CreateExampleRequest {
    fn validate(&self) -> Result<(), ApiError> {
        if self.name.is_empty() {
            return Err(ApiError::bad_request("Name cannot be empty"));
        }
        Ok(())
    }
}
```

### Standard Test Patterns

```rust
// Follow TDD - write test first!
#[tokio::test]
async fn test_create_example_success() {
    // Arrange
    let datastore = setup_test_datastore().await;
    let example = Example {
        id: Uuid::new_v4(),
        name: "test-example".to_string(),
        example_type: "test".to_string(),
        description: Some("Test description".to_string()),
        custom_data: None,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    
    // Act
    let result = datastore.create_example(example.clone()).await;
    
    // Assert
    assert!(result.is_ok());
    let created = result.unwrap();
    assert_eq!(created.name, example.name);
    assert_eq!(created.example_type, example.example_type);
    
    // Cleanup
    cleanup_test_data(&datastore).await;
}

#[tokio::test]
async fn test_create_example_duplicate_name() {
    // Test error conditions
    let datastore = setup_test_datastore().await;
    
    // Create first example
    let example1 = create_test_example(&datastore, "duplicate-name").await.unwrap();
    
    // Try to create second with same name
    let example2 = Example { name: "duplicate-name".to_string(), ..test_example() };
    let result = datastore.create_example(example2).await;
    
    assert!(result.is_err());
    // Verify specific error type
    
    cleanup_test_data(&datastore).await;
}
```

### SNMP Integration Patterns

```rust
// crates/unet-core/src/snmp/collectors/example.rs
use crate::snmp::{SnmpSession, SnmpError};
use std::time::Duration;

pub struct ExampleCollector {
    session: SnmpSession,
}

impl ExampleCollector {
    pub fn new(session: SnmpSession) -> Self {
        Self { session }
    }
    
    pub async fn collect_example_data(&mut self, node_id: &str) -> Result<ExampleData, SnmpError> {
        // 1. Define OIDs to query
        let oids = vec![
            "1.3.6.1.2.1.1.1.0".to_string(), // sysDescr
            "1.3.6.1.2.1.1.3.0".to_string(), // sysUpTime
        ];
        
        // 2. Perform bulk query with timeout
        let results = self.session
            .bulk_get(oids)
            .timeout(Duration::from_secs(30))
            .await?;
        
        // 3. Parse results with error handling
        let mut data = ExampleData::default();
        for (oid, value) in results {
            match oid.as_str() {
                "1.3.6.1.2.1.1.1.0" => {
                    data.description = value.as_string().ok();
                }
                "1.3.6.1.2.1.1.3.0" => {
                    data.uptime = value.as_counter64()
                        .map(Duration::from_centiseconds)
                        .ok();
                }
                _ => {
                    tracing::warn!("Unexpected OID in response: {}", oid);
                }
            }
        }
        
        Ok(data)
    }
}

#[derive(Debug, Default)]
pub struct ExampleData {
    pub description: Option<String>,
    pub uptime: Option<Duration>,
}
```

## Domain-Specific Patterns

### Network Automation Best Practices

#### SNMP Polling Strategy
- **Bulk operations**: Always use `bulk_get` for multiple OIDs
- **Timeout handling**: Set reasonable timeouts (30s for system info, 60s for large tables)
- **Error recovery**: Distinguish between temporary (network) and permanent (auth) failures
- **Rate limiting**: Respect device capabilities - don't overwhelm network equipment

#### Policy Evaluation Performance
- **Cache policy ASTs**: Parse policies once, evaluate many times
- **Batch evaluations**: Process multiple nodes/policies in single transaction
- **Lazy loading**: Only load data that policies actually reference
- **Early termination**: Stop evaluation as soon as decision is clear

#### State Management
- **Desired State**: User configurations, topology definitions, policy rules
- **Derived State**: SNMP data, calculated metrics, policy evaluation results
- **Never mix**: Keep clear separation between what users configure vs. what system discovers

### Common Anti-Patterns to Avoid

#### Database Anti-Patterns
```rust
// DON'T: Bypass DataStore trait
let node = Node::find_by_id(node_id).one(&db).await?;

// DO: Use DataStore abstraction
let node = datastore.get_node(&node_id).await?;

// DON'T: N+1 queries
for node in nodes {
    let location = datastore.get_location(&node.location_id).await?;
}

// DO: Bulk operations
let location_ids: Vec<_> = nodes.iter().map(|n| &n.location_id).collect();
let locations = datastore.get_locations_by_ids(&location_ids).await?;
```

#### SNMP Anti-Patterns
```rust
// DON'T: Blocking calls in async context
let result = std::thread::spawn(|| snmp_sync_call()).join();

// DO: Use async SNMP operations
let result = snmp_session.get(oid).await?;

// DON'T: Ignore timeouts
let result = snmp_session.get(oid).await?;

// DO: Set appropriate timeouts
let result = snmp_session.get(oid).timeout(Duration::from_secs(30)).await?;
```

#### Error Handling Anti-Patterns
```rust
// DON'T: Generic error messages
return Err(ApiError::internal_error("Something went wrong"));

// DO: Specific, actionable errors
return Err(ApiError::bad_request("Node name must be between 1 and 255 characters"));

// DON'T: Expose internal details
return Err(ApiError::internal_error(format!("Database error: {}", db_error)));

// DO: Log internal details, return user-friendly message
tracing::error!("Database constraint violation: {}", db_error);
return Err(ApiError::conflict("A node with this name already exists"));
```

## Debugging & Troubleshooting

### SNMP Issues

#### Connection Problems
1. **Check network connectivity**: `ping <device_ip>`
2. **Verify SNMP credentials**: Test with `snmpwalk` command
3. **Check firewall rules**: Ensure UDP 161 is accessible
4. **Validate community strings**: Ensure they match device configuration

#### Query Failures
1. **OID validation**: Verify OIDs exist on target device
2. **Permission checking**: Ensure community has read access to OIDs
3. **MIB loading**: Check if custom MIBs are needed
4. **Timeout tuning**: Increase timeouts for slow devices

#### Performance Issues
1. **Bulk query optimization**: Group related OIDs together
2. **Polling frequency**: Reduce frequency for non-critical metrics
3. **Device capability**: Some devices can't handle high query rates
4. **Network latency**: Account for WAN links with higher timeouts

### Database Performance

#### Slow Queries
1. **Check indexes**: Ensure proper indexes on filtered columns
2. **Query analysis**: Use EXPLAIN QUERY PLAN for complex queries
3. **Pagination**: Add LIMIT/OFFSET for large result sets
4. **Connection pooling**: Monitor connection pool utilization

#### Lock Contention
1. **Transaction scope**: Keep transactions as short as possible
2. **Retry logic**: Implement exponential backoff for lock timeouts
3. **Read replicas**: Use read-only queries where possible
4. **Batch operations**: Group multiple updates into single transaction

### API Performance

#### Slow Endpoints
1. **Database queries**: Profile database access patterns
2. **Serialization**: Large JSON responses can be slow
3. **External dependencies**: SNMP calls, file I/O
4. **Memory allocation**: Check for unnecessary clones/allocations

#### Memory Issues
1. **Large datasets**: Implement streaming for large responses
2. **Connection leaks**: Monitor database connection usage
3. **Background tasks**: Check for memory leaks in polling tasks
4. **Logging overhead**: Excessive debug logging can consume memory

## Task Recipes

### Adding a New SNMP OID

1. **Research the OID**
   ```bash
   # Test with snmpwalk first
   snmpwalk -v2c -c public <device_ip> <new_oid>
   ```

2. **Add to OID definitions**
   ```rust
   // crates/unet-core/src/snmp/oids/standard.rs
   pub const NEW_METRIC_OID: &str = "1.3.6.1.2.1.x.x.x";
   ```

3. **Update collector**
   ```rust
   // Add to relevant collector in crates/unet-core/src/snmp/collectors/
   let oids = vec![
       // existing OIDs...
       NEW_METRIC_OID.to_string(),
   ];
   ```

4. **Add to data model**
   ```rust
   // Update derived state entity
   pub new_metric: Option<i64>,
   ```

5. **Write tests**
   ```rust
   #[tokio::test]
   async fn test_collect_new_metric() {
       // Test successful collection
       // Test missing OID handling
       // Test parsing edge cases
   }
   ```

6. **Update migration**
   ```rust
   // Create new migration file
   cargo run --bin migration -- generate add_new_metric_column
   ```

### Adding a New Policy Rule Type

1. **Define AST node**
   ```rust
   // crates/unet-core/src/policy/ast.rs
   pub enum Condition {
       // existing conditions...
       NewRuleType { field: String, operator: ComparisonOp, value: Value },
   }
   ```

2. **Update parser**
   ```rust
   // crates/unet-core/src/policy/grammar.pest
   new_rule = { "new_rule" ~ "(" ~ field ~ operator ~ value ~ ")" }
   ```

3. **Implement evaluator**
   ```rust
   // crates/unet-core/src/policy/evaluator/conditions.rs
   Condition::NewRuleType { field, operator, value } => {
       let node_value = get_node_field(node, field)?;
       compare_values(&node_value, operator, value)
   }
   ```

4. **Add CLI support**
   ```rust
   // crates/unet-cli/src/commands/policy/
   // Add new subcommand for rule type
   ```

5. **Write comprehensive tests**
   ```rust
   #[tokio::test]
   async fn test_new_rule_evaluation() {
       // Test various value types
       // Test all operators
       // Test edge cases and errors
   }
   ```

### Extending the Data Model

1. **Plan the change**
   - Desired vs derived state?
   - Required vs optional field?
   - Index requirements?

2. **Create migration**
   ```bash
   cargo run --bin migration -- generate add_new_field_to_table
   ```

3. **Update entity model**
   ```rust
   // Add field to appropriate entity
   pub new_field: Option<String>,
   ```

4. **Update DataStore trait**
   ```rust
   // Add methods if needed
   async fn update_new_field(&self, id: &str, value: String) -> Result<()>;
   ```

5. **Implement in datastores**
   ```rust
   // Update the SQLite implementation
   ```

6. **Add API endpoints**
   ```rust
   // Update request/response types
   // Add validation logic
   // Update handlers
   ```

7. **Update CLI**
   ```rust
   // Add command line options
   // Update output formatting
   ```

8. **Write tests**
   ```rust
   // Unit tests for data model
   // Integration tests for API
   // End-to-end tests for CLI
   ```

## Performance Guidelines

### Database Optimization

#### Query Patterns
- **Single record**: Use `find_by_id()` with primary key
- **Filtered lists**: Use `find().filter()` with indexed columns
- **Counts**: Use `count()` instead of loading all records
- **Exists checks**: Use `count() > 0` for existence tests

#### Index Strategy
- **Primary keys**: Automatic unique index
- **Foreign keys**: Add index for join performance
- **Filter columns**: Index frequently filtered columns
- **Composite indexes**: For multi-column filters

#### Transaction Management
- **Read operations**: No transaction needed for single queries
- **Write operations**: Use transactions for multi-table updates
- **Long operations**: Break into smaller transactions
- **Retry logic**: Handle transaction conflicts gracefully

### Memory Management

#### Large Datasets
- **Streaming**: Process records in batches
- **Pagination**: Limit result set sizes
- **Lazy loading**: Load related data on demand
- **Connection pooling**: Reuse database connections

#### Background Tasks
- **Resource cleanup**: Properly close connections and files
- **Memory monitoring**: Check for gradual memory leaks
- **Graceful shutdown**: Handle termination signals properly
- **Error recovery**: Restart failed background tasks

### Network Optimization

#### SNMP Efficiency
- **Bulk operations**: Query multiple OIDs together
- **Connection reuse**: Maintain persistent SNMP sessions
- **Timeout tuning**: Balance responsiveness vs. reliability
- **Error handling**: Distinguish temporary vs. permanent failures

#### API Performance
- **Response size**: Minimize JSON payload size
- **Caching**: Cache expensive computations
- **Compression**: Use gzip for large responses
- **Connection limits**: Prevent resource exhaustion

This guide provides the foundation for efficient μNet development. Always refer to existing code patterns and follow the TDD practices outlined in [AGENTS.md](../AGENTS.md).