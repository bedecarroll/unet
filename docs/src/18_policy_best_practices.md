<!-- SPDX-License-Identifier: MIT -->

# Policy Best Practices Guide

This guide provides advanced patterns, best practices, and optimization techniques for μNet policy development and deployment.

## Policy Design Principles

### 1. Single Responsibility Principle

Each policy rule should have a single, well-defined purpose.

**Good:**

```
// Clear purpose: Configure SNMP for Cisco devices
WHEN node.vendor == "cisco" THEN SET custom_data.snmp.version TO "v2c"

// Clear purpose: Enable monitoring for production devices  
WHEN node.lifecycle == "live" THEN SET custom_data.monitoring.enabled TO true
```

**Avoid:**

```
// Multiple responsibilities in one rule
WHEN node.vendor == "cisco" 
THEN SET custom_data.snmp.version TO "v2c" 
     AND SET custom_data.monitoring.enabled TO true
     AND APPLY "templates/cisco-base.j2"
```

### 2. Declarative over Imperative

Focus on describing the desired state rather than the steps to achieve it.

**Good:**

```
// Declares desired state
WHEN node.role == "router" THEN SET custom_data.bgp.enabled TO true
WHEN custom_data.bgp.enabled == true THEN SET custom_data.bgp.as_number TO 65001
```

**Avoid:**

```
// Too imperative/procedural
WHEN node.role == "router" THEN SET custom_data.step1_complete TO true
WHEN custom_data.step1_complete == true THEN SET custom_data.step2_complete TO true
```

### 3. Idempotency

Policies should be safe to run multiple times without side effects.

**Good:**

```
// Safe to run repeatedly
WHEN node.vendor == "cisco" THEN SET custom_data.vendor_class TO "ios"
```

**Good (with guards):**

```
// Only applies template if not already applied
WHEN node.role == "router" AND custom_data.assigned_templates NOT CONTAINS "router-base.j2" 
THEN APPLY "templates/router-base.j2"
```

## Organizational Patterns

### Policy File Structure

#### Small Deployments (< 100 devices)

Use a single policy file organized by sections:

```
# network-policies.txt

# ============================================================================
# DEVICE CLASSIFICATION
# ============================================================================

WHEN node.vendor == "cisco" THEN SET custom_data.vendor_class TO "ios"
WHEN node.vendor == "juniper" THEN SET custom_data.vendor_class TO "junos"

# ============================================================================
# MONITORING CONFIGURATION  
# ============================================================================

WHEN node.lifecycle == "live" THEN SET custom_data.monitoring.enabled TO true
WHEN custom_data.criticality == "critical" THEN SET custom_data.monitoring.interval TO 60

# ============================================================================
# TEMPLATE ASSIGNMENT
# ============================================================================

WHEN node.role == "router" THEN APPLY "templates/router-base.j2"
WHEN node.role == "switch" THEN APPLY "templates/switch-base.j2"
```

#### Medium Deployments (100-1000 devices)

Organize by functional areas:

```
policies/
├── 01-classification.policy    # Device classification rules
├── 02-security.policy         # Security and compliance  
├── 03-monitoring.policy        # Monitoring configuration
├── 04-networking.policy        # Network-specific settings
├── 05-templates.policy         # Template assignments
└── 99-cleanup.policy          # Cleanup and maintenance
```

#### Large Deployments (1000+ devices)

Use hierarchical organization:

```
policies/
├── global/                     # Organization-wide policies
│   ├── security.policy
│   ├── monitoring.policy
│   └── compliance.policy
├── regions/                    # Regional policies
│   ├── us-east.policy
│   ├── us-west.policy
│   └── europe.policy
├── environments/               # Environment-specific  
│   ├── production.policy
│   ├── staging.policy
│   └── development.policy
└── services/                   # Service-specific
    ├── web-tier.policy
    ├── database-tier.policy
    └── network-infrastructure.policy
```

### Policy Ordering and Dependencies

#### Execution Order

Policies should be designed to work regardless of execution order, but when dependencies exist, use clear naming:

```
01-base-classification.policy
02-vendor-specific.policy  
03-role-configuration.policy
04-location-settings.policy
05-template-assignment.policy
```

#### Dependency Management

Use condition chaining for dependencies:

```
# Step 1: Base classification
WHEN node.vendor == "cisco" THEN SET custom_data.vendor_class TO "ios"

# Step 2: Build on classification  
WHEN custom_data.vendor_class == "ios" AND node.role == "switch" 
THEN SET custom_data.device_type TO "ios_switch"

# Step 3: Use device type for configuration
WHEN custom_data.device_type == "ios_switch" 
THEN APPLY "templates/cisco-switch.j2"
```

## Performance Optimization

### Condition Optimization

#### Put Selective Conditions First

```
# Good: Most selective condition first
WHEN node.serial_number == "ABC123XYZ" AND node.vendor == "cisco" 
THEN SET custom_data.special_device TO true

# Less efficient: Generic condition first  
WHEN node.vendor == "cisco" AND node.serial_number == "ABC123XYZ"
THEN SET custom_data.special_device TO true
```

#### Use Appropriate Operators

```
# Good: Simple equality is fastest
WHEN node.vendor == "cisco" THEN ...

# Good: CONTAINS for substring matching
WHEN node.model CONTAINS "2960" THEN ...

# Use carefully: Regex is slower
WHEN node.name MATCHES "^rtr-[0-9]+-.*" THEN ...

# Avoid: Complex regex for simple cases
WHEN node.name MATCHES "cisco|Cisco|CISCO" THEN ...
# Better: Use case-insensitive comparison
WHEN node.vendor == "cisco" THEN ...
```

#### Minimize Field Access

```
# Good: Access field once
WHEN custom_data.location.datacenter == "dc1" 
THEN SET custom_data.dc1_config TO true

# Less efficient: Multiple accesses to same field
WHEN custom_data.location.datacenter == "dc1" AND custom_data.location.datacenter != "dc2"
THEN SET custom_data.dc1_config TO true
```

### Memory Optimization

#### Avoid Deeply Nested Structures

```
# Good: Reasonable nesting
SET custom_data.monitoring.snmp.version TO "v3"

# Avoid: Excessive nesting  
SET custom_data.level1.level2.level3.level4.level5.value TO "deep"
```

#### Use Efficient Data Structures

```
# Good: Use arrays for lists
SET custom_data.allowed_vlans TO ["100", "200", "300"]

# Good: Use objects for structured data
SET custom_data.location.datacenter TO "dc1"
SET custom_data.location.rack TO "R42"
SET custom_data.location.unit TO "20"

# Avoid: Flat key-value pairs for structured data
SET custom_data.location_datacenter TO "dc1"
SET custom_data.location_rack TO "R42"  
SET custom_data.location_unit TO "20"
```

## Error Handling and Resilience

### Defensive Programming

#### Check Prerequisites

```
# Good: Verify prerequisites exist
WHEN node.management_ip IS NOT NULL AND node.lifecycle == "live"
THEN SET custom_data.monitoring.enabled TO true

# Good: Guard against missing data
WHEN custom_data.location IS NOT NULL AND custom_data.location.datacenter == "dc1"
THEN SET custom_data.ntp_server TO "10.1.1.10"
```

#### Handle Edge Cases

```
# Handle devices without models
WHEN node.model IS NULL OR node.model == ""
THEN SET custom_data.model_unknown TO true

# Handle numeric edge cases
WHEN custom_data.port_count IS NOT NULL AND custom_data.port_count > 0
THEN SET custom_data.ports_configured TO true

# Handle string edge cases
WHEN node.name IS NOT NULL AND node.name != ""
THEN SET custom_data.name_configured TO true
```

### Graceful Degradation

```
# Primary configuration  
WHEN custom_data.location.datacenter == "dc1"
THEN SET custom_data.ntp_server TO "10.1.1.10"

# Fallback configuration
WHEN custom_data.ntp_server IS NULL AND custom_data.location.region == "us-east"
THEN SET custom_data.ntp_server TO "pool.ntp.org"

# Final fallback
WHEN custom_data.ntp_server IS NULL
THEN SET custom_data.ntp_server TO "time.nist.gov"
```

### Error Recovery

```
# Mark problematic devices for manual review
WHEN node.vendor IS NULL OR node.vendor == ""
THEN SET custom_data.requires_manual_review TO true

# Set safe defaults
WHEN custom_data.requires_manual_review == true
THEN SET custom_data.monitoring.interval TO 300

# Skip complex processing for problematic devices
WHEN custom_data.requires_manual_review != true AND node.role == "router"
THEN APPLY "templates/router-config.j2"
```

## Security Best Practices

### Data Protection

#### Avoid Sensitive Data in Policies

```
# Good: Reference sensitive data indirectly
WHEN node.role == "radius_server" 
THEN SET custom_data.auth_method TO "certificate"

# Avoid: Hardcoded sensitive values
WHEN node.role == "radius_server" 
THEN SET custom_data.shared_secret TO "supersecret123"
```

#### Use Environment-Specific Values

```
# Good: Environment-specific references
WHEN custom_data.environment == "prod" 
THEN SET custom_data.snmp.community_ref TO "prod_snmp_community"

WHEN custom_data.environment == "dev"
THEN SET custom_data.snmp.community_ref TO "dev_snmp_community"
```

### Access Control

#### Role-Based Configuration

```
# Admin access configuration
WHEN custom_data.admin_access_required == true
THEN SET custom_data.mgmt.admin_users TO ["netadmin", "sysadmin"]

# Read-only access configuration  
WHEN custom_data.monitoring_only == true
THEN SET custom_data.mgmt.readonly_users TO ["monitor", "observer"]

# No external access
WHEN custom_data.external_access == false
THEN SET custom_data.mgmt.ssh_from_external TO false
```

#### Principle of Least Privilege

```
# Default: Minimal access
WHEN node.lifecycle == "live"
THEN SET custom_data.security.default_access TO "deny"

# Specific: Grant only needed access
WHEN node.role == "management_server"
THEN SET custom_data.security.mgmt_access TO "allow"

# Temporary: Time-bound access
WHEN custom_data.maintenance_window == true
THEN SET custom_data.security.temp_access TO "allow"
```

## Testing and Validation

### Policy Testing Strategy

#### Unit Testing Individual Rules

```bash
# Test single rule
unet policies evaluate single-rule.policy --node test-node-01 --dry-run

# Test with different node types
unet policies evaluate classification.policy --node cisco-switch-01 --dry-run
unet policies evaluate classification.policy --node juniper-router-01 --dry-run
```

#### Integration Testing

```bash
# Test policy combinations
unet policies evaluate policies/ --node production-router-01 --dry-run

# Test with real data
unet policies evaluate policies/ --node-filter "lifecycle==live" --limit 10
```

### Validation Patterns

#### Comprehensive Assertions

```
# Validate all required fields are set
WHEN node.lifecycle == "live"
THEN ASSERT custom_data.monitoring.enabled IS true

WHEN custom_data.monitoring.enabled == true
THEN ASSERT custom_data.snmp.version IS NOT NULL

WHEN node.role == "firewall"
THEN ASSERT custom_data.security.enabled IS true
```

#### Cross-Validation

```
# Ensure consistency between related fields
WHEN custom_data.environment == "prod"
THEN ASSERT custom_data.backup.enabled IS true

WHEN custom_data.criticality == "critical"
THEN ASSERT custom_data.monitoring.interval IS 60

WHEN custom_data.security_zone == "dmz"
THEN ASSERT custom_data.security.hardened IS true
```

### Rollback Testing

```
# Test policy rollback capability
WHEN custom_data.test_rollback == true
THEN SET custom_data.original_value TO "backup"

# Mark for rollback testing
WHEN custom_data.test_mode == true
THEN SET custom_data.rollback_test_data TO "test_value"
```

## Maintenance and Lifecycle

### Version Control

#### Policy Versioning

```
# Header comment with version info
# Policy Version: 2.1.0
# Last Modified: 2024-01-15  
# Author: Network Team
# Purpose: Device classification and monitoring setup

WHEN node.vendor == "cisco" THEN SET custom_data.vendor_class TO "ios"
```

#### Change Management

```bash
# Use git for policy version control
git add policies/
git commit -m "feat: add support for Arista switches in classification policy"
git tag -a v2.1.0 -m "Release 2.1.0: Arista support"
```

### Documentation

#### Self-Documenting Policies

```
# Good: Clear, descriptive conditions
WHEN node.vendor == "cisco" AND node.model CONTAINS "catalyst" AND custom_data.location.tier == "access"
THEN SET custom_data.switch_profile TO "cisco_catalyst_access"

# Good: Meaningful field names
SET custom_data.monitoring.health_check_interval TO 300
SET custom_data.backup.configuration_backup_enabled TO true
```

#### Policy Documentation Template

```
# ============================================================================
# POLICY: Device Classification
# PURPOSE: Classify network devices by vendor, role, and capabilities
# SCOPE: All managed network devices
# DEPENDENCIES: None
# AUTHOR: Network Automation Team
# LAST_UPDATED: 2024-01-15
# ============================================================================

# Cisco device classification
WHEN node.vendor == "cisco" THEN SET custom_data.vendor_class TO "ios"
WHEN custom_data.vendor_class == "ios" AND node.model CONTAINS "2960" 
THEN SET custom_data.device_family TO "catalyst_access"
```

### Monitoring and Observability

#### Policy Execution Metrics

```
# Track policy execution
WHEN custom_data.policy_executed != true
THEN SET custom_data.policy_executed TO true

WHEN custom_data.policy_executed == true
THEN SET custom_data.last_policy_run TO "2024-01-15T10:30:00Z"

# Count policy applications
WHEN custom_data.policy_application_count IS NULL
THEN SET custom_data.policy_application_count TO 1

WHEN custom_data.policy_application_count IS NOT NULL
THEN SET custom_data.policy_application_count TO custom_data.policy_application_count + 1
```

#### Health Checks

```
# Validate policy results
WHEN custom_data.monitoring.enabled == true AND custom_data.monitoring.last_check IS NULL
THEN SET custom_data.health.monitoring_check_needed TO true

# Check for policy conflicts
WHEN custom_data.assigned_templates CONTAINS "conflicting-template.j2"
THEN SET custom_data.health.template_conflict TO true
```

## Advanced Patterns

### State Machines

```
# Device provisioning state machine
WHEN node.lifecycle == "planned" AND custom_data.provisioning_stage IS NULL
THEN SET custom_data.provisioning_stage TO "initial"

WHEN custom_data.provisioning_stage == "initial" AND custom_data.ip_assigned == true
THEN SET custom_data.provisioning_stage TO "network_ready"

WHEN custom_data.provisioning_stage == "network_ready" AND custom_data.base_config_applied == true
THEN SET custom_data.provisioning_stage TO "configured"

WHEN custom_data.provisioning_stage == "configured" AND custom_data.validation_passed == true
THEN SET custom_data.provisioning_stage TO "production_ready"
```

### Feature Flags

```
# Feature flag pattern for gradual rollouts
WHEN custom_data.feature_flags.new_monitoring == true AND node.role == "router"
THEN SET custom_data.monitoring.new_system_enabled TO true

# Canary deployment pattern
WHEN custom_data.deployment_group == "canary" AND custom_data.feature_flags.beta_features == true
THEN APPLY "templates/beta/new-features.j2"

# A/B testing pattern
WHEN custom_data.ab_test_group == "group_a"
THEN SET custom_data.config_variant TO "variant_a"

WHEN custom_data.ab_test_group == "group_b"
THEN SET custom_data.config_variant TO "variant_b"
```

### Multi-Tenancy

```
# Tenant-specific configuration
WHEN custom_data.tenant == "customer_a"
THEN SET custom_data.vlans.customer_range TO "100-199"

WHEN custom_data.tenant == "customer_b"
THEN SET custom_data.vlans.customer_range TO "200-299"

# Tenant isolation
WHEN custom_data.tenant == "customer_a" AND node.role == "switch"
THEN APPLY "templates/tenants/customer-a-switch.j2"

# Shared infrastructure
WHEN custom_data.shared_infrastructure == true
THEN SET custom_data.tenant TO "shared"
```

## Common Anti-Patterns

### Anti-Pattern: Over-Complex Conditions

**Avoid:**

```
WHEN (node.vendor == "cisco" OR node.vendor == "Cisco" OR node.vendor == "CISCO") 
     AND (node.role == "switch" OR node.role == "Switch") 
     AND (custom_data.location CONTAINS "dc" OR custom_data.location CONTAINS "datacenter")
     AND NOT (node.lifecycle == "decommissioned" OR node.lifecycle == "planned")
THEN SET custom_data.complex_match TO true
```

**Better:**

```
# Normalize vendor field first
WHEN node.vendor MATCHES "(?i)cisco" THEN SET custom_data.vendor_normalized TO "cisco"

# Simple, clear conditions
WHEN custom_data.vendor_normalized == "cisco" AND node.role == "switch" 
     AND custom_data.location.type == "datacenter" AND node.lifecycle == "live"
THEN SET custom_data.datacenter_cisco_switch TO true
```

### Anti-Pattern: Magic Numbers and Strings

**Avoid:**

```
WHEN custom_data.cpu_usage > 85.7 THEN SET custom_data.alert_level TO "red"
WHEN custom_data.memory_usage > 92.3 THEN SET custom_data.alert_level TO "red"
```

**Better:**

```
# Define thresholds clearly
WHEN node.role == "router" THEN SET custom_data.thresholds.cpu_critical TO 85
WHEN node.role == "switch" THEN SET custom_data.thresholds.cpu_critical TO 90

# Use defined thresholds
WHEN custom_data.cpu_usage > custom_data.thresholds.cpu_critical
THEN SET custom_data.alert_level TO "critical"
```

### Anti-Pattern: Implicit Dependencies

**Avoid:**

```
# Hidden dependency - assumes classification already happened
WHEN custom_data.device_class == "critical" THEN SET custom_data.backup.frequency TO "hourly"
```

**Better:**

```
# Explicit prerequisite checking
WHEN custom_data.device_class IS NOT NULL AND custom_data.device_class == "critical"
THEN SET custom_data.backup.frequency TO "hourly"

# Or make dependency explicit
WHEN node.role == "core_router" AND custom_data.criticality == "high"
THEN SET custom_data.backup.frequency TO "hourly"
```

By following these best practices, you'll create maintainable, efficient, and robust policy systems that scale with your network automation needs.
