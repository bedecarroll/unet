# Policy Guide – Network Compliance and Automation

> **Audience:** Network engineers and operators writing policy rules  
> **Status:** Documents implemented policy engine (v0.1.0)

---

## Overview

μNet's policy engine enables declarative network compliance checking and automated data management through a simple Domain Specific Language (DSL). Policies help ensure your network configuration meets organizational standards and automatically apply corrective actions.

**Key Benefits:**

- **Declarative Rules**: Write what you want, not how to achieve it
- **Real-time Compliance**: Continuous evaluation of network state
- **Automated Actions**: Reduce manual configuration tasks
- **Audit Trail**: Track policy execution and compliance status

---

## Quick Start

### Your First Policy

Create a file called `basic-compliance.rules`:

```rules
WHEN node.vendor == "Cisco" THEN ASSERT custom_data.snmp_configured IS true
```

Test it:

```bash
unet policy validate basic-compliance.rules
unet policy eval basic-compliance.rules
```

This policy ensures all Cisco devices have SNMP configured.

---

## Policy Language Syntax

### Basic Structure

Every policy rule follows this pattern:

```text
WHEN <condition> THEN <action>
```

- **WHEN**: Introduces the condition that triggers the action
- **Condition**: Boolean expression evaluated against node data  
- **THEN**: Separates condition from action
- **Action**: Operation to perform when condition is true

### Example Policy Rule

```rules
WHEN node.vendor == "Cisco" AND node.lifecycle == "Production" 
THEN SET custom_data.backup_enabled TO true
```

---

## Conditions

### Field References

Access node properties using dot notation:

| Field Path | Data Type | Example |
|-----------|-----------|---------|
| `node.id` | UUID | `550e8400-e29b-41d4-a716-446655440000` |
| `node.name` | String | `"core-switch-01"` |
| `node.vendor` | String | `"Cisco"`, `"Juniper"`, `"Arista"` |
| `node.model` | String | `"ASR9000"`, `"EX4300"` |
| `node.role` | String | `"Core"`, `"Access"`, `"Edge"` |
| `node.lifecycle` | String | `"Production"`, `"Staging"` |
| `node.management_ip` | String | `"192.168.1.1"` |
| `custom_data.field` | Any | JSON field access |

### Comparison Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `==` | Equal to | `node.vendor == "Cisco"` |
| `!=` | Not equal to | `node.lifecycle != "Decommissioned"` |
| `>` | Greater than | `custom_data.port_count > 48` |
| `<` | Less than | `custom_data.cpu_percent < 80` |
| `>=` | Greater than or equal | `custom_data.uptime >= 86400` |
| `<=` | Less than or equal | `custom_data.memory_usage <= 90` |
| `CONTAINS` | String contains | `node.model CONTAINS "4000"` |
| `MATCHES` | Regex match | `node.name MATCHES "^core-"` |

### Logical Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `AND` | Both conditions true | `node.vendor == "Cisco" AND node.role == "Core"` |
| `OR` | Either condition true | `node.lifecycle == "Production" OR node.lifecycle == "Staging"` |
| `NOT` | Negate condition | `NOT (node.lifecycle == "Decommissioned")` |

### Parentheses for Grouping

```rules
WHEN (node.vendor == "Cisco" OR node.vendor == "Juniper") 
     AND node.lifecycle == "Production"
THEN ASSERT custom_data.monitoring_enabled IS true
```

---

## Actions

### ASSERT Action

Validate that a condition is true. Used for compliance checking.

**Syntax:**

```text
ASSERT <field> IS <expected_value>
ASSERT <condition>
```

**Examples:**

```rules
# Check field value
WHEN node.vendor == "Cisco" 
THEN ASSERT custom_data.snmp_enabled IS true

# Check complex condition  
WHEN node.role == "Core"
THEN ASSERT custom_data.redundancy_level >= 2

# Simple assertion with message
WHEN node.lifecycle == "Production"
THEN ASSERT "Device is properly configured for production"
```

### SET Action

Modify node data by setting field values.

**Syntax:**

```text
SET <field> TO <value>
```

**Examples:**

```rules
# Set simple value
WHEN node.vendor == "Cisco" 
THEN SET custom_data.backup_method TO "TFTP"

# Set complex data
WHEN node.role == "Access"
THEN SET custom_data.vlan_config TO {"default": 100, "voice": 200}

# Set based on condition
WHEN node.lifecycle == "Production"
THEN SET custom_data.monitoring_interval TO 300
```

### Custom Data Access

Access nested JSON data in the `custom_data` field:

```rules
# Simple field access
custom_data.location_code

# Nested field access  
custom_data.network.vlan_id
custom_data.hardware.memory_gb
custom_data.monitoring.alert_threshold

# Array access (planned feature)
custom_data.interfaces[0].name
```

---

## Data Types and Values

### String Values

```text
"Cisco"
"ASR9000" 
"192.168.1.1"
```

### Numeric Values

```text
100        # Integer
3.14       # Float
48         # Port count
80.5       # Percentage
```

### Boolean Values

```text
true
false
```

### Null Values

```text
null
```

---

## Common Policy Patterns

### Vendor-Specific Compliance

```rules
# Cisco devices should have specific SNMP settings
WHEN node.vendor == "Cisco" 
THEN SET custom_data.snmp_version TO "v3"

# Juniper devices need different backup method
WHEN node.vendor == "Juniper"
THEN SET custom_data.backup_protocol TO "SCP"
```

### Role-Based Configuration

```rules
# Core devices need redundancy
WHEN node.role == "Core"
THEN ASSERT custom_data.redundancy_enabled IS true

# Access switches have port security
WHEN node.role == "Access" 
THEN SET custom_data.port_security_enabled TO true

# Edge devices need firewall rules
WHEN node.role == "Edge"
THEN ASSERT custom_data.firewall_configured IS true
```

### Lifecycle Management

```rules
# Production devices must be monitored
WHEN node.lifecycle == "Production"
THEN SET custom_data.monitoring_enabled TO true

# Decommissioned devices should be isolated
WHEN node.lifecycle == "Decommissioned"  
THEN SET custom_data.network_access TO false

# Staging devices need test configuration
WHEN node.lifecycle == "Staging"
THEN SET custom_data.test_mode TO true
```

### Location-Based Policies

```rules
# Data center devices need specific settings
WHEN custom_data.location_type == "datacenter"
THEN SET custom_data.power_monitoring TO true

# Remote sites have different backup schedules
WHEN custom_data.location_type == "remote_office"
THEN SET custom_data.backup_schedule TO "weekly"
```

### Conditional Data Management

```rules
# Set VLAN based on device role
WHEN node.role == "Access"
THEN SET custom_data.default_vlan TO 100

WHEN node.role == "Core" 
THEN SET custom_data.default_vlan TO 1

# Configure monitoring intervals by importance
WHEN node.role == "Core" OR node.role == "Edge"
THEN SET custom_data.monitoring_interval TO 60

WHEN node.role == "Access"
THEN SET custom_data.monitoring_interval TO 300
```

---

## Policy File Organization

### Single Policy File

```rules
# File: network-compliance.rules

# Vendor compliance
WHEN node.vendor == "Cisco" THEN SET custom_data.snmp_version TO "v3"
WHEN node.vendor == "Juniper" THEN SET custom_data.backup_method TO "SCP"

# Role-based settings  
WHEN node.role == "Core" THEN ASSERT custom_data.redundancy IS true
WHEN node.role == "Access" THEN SET custom_data.port_security TO true
```

### Multiple Policy Files

```bash
policies/
├── vendor-compliance.rules      # Vendor-specific rules
├── role-configuration.rules     # Role-based configuration  
├── lifecycle-management.rules   # Lifecycle policies
└── security-baseline.rules     # Security requirements
```

### Policy File Comments

```rules
# This policy ensures core devices have redundancy
WHEN node.role == "Core" 
THEN ASSERT custom_data.redundancy_level >= 2

# TODO: Add environmental monitoring for data center devices
# WHEN custom_data.location == "datacenter"
# THEN ASSERT custom_data.temperature_monitoring IS true
```

---

## Working with Policies

### Validating Policy Syntax

```bash
# Validate single file
unet policy validate network-compliance.rules

# Validate directory of policies
unet policy validate policies/

# Validate with detailed output
unet policy validate policies/ --verbose
```

### Evaluating Policies

```bash
# Evaluate policies against all nodes
unet policy eval network-compliance.rules

# Evaluate against specific node
unet policy eval policies/ --node core-switch-01

# Show only failures
unet policy eval policies/ --failures-only

# Detailed evaluation output
unet policy eval policies/ --verbose
```

### Viewing Policy Results

```bash
# List policy files and their rules
unet policy list policies/

# Show policy file contents
unet policy show network-compliance.rules

# Show parsed rules (AST)
unet policy show network-compliance.rules --ast
```

---

## API Integration

### Evaluate Policies via API

```bash
curl -X POST http://localhost:8080/api/v1/policies/evaluate \
  -H "Content-Type: application/json" \
  -d '{
    "policies": [
      {
        "id": "cisco-compliance",
        "condition": "node.vendor == \"Cisco\"",
        "action": "assert(\"SNMP configured\")"
      }
    ]
  }'
```

### Get Policy Results

```bash
curl http://localhost:8080/api/v1/policies/results?node_id=550e8400-e29b-41d4-a716-446655440000
```

---

## Best Practices

### Policy Design

1. **Start Simple**: Begin with basic compliance checks before complex logic
2. **Use Descriptive Names**: Make policy intent clear from the rule
3. **Group Related Rules**: Organize policies by vendor, role, or function
4. **Test Incrementally**: Validate syntax before adding to production

### Rule Organization

```rules
# Good: Clear, specific rule
WHEN node.vendor == "Cisco" AND node.role == "Core"
THEN ASSERT custom_data.hsrp_configured IS true

# Avoid: Complex nested conditions
WHEN (node.vendor == "Cisco" OR node.vendor == "Juniper") 
     AND (node.role == "Core" OR node.role == "Distribution")
     AND (custom_data.location == "datacenter" OR custom_data.location == "colo")
THEN SET custom_data.complex_setting TO true
```

### Error Handling

1. **Use ASSERT for Compliance**: Check requirements with clear messages
2. **Handle Missing Data**: Consider what happens with null/undefined values
3. **Test Edge Cases**: Verify policies work with minimal data sets

### Version Control

```bash
# Store policies in Git
git add policies/
git commit -m "Add network compliance policies"

# Review policy changes
git diff policies/network-compliance.rules
```

---

## Troubleshooting

### Common Syntax Errors

```rules
# Error: Missing quotes around string
WHEN node.vendor == Cisco  # ❌ Wrong
WHEN node.vendor == "Cisco"  # ✅ Correct

# Error: Invalid field reference  
WHEN node.invalid_field == "value"  # ❌ Wrong
WHEN custom_data.valid_field == "value"  # ✅ Correct

# Error: Missing THEN keyword
WHEN node.vendor == "Cisco" SET custom_data.x TO true  # ❌ Wrong  
WHEN node.vendor == "Cisco" THEN SET custom_data.x TO true  # ✅ Correct
```

### Debugging Policy Evaluation

```bash
# Check if policies parse correctly
unet policy validate policies/ --verbose

# Test against specific node  
unet policy eval policies/ --node test-device-01 --verbose

# Examine policy AST
unet policy show policies/debug.rules --ast
```

### Common Issues

1. **Policy Not Triggering**: Check condition syntax and field names
2. **SET Action Failing**: Verify field path and value type
3. **ASSERT Always Failing**: Check for null/undefined values

---

## Performance Considerations

### Efficient Policy Design

- **Use Specific Conditions**: Avoid overly broad conditions that match many nodes
- **Limit Custom Data Access**: Deep JSON traversal can be slow
- **Batch Similar Rules**: Group related checks in single policies

### Evaluation Performance

- **Node Count**: Policies evaluate against all matching nodes
- **Rule Complexity**: Complex conditions take more time to evaluate  
- **Custom Data Size**: Large JSON objects slow down field access

---

## Limitations (Current Version)

- **Array Access**: Custom data array indexing not yet supported
- **Function Calls**: No built-in functions (length, contains, etc.)
- **Cross-Node References**: Cannot reference other nodes in conditions
- **Template Integration**: APPLY action is infrastructure-ready but templates not implemented

---

## Examples

### Network Security Baseline

```rules
# File: security-baseline.rules

# All production devices must have SNMP v3
WHEN node.lifecycle == "Production"
THEN ASSERT custom_data.snmp_version == "v3"

# Core devices need access control
WHEN node.role == "Core"  
THEN ASSERT custom_data.acl_configured IS true

# Edge devices require firewall
WHEN node.role == "Edge"
THEN ASSERT custom_data.firewall_enabled IS true

# Management interfaces should be secured
WHEN node.management_ip != null
THEN SET custom_data.mgmt_security_enabled TO true
```

### Device Lifecycle Management

```rules
# File: lifecycle-management.rules

# New devices start in staging
WHEN custom_data.deployment_status == "new"
THEN SET node.lifecycle TO "Staging"

# Production readiness checks
WHEN node.lifecycle == "Staging" AND custom_data.testing_complete IS true
THEN SET node.lifecycle TO "Production"

# Decommissioning workflow
WHEN custom_data.scheduled_replacement != null
THEN SET custom_data.decommission_planned TO true

# Remove access for decommissioned devices
WHEN node.lifecycle == "Decommissioned"
THEN SET custom_data.network_access TO false
```

### Vendor-Specific Configuration

```rules
# File: vendor-configuration.rules

# Cisco device standards
WHEN node.vendor == "Cisco"
THEN SET custom_data.config_backup_method TO "TFTP"

WHEN node.vendor == "Cisco" AND node.role == "Core"  
THEN SET custom_data.routing_protocol TO "OSPF"

# Juniper device standards
WHEN node.vendor == "Juniper"
THEN SET custom_data.config_backup_method TO "SCP"

WHEN node.vendor == "Juniper" AND node.role == "Edge"
THEN SET custom_data.security_zones TO ["DMZ", "INTERNAL"]

# Arista device standards  
WHEN node.vendor == "Arista"
THEN SET custom_data.management_protocol TO "eAPI"
```

For more advanced examples and integration patterns, see the [API Reference](api_reference.md) and [CLI Reference](cli_reference.md).
