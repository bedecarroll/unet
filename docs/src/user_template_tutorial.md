<!-- SPDX-License-Identifier: MIT -->

# Template Usage Tutorial

> **Audience:** Network engineers who want to master μNet's template system for generating network configurations.  
> **Objective:** Learn advanced templating techniques to create flexible, maintainable, and reusable configuration templates.

---

## Table of Contents

1. [Template Fundamentals](#template-fundamentals)
2. [Jinja2 Syntax for Networks](#jinja2-syntax-for-networks)
3. [Building Your First Template](#building-your-first-template)
4. [Advanced Template Patterns](#advanced-template-patterns)
5. [Template Organization](#template-organization)
6. [Data-Driven Templates](#data-driven-templates)
7. [Testing and Debugging](#testing-and-debugging)
8. [Best Practices](#best-practices)

---

## Template Fundamentals

### What are Templates?

Templates in μNet are Jinja2-based configuration generators that:

- **Generate configurations** from structured data
- **Reduce repetition** through reusable patterns
- **Ensure consistency** across similar devices
- **Enable mass changes** through centralized updates
- **Support multiple vendors** with shared logic

### Template Components

Every template can use:

1. **Variables** - Node attributes and custom data: `{{ node.name }}`
2. **Filters** - Transform data: `{{ ip_address | ipaddr('network') }}`
3. **Control structures** - Loops and conditions: `{% if %}`, `{% for %}`
4. **Macros** - Reusable functions: `{% macro interface_config() %}`
5. **Includes** - External template files: `{% include "common/snmp.j2" %}`
6. **Inheritance** - Template extension: `{% extends "base.j2" %}`

### Template Context

Templates have access to:

```python
{
    "node": {
        "name": "router-01",
        "type": "router", 
        "vendor": "cisco",
        "management_ip": "192.168.1.1",
        "custom_data": {...}
    },
    "config": {
        "global": {...},
        "environment": "production"
    },
    "secrets": {
        # Securely managed credentials
    }
}
```

---

## Jinja2 Syntax for Networks

### Variables and Expressions

**Basic variable substitution:**

```jinja2
hostname {{ node.name }}
!
interface {{ node.mgmt_interface | default('GigabitEthernet0/0') }}
 description Management Interface
 ip address {{ node.management_ip }} {{ node.mgmt_netmask | default('255.255.255.0') }}
 no shutdown
```

**Attribute access:**

```jinja2
{# Direct attribute access #}
{{ node.vendor }}

{# Dictionary-style access #}
{{ node['management_ip'] }}

{# Nested attributes #}
{{ node.location.datacenter }}
{{ node.interfaces.mgmt.ip }}

{# Safe attribute access (won't error if missing) #}
{{ node.optional_field | default('N/A') }}
```

### Control Structures

**Conditional blocks:**

```jinja2
hostname {{ node.name }}
!
{% if node.vendor == "cisco" %}
enable secret {{ secrets.enable_password }}
!
service password-encryption
service timestamps debug datetime msec
service timestamps log datetime msec
{% elif node.vendor == "juniper" %}
system {
    host-name {{ node.name }};
    root-authentication {
        encrypted-password "{{ secrets.root_password_hash }}";
    }
}
{% endif %}
```

**Loops:**

```jinja2
{# Loop through interfaces #}
{% for interface in node.interfaces %}
interface {{ interface.name }}
 description {{ interface.description }}
 {% if interface.ip %}
 ip address {{ interface.ip }} {{ interface.netmask }}
 {% endif %}
 {% if interface.vlan %}
 switchport access vlan {{ interface.vlan }}
 {% endif %}
 no shutdown
!
{% endfor %}

{# Loop with conditions #}
{% for vlan in node.vlans if vlan.enabled %}
vlan {{ vlan.id }}
 name {{ vlan.name }}
{% if vlan.description %}
 description {{ vlan.description }}
{% endif %}
!
{% endfor %}
```

### Filters

**Common network filters:**

```jinja2
{# IP address manipulation #}
{{ "192.168.1.1/24" | ipaddr('network') }}        {# 192.168.1.0/24 #}
{{ "192.168.1.1/24" | ipaddr('netmask') }}        {# 255.255.255.0 #}
{{ "192.168.1.1/24" | ipaddr('broadcast') }}      {# 192.168.1.255 #}
{{ "192.168.1.1/24" | ipaddr('size') }}           {# 256 #}

{# String manipulation #}
{{ node.name | upper }}                            {# ROUTER-01 #}
{{ interface.description | truncate(32) }}        {# Truncate long descriptions #}
{{ node.location | replace('-', '_') }}           {# Replace characters #}

{# List operations #}
{{ node.vlans | length }}                          {# Count items #}
{{ node.interfaces | selectattr('type', 'equalto', 'ethernet') | list }}
{{ node.vlans | map(attribute='id') | join(',') }} {# 10,20,30,40 #}
```

**Custom filters for networking:**

```jinja2
{# Convert CIDR to wildcard mask #}
{{ "192.168.1.0/24" | cidr_to_wildcard }}         {# 0.0.0.255 #}

{# Generate interface ranges #}
{{ interfaces | interface_range }}                 {# Gi1/0-48 #}

{# VLAN list compression #}
{{ [10, 11, 12, 20, 21, 30] | compress_vlans }}   {# 10-12,20-21,30 #}
```

### Macros

**Reusable configuration blocks:**

```jinja2
{# Define a macro for interface configuration #}
{% macro interface_config(name, ip=none, description=none, vlan=none) %}
interface {{ name }}
{% if description %}
 description {{ description }}
{% endif %}
{% if ip %}
 ip address {{ ip.address }} {{ ip.netmask }}
{% endif %}
{% if vlan %}
 switchport access vlan {{ vlan }}
{% endif %}
 no shutdown
!
{% endmacro %}

{# Use the macro #}
{{ interface_config('GigabitEthernet0/1', 
                   ip={'address': '192.168.1.1', 'netmask': '255.255.255.0'}, 
                   description='Management Interface') }}
```

**More complex macros:**

```jinja2
{# OSPF configuration macro #}
{% macro ospf_config(process_id, router_id, areas) %}
router ospf {{ process_id }}
 router-id {{ router_id }}
{% for area in areas %}
{% for network in area.networks %}
 network {{ network.address }} {{ network.wildcard }} area {{ area.id }}
{% endfor %}
{% endfor %}
!
{% endmacro %}

{# Usage #}
{{ ospf_config(
    process_id=1,
    router_id=node.router_id,
    areas=[
        {
            'id': 0,
            'networks': [
                {'address': '192.168.1.0', 'wildcard': '0.0.0.255'},
                {'address': '10.0.0.0', 'wildcard': '0.255.255.255'}
            ]
        }
    ]
) }}
```

---

## Building Your First Template

### Step 1: Analyze Configuration Patterns

Before creating templates, analyze your existing configurations:

```bash
# Look at similar device configurations
unet config show router-01 > /tmp/r1.cfg
unet config show router-02 > /tmp/r2.cfg
diff /tmp/r1.cfg /tmp/r2.cfg

# Identify common patterns and differences
# Common: hostname, interface types, protocols
# Different: IP addresses, interface counts, VLAN assignments
```

### Step 2: Extract Variables

Identify what should be templated:

**Original configuration:**

```
hostname border-router-01
!
interface GigabitEthernet0/0
 description Management Interface
 ip address 192.168.1.1 255.255.255.0
 no shutdown
!
interface GigabitEthernet0/1
 description WAN Interface to ISP-A
 ip address 203.0.113.1 255.255.255.252
 no shutdown
!
router ospf 1
 router-id 192.168.1.1
 network 192.168.1.0 0.0.0.255 area 0
 network 203.0.113.0 0.0.0.3 area 0
```

**Identified variables:**

- Hostname: `border-router-01`
- Management IP: `192.168.1.1`
- WAN IP: `203.0.113.1`
- Router ID: `192.168.1.1`
- Interface descriptions: `Management Interface`, `WAN Interface to ISP-A`

### Step 3: Create Basic Template

**templates/cisco/basic-router.j2:**

```jinja2
hostname {{ node.name }}
!
interface {{ node.mgmt_interface | default('GigabitEthernet0/0') }}
 description {{ node.mgmt_description | default('Management Interface') }}
 ip address {{ node.management_ip }} {{ node.mgmt_netmask | default('255.255.255.0') }}
 no shutdown
!
{% if node.wan_interface %}
interface {{ node.wan_interface }}
 description {{ node.wan_description | default('WAN Interface') }}
 ip address {{ node.wan_ip }} {{ node.wan_netmask | default('255.255.255.252') }}
 no shutdown
!
{% endif %}
{% if node.router_id %}
router ospf {{ node.ospf_process | default(1) }}
 router-id {{ node.router_id }}
{% for network in node.ospf_networks %}
 network {{ network.address }} {{ network.wildcard }} area {{ network.area | default(0) }}
{% endfor %}
!
{% endif %}
```

### Step 4: Test Your Template

```bash
# Create test node data
unet nodes add \
  --name "test-router" \
  --management-ip "192.168.1.100" \
  --custom-data '{
    "mgmt_interface": "GigabitEthernet0/0",
    "wan_interface": "GigabitEthernet0/1", 
    "wan_ip": "203.0.113.5",
    "router_id": "192.168.1.100",
    "ospf_networks": [
      {"address": "192.168.1.0", "wildcard": "0.0.0.255"},
      {"address": "203.0.113.4", "wildcard": "0.0.0.3"}
    ]
  }'

# Generate configuration
unet config generate test-router --template cisco/basic-router

# Review output and refine template
```

---

## Advanced Template Patterns

### Template Inheritance

Create a base template that others can extend:

**templates/base/device.j2:**

```jinja2
{# Base device template with common elements #}
{% block hostname %}
hostname {{ node.name }}
{% endblock %}
!
{% block global_settings %}
no ip domain-lookup
ip domain-name {{ config.domain_name | default('example.com') }}
service password-encryption
service timestamps debug datetime msec
service timestamps log datetime msec
{% endblock %}
!
{% block management %}
{# Override in child templates #}
{% endblock %}
!
{% block ntp %}
{% for server in config.ntp_servers | default(['10.1.1.100', '10.1.1.101']) %}
ntp server {{ server }}
{% endfor %}
{% endblock %}
!
{% block logging %}
{% for server in config.syslog_servers | default([]) %}
logging host {{ server }}
{% endfor %}
logging buffered 32768 warnings
{% endblock %}
!
{% block additional %}
{# Device-specific configuration #}
{% endblock %}
```

**templates/cisco/router.j2:**

```jinja2
{% extends "base/device.j2" %}

{% block management %}
interface {{ node.mgmt_interface | default('GigabitEthernet0/0') }}
 description Management Interface  
 ip address {{ node.management_ip }} {{ node.mgmt_netmask | default('255.255.255.0') }}
 no shutdown
{% endblock %}

{% block additional %}
{# Router-specific configuration #}
ip routing
ip cef

{% if node.ospf_enabled %}
router ospf {{ node.ospf_process | default(1) }}
 router-id {{ node.router_id | default(node.management_ip) }}
{% for area in node.ospf_areas | default([]) %}
{% for network in area.networks %}
 network {{ network.address }} {{ network.wildcard }} area {{ area.id }}
{% endfor %}
{% endfor %}
!
{% endif %}

{% if node.bgp_enabled %}
router bgp {{ node.bgp_asn }}
 bgp router-id {{ node.router_id | default(node.management_ip) }}
{% for neighbor in node.bgp_neighbors | default([]) %}
 neighbor {{ neighbor.ip }} remote-as {{ neighbor.asn }}
 neighbor {{ neighbor.ip }} description {{ neighbor.description }}
{% endfor %}
!
{% endif %}
{% endblock %}
```

### Conditional Includes

Include different templates based on conditions:

**templates/cisco/switch.j2:**

```jinja2
{% extends "base/device.j2" %}

{% block additional %}
{# Include role-specific configuration #}
{% if node.role == "access" %}
{% include "cisco/switch-access.j2" %}
{% elif node.role == "distribution" %}
{% include "cisco/switch-distribution.j2" %}
{% elif node.role == "core" %}
{% include "cisco/switch-core.j2" %}
{% endif %}

{# Include vendor-specific features #}
{% if node.features %}
{% for feature in node.features %}
{% include "cisco/features/" + feature + ".j2" ignore missing %}
{% endfor %}
{% endif %}
{% endblock %}
```

### Dynamic Template Selection

**templates/auto-select.j2:**

```jinja2
{# Automatically select appropriate template based on device attributes #}
{% set template_map = {
    'cisco': {
        'router': 'cisco/router.j2',
        'switch': 'cisco/switch.j2',
        'firewall': 'cisco/asa.j2'
    },
    'juniper': {
        'router': 'juniper/router.j2',
        'switch': 'juniper/switch.j2', 
        'firewall': 'juniper/srx.j2'
    }
} %}

{% set template_path = template_map[node.vendor][node.type] %}
{% include template_path %}
```

### Environment-Specific Templates

**templates/environments/production.j2:**

```jinja2
{# Production-specific overrides #}
{% extends node.vendor + "/" + node.type + ".j2" %}

{% block logging %}
{# Enhanced logging for production #}
{{ super() }}
logging trap warnings
logging facility local0
{% for server in config.production.syslog_servers %}
logging host {{ server }} transport tcp port 6514
{% endfor %}
{% endblock %}

{% block snmp %}
{# Production SNMP configuration #}
snmp-server community {{ secrets.snmp_ro_community }} RO 99
snmp-server community {{ secrets.snmp_rw_community }} RW 98
{% for server in config.production.snmp_servers %}
snmp-server host {{ server }} version 2c {{ secrets.snmp_ro_community }}
{% endfor %}
snmp-server enable traps
{% endblock %}

{% block security %}
{# Enhanced security for production #}
{{ super() }}
banner motd ^
UNAUTHORIZED ACCESS PROHIBITED
This system is for authorized users only.
All activities are logged and monitored.
^
{% endblock %}
```

---

## Template Organization

### Directory Structure

Organize templates logically:

```
templates/
├── base/                   # Base templates for inheritance
│   ├── device.j2          # Common device configuration
│   ├── router.j2          # Base router template
│   └── switch.j2          # Base switch template
├── cisco/                 # Cisco-specific templates
│   ├── ios/
│   │   ├── router.j2
│   │   ├── switch.j2
│   │   └── features/
│   │       ├── ospf.j2
│   │       ├── bgp.j2
│   │       └── hsrp.j2
│   ├── asa/               # ASA firewall templates
│   └── nxos/              # Nexus switch templates
├── juniper/               # Juniper-specific templates
│   ├── junos/
│   └── srx/
├── arista/                # Arista-specific templates
│   └── eos/
├── common/                # Shared template components
│   ├── management.j2      # Management interface config
│   ├── ntp.j2            # NTP configuration
│   ├── logging.j2        # Logging configuration
│   ├── snmp.j2           # SNMP configuration
│   └── security.j2       # Basic security settings
├── environments/          # Environment-specific overrides
│   ├── development.j2
│   ├── staging.j2
│   └── production.j2
└── macros/               # Reusable macro libraries
    ├── interfaces.j2
    ├── routing.j2
    └── vlans.j2
```

### Template Naming Conventions

Use consistent naming:

```
# Format: vendor/platform/device-type-role.j2
cisco/ios/router-edge.j2
cisco/ios/switch-access.j2
cisco/nxos/switch-core.j2
juniper/junos/router-pe.j2

# For features/modules
cisco/features/ospf.j2
cisco/features/bgp.j2
common/management.j2

# For environments
environments/prod-security.j2
environments/dev-logging.j2
```

### Template Metadata

Add metadata to templates:

```jinja2
{#
Template: Cisco IOS Router Base Configuration
Author: Network Team
Version: 2.1
Last Updated: 2024-01-15
Description: Base configuration for Cisco IOS routers with OSPF and BGP support

Required node attributes:
  - name: Device hostname
  - management_ip: Management interface IP
  - router_id: OSPF/BGP router ID
  
Optional node attributes:
  - mgmt_interface: Management interface name (default: Gi0/0)
  - ospf_process: OSPF process ID (default: 1)
  - bgp_asn: BGP AS number (enables BGP if present)

Example usage:
  unet config generate router-01 --template cisco/ios/router
#}

hostname {{ node.name }}
{# ... rest of template ... #}
```

---

## Data-Driven Templates

### Using External Data Sources

**Integration with IPAM systems:**

```jinja2
{# Fetch IP allocation from IPAM #}
{% set ipam_data = ipam.get_allocation(node.name) %}

interface {{ node.mgmt_interface }}
 description {{ ipam_data.description }}
 ip address {{ ipam_data.ip }} {{ ipam_data.netmask }}
 no shutdown

{# Loop through all allocated interfaces #}
{% for allocation in ipam_data.interfaces %}
interface {{ allocation.interface }}
 description {{ allocation.description }}
 ip address {{ allocation.ip }} {{ allocation.netmask }}
 no shutdown
{% endfor %}
```

**Database-driven VLANs:**

```jinja2
{# Query VLAN database #}
{% set site_vlans = db.query('SELECT id, name, description FROM vlans WHERE site_id = ?', node.site_id) %}

{# Generate VLAN configuration #}
{% for vlan in site_vlans %}
vlan {{ vlan.id }}
 name {{ vlan.name }}
{% if vlan.description %}
 description {{ vlan.description }}
{% endif %}
!
{% endfor %}
```

### Complex Data Structures

**Hierarchical interface configuration:**

```yaml
# Node data structure
interfaces:
  management:
    name: "GigabitEthernet0/0"
    ip: "192.168.1.1"
    netmask: "255.255.255.0"
    description: "Management Interface"
  wan:
    - name: "GigabitEthernet0/1"
      ip: "203.0.113.1"
      netmask: "255.255.255.252"
      description: "WAN to ISP-A"
      bandwidth: 1000000
    - name: "GigabitEthernet0/2"
      ip: "203.0.113.5"
      netmask: "255.255.255.252"
      description: "WAN to ISP-B"
      bandwidth: 500000
  lan:
    - name: "GigabitEthernet0/3"
      description: "LAN Interface"
      vlans: [10, 20, 30]
```

**Template to process hierarchical data:**

```jinja2
{# Management interface #}
{% if node.interfaces.management %}
{% set mgmt = node.interfaces.management %}
interface {{ mgmt.name }}
 description {{ mgmt.description }}
 ip address {{ mgmt.ip }} {{ mgmt.netmask }}
 no shutdown
!
{% endif %}

{# WAN interfaces #}
{% for wan in node.interfaces.wan | default([]) %}
interface {{ wan.name }}
 description {{ wan.description }}
 ip address {{ wan.ip }} {{ wan.netmask }}
{% if wan.bandwidth %}
 bandwidth {{ wan.bandwidth }}
{% endif %}
 no shutdown
!
{% endfor %}

{# LAN interfaces #}
{% for lan in node.interfaces.lan | default([]) %}
interface {{ lan.name }}
 description {{ lan.description }}
 switchport mode trunk
 switchport trunk allowed vlan {{ lan.vlans | join(',') }}
 no shutdown
!
{% endfor %}
```

### Configuration Generation Pipelines

**Multi-stage template processing:**

```jinja2
{# Stage 1: Base configuration #}
{% set base_config %}
{% include "base/device.j2" %}
{% endset %}

{# Stage 2: Apply security hardening #}
{% set hardened_config %}
{{ base_config }}
{% include "security/hardening.j2" %}
{% endset %}

{# Stage 3: Apply environment-specific settings #}
{% set final_config %}
{{ hardened_config }}
{% include "environments/" + node.environment + ".j2" %}
{% endset %}

{{ final_config }}
```

---

## Testing and Debugging

### Template Development Workflow

**1. Start with static examples:**

```jinja2
{# Test with known values first #}
hostname test-router
!
interface GigabitEthernet0/0
 description Management Interface
 ip address 192.168.1.1 255.255.255.0
 no shutdown
```

**2. Add simple variables:**

```jinja2
hostname {{ node.name | default('test-router') }}
!
interface GigabitEthernet0/0
 description Management Interface
 ip address {{ node.management_ip | default('192.168.1.1') }} 255.255.255.0
 no shutdown
```

**3. Add conditional logic:**

```jinja2
hostname {{ node.name | default('test-router') }}
!
{% if node.management_ip %}
interface GigabitEthernet0/0
 description Management Interface
 ip address {{ node.management_ip }} {{ node.mgmt_netmask | default('255.255.255.0') }}
 no shutdown
!
{% endif %}
```

### Debugging Techniques

**Add debug output:**

```jinja2
{# Debug: Show available variables #}
{% if config.debug %}
{# DEBUG: Node attributes:
{%   for key, value in node.items() %}
{#   {{ key }}: {{ value }}
{%   endfor %}
#}
{% endif %}

hostname {{ node.name }}
```

**Test filter operations:**

```bash
# Test filters in isolation
unet templates test-filter "192.168.1.1/24 | ipaddr('network')"
unet templates test-filter "['10', '20', '30'] | join(',')"
```

**Validate template syntax:**

```bash
# Check template syntax without generating config
unet templates validate cisco/router.j2

# Test with minimal data
unet templates test cisco/router.j2 --data '{"node": {"name": "test"}}'
```

### Common Template Issues

**Issue: Undefined variables**

```jinja2
{# Problem: Variable doesn't exist #}
hostname {{ node.hostname }}  # Error if hostname not defined

{# Solution: Use defaults #}
hostname {{ node.hostname | default(node.name) }}
```

**Issue: Empty loops**

```jinja2
{# Problem: Loop generates nothing when list is empty #}
{% for vlan in node.vlans %}
vlan {{ vlan.id }}
{% endfor %}

{# Solution: Check if list exists and has items #}
{% if node.vlans and node.vlans | length > 0 %}
{% for vlan in node.vlans %}
vlan {{ vlan.id }}
{% endfor %}
{% endif %}
```

**Issue: Whitespace control**

```jinja2
{# Problem: Extra blank lines #}
{% for interface in node.interfaces %}
interface {{ interface.name }}
 description {{ interface.description }}
!
{% endfor %}

{# Solution: Use whitespace control #}
{% for interface in node.interfaces -%}
interface {{ interface.name }}
 description {{ interface.description }}
!
{% endfor %}
```

### Template Testing Framework

**Create template tests:**

```yaml
# tests/templates/cisco-router.yaml
template: "cisco/router.j2"
tests:
  - name: "Basic router configuration"
    data:
      node:
        name: "test-router"
        management_ip: "192.168.1.1"
        vendor: "cisco"
    expected_contains:
      - "hostname test-router"
      - "ip address 192.168.1.1"
    expected_not_contains:
      - "hostname undefined"
      
  - name: "Router with OSPF"
    data:
      node:
        name: "ospf-router"
        management_ip: "192.168.1.1"
        router_id: "1.1.1.1"
        ospf_process: 1
    expected_contains:
      - "router ospf 1"
      - "router-id 1.1.1.1"
```

**Run template tests:**

```bash
# Run all template tests
unet templates test-suite tests/templates/

# Run specific template test
unet templates test-suite tests/templates/cisco-router.yaml
```

---

## Best Practices

### Template Design Principles

1. **Keep templates focused** - One template per device type/role
2. **Use inheritance** - Share common patterns through base templates
3. **Provide defaults** - Make templates work with minimal data
4. **Be defensive** - Check for required data before using it
5. **Document expectations** - Clearly specify required node attributes

### Data Organization

1. **Separate data from templates** - Don't hardcode values in templates
2. **Use structured data** - Organize complex configurations hierarchically
3. **Normalize naming** - Use consistent attribute names across templates
4. **Validate data** - Ensure required fields are present and valid

### Performance Optimization

1. **Minimize database queries** - Fetch data once, use throughout template
2. **Avoid complex loops** - Keep iteration simple and efficient
3. **Use appropriate filters** - Don't over-process data
4. **Cache compiled templates** - Reuse compiled templates when possible

### Security Considerations

1. **Never hardcode secrets** - Use the secrets system
2. **Validate inputs** - Sanitize data before using in templates
3. **Limit template access** - Don't expose sensitive functions
4. **Review generated configs** - Ensure templates don't leak sensitive data

### Maintenance

1. **Version control templates** - Track changes like code
2. **Test before deploying** - Validate templates with real data
3. **Monitor template performance** - Profile slow templates
4. **Regular reviews** - Keep templates current with infrastructure changes

---

## Quick Reference

### Essential Template Patterns

```jinja2
{# Variable with default #}
{{ node.attribute | default('default_value') }}

{# Conditional block #}
{% if condition %}
configuration here
{% endif %}

{# Loop with items #}
{% for item in list %}
{{ item.property }}
{% endfor %}

{# Include other template #}
{% include "common/template.j2" %}

{# Extend base template #}
{% extends "base/template.j2" %}
```

### Common Filters

```jinja2
{# String operations #}
{{ string | upper | lower | title }}
{{ string | replace('old', 'new') }}
{{ string | truncate(50) }}

{# List operations #}
{{ list | length }}
{{ list | join(',') }}
{{ list | first | last }}

{# IP operations #}
{{ ip | ipaddr('network') }}
{{ ip | ipaddr('netmask') }}
{{ ip | ipaddr('broadcast') }}
```

---

## Next Steps

You now have comprehensive knowledge of μNet's template system. Continue your learning with:

- **[Example Configurations](user_examples.md)** - Real-world template examples
- **[Policy Creation Guide](user_policy_guide.md)** - Validate your generated configurations
- **[Configuration Management Tutorial](user_config_tutorial.md)** - Integrate templates into workflows

For advanced template features, refer to the **[Template Engine](04_template_engine.md)** documentation and **[API Reference](api_reference.md)**.
