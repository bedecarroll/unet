<!-- SPDX-License-Identifier: MIT -->

# Example Configurations

> **Audience:** Network engineers looking for real-world μNet configuration examples.  
> **Objective:** Provide practical templates, policies, and workflows that can be adapted for your environment.

---

## Table of Contents

1. [Basic Examples](#basic-examples)
2. [Multi-vendor Templates](#multi-vendor-templates)
3. [Advanced Configuration Patterns](#advanced-configuration-patterns)
4. [Policy Examples](#policy-examples)
5. [Real-world Scenarios](#real-world-scenarios)
6. [Complete Workflows](#complete-workflows)

---

## Basic Examples

### Simple Router Configuration

**Node Data:**

```yaml
name: "branch-router-01"
type: "router"
vendor: "cisco"
management_ip: "192.168.1.1"
location: "branch-office-seattle"
wan_interfaces:
  - name: "GigabitEthernet0/1"
    ip: "203.0.113.1"
    netmask: "255.255.255.252"
    description: "WAN to Headquarters"
lan_interface:
  name: "GigabitEthernet0/0"
  ip: "10.10.1.1"
  netmask: "255.255.255.0"
  description: "LAN Interface"
```

**Template (templates/cisco/branch-router.j2):**

```jinja2
hostname {{ node.name }}
!
enable secret {{ secrets.enable_password }}
service password-encryption
!
interface {{ node.lan_interface.name }}
 description {{ node.lan_interface.description }}
 ip address {{ node.lan_interface.ip }} {{ node.lan_interface.netmask }}
 no shutdown
!
{% for wan in node.wan_interfaces %}
interface {{ wan.name }}
 description {{ wan.description }}
 ip address {{ wan.ip }} {{ wan.netmask }}
 no shutdown
!
{% endfor %}
!
router ospf 1
 router-id {{ node.management_ip }}
 network {{ node.lan_interface.ip | ipaddr('network') }} {{ node.lan_interface.netmask | netmask_to_wildcard }} area 0
{% for wan in node.wan_interfaces %}
 network {{ wan.ip | ipaddr('network') }} {{ wan.netmask | netmask_to_wildcard }} area 0
{% endfor %}
!
ip route 0.0.0.0 0.0.0.0 {{ node.wan_interfaces[0].ip | ipaddr('next_hop') }}
!
line vty 0 4
 transport input ssh
 exec-timeout 10 0
!
ntp server 8.8.8.8
logging host {{ config.syslog_server }}
```

**Generated Configuration:**

```
hostname branch-router-01
!
enable secret $1$abcd$xyz123
service password-encryption
!
interface GigabitEthernet0/0
 description LAN Interface
 ip address 10.10.1.1 255.255.255.0
 no shutdown
!
interface GigabitEthernet0/1
 description WAN to Headquarters
 ip address 203.0.113.1 255.255.255.252
 no shutdown
!
!
router ospf 1
 router-id 192.168.1.1
 network 10.10.1.0 0.0.0.255 area 0
 network 203.0.113.0 0.0.0.3 area 0
!
ip route 0.0.0.0 0.0.0.0 203.0.113.2
!
line vty 0 4
 transport input ssh
 exec-timeout 10 0
!
ntp server 8.8.8.8
logging host 10.1.1.200
```

### Access Switch Configuration

**Node Data:**

```yaml
name: "access-sw-01"
type: "switch"
vendor: "cisco"
role: "access"
management_ip: "192.168.1.10"
mgmt_vlan: 99
uplinks:
  - interface: "GigabitEthernet0/47"
    description: "Uplink to Distribution Switch 1"
  - interface: "GigabitEthernet0/48" 
    description: "Uplink to Distribution Switch 2"
access_ports:
  - interface: "GigabitEthernet0/1-24"
    vlan: 10
    description: "User Workstations"
  - interface: "GigabitEthernet0/25-36"
    vlan: 20
    description: "Printers and Shared Devices"
vlans:
  - id: 10
    name: "USERS"
  - id: 20
    name: "PRINTERS"
  - id: 99
    name: "MANAGEMENT"
```

**Template (templates/cisco/access-switch.j2):**

```jinja2
hostname {{ node.name }}
!
{% for vlan in node.vlans %}
vlan {{ vlan.id }}
 name {{ vlan.name }}
!
{% endfor %}
!
interface Vlan{{ node.mgmt_vlan }}
 description Management Interface
 ip address {{ node.management_ip }} 255.255.255.0
 no shutdown
!
{% for port_group in node.access_ports %}
interface range {{ port_group.interface }}
 description {{ port_group.description }}
 switchport mode access
 switchport access vlan {{ port_group.vlan }}
 spanning-tree portfast
 spanning-tree bpduguard enable
!
{% endfor %}
!
{% for uplink in node.uplinks %}
interface {{ uplink.interface }}
 description {{ uplink.description }}
 switchport mode trunk
 switchport trunk allowed vlan 10,20,99
!
{% endfor %}
!
spanning-tree mode rapid-pvst
spanning-tree extend system-id
!
ip default-gateway 192.168.1.1
!
line vty 0 4
 transport input ssh
 exec-timeout 10 0
```

---

## Multi-vendor Templates

### Vendor-Agnostic Interface Template

**Data Structure:**

```yaml
name: "core-device-01"
vendor: "cisco"  # or "juniper", "arista"
interfaces:
  - name: "interface1"
    type: "ethernet"
    speed: "1000"
    description: "Server Connection"
    ip: "10.1.1.1"
    netmask: "255.255.255.0"
  - name: "interface2"
    type: "ethernet" 
    speed: "10000"
    description: "Uplink to Core"
    vlan_mode: "trunk"
    allowed_vlans: [10, 20, 30]
```

**Cisco Template (templates/cisco/interfaces.j2):**

```jinja2
{% for interface in node.interfaces %}
interface {{ interface.name }}
 description {{ interface.description }}
{% if interface.ip %}
 ip address {{ interface.ip }} {{ interface.netmask }}
{% elif interface.vlan_mode == "access" %}
 switchport mode access
 switchport access vlan {{ interface.access_vlan }}
{% elif interface.vlan_mode == "trunk" %}
 switchport mode trunk
 switchport trunk allowed vlan {{ interface.allowed_vlans | join(',') }}
{% endif %}
{% if interface.speed %}
 speed {{ interface.speed }}
{% endif %}
 no shutdown
!
{% endfor %}
```

**Juniper Template (templates/juniper/interfaces.j2):**

```jinja2
interfaces {
{% for interface in node.interfaces %}
    {{ interface.name }} {
        description "{{ interface.description }}";
{% if interface.ip %}
        unit 0 {
            family inet {
                address {{ interface.ip }}/{{ interface.netmask | netmask_to_cidr }};
            }
        }
{% elif interface.vlan_mode == "access" %}
        unit 0 {
            family ethernet-switching {
                vlan {
                    members vlan-{{ interface.access_vlan }};
                }
            }
        }
{% elif interface.vlan_mode == "trunk" %}
        unit 0 {
            family ethernet-switching {
                interface-mode trunk;
                vlan {
                    members [ {% for vlan in interface.allowed_vlans %}vlan-{{ vlan }}{% if not loop.last %} {% endif %}{% endfor %} ];
                }
            }
        }
{% endif %}
    }
{% endfor %}
}
```

**Arista Template (templates/arista/interfaces.j2):**

```jinja2
{% for interface in node.interfaces %}
interface {{ interface.name }}
   description {{ interface.description }}
{% if interface.ip %}
   ip address {{ interface.ip }}/{{ interface.netmask | netmask_to_cidr }}
{% elif interface.vlan_mode == "access" %}
   switchport mode access
   switchport access vlan {{ interface.access_vlan }}
{% elif interface.vlan_mode == "trunk" %}
   switchport mode trunk
   switchport trunk allowed vlan {{ interface.allowed_vlans | join(',') }}
{% endif %}
{% if interface.speed %}
   speed {{ interface.speed }}
{% endif %}
   no shutdown
!
{% endfor %}
```

---

## Advanced Configuration Patterns

### Dynamic OSPF Configuration

**Node Data:**

```yaml
name: "core-router-01"
vendor: "cisco"
ospf:
  process_id: 1
  router_id: "1.1.1.1"
  areas:
    - id: 0
      type: "backbone"
      interfaces:
        - name: "Loopback0"
          ip: "1.1.1.1"
          netmask: "255.255.255.255"
        - name: "GigabitEthernet0/0"
          ip: "10.1.1.1"
          netmask: "255.255.255.0"
    - id: 10
      type: "standard"
      interfaces:
        - name: "GigabitEthernet0/1"
          ip: "10.10.1.1" 
          netmask: "255.255.255.0"
      authentication:
        type: "message-digest"
        key_id: 1
```

**Template (templates/cisco/ospf-dynamic.j2):**

```jinja2
{% set ospf = node.ospf %}
router ospf {{ ospf.process_id }}
 router-id {{ ospf.router_id }}
{% for area in ospf.areas %}
{% for interface in area.interfaces %}
 network {{ interface.ip | ipaddr('network') }} {{ interface.netmask | netmask_to_wildcard }} area {{ area.id }}
{% endfor %}
{% if area.authentication %}
 area {{ area.id }} authentication {{ area.authentication.type }}
{% endif %}
{% endfor %}
!
{% for area in ospf.areas %}
{% for interface in area.interfaces %}
interface {{ interface.name }}
 ip address {{ interface.ip }} {{ interface.netmask }}
{% if area.authentication and area.authentication.type == "message-digest" %}
 ip ospf message-digest-key {{ area.authentication.key_id }} md5 {{ secrets.ospf_auth_key }}
{% endif %}
 ip ospf {{ ospf.process_id }} area {{ area.id }}
 no shutdown
!
{% endfor %}
{% endfor %}
```

### BGP with Route Policies

**Node Data:**

```yaml
name: "edge-router-01"
vendor: "cisco"
bgp:
  asn: 65001
  router_id: "1.1.1.1"
  neighbors:
    - ip: "203.0.113.2"
      asn: 65002
      description: "ISP-A Primary"
      type: "external"
      policies:
        inbound: "ISP-A-IN"
        outbound: "ISP-A-OUT"
    - ip: "203.0.113.6"
      asn: 65003
      description: "ISP-B Backup"
      type: "external"
      local_pref: 50
      policies:
        inbound: "ISP-B-IN"
        outbound: "ISP-B-OUT"
  route_maps:
    - name: "ISP-A-IN"
      sequences:
        - seq: 10
          action: "permit"
          match: "ip address prefix-list ISP-A-PREFIXES"
          set: "local-preference 100"
    - name: "ISP-A-OUT"
      sequences:
        - seq: 10
          action: "permit"
          match: "ip address prefix-list OUR-PREFIXES"
```

**Template (templates/cisco/bgp-advanced.j2):**

```jinja2
{% set bgp = node.bgp %}
!
{# Prefix lists and route maps first #}
{% for route_map in bgp.route_maps %}
route-map {{ route_map.name }}
{% for seq in route_map.sequences %}
 {{ seq.action }} {{ seq.seq }}
{% if seq.match %}
  match {{ seq.match }}
{% endif %}
{% if seq.set %}
  set {{ seq.set }}
{% endif %}
{% endfor %}
!
{% endfor %}
!
router bgp {{ bgp.asn }}
 bgp router-id {{ bgp.router_id }}
 bgp log-neighbor-changes
!
{% for neighbor in bgp.neighbors %}
 neighbor {{ neighbor.ip }} remote-as {{ neighbor.asn }}
 neighbor {{ neighbor.ip }} description {{ neighbor.description }}
{% if neighbor.type == "external" %}
 neighbor {{ neighbor.ip }} ebgp-multihop 2
{% endif %}
{% if neighbor.local_pref %}
 neighbor {{ neighbor.ip }} route-map SET-LOCAL-PREF-{{ neighbor.local_pref }} in
{% endif %}
{% if neighbor.policies.inbound %}
 neighbor {{ neighbor.ip }} route-map {{ neighbor.policies.inbound }} in
{% endif %}
{% if neighbor.policies.outbound %}
 neighbor {{ neighbor.ip }} route-map {{ neighbor.policies.outbound }} out
{% endif %}
!
{% endfor %}
```

### Data Center Fabric Configuration

**Node Data (Spine Switch):**

```yaml
name: "spine-01"
type: "switch"
vendor: "arista"
role: "spine"
fabric:
  underlay_protocol: "bgp"
  overlay_protocol: "vxlan-evpn"
  asn: 65100
  router_id: "1.1.1.1"
leaf_connections:
  - leaf_name: "leaf-01"
    local_interface: "Ethernet1"
    remote_interface: "Ethernet49"
    link_ip: "10.1.1.0/31"
    leaf_asn: 65101
  - leaf_name: "leaf-02"
    local_interface: "Ethernet2"
    remote_interface: "Ethernet49"
    link_ip: "10.1.1.2/31"
    leaf_asn: 65102
```

**Template (templates/arista/spine-bgp-evpn.j2):**

```jinja2
hostname {{ node.name }}
!
{% for connection in node.fabric.leaf_connections %}
interface {{ connection.local_interface }}
   description Connection to {{ connection.leaf_name }}
   no switchport
   ip address {{ connection.link_ip | ipaddr('0') }}/31
   no shutdown
!
{% endfor %}
!
interface Loopback0
   description Router ID
   ip address {{ node.fabric.router_id }}/32
!
router bgp {{ node.fabric.asn }}
   router-id {{ node.fabric.router_id }}
   maximum-paths 64
   distance bgp 20 200 200
   !
   {% for connection in node.fabric.leaf_connections %}
   neighbor {{ connection.link_ip | ipaddr('1') }} remote-as {{ connection.leaf_asn }}
   neighbor {{ connection.link_ip | ipaddr('1') }} description {{ connection.leaf_name }}
   neighbor {{ connection.link_ip | ipaddr('1') }} maximum-routes 12000
   !
   {% endfor %}
   address-family evpn
      {% for connection in node.fabric.leaf_connections %}
      neighbor {{ connection.link_ip | ipaddr('1') }} activate
      {% endfor %}
   !
   address-family ipv4
      {% for connection in node.fabric.leaf_connections %}
      neighbor {{ connection.link_ip | ipaddr('1') }} activate
      {% endfor %}
      redistribute connected
```

---

## Policy Examples

### Security Compliance Policies

**policies/security-baseline.rules:**

```
# Basic security requirements
assert config contains "service password-encryption"
  message "Password encryption must be enabled"

assert config not contains "username.*password "
  message "Plaintext passwords are prohibited - use 'secret' instead"

assert config contains "exec-timeout"
  message "Session timeouts must be configured"

assert config not contains "snmp-server community public"
assert config not contains "snmp-server community private"
  message "Default SNMP community strings are prohibited"

# SSH requirements
assert config contains "transport input ssh" or config contains "ip ssh"
  message "SSH must be enabled for remote management"

assert config not contains "transport input telnet"
  message "Telnet is prohibited - use SSH only"

# Login security
assert config contains "login block-for" or config contains "aaa new-model"
  message "Login attack protection must be configured"
```

### Operational Standards Policies

**policies/operational-standards.rules:**

```
# Naming conventions
assert node.name matches "^[a-z]+-[a-z]+-[0-9]+$"
  message "Device names must follow format: role-location-number (e.g., core-dc1-01)"

# Required management configuration
assert node.management_ip is not null
  message "All devices must have a management IP address"

assert config contains "ntp server"
  message "NTP must be configured for time synchronization"

assert config contains "logging host" or config contains "logging buffered"
  message "Logging must be configured (local or remote)"

# Interface descriptions
assert config matches "interface.*" -> config matches "interface.*\\n\\s+description"
  message "All configured interfaces must have descriptions"

# VLAN validation for switches
assert node.type == "switch" -> (
  config contains "vlan" and 
  config not contains "vlan 1 "
)
  message "Switches must have VLANs configured and VLAN 1 should not be used"
```

### Role-based Policies

**policies/role-access-switch.rules:**

```
# Access switch specific requirements
assert node.role == "access" -> config contains "spanning-tree portfast"
  message "Access switches must have PortFast enabled on access ports"

assert node.role == "access" -> config contains "spanning-tree bpduguard enable"
  message "Access switches must have BPDU Guard enabled"

assert node.role == "access" -> config contains "storm-control"
  message "Access switches must have storm control configured"

# Port security for access switches
assert node.role == "access" and config contains "switchport mode access" ->
       config contains "switchport port-security"
  message "Access ports should have port security enabled"

# Uplink configuration
assert node.role == "access" -> config.count("switchport mode trunk") >= 1
  message "Access switches must have at least one trunk uplink"
```

---

## Real-world Scenarios

### Branch Office Deployment

**Scenario:** Deploy standardized configuration to 50 branch offices with minimal local variation.

**Base Template (templates/branch/router-base.j2):**

```jinja2
{% extends "base/cisco-router.j2" %}

{# Branch-specific global settings #}
{% block global_settings %}
{{ super() }}
ip dhcp excluded-address {{ branch.lan_network | ipaddr('1') }} {{ branch.lan_network | ipaddr('10') }}
!
ip dhcp pool BRANCH-LAN
 network {{ branch.lan_network | ipaddr('network') }} {{ branch.lan_network | ipaddr('netmask') }}
 default-router {{ branch.lan_network | ipaddr('1') }}
 dns-server {{ config.dns_servers | join(' ') }}
 lease 7
!
{% endblock %}

{# WAN configuration #}
{% block wan_interfaces %}
interface {{ branch.wan_interface }}
 description WAN to {{ branch.isp_name }}
 ip address dhcp
 ip nat outside
 no shutdown
!
{% endblock %}

{# LAN configuration #}
{% block lan_interfaces %}
interface {{ branch.lan_interface }}
 description LAN Interface
 ip address {{ branch.lan_network | ipaddr('1') }} {{ branch.lan_network | ipaddr('netmask') }}
 ip nat inside
 no shutdown
!
{% endblock %}

{# NAT configuration #}
{% block nat %}
ip nat inside source list 1 interface {{ branch.wan_interface }} overload
!
access-list 1 permit {{ branch.lan_network | ipaddr('network') }} {{ branch.lan_network | ipaddr('hostmask') }}
!
{% endblock %}

{# Site-to-site VPN #}
{% block vpn %}
crypto isakmp policy 10
 encr aes 256
 hash sha256
 authentication pre-share
 group 14
 lifetime 86400
!
crypto isakmp key {{ secrets.vpn_preshared_key }} address {{ config.headquarters.vpn_peer }}
!
crypto ipsec transform-set ESP-AES256-SHA256 esp-aes 256 esp-sha256-hmac
 mode tunnel
!
crypto map SITE-TO-SITE 10 ipsec-isakmp
 set peer {{ config.headquarters.vpn_peer }}
 set transform-set ESP-AES256-SHA256
 match address VPN-TRAFFIC
!
interface {{ branch.wan_interface }}
 crypto map SITE-TO-SITE
!
ip access-list extended VPN-TRAFFIC
 permit ip {{ branch.lan_network | ipaddr('network') }} {{ branch.lan_network | ipaddr('hostmask') }} {{ config.headquarters.lan_network | ipaddr('network') }} {{ config.headquarters.lan_network | ipaddr('hostmask') }}
{% endblock %}
```

**Per-site Data:**

```yaml
# sites/seattle.yaml
name: "branch-seattle-01"
branch:
  lan_network: "10.100.1.0/24"
  lan_interface: "GigabitEthernet0/1"
  wan_interface: "GigabitEthernet0/0"
  isp_name: "Local ISP"
location: "Seattle Branch Office"
```

### Data Center Migration

**Scenario:** Migrate from legacy flat network to modern segmented architecture.

**Migration Template (templates/migration/legacy-to-segmented.j2):**

```jinja2
{# Phase 1: Add new VLANs while maintaining legacy #}
{% if migration.phase == 1 %}
{# Legacy VLAN (will be removed in phase 3) #}
vlan {{ migration.legacy_vlan }}
 name LEGACY-{{ migration.legacy_vlan }}
!
{# New segmented VLANs #}
{% for segment in migration.new_segments %}
vlan {{ segment.vlan_id }}
 name {{ segment.name }}
!
{% endfor %}

{# Dual-homed configuration during migration #}
{% for segment in migration.new_segments %}
interface Vlan{{ segment.vlan_id }}
 description {{ segment.description }}
 ip address {{ segment.gateway_ip }} {{ segment.netmask }}
 ip helper-address {{ config.dhcp_server }}
 no shutdown
!
{% endfor %}

{% elif migration.phase == 2 %}
{# Phase 2: Move devices to new VLANs #}
{% for port_migration in migration.port_moves %}
interface {{ port_migration.interface }}
 description Migrated: {{ port_migration.description }}
 switchport access vlan {{ port_migration.new_vlan }}
!
{% endfor %}

{% elif migration.phase == 3 %}
{# Phase 3: Remove legacy configuration #}
no vlan {{ migration.legacy_vlan }}
!
{% for cleanup in migration.cleanup_interfaces %}
interface {{ cleanup.interface }}
 no description {{ cleanup.old_description }}
!
{% endfor %}
{% endif %}
```

### Disaster Recovery Automation

**Scenario:** Rapid deployment of backup configurations during outages.

**DR Template (templates/dr/emergency-config.j2):**

```jinja2
{# Emergency configuration for disaster recovery #}
hostname {{ node.name }}-DR-{{ dr.activation_time | strftime('%Y%m%d') }}
!
{# Minimal essential configuration #}
{% for essential_interface in dr.essential_interfaces %}
interface {{ essential_interface.name }}
 description DR: {{ essential_interface.description }}
 ip address {{ essential_interface.backup_ip }} {{ essential_interface.netmask }}
 no shutdown
!
{% endfor %}

{# Emergency routing - simple static routes #}
{% for route in dr.emergency_routes %}
ip route {{ route.network }} {{ route.netmask }} {{ route.next_hop }}
!
{% endfor %}

{# Simplified protocols for quick convergence #}
{% if dr.routing_protocol == "static" %}
{# Static routing only during DR #}
{% for static_route in dr.static_routes %}
ip route {{ static_route.destination }} {{ static_route.netmask }} {{ static_route.next_hop }}
{% endfor %}
{% else %}
{# Minimal OSPF with fast convergence #}
router ospf 1
 router-id {{ node.dr_router_id }}
{% for network in dr.ospf_networks %}
 network {{ network.address }} {{ network.wildcard }} area 0
{% endfor %}
 area 0 stub
 timers throttle spf 1 1 1
!
{% endif %}

banner motd ^
DISASTER RECOVERY MODE ACTIVE
Activated: {{ dr.activation_time | strftime('%Y-%m-%d %H:%M:%S UTC') }}
Contact NOC: {{ config.noc_contact }}
^
```

---

## Complete Workflows

### CI/CD Integration Example

**1. Template Development Workflow:**

```bash
#!/bin/bash
# scripts/template-ci.sh

echo "Running template validation pipeline..."

# Step 1: Validate template syntax
echo "Validating template syntax..."
for template in templates/**/*.j2; do
    unet templates validate "$template" || exit 1
done

# Step 2: Run template tests
echo "Running template test suite..."
unet templates test-suite tests/templates/ || exit 1

# Step 3: Generate test configurations
echo "Generating test configurations..."
mkdir -p test-output/
unet config generate --all --output-dir test-output/ --template-dir templates/

# Step 4: Validate generated configs against policies
echo "Validating generated configurations..."
unet policies check --all --config-dir test-output/ --policy-dir policies/

# Step 5: Diff against baseline
echo "Comparing with baseline configurations..."
if [ -d baseline-configs/ ]; then
    for config in test-output/*.cfg; do
        baseline="baseline-configs/$(basename "$config")"
        if [ -f "$baseline" ]; then
            unet config diff "$baseline" "$config" --output "diffs/$(basename "$config" .cfg).diff"
        fi
    done
fi

echo "Template validation pipeline completed successfully!"
```

**2. Automated Deployment Pipeline:**

```bash
#!/bin/bash
# scripts/deploy-pipeline.sh

ENVIRONMENT=${1:-staging}
DRY_RUN=${2:-true}

echo "Starting deployment pipeline for $ENVIRONMENT environment..."

# Step 1: Generate configurations for environment
echo "Generating configurations..."
unet config generate --all \
    --environment "$ENVIRONMENT" \
    --output-dir "deploy/$ENVIRONMENT/"

# Step 2: Validate configurations
echo "Running policy validation..."
unet policies check --all \
    --config-dir "deploy/$ENVIRONMENT/" \
    --policy-set "critical,standard"

# Step 3: Create deployment plan
echo "Creating deployment plan..."
unet deployment plan \
    --config-dir "deploy/$ENVIRONMENT/" \
    --output "deploy/$ENVIRONMENT/deployment-plan.yaml"

# Step 4: Execute deployment (if not dry run)
if [ "$DRY_RUN" = "false" ]; then
    echo "Executing deployment..."
    unet deployment execute \
        --plan "deploy/$ENVIRONMENT/deployment-plan.yaml" \
        --parallel 5 \
        --rollback-on-failure
else
    echo "Dry run completed. Use '--dry-run false' to execute deployment."
fi
```

### Monitoring and Alerting Integration

**Template with Monitoring Hooks:**

```jinja2
{# templates/monitored-device.j2 #}
{% extends "base/device.j2" %}

{% block monitoring %}
{# SNMP configuration for monitoring #}
snmp-server community {{ secrets.snmp_ro_community }} RO 99
snmp-server location {{ node.location }}
snmp-server contact {{ node.owner | default(config.default_contact) }}
{% for monitor in config.monitoring.snmp_servers %}
snmp-server host {{ monitor.ip }} version 2c {{ secrets.snmp_ro_community }}
{% endfor %}
snmp-server enable traps snmp authentication linkdown linkup coldstart warmstart
snmp-server enable traps config
!
{# Syslog configuration #}
{% for syslog in config.monitoring.syslog_servers %}
logging host {{ syslog.ip }}
{% endfor %}
logging trap warnings
logging facility local7
service timestamps log datetime msec
!
{# Performance monitoring #}
ip sla 1
 icmp-echo {{ config.monitoring.ping_target }}
 frequency 60
ip sla schedule 1 life forever start-time now
!
track 1 ip sla 1 reachability
!
{% endblock %}

{% block post_deployment_hooks %}
{# Notify monitoring system of configuration change #}
{# This would trigger external webhook/API call #}
event manager applet CONFIG-DEPLOYED
 event syslog pattern "SYS-5-CONFIG_I"
 action 1.0 cli command "send log Configuration deployed on {{ node.name }} at {{ ansible_date_time.iso8601 }}"
!
{% endblock %}
```

**Validation with Monitoring Integration:**

```
# policies/monitoring-compliance.rules

# Ensure SNMP is configured for monitoring
assert config contains "snmp-server host"
  message "SNMP monitoring targets must be configured"

assert config contains "snmp-server enable traps config"
  message "Configuration change traps must be enabled"

# Ensure logging is configured
assert config contains "logging host"
  message "Remote logging must be configured"

assert config contains "service timestamps log datetime"
  message "Log timestamps must be enabled"

# Ensure monitoring connectivity
assert config contains "ip sla" and config contains "track"
  message "SLA monitoring must be configured for uptime tracking"
```

---

This comprehensive example collection demonstrates real-world usage patterns for μNet. Each example can be adapted for your specific environment by modifying the data structures and templates to match your network architecture and requirements.

For more examples and patterns, see the existing **[Policy Examples](16_policy_examples.md)** and **[DSL Syntax Reference](17_dsl_syntax_reference.md)** documentation.
