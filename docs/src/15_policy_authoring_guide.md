<!-- SPDX-License-Identifier: MIT -->

# Policy Authoring Guide

This guide provides comprehensive instructions for writing μNet policy rules using the Domain Specific Language (DSL).

## Overview

μNet policies are declarative rules that define network configuration compliance requirements and automated actions. They follow a simple WHEN/THEN structure that makes them easy to read and maintain.

### Basic Structure

```
WHEN <condition> THEN <action>
```

Every policy rule consists of:

- **Condition**: A boolean expression that evaluates node properties
- **Action**: An operation to perform when the condition is met

## Getting Started

### Your First Policy

Let's start with a simple policy that ensures all Cisco devices have a specific SNMP community:

```
WHEN node.vendor == "cisco" THEN SET custom_data.snmp.community TO "public"
```

This policy:

1. Checks if a node's vendor is "cisco"
2. Sets the SNMP community in the node's custom data

### Running Policies

Policies can be executed through:

- **CLI**: `unet policies evaluate <policy-file> --node <node-id>`
- **API**: POST to `/api/v1/policies/evaluate`
- **Automatic**: Background policy evaluation (coming in future releases)

## Writing Conditions

Conditions determine when a policy rule should execute. They can be simple comparisons or complex logical expressions.

### Field References

Access node properties using dot notation:

```
node.vendor          # Device vendor (cisco, juniper, arista, etc.)
node.model           # Device model
node.role            # Device role (router, switch, firewall, etc.)
node.lifecycle       # Lifecycle state (planned, live, decommissioned)
node.management_ip   # Management IP address
custom_data.field    # Custom data fields
derived.interfaces   # Derived data from SNMP polling
```

### Comparison Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `==` | Equal | `node.vendor == "cisco"` |
| `!=` | Not equal | `node.role != "switch"` |
| `<` | Less than | `node.port_count < 24` |
| `<=` | Less than or equal | `node.port_count <= 48` |
| `>` | Greater than | `custom_data.uptime > 86400` |
| `>=` | Greater than or equal | `custom_data.cpu_usage >= 80` |
| `CONTAINS` | String contains | `node.model CONTAINS "2960"` |
| `MATCHES` | Regex match | `node.name MATCHES "^rtr-.*"` |

### String Operations

#### Contains Check

```
WHEN node.model CONTAINS "catalyst" THEN SET custom_data.switch_type TO "cisco_catalyst"
```

#### Regex Matching

```
WHEN node.name MATCHES "^(rtr|router)-.*" THEN SET custom_data.device_type TO "router"
```

### Null Value Checks

Check if fields exist or are null:

```
WHEN node.management_ip IS NOT NULL THEN SET custom_data.monitoring.enabled TO true
WHEN custom_data.location IS NULL THEN SET custom_data.location TO "unknown"
```

### Logical Operators

#### AND Conditions

```
WHEN node.vendor == "cisco" AND node.role == "switch" 
THEN SET custom_data.management.snmp_version TO "v3"
```

#### OR Conditions

```
WHEN node.vendor == "cisco" OR node.vendor == "arista" 
THEN SET custom_data.os_type TO "ios_like"
```

#### NOT Conditions

```
WHEN NOT (node.lifecycle == "decommissioned") 
THEN SET custom_data.monitoring.enabled TO true
```

#### Complex Expressions

```
WHEN (node.vendor == "cisco" AND node.model CONTAINS "2960") 
  OR (node.vendor == "arista" AND node.role == "switch")
THEN APPLY "templates/layer2-switch.j2"
```

## Actions

Actions define what to do when a condition is satisfied. μNet supports three types of actions.

### SET Action

Updates custom_data fields with new values.

#### Syntax

```
SET <field_path> TO <value>
```

#### Examples

**Simple field assignment:**

```
SET custom_data.location TO "datacenter-1"
```

**Nested field assignment:**

```
SET custom_data.snmp.community TO "private"
SET custom_data.monitoring.enabled TO true
SET custom_data.thresholds.cpu_warning TO 75
```

**Creating nested structures:**

```
SET custom_data.compliance.last_check TO "2024-01-15"
SET custom_data.compliance.status TO "passed"
```

### ASSERT Action

Validates that a field matches an expected value. Used for compliance checking.

#### Syntax

```
ASSERT <field_path> IS <expected_value>
```

#### Examples

**Compliance validation:**

```
WHEN node.vendor == "cisco" THEN ASSERT node.version IS "15.1"
```

**Configuration validation:**

```
WHEN node.role == "firewall" THEN ASSERT custom_data.security.enabled IS true
```

### APPLY Action

Assigns configuration templates to nodes for later generation.

#### Syntax

```
APPLY "<template_path>"
```

#### Examples

**Basic template assignment:**

```
WHEN node.role == "router" THEN APPLY "templates/base-router.j2"
```

**Vendor-specific templates:**

```
WHEN node.vendor == "cisco" AND node.role == "switch" 
THEN APPLY "templates/cisco-switch-base.j2"
```

## Data Types and Values

### Supported Value Types

| Type | Example | Notes |
|------|---------|-------|
| String | `"cisco"`, `"datacenter-1"` | Use double quotes |
| Number | `24`, `100.5`, `-1` | Integer or decimal |
| Boolean | `true`, `false` | Lowercase only |
| Null | `null` | Represents absence of value |

### Working with Numbers

```
WHEN custom_data.port_count == 24 THEN SET custom_data.switch_class TO "access"
WHEN custom_data.cpu_usage > 85.5 THEN SET custom_data.alert_level TO "critical"
```

### Working with Booleans

```
WHEN custom_data.monitoring.enabled == false THEN SET custom_data.monitoring.enabled TO true
WHEN node.lifecycle == "live" THEN SET custom_data.production_ready TO true
```

## Advanced Patterns

### Conditional Field Creation

Create fields only when certain conditions are met:

```
WHEN node.vendor == "cisco" AND custom_data.snmp IS NULL 
THEN SET custom_data.snmp.version TO "v2c"
```

### Multi-step Configuration

Use multiple policies to build up configuration:

```
# Step 1: Basic monitoring setup
WHEN node.lifecycle == "live" THEN SET custom_data.monitoring.enabled TO true

# Step 2: Vendor-specific monitoring
WHEN node.vendor == "cisco" AND custom_data.monitoring.enabled == true 
THEN SET custom_data.monitoring.snmp_oids TO "1.3.6.1.2.1.1.1.0"

# Step 3: Role-specific monitoring
WHEN node.role == "router" AND custom_data.monitoring.enabled == true 
THEN SET custom_data.monitoring.bgp_check TO true
```

### Template Hierarchies

Apply multiple templates in order:

```
# Base template for all devices
WHEN node.lifecycle == "live" THEN APPLY "templates/base-config.j2"

# Vendor-specific template
WHEN node.vendor == "cisco" THEN APPLY "templates/cisco-base.j2"

# Role-specific template  
WHEN node.role == "router" THEN APPLY "templates/router-config.j2"

# Location-specific template
WHEN custom_data.location == "datacenter-1" THEN APPLY "templates/dc1-config.j2"
```

## Common Patterns

### Device Classification

```
# Classify switch types by port count
WHEN node.role == "switch" AND custom_data.port_count <= 24 
THEN SET custom_data.switch_type TO "access"

WHEN node.role == "switch" AND custom_data.port_count > 24 AND custom_data.port_count <= 48 
THEN SET custom_data.switch_type TO "distribution"

WHEN node.role == "switch" AND custom_data.port_count > 48 
THEN SET custom_data.switch_type TO "core"
```

### Compliance Enforcement

```
# Ensure all production devices have monitoring
WHEN node.lifecycle == "live" AND custom_data.monitoring.enabled != true 
THEN SET custom_data.monitoring.enabled TO true

# Validate SNMP configuration
WHEN node.lifecycle == "live" AND custom_data.monitoring.enabled == true 
THEN ASSERT custom_data.snmp.version IS "v3"

# Ensure security settings
WHEN node.role == "firewall" 
THEN ASSERT custom_data.security.enabled IS true
```

### Environment-Specific Configuration

```
# Development environment
WHEN custom_data.environment == "dev" 
THEN SET custom_data.logging.level TO "debug"

# Production environment  
WHEN custom_data.environment == "prod" 
THEN SET custom_data.logging.level TO "warn"

# Apply environment-specific templates
WHEN custom_data.environment == "prod" 
THEN APPLY "templates/production-hardening.j2"
```

## Best Practices

### 1. Use Descriptive Conditions

**Good:**

```
WHEN node.vendor == "cisco" AND node.model CONTAINS "catalyst" 
THEN SET custom_data.switch_family TO "cisco_catalyst"
```

**Avoid:**

```
WHEN node.vendor == "cisco" AND node.model CONTAINS "cat" 
THEN SET custom_data.type TO "cat"
```

### 2. Structure Custom Data Logically

Use nested objects to organize custom data:

```
# Good structure
SET custom_data.monitoring.enabled TO true
SET custom_data.monitoring.snmp_version TO "v3"
SET custom_data.compliance.last_check TO "2024-01-15"
SET custom_data.compliance.status TO "passed"

# Avoid flat structure
SET custom_data.monitoring_enabled TO true
SET custom_data.snmp_version TO "v3"
SET custom_data.last_compliance_check TO "2024-01-15"
```

### 3. Use Explicit Comparisons

**Good:**

```
WHEN custom_data.monitoring.enabled == true THEN ...
WHEN node.lifecycle != "decommissioned" THEN ...
```

**Avoid:**

```
WHEN custom_data.monitoring.enabled THEN ...
WHEN NOT node.lifecycle THEN ...
```

### 4. Test Policies Incrementally

Start with simple policies and build complexity:

```
# Start simple
WHEN node.vendor == "cisco" THEN SET custom_data.vendor_class TO "ios"

# Add complexity
WHEN node.vendor == "cisco" AND node.role == "router" 
THEN SET custom_data.vendor_class TO "ios_router"

# Add more conditions
WHEN node.vendor == "cisco" AND node.role == "router" AND custom_data.location == "datacenter-1" 
THEN APPLY "templates/cisco-router-dc1.j2"
```

### 5. Use Comments for Complex Logic

While the DSL doesn't support inline comments, use policy file comments:

```
# Cisco device classification policies
# These policies set vendor-specific attributes for Cisco devices

WHEN node.vendor == "cisco" AND node.model CONTAINS "2960" 
THEN SET custom_data.switch_family TO "catalyst_access"

WHEN node.vendor == "cisco" AND node.model CONTAINS "3850" 
THEN SET custom_data.switch_family TO "catalyst_distribution"
```

## Troubleshooting

### Common Errors

#### 1. Field Not Found

```
Error: Field not found: custom_data.nonexistent.field
```

**Solution:** Check field paths and ensure parent objects exist.

#### 2. Type Mismatch

```
Error: Type mismatch: expected String, got Number
```

**Solution:** Ensure value types match field expectations.

#### 3. Invalid Regex

```
Error: Invalid regex: [invalid-pattern
```

**Solution:** Test regex patterns before using in policies.

### Debugging Tips

1. **Start Simple:** Test basic conditions before adding complexity
2. **Check Field Paths:** Verify fields exist in your data model
3. **Validate Syntax:** Use the policy parser to check syntax
4. **Test with Sample Data:** Use known node data for testing

### Testing Policies

Use the CLI to test policies against specific nodes:

```bash
# Test a single policy
unet policies evaluate policy.txt --node node-id-123

# Test with verbose output  
unet policies evaluate policy.txt --node node-id-123 --verbose

# Dry run (don't apply changes)
unet policies evaluate policy.txt --node node-id-123 --dry-run
```

## Policy File Organization

### Single File Approach

For simple deployments, keep all policies in one file:

```
# network-policies.txt

# Device classification
WHEN node.vendor == "cisco" THEN SET custom_data.vendor_class TO "ios"
WHEN node.vendor == "juniper" THEN SET custom_data.vendor_class TO "junos"

# Monitoring setup
WHEN node.lifecycle == "live" THEN SET custom_data.monitoring.enabled TO true

# Template assignment
WHEN node.role == "router" THEN APPLY "templates/router-base.j2"
```

### Multi-File Approach

For complex deployments, organize by function:

```
policies/
├── classification.policy      # Device classification rules
├── compliance.policy         # Compliance validation rules  
├── monitoring.policy         # Monitoring configuration
├── templates.policy          # Template assignments
└── security.policy          # Security hardening rules
```

## Integration with Templates

Policies and templates work together to provide complete configuration management:

### Template Assignment

```
# Assign base template to all devices
WHEN node.lifecycle == "live" THEN APPLY "templates/base.j2"

# Assign vendor-specific templates
WHEN node.vendor == "cisco" THEN APPLY "templates/cisco-base.j2"
WHEN node.vendor == "juniper" THEN APPLY "templates/juniper-base.j2"
```

### Data Preparation for Templates

Use policies to prepare data that templates will use:

```
# Set template variables
WHEN node.role == "router" THEN SET custom_data.template_vars.enable_bgp TO true
WHEN custom_data.location == "datacenter-1" THEN SET custom_data.template_vars.ntp_server TO "10.1.1.1"

# Apply template that uses these variables
WHEN custom_data.template_vars IS NOT NULL THEN APPLY "templates/router-config.j2"
```

## Migration and Rollback

### Policy Rollback

μNet supports automatic rollback of policy changes:

```rust
// Execute policies with transaction support
let (results, transaction) = PolicyEvaluator::execute_rules_with_transaction(
    &policies, &context, &datastore, &node_id
).await?;

// Rollback if needed
if should_rollback {
    PolicyEvaluator::rollback_transaction(&transaction, &datastore).await?;
}
```

### Migration Strategies

When updating policies:

1. **Test in Development:** Always test policy changes in a development environment
2. **Gradual Rollout:** Apply to small groups of devices first
3. **Monitor Results:** Watch for unexpected changes or compliance failures
4. **Keep Backups:** Maintain backups of node custom_data before policy changes

## Next Steps

- Review the [DSL Syntax Reference](16_dsl_syntax_reference.md) for complete syntax details
- Study [Policy Examples](17_policy_examples.md) for real-world scenarios  
- Check [Best Practices Guide](18_policy_best_practices.md) for advanced patterns
- Explore the [Template Engine documentation](04_template_engine.md) for integration patterns
