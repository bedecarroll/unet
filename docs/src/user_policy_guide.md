# Policy Creation Guide

> **Audience:** Network engineers who want to implement automated configuration validation using μNet's policy engine.  
> **Objective:** Learn to write effective policies that enforce standards, security, and compliance across your network infrastructure.

---

## Table of Contents

1. [Policy Fundamentals](#policy-fundamentals)
2. [Policy Language Syntax](#policy-language-syntax)
3. [Writing Your First Policy](#writing-your-first-policy)
4. [Advanced Policy Patterns](#advanced-policy-patterns)
5. [Security Policies](#security-policies)
6. [Compliance Policies](#compliance-policies)
7. [Performance and Optimization](#performance-and-optimization)
8. [Testing and Debugging](#testing-and-debugging)

---

## Policy Fundamentals

### What are Policies?

Policies in μNet are rules that automatically validate network configurations against your organization's standards. They help ensure:

- **Security compliance** - Enforce security best practices
- **Configuration consistency** - Maintain standards across devices
- **Regulatory compliance** - Meet industry requirements (SOX, PCI, etc.)
- **Error prevention** - Catch misconfigurations before deployment

### Policy Components

Every policy consists of:

1. **Assertion** - The rule being checked (`assert`)
2. **Condition** - What triggers the rule (optional)
3. **Expression** - What is being validated
4. **Message** - Description of the rule (optional)

Basic structure:

```
assert <condition> -> <expression>
```

### Policy Evaluation

Policies are evaluated:

- **Before deployment** - Prevent bad configurations from being deployed
- **On configuration generation** - Catch template issues early
- **During audits** - Verify existing configurations meet standards
- **In CI/CD pipelines** - Automate compliance checking

---

## Policy Language Syntax

### Basic Assertions

**Simple content checks:**

```
# Check if configuration contains specific text
assert config contains "ntp server"

# Check if configuration does NOT contain something
assert config not contains "snmp-server community public"

# Check for exact matches
assert config matches "hostname [a-zA-Z0-9-]+"
```

**Node attribute checks:**

```
# Verify node attributes exist
assert node.management_ip is not null
assert node.location is not empty

# Check attribute values
assert node.type == "router"
assert node.vendor in ["cisco", "juniper", "arista"]
assert node.os_version >= "15.1"
```

### Conditional Logic

**If-then patterns:**

```
# Apply rules conditionally
assert node.type == "router" -> config contains "router ospf"
assert node.location == "datacenter" -> config contains "ntp server 10.1.1.100"
assert node.security_level == "high" -> config contains "aaa new-model"
```

**Complex conditions:**

```
# Multiple conditions with AND
assert (node.type == "switch" and node.role == "access") -> 
       config contains "spanning-tree portfast"

# Multiple conditions with OR
assert (node.vendor == "cisco" or node.vendor == "arista") -> 
       config contains "enable secret"

# Negated conditions
assert not (node.type == "firewall") or config contains "access-list"
```

### String Operations

**Pattern matching:**

```
# Regular expressions
assert config matches "interface GigabitEthernet[0-9]+/[0-9]+"
assert config matches "ip address [0-9.]+\\s+[0-9.]+"

# Wildcard patterns
assert config contains "vlan *"
assert config contains "interface * description"
```

**String functions:**

```
# Count occurrences
assert config.count("interface") >= 4

# Length checks
assert node.name.length >= 3
assert node.name.length <= 32

# Case handling
assert config.lower contains "hostname"
assert node.name.upper == node.name  # All uppercase
```

### List and Set Operations

**Membership tests:**

```
# Check if value is in a list
assert node.vendor in ["cisco", "juniper", "arista", "nokia"]
assert node.type in ["router", "switch", "firewall"]

# Check if none of the values are present
assert node.os_version not in ["12.1", "12.2", "12.3"]  # Vulnerable versions
```

**List operations:**

```
# Check list properties
assert node.vlans.length >= 1
assert node.interfaces.length <= 48

# All/any operations
assert all(interface.speed >= 1000 for interface in node.interfaces)
assert any(interface.type == "management" for interface in node.interfaces)
```

---

## Writing Your First Policy

### Step 1: Identify Requirements

Start by documenting your organization's standards:

```
Network Configuration Standards:
1. All devices must have NTP configured
2. Management interfaces must have descriptions
3. SNMP community strings must not be default values
4. Console timeouts must be configured
5. SSH must be enabled, telnet disabled
```

### Step 2: Create Basic Policies

**policies/basic-standards.rules:**

```
# NTP requirement
assert config contains "ntp server"
  message "All devices must have NTP configured for time synchronization"

# Management interface descriptions
assert node.management_ip -> config contains "description Management"
  message "Management interfaces must have proper descriptions"

# No default SNMP communities
assert config not contains "snmp-server community public"
assert config not contains "snmp-server community private"
  message "Default SNMP community strings are prohibited"

# Console timeout
assert config contains "exec-timeout"
  message "Console/VTY sessions must have timeout configured"

# SSH enabled, telnet disabled
assert config contains "transport input ssh"
assert config not contains "transport input telnet"
  message "Only SSH is allowed for remote access"
```

### Step 3: Test Your Policies

```bash
# Test against a single device
unet policies check router-01 --policy basic-standards.rules

# Test against all devices
unet policies check --all --policy basic-standards.rules

# Debug policy evaluation
unet policies check router-01 --policy basic-standards.rules --debug
```

### Step 4: Refine and Expand

Based on test results, refine your policies:

```
# More specific NTP requirement
assert config contains "ntp server" and 
       (config contains "ntp server 10.1.1.100" or config contains "ntp server 10.2.1.100")
  message "Must use approved NTP servers: 10.1.1.100 or 10.2.1.100"

# Conditional management interface requirements
assert node.type in ["router", "switch"] -> 
       config matches "interface.*description.*[Mm]anagement"
  message "Routers and switches must have management interface descriptions"
```

---

## Advanced Policy Patterns

### Role-based Policies

**policies/role-based.rules:**

```
# Core router requirements
assert node.role == "core" -> (
  config contains "router ospf" and
  config contains "redistribute connected" and
  config.count("interface Loopback") >= 1
)

# Access switch requirements  
assert node.role == "access" -> (
  config contains "spanning-tree mode rapid-pvst" and
  config contains "spanning-tree portfast" and
  config contains "spanning-tree bpduguard enable"
)

# Distribution switch requirements
assert node.role == "distribution" -> (
  config contains "spanning-tree vlan" and
  config matches "spanning-tree vlan.*priority [0-9]+" and
  config.count("interface Vlan") >= 2
)

# Firewall requirements
assert node.role == "firewall" -> (
  config contains "access-list" and
  config contains "object-group" and
  config not contains "access-list.*permit ip any any"
)
```

### Location-based Policies

**policies/location-based.rules:**

```
# Datacenter-specific requirements
assert node.location == "datacenter" -> (
  config contains "ntp server 10.1.1.100" and
  config contains "logging host 10.1.1.200" and
  config contains "snmp-server host 10.1.1.300"
)

# Branch office requirements
assert node.location.startswith("branch-") -> (
  config contains "ntp server 8.8.8.8" and
  config contains "ip route 0.0.0.0 0.0.0.0" and
  config contains "crypto map"
)

# DMZ requirements
assert node.location == "dmz" -> (
  config contains "access-list" and
  config not contains "snmp-server community" and
  config contains "logging buffered warnings"
)
```

### Interface Policies

**policies/interface-standards.rules:**

```
# All interfaces must have descriptions
assert config matches "interface.*" -> 
       config matches "interface.*\\n\\s+description"
  message "All configured interfaces must have descriptions"

# Access ports must be in specific VLANs
assert config contains "switchport mode access" ->
       config matches "switchport access vlan (10|20|30|100)"
  message "Access ports must use approved VLANs: 10, 20, 30, or 100"

# Trunk ports must have allowed VLAN lists
assert config contains "switchport mode trunk" ->
       config contains "switchport trunk allowed vlan"
  message "Trunk ports must have explicit allowed VLAN configuration"

# Management interfaces must be in management VLAN
assert config matches "interface.*[Mm]anagement" ->
       config contains "switchport access vlan 999"
  message "Management interfaces must be in VLAN 999"
```

### Version-specific Policies

**policies/version-compliance.rules:**

```
# IOS version requirements
assert node.vendor == "cisco" and node.type == "router" ->
       node.os_version >= "15.1"
  message "Cisco routers must run IOS 15.1 or later"

# Security patches
assert node.vendor == "cisco" and node.os_version.startswith("15.1") ->
       node.os_version not in ["15.1(4)M1", "15.1(4)M2", "15.1(4)M3"]
  message "IOS versions 15.1(4)M1-M3 have known vulnerabilities"

# Feature set requirements
assert node.vendor == "cisco" and node.role == "edge" ->
       node.feature_set == "securityk9"
  message "Edge routers must have security feature set"
```

---

## Security Policies

### Authentication and Authorization

**policies/security-auth.rules:**

```
# AAA model required for production
assert node.environment == "production" -> config contains "aaa new-model"
  message "Production devices must use AAA authentication model"

# No default passwords
assert config not contains "username admin password"
assert config not contains "enable password"
  message "Default passwords are prohibited"

# Strong password requirements
assert config contains "username" -> config contains "secret"
  message "User passwords must use 'secret' (hashed) not 'password' (cleartext)"

# Privilege separation
assert config not contains "username.*privilege 15"
  message "Direct privilege 15 assignment is prohibited - use role-based access"

# Login security
assert config contains "login block-for"
assert config contains "login quiet-mode access-class"
  message "Login security features must be enabled"
```

### Network Security

**policies/security-network.rules:**

```
# Disable unnecessary services
assert config not contains "ip http server"
assert config not contains "cdp run"
assert config not contains "service finger"
  message "Unnecessary network services must be disabled"

# Secure management protocols only
assert config not contains "transport input telnet"
assert config not contains "snmp-server community.*RW"
  message "Only secure management protocols allowed"

# Control plane protection
assert node.type == "router" -> config contains "control-plane"
  message "Routers must have control plane protection configured"

# Interface security
assert config matches "interface.*Ethernet" -> (
  config contains "storm-control" or
  config contains "switchport port-security"
)
  message "Access interfaces must have storm control or port security"
```

### Encryption and Privacy

**policies/security-crypto.rules:**

```
# SSH version 2 only
assert config contains "ip ssh" -> config contains "ip ssh version 2"
  message "Only SSH version 2 is permitted"

# Strong encryption algorithms
assert config contains "crypto" -> (
  config not contains "des" and
  config not contains "md5"
)
  message "Weak encryption algorithms (DES, MD5) are prohibited"

# Certificate validation
assert config contains "crypto pki" -> config contains "revocation-check"
  message "PKI configurations must include certificate revocation checking"
```

---

## Compliance Policies

### SOX Compliance

**policies/compliance-sox.rules:**

```
# Change control requirements
assert node.compliance.sox == true -> (
  config contains "archive" and
  config contains "logging enable" and
  config contains "snmp-server enable traps config"
)
  message "SOX-compliant devices must have change tracking enabled"

# Access logging
assert node.compliance.sox == true -> (
  config contains "aaa accounting" and
  config contains "logging buffered" and
  config contains "service timestamps log datetime msec"
)
  message "SOX compliance requires comprehensive access logging"

# No shared accounts
assert node.compliance.sox == true ->
       config not matches "username (admin|shared|generic)"
  message "SOX compliance prohibits shared administrative accounts"
```

### PCI DSS Compliance

**policies/compliance-pci.rules:**

```
# Network segmentation
assert node.compliance.pci == true and node.role == "firewall" -> (
  config contains "access-list.*permit.*tcp.*eq 443" and
  config not contains "access-list.*permit ip any any"
)
  message "PCI environments require proper network segmentation"

# Strong authentication
assert node.compliance.pci == true -> (
  config contains "aaa new-model" and
  config contains "login block-for" and
  config not contains "username.*password"
)
  message "PCI compliance requires strong authentication controls"

# Logging requirements
assert node.compliance.pci == true -> (
  config contains "logging host" and
  config contains "logging trap warnings" and
  config contains "service timestamps log datetime msec"
)
  message "PCI compliance requires centralized logging with timestamps"
```

### NIST Cybersecurity Framework

**policies/compliance-nist.rules:**

```
# Asset identification (IDENTIFY)
assert node.compliance.nist == true -> (
  node.asset_tag is not null and
  node.owner is not null and
  node.criticality in ["low", "medium", "high", "critical"]
)
  message "NIST compliance requires complete asset identification"

# Access control (PROTECT)
assert node.compliance.nist == true -> (
  config contains "aaa" and
  config contains "exec-timeout" and
  config not contains "no exec-timeout"
)
  message "NIST compliance requires access controls and session timeouts"

# Monitoring and detection (DETECT)
assert node.compliance.nist == true -> (
  config contains "logging" and
  config contains "snmp-server enable traps" and
  config.count("logging host") >= 1
)
  message "NIST compliance requires comprehensive monitoring"
```

---

## Performance and Optimization

### Efficient Policy Writing

**Use specific conditions to reduce evaluation overhead:**

```
# Good: Specific condition first
assert node.vendor == "cisco" and node.type == "router" ->
       config contains "router ospf"

# Less efficient: Generic condition first  
assert config contains "interface" and node.vendor == "cisco" ->
       config contains "description"
```

**Group related policies:**

```
# Combine related checks
assert node.type == "switch" -> (
  config contains "spanning-tree mode" and
  config contains "vlan" and
  config not contains "spanning-tree mode spanning-tree"
)

# Instead of separate assertions:
# assert node.type == "switch" -> config contains "spanning-tree mode"
# assert node.type == "switch" -> config contains "vlan"  
# assert node.type == "switch" -> config not contains "spanning-tree mode spanning-tree"
```

### Policy Organization

**Structure policies by evaluation frequency:**

```
policies/
├── critical/           # Evaluated on every config change
│   ├── security.rules
│   └── safety.rules
├── standard/          # Evaluated before deployment
│   ├── naming.rules
│   ├── interfaces.rules
│   └── protocols.rules
└── audit/             # Evaluated during compliance audits
    ├── sox.rules
    ├── pci.rules
    └── reporting.rules
```

**Use policy sets for different scenarios:**

```bash
# Critical policies for immediate validation
unet policies check --policy-set critical --all

# Full validation before deployment
unet policies check --policy-set standard,critical --all

# Compliance audit
unet policies check --policy-set audit --all --output compliance-report.json
```

---

## Testing and Debugging

### Policy Development Workflow

**1. Start with simple tests:**

```
# Test basic syntax
assert true  # Should always pass
assert false message "This should fail"  # Should always fail
```

**2. Test against known configurations:**

```bash
# Create test configuration
cat > test-config.txt << EOF
hostname test-router
interface GigabitEthernet0/0
 description Management Interface
 ip address 192.168.1.1 255.255.255.0
EOF

# Test policy against known config
unet policies test "assert config contains 'hostname'" --config test-config.txt
```

**3. Use debug mode:**

```bash
# See policy evaluation details
unet policies check router-01 --policy test.rules --debug

# Output shows:
# - Which policies were evaluated
# - How conditions were resolved
# - Why policies passed or failed
```

### Common Policy Issues

**Issue: Policy never triggers**

```
# Problem: Condition too specific
assert node.vendor == "Cisco" -> config contains "enable secret"

# Solution: Check exact case and values
assert node.vendor == "cisco" -> config contains "enable secret"
```

**Issue: Policy fails unexpectedly**

```
# Problem: Case sensitivity
assert config contains "Hostname"

# Solution: Use case-insensitive matching
assert config.lower contains "hostname"
```

**Issue: Regular expression errors**

```
# Problem: Unescaped special characters
assert config matches "ip address [0-9.]+ [0-9.]+"

# Solution: Proper escaping
assert config matches "ip address [0-9\\.]+\\s+[0-9\\.]+"
```

### Policy Testing Tools

**Interactive testing:**

```bash
# Test policy expressions interactively
unet policies console

> config = load_config("router-01.cfg")
> config contains "ntp server"
True
> config.count("interface")
4
> node.vendor == "cisco"
True
```

**Automated testing:**

```bash
# Create policy test suite
cat > policy-tests.yaml << EOF
tests:
  - name: "NTP server check"
    policy: "assert config contains 'ntp server'"
    config: "test-configs/router-with-ntp.cfg"
    should_pass: true
  - name: "Default password check"
    policy: "assert config not contains 'username admin password'"
    config: "test-configs/router-default-password.cfg"
    should_pass: false
EOF

# Run test suite
unet policies test-suite policy-tests.yaml
```

### Debugging Complex Policies

**Break down complex assertions:**

```
# Complex policy (hard to debug)
assert (node.type == "router" and node.role == "edge") -> 
       (config contains "crypto map" and 
        config contains "tunnel" and 
        config.count("access-list") >= 2)

# Break into smaller pieces for debugging
assert node.type == "router" message "Device is router"
assert node.role == "edge" message "Device is edge router"  
assert config contains "crypto map" message "Crypto map configured"
assert config contains "tunnel" message "Tunnel configured"
assert config.count("access-list") >= 2 message "Sufficient access lists"
```

**Use descriptive messages:**

```
# Poor: Generic message
assert config contains "ntp server"

# Better: Specific, actionable message
assert config contains "ntp server" 
  message "NTP server must be configured for time synchronization. Add 'ntp server <ip>' to configuration."
```

---

## Best Practices

### Policy Development

1. **Start simple** - Begin with basic requirements, add complexity gradually
2. **Test thoroughly** - Validate policies against known good and bad configurations
3. **Use meaningful names** - Policy file names should indicate their purpose
4. **Document exceptions** - Comment why certain policies have exceptions
5. **Version control** - Track policy changes like code changes

### Policy Organization

1. **Separate by concern** - Security, compliance, operational standards
2. **Use inheritance** - Common policies in base files, specific ones in extensions
3. **Environment-specific** - Different policies for dev, staging, production
4. **Regular reviews** - Policies should evolve with your infrastructure

### Performance Considerations

1. **Order conditions efficiently** - Most specific conditions first
2. **Avoid redundant checks** - Combine related validations
3. **Use policy sets** - Don't run all policies for every validation
4. **Monitor execution time** - Profile policy performance regularly

---

## Next Steps

You now understand how to create effective policies for network configuration validation. Continue your learning with:

- **[Template Usage Tutorial](user_template_tutorial.md)** - Learn how templates and policies work together
- **[Example Configurations](user_examples.md)** - See real-world policy examples
- **[API Reference](api_reference.md)** - Advanced policy management via API

For more complex policy scenarios, refer to the existing **[Policy Examples](16_policy_examples.md)** and **[DSL Syntax Reference](17_dsl_syntax_reference.md)**.
