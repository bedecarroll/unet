# Policy Examples

This document provides real-world examples of μNet policy rules for common network automation scenarios.

## Basic Examples

### Device Classification

```
# Classify Cisco devices by product family
WHEN node.vendor == "cisco" AND node.model CONTAINS "2960" 
THEN SET custom_data.device_family TO "catalyst_access"

WHEN node.vendor == "cisco" AND node.model CONTAINS "3850" 
THEN SET custom_data.device_family TO "catalyst_distribution"

WHEN node.vendor == "cisco" AND node.model CONTAINS "9300" 
THEN SET custom_data.device_family TO "catalyst_next_gen"

# Classify by role and capabilities  
WHEN node.role == "switch" AND custom_data.port_count <= 24 
THEN SET custom_data.switch_tier TO "access"

WHEN node.role == "switch" AND custom_data.port_count > 24 AND custom_data.port_count <= 48 
THEN SET custom_data.switch_tier TO "distribution"

WHEN node.role == "switch" AND custom_data.port_count > 48 
THEN SET custom_data.switch_tier TO "core"
```

### SNMP Configuration

```
# Set SNMP version based on security requirements
WHEN node.lifecycle == "live" AND custom_data.security_zone == "dmz" 
THEN SET custom_data.snmp.version TO "v3"

WHEN node.lifecycle == "live" AND custom_data.security_zone == "internal" 
THEN SET custom_data.snmp.version TO "v2c"

# Configure SNMP communities
WHEN custom_data.snmp.version == "v2c" 
THEN SET custom_data.snmp.read_community TO "public"

WHEN custom_data.snmp.version == "v3" 
THEN SET custom_data.snmp.auth_protocol TO "SHA"
```

### Monitoring Setup

```
# Enable monitoring for production devices
WHEN node.lifecycle == "live" 
THEN SET custom_data.monitoring.enabled TO true

# Set monitoring intervals based on device criticality
WHEN custom_data.criticality == "critical" 
THEN SET custom_data.monitoring.poll_interval TO 60

WHEN custom_data.criticality == "standard" 
THEN SET custom_data.monitoring.poll_interval TO 300

# Configure vendor-specific monitoring
WHEN node.vendor == "cisco" AND custom_data.monitoring.enabled == true 
THEN SET custom_data.monitoring.snmp_oids TO "1.3.6.1.4.1.9.9.48.1.1.1.5"

WHEN node.vendor == "juniper" AND custom_data.monitoring.enabled == true 
THEN SET custom_data.monitoring.snmp_oids TO "1.3.6.1.4.1.2636.3.1.13.1.8"
```

## Compliance and Security

### Security Hardening

```
# Enforce SSH version for management access
WHEN node.lifecycle == "live" 
THEN SET custom_data.mgmt.ssh_version TO "2"

# Disable insecure protocols
WHEN node.lifecycle == "live" 
THEN SET custom_data.security.telnet_enabled TO false

WHEN node.lifecycle == "live" 
THEN SET custom_data.security.http_enabled TO false

# Set password policies
WHEN node.role == "router" OR node.role == "switch" 
THEN SET custom_data.security.min_password_length TO 12

# Configure logging for security events
WHEN custom_data.security_zone == "dmz" 
THEN SET custom_data.logging.security_events TO true
```

### Compliance Validation

```
# Validate required software versions
WHEN node.vendor == "cisco" AND node.role == "router" 
THEN ASSERT node.version IS "15.1"

WHEN node.vendor == "juniper" AND node.role == "router" 
THEN ASSERT node.version IS "18.4R2"

# Validate security configurations
WHEN node.lifecycle == "live" 
THEN ASSERT custom_data.security.ssh_version IS "2"

WHEN custom_data.security_zone == "dmz" 
THEN ASSERT custom_data.security.telnet_enabled IS false

# Validate backup configurations
WHEN node.lifecycle == "live" 
THEN ASSERT custom_data.backup.enabled IS true

WHEN custom_data.backup.enabled == true 
THEN ASSERT custom_data.backup.frequency IS "daily"
```

### Audit and Governance

```
# Tag devices requiring PCI compliance
WHEN custom_data.environment == "prod" AND custom_data.handles_payment_data == true 
THEN SET custom_data.compliance.pci_required TO true

# Set data retention policies
WHEN custom_data.compliance.pci_required == true 
THEN SET custom_data.logging.retention_days TO 365

WHEN custom_data.compliance.pci_required != true 
THEN SET custom_data.logging.retention_days TO 90

# Enforce change management
WHEN node.lifecycle == "live" 
THEN SET custom_data.change_mgmt.approval_required TO true
```

## Network Architecture

### Location-Based Configuration

```
# Configure NTP servers by location
WHEN custom_data.location.region == "us-east" 
THEN SET custom_data.ntp.primary_server TO "10.1.1.10"

WHEN custom_data.location.region == "us-west" 
THEN SET custom_data.ntp.primary_server TO "10.2.1.10"

WHEN custom_data.location.region == "europe" 
THEN SET custom_data.ntp.primary_server TO "10.3.1.10"

# Set DNS servers by datacenter
WHEN custom_data.location.datacenter == "dc1" 
THEN SET custom_data.dns.primary TO "10.1.1.53"

WHEN custom_data.location.datacenter == "dc2" 
THEN SET custom_data.dns.primary TO "10.2.1.53"

# Configure syslog destinations
WHEN custom_data.location.site == "headquarters" 
THEN SET custom_data.logging.syslog_server TO "10.1.1.100"

WHEN custom_data.location.site != "headquarters" 
THEN SET custom_data.logging.syslog_server TO "10.1.1.101"
```

### VLAN and Network Segmentation

```
# Assign management VLANs by location
WHEN custom_data.location.datacenter == "dc1" 
THEN SET custom_data.vlans.management TO 100

WHEN custom_data.location.datacenter == "dc2" 
THEN SET custom_data.vlans.management TO 200

# Configure security zones
WHEN node.role == "firewall" 
THEN SET custom_data.zones.dmz_vlan TO 10

WHEN node.role == "firewall" 
THEN SET custom_data.zones.internal_vlan TO 20

# Set VLAN ranges for access switches
WHEN custom_data.switch_tier == "access" AND custom_data.location.floor == 1 
THEN SET custom_data.vlans.user_range TO "101-110"

WHEN custom_data.switch_tier == "access" AND custom_data.location.floor == 2 
THEN SET custom_data.vlans.user_range TO "111-120"
```

### Routing Configuration

```
# Configure BGP for edge routers
WHEN node.role == "router" AND custom_data.router_type == "edge" 
THEN SET custom_data.bgp.enabled TO true

WHEN custom_data.bgp.enabled == true AND custom_data.location.region == "us-east" 
THEN SET custom_data.bgp.as_number TO 65001

WHEN custom_data.bgp.enabled == true AND custom_data.location.region == "us-west" 
THEN SET custom_data.bgp.as_number TO 65002

# Configure OSPF areas
WHEN node.role == "router" AND custom_data.router_type == "internal" 
THEN SET custom_data.ospf.enabled TO true

WHEN custom_data.ospf.enabled == true AND custom_data.location.datacenter == "dc1" 
THEN SET custom_data.ospf.area TO "0.0.0.1"

WHEN custom_data.ospf.enabled == true AND custom_data.location.datacenter == "dc2" 
THEN SET custom_data.ospf.area TO "0.0.0.2"
```

## Template Assignment

### Hierarchical Template Application

```
# Base configuration for all devices
WHEN node.lifecycle == "live" 
THEN APPLY "templates/base-config.j2"

# Vendor-specific base templates
WHEN node.vendor == "cisco" 
THEN APPLY "templates/cisco/base.j2"

WHEN node.vendor == "juniper" 
THEN APPLY "templates/juniper/base.j2"

WHEN node.vendor == "arista" 
THEN APPLY "templates/arista/base.j2"

# Role-specific templates
WHEN node.role == "router" 
THEN APPLY "templates/router/base.j2"

WHEN node.role == "switch" 
THEN APPLY "templates/switch/base.j2"

WHEN node.role == "firewall" 
THEN APPLY "templates/firewall/base.j2"

# Specialized configurations
WHEN node.role == "router" AND custom_data.bgp.enabled == true 
THEN APPLY "templates/router/bgp.j2"

WHEN node.role == "switch" AND custom_data.switch_tier == "access" 
THEN APPLY "templates/switch/access-layer.j2"

WHEN custom_data.security_zone == "dmz" 
THEN APPLY "templates/security/dmz-hardening.j2"
```

### Environment-Specific Templates

```
# Development environment
WHEN custom_data.environment == "dev" 
THEN APPLY "templates/environments/development.j2"

# Staging environment
WHEN custom_data.environment == "staging" 
THEN APPLY "templates/environments/staging.j2"

# Production environment
WHEN custom_data.environment == "prod" 
THEN APPLY "templates/environments/production.j2"

# High availability configurations
WHEN custom_data.environment == "prod" AND custom_data.criticality == "critical" 
THEN APPLY "templates/ha/active-standby.j2"

# Disaster recovery configurations
WHEN custom_data.environment == "prod" AND custom_data.dr_site == true 
THEN APPLY "templates/dr/backup-site.j2"
```

## Advanced Scenarios

### Multi-Vendor Environment

```
# Cisco-specific configurations
WHEN node.vendor == "cisco" AND node.role == "switch" 
THEN SET custom_data.features.spanning_tree TO "rapid-pvst"

WHEN node.vendor == "cisco" AND node.role == "router" 
THEN SET custom_data.features.routing_protocol TO "eigrp"

# Juniper-specific configurations  
WHEN node.vendor == "juniper" AND node.role == "switch" 
THEN SET custom_data.features.spanning_tree TO "rstp"

WHEN node.vendor == "juniper" AND node.role == "router" 
THEN SET custom_data.features.routing_protocol TO "ospf"

# Arista-specific configurations
WHEN node.vendor == "arista" AND node.role == "switch" 
THEN SET custom_data.features.spanning_tree TO "mst"

# Cross-vendor interoperability
WHEN node.role == "router" 
THEN SET custom_data.interop.bgp_standard TO "rfc4271"

WHEN node.role == "switch" 
THEN SET custom_data.interop.lldp_enabled TO true
```

### Dynamic Configuration Based on Discovery

```
# Configure based on discovered interface count
WHEN derived.interface_count <= 8 
THEN SET custom_data.device_class TO "small"

WHEN derived.interface_count > 8 AND derived.interface_count <= 24 
THEN SET custom_data.device_class TO "medium"

WHEN derived.interface_count > 24 
THEN SET custom_data.device_class TO "large"

# Configure based on discovered capabilities
WHEN derived.capabilities CONTAINS "bridge" 
THEN SET custom_data.layer2_capable TO true

WHEN derived.capabilities CONTAINS "router" 
THEN SET custom_data.layer3_capable TO true

# Configure based on neighbor discovery
WHEN derived.neighbors.count > 10 
THEN SET custom_data.network_density TO "high"

WHEN derived.lldp_neighbors CONTAINS "firewall" 
THEN SET custom_data.security_adjacent TO true
```

### Service Chain Configuration

```
# Identify service chain position
WHEN derived.neighbors CONTAINS "load-balancer" AND derived.neighbors CONTAINS "firewall" 
THEN SET custom_data.service_chain.position TO "middle"

WHEN derived.neighbors CONTAINS "internet-gateway" 
THEN SET custom_data.service_chain.position TO "edge"

# Configure service insertion
WHEN custom_data.service_chain.position == "edge" 
THEN SET custom_data.services.traffic_inspection TO true

WHEN custom_data.service_chain.position == "middle" 
THEN SET custom_data.services.load_balancing TO true

# Apply service-specific templates
WHEN custom_data.services.traffic_inspection == true 
THEN APPLY "templates/services/traffic-inspection.j2"

WHEN custom_data.services.load_balancing == true 
THEN APPLY "templates/services/load-balancing.j2"
```

## Lifecycle Management

### Device Provisioning

```
# Initial provisioning setup
WHEN node.lifecycle == "planned" 
THEN SET custom_data.provisioning.stage TO "initial"

# Pre-deployment validation
WHEN node.lifecycle == "implementing" 
THEN SET custom_data.validation.required TO true

WHEN custom_data.validation.required == true 
THEN ASSERT custom_data.management_ip IS NOT NULL

# Production readiness
WHEN node.lifecycle == "implementing" AND custom_data.validation.passed == true 
THEN SET custom_data.ready_for_production TO true

# Automatic lifecycle progression
WHEN custom_data.ready_for_production == true 
THEN SET node.lifecycle TO "live"
```

### Maintenance and Updates

```
# Schedule maintenance windows
WHEN custom_data.maintenance.due == true AND custom_data.criticality == "standard" 
THEN SET custom_data.maintenance.window TO "weekend"

WHEN custom_data.maintenance.due == true AND custom_data.criticality == "critical" 
THEN SET custom_data.maintenance.window TO "planned_outage"

# Software update policies
WHEN node.vendor == "cisco" AND custom_data.software.current_version != "15.1" 
THEN SET custom_data.software.update_required TO true

# Backup before changes
WHEN custom_data.software.update_required == true 
THEN SET custom_data.backup.pre_change TO true
```

### Decommissioning

```
# Prepare for decommissioning
WHEN node.lifecycle == "decommissioned" 
THEN SET custom_data.monitoring.enabled TO false

WHEN node.lifecycle == "decommissioned" 
THEN SET custom_data.backup.enabled TO false

# Remove from production services
WHEN node.lifecycle == "decommissioned" 
THEN SET custom_data.dns.remove_records TO true

WHEN node.lifecycle == "decommissioned" 
THEN SET custom_data.monitoring.remove_checks TO true

# Data retention for decommissioned devices
WHEN node.lifecycle == "decommissioned" 
THEN SET custom_data.data_retention.config_backup TO "30_days"

WHEN node.lifecycle == "decommissioned" 
THEN SET custom_data.data_retention.logs TO "90_days"
```

## Integration Examples

### ITSM Integration

```
# Create change requests for configuration changes
WHEN custom_data.config.change_pending == true AND custom_data.criticality == "critical" 
THEN SET custom_data.itsm.change_request_required TO true

# Set approval requirements
WHEN custom_data.itsm.change_request_required == true 
THEN SET custom_data.itsm.approval_level TO "manager"

WHEN custom_data.environment == "prod" 
THEN SET custom_data.itsm.approval_level TO "change_board"

# Track change implementation
WHEN custom_data.itsm.approved == true 
THEN SET custom_data.config.deployment_authorized TO true
```

### Monitoring System Integration

```
# Configure monitoring based on device importance
WHEN custom_data.criticality == "critical" 
THEN SET custom_data.monitoring.alerting.escalation_time TO 5

WHEN custom_data.criticality == "standard" 
THEN SET custom_data.monitoring.alerting.escalation_time TO 15

# Set monitoring thresholds
WHEN node.role == "router" 
THEN SET custom_data.monitoring.thresholds.cpu_warning TO 70

WHEN node.role == "switch" 
THEN SET custom_data.monitoring.thresholds.cpu_warning TO 80

# Configure custom monitoring checks
WHEN custom_data.bgp.enabled == true 
THEN SET custom_data.monitoring.custom_checks.bgp_peers TO true

WHEN custom_data.ospf.enabled == true 
THEN SET custom_data.monitoring.custom_checks.ospf_neighbors TO true
```

### Backup and Recovery

```
# Configure backup schedules
WHEN custom_data.criticality == "critical" 
THEN SET custom_data.backup.frequency TO "daily"

WHEN custom_data.criticality == "standard" 
THEN SET custom_data.backup.frequency TO "weekly"

# Set retention policies
WHEN custom_data.backup.frequency == "daily" 
THEN SET custom_data.backup.retention_count TO 30

WHEN custom_data.backup.frequency == "weekly" 
THEN SET custom_data.backup.retention_count TO 12

# Configure backup verification
WHEN custom_data.backup.enabled == true 
THEN SET custom_data.backup.verification_required TO true

# Disaster recovery priority
WHEN custom_data.criticality == "critical" 
THEN SET custom_data.dr.recovery_priority TO 1

WHEN custom_data.criticality == "standard" 
THEN SET custom_data.dr.recovery_priority TO 3
```

## Testing and Validation

### Policy Testing Patterns

```bash
# Test basic classification
unet policies evaluate examples/classification.policy --node cisco-switch-01 --dry-run

# Test compliance validation  
unet policies evaluate examples/compliance.policy --node core-router-01 --verbose

# Test template assignment
unet policies evaluate examples/templates.policy --node edge-router-01 --dry-run
```

### Validation Examples

```
# Validate that monitoring is properly configured
WHEN node.lifecycle == "live" 
THEN ASSERT custom_data.monitoring.enabled IS true

# Validate security configurations
WHEN custom_data.security_zone == "dmz" 
THEN ASSERT custom_data.security.ssh_version IS "2"

# Validate backup configurations
WHEN custom_data.criticality == "critical" 
THEN ASSERT custom_data.backup.frequency IS "daily"

# Validate template assignments
WHEN node.role == "router" 
THEN ASSERT custom_data.assigned_templates CONTAINS "router-base.j2"
```

## Performance Considerations

### Efficient Policy Design

```
# Good: Specific conditions first
WHEN node.vendor == "cisco" AND node.model == "2960X" AND custom_data.location == "building-a" 
THEN SET custom_data.specific_config TO "value"

# Good: Use hierarchical conditions
WHEN node.vendor == "cisco" 
THEN SET custom_data.vendor_class TO "ios"

WHEN custom_data.vendor_class == "ios" AND node.role == "switch" 
THEN SET custom_data.device_type TO "ios_switch"

# Avoid: Overly complex single conditions
# WHEN (node.vendor == "cisco" OR node.vendor == "arista") AND (node.role == "switch" OR node.role == "router") AND (custom_data.location CONTAINS "dc" OR custom_data.location CONTAINS "office") 
# THEN SET custom_data.complex_match TO true
```

### Batch Processing

```
# Process devices by groups for efficiency
WHEN custom_data.processing_batch == "group1" 
THEN SET custom_data.batch_processed TO true

WHEN custom_data.processing_batch == "group2" 
THEN SET custom_data.batch_processed TO true

# Use conditional processing
WHEN custom_data.needs_update == true 
THEN SET custom_data.update_applied TO true

WHEN custom_data.update_applied == true 
THEN SET custom_data.needs_update TO false
```

These examples demonstrate the flexibility and power of μNet's policy engine for various network automation scenarios. Use them as starting points for your own policy implementations.
