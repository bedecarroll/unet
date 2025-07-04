<!-- SPDX-License-Identifier: MIT -->

# Configuration Management Tutorial

> **Audience:** Network engineers ready to dive deeper into μNet's configuration management capabilities.  
> **Objective:** Master advanced configuration templating, validation, and deployment workflows.

---

## Table of Contents

1. [Advanced Templating](#advanced-templating)
2. [Configuration Validation](#configuration-validation)
3. [Change Management](#change-management)
4. [Multi-vendor Support](#multi-vendor-support)
5. [Deployment Workflows](#deployment-workflows)
6. [Troubleshooting](#troubleshooting)

---

## Advanced Templating

### Template Inheritance

Create base templates that can be extended:

**templates/base/router.j2:**

```jinja2
{# Base router configuration #}
hostname {{ node.name }}
!
{% block management %}
interface {{ node.mgmt_interface | default('GigabitEthernet0/0') }}
 description Management Interface
 ip address {{ node.management_ip }} {{ node.mgmt_netmask | default('255.255.255.0') }}
 no shutdown
{% endblock %}
!
{% block ntp %}
{% for server in ntp_servers %}
ntp server {{ server }}
{% endfor %}
{% endblock %}
!
{% block additional %}
{# Override in child templates #}
{% endblock %}
```

**templates/cisco/edge-router.j2:**

```jinja2
{% extends "base/router.j2" %}

{% block additional %}
{# Edge router specific configuration #}
router ospf 1
 router-id {{ node.router_id }}
 network {{ node.management_ip }} 0.0.0.0 area 0
!
{% for interface in node.wan_interfaces %}
interface {{ interface.name }}
 description {{ interface.description }}
 ip address {{ interface.ip }} {{ interface.netmask }}
 {% if interface.type == "wan" %}
 bandwidth {{ interface.bandwidth | default(100000) }}
 {% endif %}
 no shutdown
!
{% endfor %}
{% endblock %}
```

### Template Functions and Filters

Create custom filters for common operations:

**templates/functions.j2:**

```jinja2
{# Custom Jinja2 filters and functions #}

{# IP address calculations #}
{% macro subnet_id(ip, netmask) %}
{{ ip | ipaddr('network') }}
{% endmacro %}

{# VLAN configuration generator #}
{% macro vlan_config(vlans) %}
{% for vlan in vlans %}
vlan {{ vlan.id }}
 name {{ vlan.name }}
{% if vlan.description %}
 description {{ vlan.description }}
{% endif %}
!
{% endfor %}
{% endmacro %}

{# Interface range configuration #}
{% macro interface_range(start, end, config) %}
interface range GigabitEthernet{{ start }}-{{ end }}
{% for line in config %}
 {{ line }}
{% endfor %}
!
{% endmacro %}
```

### Conditional Configuration

Handle different scenarios with conditional blocks:

**templates/cisco/switch.j2:**

```jinja2
hostname {{ node.name }}
!
{# Different configurations based on role #}
{% if node.role == "access" %}
  {# Access switch configuration #}
  spanning-tree mode rapid-pvst
  spanning-tree extend system-id
  
  {% for port in node.access_ports %}
  interface {{ port.interface }}
   description {{ port.description }}
   switchport mode access
   switchport access vlan {{ port.vlan }}
   {% if port.voice_vlan %}
   switchport voice vlan {{ port.voice_vlan }}
   {% endif %}
   spanning-tree portfast
   spanning-tree bpduguard enable
  !
  {% endfor %}

{% elif node.role == "distribution" %}
  {# Distribution switch configuration #}
  spanning-tree mode rapid-pvst
  spanning-tree vlan 1-4094 priority 4096
  
  {% for trunk in node.trunk_ports %}
  interface {{ trunk.interface }}
   description {{ trunk.description }}
   switchport mode trunk
   switchport trunk allowed vlan {{ trunk.allowed_vlans | join(',') }}
  !
  {% endfor %}

{% elif node.role == "core" %}
  {# Core switch configuration #}
  spanning-tree mode rapid-pvst
  spanning-tree vlan 1-4094 priority 0
  
  {# Layer 3 interfaces for core #}
  {% for l3_int in node.l3_interfaces %}
  interface {{ l3_int.interface }}
   description {{ l3_int.description }}
   no switchport
   ip address {{ l3_int.ip }} {{ l3_int.netmask }}
  !
  {% endfor %}
{% endif %}

{# Common configuration for all switch types #}
{% include "common/management.j2" %}
{% include "common/logging.j2" %}
{% include "common/snmp.j2" %}
```

### Data-driven Configuration

Use structured data to drive complex configurations:

**Node data (in database or YAML):**

```yaml
name: "core-switch-01"
role: "core"
vlans:
  - id: 10
    name: "USERS"
    description: "User workstations"
    gateway: "10.10.10.1"
  - id: 20
    name: "SERVERS"
    description: "Server network"
    gateway: "10.20.20.1"
l3_interfaces:
  - interface: "Vlan10"
    ip: "10.10.10.1"
    netmask: "255.255.255.0"
    hsrp_group: 10
    hsrp_priority: 110
  - interface: "Vlan20"
    ip: "10.20.20.1"
    netmask: "255.255.255.0"
    hsrp_group: 20
    hsrp_priority: 110
```

**templates/cisco/l3-switch.j2:**

```jinja2
hostname {{ node.name }}
!
ip routing
!
{# Create VLANs #}
{% for vlan in node.vlans %}
vlan {{ vlan.id }}
 name {{ vlan.name }}
{% if vlan.description %}
 description {{ vlan.description }}
{% endif %}
!
{% endfor %}

{# Configure SVI interfaces with HSRP #}
{% for interface in node.l3_interfaces %}
interface {{ interface.interface }}
 description {{ interface.description | default('L3 interface') }}
 ip address {{ interface.ip }} {{ interface.netmask }}
 {% if interface.hsrp_group %}
 standby {{ interface.hsrp_group }} ip {{ interface.hsrp_vip | default(interface.ip | ipaddr('1') | ipaddr('address')) }}
 standby {{ interface.hsrp_group }} priority {{ interface.hsrp_priority | default(100) }}
 standby {{ interface.hsrp_group }} preempt
 {% endif %}
 no shutdown
!
{% endfor %}
```

---

## Configuration Validation

### Writing Comprehensive Policies

**policies/cisco-best-practices.rules:**

```
# Basic security requirements
assert config contains "service password-encryption"
assert config contains "no ip http server"
assert config contains "login block-for"

# Management interface requirements
assert node.type in ["router", "switch"] -> config contains "management-ip"
assert config contains "exec-timeout"

# SNMP security
assert config contains "snmp-server community" -> config contains "RO"
assert config not contains "snmp-server community public"
assert config not contains "snmp-server community private"

# Conditional requirements based on node role
assert node.role == "core" -> config contains "spanning-tree vlan.*priority 0"
assert node.role == "distribution" -> config contains "spanning-tree vlan.*priority 4096"
assert node.role == "access" -> config contains "spanning-tree portfast"

# Interface validation
assert node.mgmt_interface -> config contains node.mgmt_interface
assert node.type == "switch" -> config contains "spanning-tree mode"

# Logging requirements
assert config contains "logging buffered"
assert config contains "logging host"

# NTP validation
assert config contains "ntp server"
```

### Policy Organization

Structure your policies for maintainability:

```
policies/
├── common/
│   ├── security.rules      # Universal security requirements
│   ├── management.rules    # Management interface standards
│   └── logging.rules       # Logging requirements
├── cisco/
│   ├── ios-security.rules  # Cisco IOS specific security
│   ├── switching.rules     # Switch configuration standards
│   └── routing.rules       # Router configuration standards
├── juniper/
│   ├── junos-security.rules
│   └── interface.rules
└── compliance/
    ├── sox.rules           # SOX compliance requirements
    ├── pci.rules           # PCI DSS requirements
    └── iso27001.rules      # ISO 27001 requirements
```

### Custom Validation Logic

**policies/advanced-validation.rules:**

```
# Complex conditional validation
assert node.location == "datacenter" -> (
  config contains "ntp server 10.1.1.100" and
  config contains "logging host 10.1.1.200" and
  config contains "snmp-server host 10.1.1.300"
)

# Interface count validation
assert node.type == "access-switch" -> config.count("interface GigabitEthernet") >= 24
assert node.type == "core-switch" -> config.count("interface TenGigabitEthernet") >= 4

# VLAN consistency
assert config contains "vlan 10" -> config contains "interface Vlan10"

# Security compliance
assert node.compliance_level == "high" -> (
  config contains "aaa new-model" and
  config contains "aaa authentication login default group radius local" and
  config not contains "username.*privilege 15"
)

# Performance requirements
assert node.cpu_intensive == true -> config contains "process cpu threshold type total rising"
```

---

## Change Management

### Version Control Integration

Track all configuration changes:

```bash
# Initialize git integration
unet git init --remote git@github.com:company/network-configs.git

# View change history
unet git log --node border-router-01

# Create a change proposal
unet changes create \
  --title "Update NTP servers for datacenter migration" \
  --description "Changing primary NTP from 10.1.1.100 to 10.2.1.100" \
  --nodes "border-router-01,core-switch-01" \
  --template "datacenter-migration"

# Review pending changes
unet changes list --status pending

# Approve changes
unet changes approve --id 12345 --reviewer "john.doe"

# Deploy approved changes
unet changes deploy --id 12345
```

### Change Validation Workflow

**1. Pre-deployment Validation:**

```bash
# Validate proposed changes
unet config validate --staged

# Generate diff to review changes
unet config diff --staged --output changes-review.txt

# Run policy checks on new configuration
unet policies check --all --staged
```

**2. Rollback Procedures:**

```bash
# View deployment history
unet deployments list --node border-router-01

# Rollback to previous configuration
unet deployments rollback --deployment-id 67890

# Emergency rollback (last known good)
unet config rollback --node border-router-01 --emergency
```

---

## Multi-vendor Support

### Vendor-specific Templates

**Cisco IOS:**

```jinja2
{# templates/cisco/ios-base.j2 #}
hostname {{ node.name }}
!
enable secret {{ secrets.enable_password }}
!
{% for user in node.users %}
username {{ user.name }} privilege {{ user.privilege }} secret {{ user.password }}
{% endfor %}
!
interface {{ node.mgmt_interface }}
 description Management Interface
 ip address {{ node.management_ip }} {{ node.mgmt_netmask }}
 no shutdown
!
line vty 0 4
 transport input ssh
 exec-timeout 10 0
```

**Juniper JunOS:**

```jinja2
{# templates/juniper/junos-base.j2 #}
system {
    host-name {{ node.name }};
    root-authentication {
        encrypted-password "{{ secrets.root_password_hash }}";
    }
    {% for user in node.users %}
    login {
        user {{ user.name }} {
            class {{ user.class }};
            authentication {
                encrypted-password "{{ user.password_hash }}";
            }
        }
    }
    {% endfor %}
}

interfaces {
    {{ node.mgmt_interface }} {
        description "Management Interface";
        unit 0 {
            family inet {
                address {{ node.management_ip }}/{{ node.mgmt_prefix }};
            }
        }
    }
}
```

**Arista EOS:**

```jinja2
{# templates/arista/eos-base.j2 #}
hostname {{ node.name }}
!
{% for user in node.users %}
username {{ user.name }} privilege {{ user.privilege }} secret sha512 {{ user.password_hash }}
{% endfor %}
!
interface Management1
   description Management Interface
   ip address {{ node.management_ip }}/{{ node.mgmt_prefix }}
   no shutdown
!
management api http-commands
   protocol https
   no shutdown
```

### Vendor-agnostic Abstractions

Create vendor-neutral data structures:

```yaml
# Node definition (vendor-agnostic)
name: "edge-router-01"
vendor: "cisco"
platform: "ISR4000"
interfaces:
  - name: "mgmt"
    type: "management"
    ip: "192.168.1.10"
    netmask: "255.255.255.0"
  - name: "wan1"
    type: "wan"
    ip: "203.0.113.10"
    netmask: "255.255.255.252"
    description: "ISP-A Primary Link"
  - name: "wan2"
    type: "wan"
    ip: "203.0.113.14"
    netmask: "255.255.255.252"
    description: "ISP-B Backup Link"
    
routing:
  ospf:
    process_id: 1
    router_id: "192.168.1.10"
    areas:
      - area: 0
        networks:
          - "192.168.1.0/24"
          - "10.0.0.0/8"
```

Then use vendor-specific templates to render the same data:

```bash
# Generate Cisco IOS configuration
unet config generate edge-router-01 --template cisco/router --vendor cisco

# Generate Juniper JunOS configuration  
unet config generate edge-router-01 --template juniper/router --vendor juniper
```

---

## Deployment Workflows

### Automated Deployment Pipeline

**1. Configuration Generation:**

```bash
#!/bin/bash
# scripts/generate-configs.sh

echo "Generating configurations for all devices..."
unet config generate --all --output-dir deploy/staging/

echo "Validating generated configurations..."
unet config validate --all --config-dir deploy/staging/

echo "Running policy checks..."
unet policies check --all --config-dir deploy/staging/

if [ $? -eq 0 ]; then
    echo "All validations passed. Ready for deployment."
    cp -r deploy/staging/* deploy/production/
else
    echo "Validation failed. Check errors above."
    exit 1
fi
```

**2. Staged Deployment:**

```bash
#!/bin/bash
# scripts/staged-deploy.sh

STAGE=${1:-dev}
NODES_FILE=${2:-nodes-${STAGE}.txt}

echo "Deploying to $STAGE environment..."

while read -r node; do
    echo "Deploying configuration to $node..."
    
    # Backup current configuration
    unet config backup --node "$node"
    
    # Deploy new configuration
    unet config deploy --node "$node" --config "deploy/production/${node}.cfg"
    
    # Verify deployment
    if unet config verify --node "$node"; then
        echo "✓ $node deployment successful"
        unet changes mark-deployed --node "$node"
    else
        echo "✗ $node deployment failed, rolling back..."
        unet config rollback --node "$node"
        exit 1
    fi
    
    # Wait between deployments to avoid overloading network
    sleep 30
done < "$NODES_FILE"

echo "Staged deployment to $STAGE completed successfully."
```

**3. Zero-downtime Deployment:**

```bash
# For critical infrastructure
unet config deploy \
  --node core-switch-01 \
  --strategy gradual \
  --verify-interval 30 \
  --rollback-on-failure \
  --notification-webhook https://alerts.company.com/webhook
```

### Integration with External Systems

**Network Monitoring Integration:**

```bash
# Pre-deployment checks
unet integrations monitoring pre-check --nodes "router-01,router-02"

# Post-deployment verification
unet integrations monitoring verify --nodes "router-01,router-02" --timeout 300
```

**ITSM Integration:**

```bash
# Create change request in ServiceNow
unet integrations servicenow create-cr \
  --summary "Update router configurations for security patch" \
  --nodes "router-01,router-02" \
  --change-window "2024-01-15T02:00:00Z/2024-01-15T04:00:00Z"

# Update change request with deployment results
unet integrations servicenow update-cr --cr-number CHG123456 --status completed
```

---

## Troubleshooting

### Common Issues and Solutions

**Template Rendering Errors:**

```bash
# Debug template rendering
unet config generate border-router-01 --template cisco-router --debug

# Common issues:
# 1. Missing variables - check node attributes
unet nodes show border-router-01 --format yaml

# 2. Template syntax errors - validate template
unet templates validate cisco-router

# 3. Filter errors - test filters separately
unet templates test-filter "192.168.1.1 | ipaddr('network')"
```

**Policy Validation Failures:**

```bash
# Debug policy evaluation
unet policies check border-router-01 --debug

# Show which policies failed
unet policies check border-router-01 --verbose

# Test individual policy rules
unet policies test "assert config contains 'ntp server'" --config border-router-01.cfg
```

**Deployment Issues:**

```bash
# Check connectivity to device
unet nodes ping border-router-01

# Verify credentials
unet nodes test-auth border-router-01

# Check deployment status
unet deployments status --node border-router-01

# View deployment logs
unet deployments logs --deployment-id 12345
```

### Performance Optimization

**Large-scale Operations:**

```bash
# Parallel processing for multiple devices
unet config generate --all --parallel 10

# Batch operations
unet config validate --batch-size 50 --nodes-file large-deployment.txt

# Memory optimization for large templates
unet config generate --streaming --template large-template --nodes-file nodes.txt
```

**Template Optimization:**

```bash
# Profile template rendering performance
unet templates profile cisco-router-complex --iterations 100

# Cache compiled templates
unet templates compile --all --cache-dir /tmp/template-cache
```

### Debugging Tips

1. **Use verbose output:** Add `--verbose` or `--debug` to most commands
2. **Check logs:** `unet logs tail` shows recent activity
3. **Validate incrementally:** Test templates and policies separately before combining
4. **Use dry-run mode:** Most deployment commands support `--dry-run`
5. **Monitor resource usage:** `unet system status` shows memory and CPU usage

---

## Next Steps

You now have a solid foundation in μNet's configuration management capabilities. Continue learning with:

- **[Policy Creation Guide](user_policy_guide.md)** - Master the policy language
- **[Template Usage Tutorial](user_template_tutorial.md)** - Advanced templating techniques
- **[Example Configurations](user_examples.md)** - Real-world examples and patterns

For complex scenarios, refer to the **[API Reference](api_reference.md)** and **[Troubleshooting Guide](troubleshooting_guide.md)**.
