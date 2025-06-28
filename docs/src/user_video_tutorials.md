# Video Tutorial Scripts and Walkthroughs

> **Audience:** Content creators and trainers who want to create video tutorials for μNet.  
> **Objective:** Provide detailed scripts and walkthroughs for creating comprehensive video tutorials covering μNet's features.

---

## Table of Contents

1. [Video Tutorial Overview](#video-tutorial-overview)
2. [Getting Started Series](#getting-started-series)
3. [Configuration Management Series](#configuration-management-series)
4. [Advanced Topics Series](#advanced-topics-series)
5. [Real-world Scenarios Series](#real-world-scenarios-series)
6. [Production Guidelines](#production-guidelines)

---

## Video Tutorial Overview

### Tutorial Series Structure

**Total Duration:** ~6 hours across 12 videos
**Target Audience:** Network engineers new to μNet
**Prerequisites:** Basic networking knowledge, familiarity with CLI tools

### Equipment Needed

- **Screen Recording Software:** OBS Studio, Camtasia, or similar
- **Microphone:** Good quality USB microphone for clear audio
- **Test Environment:** Virtual machines or containers running μNet
- **Demo Network:** Simulated network devices (GNS3, EVE-NG, or Containerlab)

### Common Demo Environment Setup

```bash
# Create demo workspace
mkdir unet-demo
cd unet-demo

# Initialize μNet workspace
unet init

# Add sample devices for demonstrations
unet nodes add --name "demo-router-01" --type router --vendor cisco --management-ip 192.168.1.1
unet nodes add --name "demo-switch-01" --type switch --vendor cisco --management-ip 192.168.1.10
unet nodes add --name "demo-firewall-01" --type firewall --vendor cisco --management-ip 192.168.1.20
```

---

## Getting Started Series

### Video 1: "Introduction to μNet" (15 minutes)

**Learning Objectives:**

- Understand what μNet is and why it's useful
- See the problems it solves in network management
- Get excited about the possibilities

**Script Outline:**

**[0:00-2:00] Hook and Problem Statement**

```
"Imagine managing 100 network devices, each with slightly different configurations. 
One wrong command could take down your entire network. Sound familiar?

Today I'll show you μNet - a tool that transforms network configuration management 
from a manual, error-prone process into an automated, reliable system."
```

**[2:00-5:00] What is μNet?**

- Show the μNet logo and project overview
- Explain the core concepts: templates, policies, nodes
- Demonstrate the single binary approach

**[5:00-8:00] Quick Demo - "The Magic Moment"**

```bash
# Show empty directory
ls -la

# Initialize μNet
unet init

# Show what was created
tree .

# Add a device
unet nodes add --name "quick-demo" --type router --vendor cisco

# Generate configuration
unet config generate quick-demo --template cisco/basic
```

**[8:00-12:00] Key Benefits Walkthrough**

- Configuration consistency across devices
- Version control integration
- Policy-driven validation
- Multi-vendor support
- Template reusability

**[12:00-15:00] What You'll Learn Next**

- Preview of upcoming tutorials
- Call to action to follow along

**Demo Files Needed:**

- Pre-built templates showing different vendor outputs
- Sample policy violations and fixes
- Before/after configuration comparisons

---

### Video 2: "Installation and First Steps" (12 minutes)

**Learning Objectives:**

- Install μNet on different platforms
- Initialize first workspace
- Understand the file structure

**Script Outline:**

**[0:00-1:00] Introduction**

```
"In this tutorial, you'll get μNet installed and running on your system in under 10 minutes. 
By the end, you'll have generated your first network configuration."
```

**[1:00-4:00] Installation Options**
Show all three methods:

1. **Pre-built Binary (Recommended)**

```bash
# Download and install
curl -L https://github.com/your-org/unet/releases/latest/download/unet-linux-x86_64.tar.gz | tar xz
sudo mv unet /usr/local/bin/
unet --version
```

2. **Package Manager**

```bash
# Ubuntu/Debian
curl -fsSL https://packages.unet.dev/gpg | sudo apt-key add -
echo "deb https://packages.unet.dev/apt stable main" | sudo tee /etc/apt/sources.list.d/unet.list
sudo apt update && sudo apt install unet

# macOS
brew install unet/tap/unet
```

3. **Build from Source**

```bash
git clone https://github.com/your-org/unet.git
cd unet
cargo build --release
```

**[4:00-7:00] First Workspace Setup**

```bash
# Create project directory
mkdir my-network-configs
cd my-network-configs

# Initialize μNet workspace
unet init

# Explore what was created
ls -la
cat unet.toml
```

**[7:00-10:00] Understanding the Structure**

```bash
# Show directory structure
tree .

# Explain each directory
echo "nodes/ - Device configurations"
echo "templates/ - Configuration templates" 
echo "policies/ - Validation rules"
echo "unet.toml - Main configuration file"
```

**[10:00-12:00] First Configuration**

```bash
# Add your first device
unet nodes add \
  --name "my-first-router" \
  --type router \
  --vendor cisco \
  --management-ip 192.168.1.1

# Generate a basic configuration
unet config generate my-first-router --template basic

# Show the generated config
cat nodes/my-first-router.cfg
```

**Common Issues to Address:**

- Permission errors during installation
- PATH configuration for binary installation
- Firewall/antivirus false positives

---

### Video 3: "Your First Network Device" (18 minutes)

**Learning Objectives:**

- Add devices with different attributes
- Understand node data structures
- Generate and customize configurations

**Script Outline:**

**[0:00-2:00] Introduction and Setup**

```
"Now that μNet is installed, let's add our first real network device and generate a production-ready configuration."
```

**[2:00-6:00] Adding a Router**

```bash
# Add a border router with comprehensive attributes
unet nodes add \
  --name "border-router-01" \
  --type "router" \
  --vendor "cisco" \
  --management-ip "192.168.1.1" \
  --location "datacenter-east" \
  --custom-data '{
    "wan_interface": "GigabitEthernet0/1",
    "wan_ip": "203.0.113.1",
    "lan_interface": "GigabitEthernet0/0", 
    "lan_ip": "10.1.1.1",
    "router_id": "1.1.1.1"
  }'

# Show what was stored
unet nodes show border-router-01
```

**[6:00-10:00] Creating a Template**

```bash
# Create templates directory structure
mkdir -p templates/cisco

# Create a router template
cat > templates/cisco/border-router.j2 << 'EOF'
hostname {{ node.name }}
!
interface {{ node.custom_data.wan_interface }}
 description WAN Interface
 ip address {{ node.custom_data.wan_ip }} 255.255.255.252
 no shutdown
!
interface {{ node.custom_data.lan_interface }}
 description LAN Interface
 ip address {{ node.custom_data.lan_ip }} 255.255.255.0
 no shutdown
!
router ospf 1
 router-id {{ node.custom_data.router_id }}
 network {{ node.custom_data.lan_ip | ipaddr('network') }} 0.0.0.255 area 0
!
EOF
```

**[10:00-14:00] Generating Configuration**

```bash
# Generate the configuration
unet config generate border-router-01 --template cisco/border-router

# Show the generated config
cat nodes/border-router-01.cfg

# Explain how template variables were substituted
echo "Notice how {{ node.name }} became border-router-01"
echo "And {{ node.custom_data.wan_ip }} became 203.0.113.1"
```

**[14:00-18:00] Adding More Devices**

```bash
# Add an access switch
unet nodes add \
  --name "access-sw-floor1" \
  --type "switch" \
  --vendor "cisco" \
  --management-ip "192.168.1.10" \
  --custom-data '{
    "vlans": [
      {"id": 10, "name": "USERS"},
      {"id": 20, "name": "PRINTERS"}
    ],
    "uplink_interface": "GigabitEthernet0/48"
  }'

# Show the devices list
unet nodes list

# Generate config for the switch
unet config generate access-sw-floor1 --template cisco/access-switch
```

**Key Teaching Points:**

- How custom_data extends basic node attributes
- Template variable substitution
- File organization best practices

---

## Configuration Management Series

### Video 4: "Template Mastery" (25 minutes)

**Learning Objectives:**

- Master Jinja2 templating for networks
- Understand loops, conditions, and filters
- Create reusable template components

**Script Outline:**

**[0:00-3:00] Template Fundamentals**

```
"Templates are the heart of μNet. They transform your network data into actual device configurations. 
In this tutorial, you'll learn to create powerful, flexible templates that work across your entire network."
```

**[3:00-8:00] Basic Template Syntax**

```jinja2
{# Show progression from simple to complex #}

{# 1. Simple variable substitution #}
hostname {{ node.name }}

{# 2. Variable with default #}
interface {{ node.mgmt_interface | default('GigabitEthernet0/0') }}

{# 3. Conditional logic #}
{% if node.vendor == "cisco" %}
enable secret {{ secrets.enable_password }}
{% endif %}

{# 4. Loops #}
{% for vlan in node.vlans %}
vlan {{ vlan.id }}
 name {{ vlan.name }}
{% endfor %}
```

**[8:00-15:00] Advanced Patterns - Live Coding**

Create a comprehensive switch template:

```jinja2
{# templates/cisco/advanced-switch.j2 #}
hostname {{ node.name }}
!
{# VLAN Configuration #}
{% for vlan in node.vlans | default([]) %}
vlan {{ vlan.id }}
 name {{ vlan.name }}
{% if vlan.description %}
 description {{ vlan.description }}
{% endif %}
!
{% endfor %}

{# Interface Configuration #}
{% for interface in node.interfaces | default([]) %}
interface {{ interface.name }}
 description {{ interface.description | default('Configured by μNet') }}
{% if interface.type == "access" %}
 switchport mode access
 switchport access vlan {{ interface.vlan }}
 spanning-tree portfast
{% elif interface.type == "trunk" %}
 switchport mode trunk
 switchport trunk allowed vlan {{ interface.allowed_vlans | join(',') }}
{% endif %}
 no shutdown
!
{% endfor %}

{# Management Interface #}
interface Vlan{{ node.mgmt_vlan | default(999) }}
 description Management Interface
 ip address {{ node.management_ip }} 255.255.255.0
 no shutdown
```

**[15:00-20:00] Template Testing and Debugging**

```bash
# Test template with sample data
unet templates test cisco/advanced-switch --data '{
  "node": {
    "name": "test-switch",
    "management_ip": "192.168.1.10",
    "vlans": [{"id": 10, "name": "USERS"}],
    "interfaces": [
      {"name": "Gi0/1", "type": "access", "vlan": 10}
    ]
  }
}'

# Show debugging techniques
unet templates validate cisco/advanced-switch
```

**[20:00-25:00] Template Organization Best Practices**

```bash
# Show recommended structure
templates/
├── base/
│   └── cisco-device.j2
├── cisco/
│   ├── router.j2
│   ├── switch.j2
│   └── features/
│       ├── ospf.j2
│       └── vlans.j2
└── common/
    ├── management.j2
    └── logging.j2
```

**Demo Data Files:**

- Complex device with multiple interfaces and VLANs
- Before/after template improvements
- Common template errors and fixes

---

### Video 5: "Policy-Driven Validation" (20 minutes)

**Learning Objectives:**

- Write effective policies for network validation
- Understand policy language syntax
- Implement compliance checking

**Script Outline:**

**[0:00-2:00] Why Policies Matter**

```
"Imagine deploying a configuration that accidentally disables SSH access to all your devices. 
Policies in μNet prevent these disasters by automatically validating configurations before deployment."
```

**[2:00-7:00] Writing Your First Policy**

```bash
# Create policies directory
mkdir -p policies

# Create basic security policy
cat > policies/security-basics.rules << 'EOF'
# Ensure SSH is enabled
assert config contains "transport input ssh"
  message "SSH must be enabled for remote management"

# Ensure telnet is disabled  
assert config not contains "transport input telnet"
  message "Telnet is prohibited for security reasons"

# Ensure NTP is configured
assert config contains "ntp server"
  message "NTP must be configured for time synchronization"

# Ensure management interfaces have descriptions
assert node.management_ip -> config contains "description Management"
  message "Management interfaces must have proper descriptions"
EOF

# Test the policy
unet policies check border-router-01 --policy security-basics.rules
```

**[7:00-12:00] Advanced Policy Patterns**

```bash
# Role-based policies
cat > policies/router-requirements.rules << 'EOF'
# Router-specific requirements
assert node.type == "router" -> config contains "router ospf"
  message "Routers must have OSPF configured"

assert node.type == "router" -> config contains "ip routing"
  message "IP routing must be enabled on routers"

# Conditional requirements based on location
assert node.location == "datacenter" -> 
       config contains "ntp server 10.1.1.100"
  message "Datacenter devices must use internal NTP server"

# Interface validation
assert config matches "interface.*" -> 
       config matches "interface.*\\n\\s+description"
  message "All interfaces must have descriptions"
EOF
```

**[12:00-17:00] Policy Testing and Debugging**

```bash
# Run policies against all devices
unet policies check --all

# Show policy failure and fix
echo "Demonstrating a policy violation..."

# Create a bad configuration
cat > nodes/bad-config.cfg << 'EOF'
hostname bad-device
interface GigabitEthernet0/1
 ip address 192.168.1.1 255.255.255.0
! (No description - policy violation)
EOF

# Show the violation
unet policies check bad-config --policy security-basics.rules

# Fix the configuration
cat >> nodes/bad-config.cfg << 'EOF'
interface GigabitEthernet0/1
 description Fixed - Added description
EOF

# Show the fix worked
unet policies check bad-config --policy security-basics.rules
```

**[17:00-20:00] Compliance Integration**

```bash
# Create compliance policy sets
mkdir -p policies/compliance

cat > policies/compliance/sox.rules << 'EOF'
# SOX compliance requirements
assert node.compliance.sox == true -> (
  config contains "logging enable" and
  config contains "archive" and
  config contains "snmp-server enable traps config"
)
  message "SOX compliance requires change tracking"
EOF

# Run compliance checks
unet policies check --all --policy-set compliance
```

---

### Video 6: "Multi-vendor Configuration Management" (22 minutes)

**Learning Objectives:**

- Create vendor-agnostic data structures
- Build multi-vendor templates
- Handle vendor-specific differences

**Script Outline:**

**[0:00-2:00] The Multi-vendor Challenge**

```
"Your network probably isn't all one vendor. Cisco, Juniper, Arista - each has different syntax 
but similar concepts. Let's see how μNet handles this complexity elegantly."
```

**[2:00-8:00] Vendor-Agnostic Data Modeling**

```bash
# Add devices from different vendors
unet nodes add \
  --name "cisco-router-01" \
  --vendor "cisco" \
  --type "router" \
  --custom-data '{
    "interfaces": [
      {
        "name": "GigabitEthernet0/0",
        "ip": "192.168.1.1",
        "netmask": "255.255.255.0",
        "description": "Management Interface"
      }
    ]
  }'

unet nodes add \
  --name "juniper-router-01" \
  --vendor "juniper" \
  --type "router" \
  --custom-data '{
    "interfaces": [
      {
        "name": "ge-0/0/0",
        "ip": "192.168.1.2", 
        "netmask": "255.255.255.0",
        "description": "Management Interface"
      }
    ]
  }'
```

**[8:00-14:00] Creating Vendor-Specific Templates**

**Cisco Template:**

```jinja2
{# templates/cisco/router.j2 #}
hostname {{ node.name }}
!
{% for interface in node.custom_data.interfaces %}
interface {{ interface.name }}
 description {{ interface.description }}
 ip address {{ interface.ip }} {{ interface.netmask }}
 no shutdown
!
{% endfor %}
```

**Juniper Template:**

```jinja2
{# templates/juniper/router.j2 #}
system {
    host-name {{ node.name }};
}

interfaces {
{% for interface in node.custom_data.interfaces %}
    {{ interface.name }} {
        description "{{ interface.description }}";
        unit 0 {
            family inet {
                address {{ interface.ip }}/{{ interface.netmask | netmask_to_cidr }};
            }
        }
    }
{% endfor %}
}
```

**[14:00-18:00] Automatic Vendor Selection**

```jinja2
{# templates/auto-router.j2 #}
{% if node.vendor == "cisco" %}
{% include "cisco/router.j2" %}
{% elif node.vendor == "juniper" %}
{% include "juniper/router.j2" %}
{% elif node.vendor == "arista" %}
{% include "arista/router.j2" %}
{% else %}
{# Error: Unsupported vendor #}
! ERROR: Vendor {{ node.vendor }} not supported
{% endif %}
```

**[18:00-22:00] Demonstration and Testing**

```bash
# Generate configurations for different vendors
unet config generate cisco-router-01 --template cisco/router
unet config generate juniper-router-01 --template juniper/router

# Show the different outputs
echo "=== Cisco Configuration ==="
cat nodes/cisco-router-01.cfg

echo "=== Juniper Configuration ==="
cat nodes/juniper-router-01.cfg

# Use auto-selection template
unet config generate cisco-router-01 --template auto-router
unet config generate juniper-router-01 --template auto-router
```

---

## Advanced Topics Series

### Video 7: "Git Integration and Change Management" (18 minutes)

**Learning Objectives:**

- Integrate μNet with Git workflows
- Implement change approval processes
- Track configuration history

**Script Outline:**

**[0:00-2:00] Why Version Control Matters**

```
"Every change to your network should be tracked, reviewed, and reversible. 
Git integration in μNet makes this automatic and effortless."
```

**[2:00-6:00] Git Setup and Integration**

```bash
# Initialize git repository
git init
git remote add origin git@github.com:company/network-configs.git

# Configure μNet for git integration
cat >> unet.toml << 'EOF'
[git]
auto_commit = true
remote_url = "git@github.com:company/network-configs.git"
commit_message_template = "μNet: {{ action }} {{ node_name }}"
EOF

# Make first commit
git add .
git commit -m "Initial μNet workspace setup"
git push -u origin main
```

**[6:00-10:00] Change Workflow Demonstration**

```bash
# Create a configuration change
unet nodes add \
  --name "new-router" \
  --type "router" \
  --vendor "cisco"

# Generate configuration
unet config generate new-router --template cisco/router

# Show git status
git status

# Commit the change
git add .
git commit -m "Add new-router configuration"

# Show git log
git log --oneline
```

**[10:00-14:00] Change Management API**

```bash
# Create a change proposal
unet changes create \
  --title "Update NTP servers" \
  --description "Changing from public to internal NTP servers" \
  --nodes "border-router-01,core-switch-01"

# List pending changes
unet changes list --status pending

# Approve change (in real workflow, different person)
unet changes approve --id 1 --reviewer "network-admin"

# Deploy approved change
unet changes deploy --id 1
```

**[14:00-18:00] Rollback and Recovery**

```bash
# Show deployment history
unet deployments list

# Demonstrate rollback
unet deployments rollback --deployment-id 123

# Show git history for specific device
git log --follow nodes/border-router-01.cfg

# Compare configurations across time
git diff HEAD~1:nodes/border-router-01.cfg nodes/border-router-01.cfg
```

---

### Video 8: "Automation and CI/CD Integration" (25 minutes)

**Learning Objectives:**

- Integrate μNet with CI/CD pipelines
- Automate configuration validation
- Implement deployment automation

**Script Outline:**

**[0:00-3:00] The Power of Automation**

```
"Manual configuration deployment doesn't scale. In this tutorial, you'll see how to automate 
your entire configuration workflow from development to production."
```

**[3:00-8:00] CI/CD Pipeline Setup**

```yaml
# .github/workflows/unet-ci.yml
name: μNet Configuration CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Install μNet
      run: |
        curl -L https://github.com/your-org/unet/releases/latest/download/unet-linux-x86_64.tar.gz | tar xz
        sudo mv unet /usr/local/bin/
    
    - name: Validate Templates
      run: |
        for template in templates/**/*.j2; do
          unet templates validate "$template"
        done
    
    - name: Generate Test Configurations
      run: |
        unet config generate --all --output-dir test-configs/
    
    - name: Run Policy Checks
      run: |
        unet policies check --all --config-dir test-configs/
    
    - name: Upload Configurations
      uses: actions/upload-artifact@v3
      with:
        name: generated-configs
        path: test-configs/
```

**[8:00-15:00] Automated Testing**

```bash
# Create test suite
mkdir -p tests/

cat > tests/template-tests.yaml << 'EOF'
tests:
  - name: "Basic router template"
    template: "cisco/router.j2"
    data:
      node:
        name: "test-router"
        management_ip: "192.168.1.1"
    expected_contains:
      - "hostname test-router"
      - "ip address 192.168.1.1"
    expected_not_contains:
      - "undefined"

  - name: "Switch VLAN configuration"
    template: "cisco/switch.j2"
    data:
      node:
        name: "test-switch"
        vlans:
          - id: 10
            name: "USERS"
    expected_contains:
      - "vlan 10"
      - "name USERS"
EOF

# Run tests
unet templates test-suite tests/template-tests.yaml
```

**[15:00-20:00] Deployment Pipeline**

```bash
# Create deployment script
cat > scripts/deploy.sh << 'EOF'
#!/bin/bash
set -e

ENVIRONMENT=${1:-staging}
DRY_RUN=${2:-true}

echo "Deploying to $ENVIRONMENT environment..."

# Generate configurations
unet config generate --all --environment "$ENVIRONMENT"

# Validate configurations
unet policies check --all --policy-set "critical,standard"

# Create deployment plan
unet deployment plan --output "deploy-plan-$ENVIRONMENT.yaml"

# Execute deployment
if [ "$DRY_RUN" = "false" ]; then
    unet deployment execute --plan "deploy-plan-$ENVIRONMENT.yaml"
else
    echo "Dry run completed. Use 'false' as second argument to deploy."
fi
EOF

chmod +x scripts/deploy.sh

# Test deployment
./scripts/deploy.sh staging true
```

**[20:00-25:00] Monitoring and Alerting Integration**

```bash
# Add webhook notifications to deployment
cat >> unet.toml << 'EOF'
[notifications]
webhook_url = "https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK"
notify_on = ["deployment_start", "deployment_complete", "deployment_failed"]
EOF

# Show integration with monitoring
unet deployment execute \
  --plan deploy-plan-production.yaml \
  --notify-webhook https://monitoring.company.com/webhooks/unet
```

---

## Real-world Scenarios Series

### Video 9: "Branch Office Standardization" (20 minutes)

**Learning Objectives:**

- Deploy consistent configurations across multiple sites
- Handle site-specific variations
- Manage large-scale deployments

**Script Outline:**

**[0:00-2:00] The Challenge**

```
"You have 50 branch offices, each slightly different, but all needing consistent security and management. 
Let's see how μNet makes this manageable."
```

**[2:00-8:00] Creating the Branch Template**

```jinja2
{# templates/branch/router.j2 #}
hostname {{ node.name }}
!
{# Standard security configuration #}
enable secret {{ secrets.enable_password }}
service password-encryption
no ip http server
!
{# Site-specific WAN configuration #}
interface {{ branch.wan_interface }}
 description WAN to {{ branch.isp_name }}
 ip address dhcp
 no shutdown
!
{# Site-specific LAN configuration #}  
interface {{ branch.lan_interface }}
 description LAN Interface
 ip address {{ branch.lan_network | ipaddr('1') }} {{ branch.lan_network | ipaddr('netmask') }}
 no shutdown
!
{# Standard DHCP configuration #}
ip dhcp excluded-address {{ branch.lan_network | ipaddr('1') }} {{ branch.lan_network | ipaddr('10') }}
ip dhcp pool BRANCH-LAN
 network {{ branch.lan_network | ipaddr('network') }} {{ branch.lan_network | ipaddr('netmask') }}
 default-router {{ branch.lan_network | ipaddr('1') }}
 dns-server {{ config.dns_servers | join(' ') }}
!
{# Standard routing #}
ip route 0.0.0.0 0.0.0.0 dhcp
!
{# Standard management #}
ntp server {{ config.ntp_server }}
logging host {{ config.syslog_server }}
snmp-server community {{ secrets.snmp_community }} RO
```

**[8:00-12:00] Site-Specific Data Management**

```bash
# Create sites directory
mkdir -p sites/

# Create individual site files
cat > sites/seattle.yaml << 'EOF'
name: "branch-seattle-01"
location: "Seattle Branch Office"
branch:
  wan_interface: "GigabitEthernet0/0"
  lan_interface: "GigabitEthernet0/1"
  lan_network: "10.100.1.0/24"
  isp_name: "Local ISP Seattle"
EOF

cat > sites/portland.yaml << 'EOF'
name: "branch-portland-01"
location: "Portland Branch Office"  
branch:
  wan_interface: "GigabitEthernet0/0"
  lan_interface: "GigabitEthernet0/1"
  lan_network: "10.101.1.0/24"
  isp_name: "Local ISP Portland"
EOF

# Bulk add sites
for site in sites/*.yaml; do
    unet nodes import "$site"
done
```

**[12:00-17:00] Mass Configuration Generation**

```bash
# Generate configurations for all branch offices
unet config generate --all --template branch/router --filter "location contains 'Branch'"

# Show generated configurations
ls nodes/branch-*.cfg

# Validate all branch configurations
unet policies check --all --filter "location contains 'Branch'" --policy-set branch-standards

# Create deployment groups
unet deployment group create \
  --name "branch-offices" \
  --filter "location contains 'Branch'" \
  --rollout-strategy "canary" \
  --canary-percentage 10
```

**[17:00-20:00] Staged Deployment**

```bash
# Deploy to canary sites first
unet deployment execute \
  --group branch-offices \
  --stage canary \
  --max-parallel 2

# Monitor canary deployment
unet deployment status --group branch-offices --stage canary

# If successful, deploy to remaining sites
unet deployment execute \
  --group branch-offices \
  --stage production \
  --max-parallel 5
```

---

### Video 10: "Data Center Migration" (22 minutes)

**Learning Objectives:**

- Plan and execute complex network migrations
- Handle configuration dependencies
- Implement rollback procedures

**Script Outline:**

**[0:00-3:00] Migration Overview**

```
"Data center migrations are complex, high-risk operations. μNet helps you plan, execute, 
and verify migrations with confidence. Let's walk through a complete DC migration."
```

**[3:00-8:00] Migration Planning**

```bash
# Create migration project
mkdir -p migrations/dc-east-to-west/

# Define migration phases
cat > migrations/dc-east-to-west/phases.yaml << 'EOF'
migration:
  name: "DC East to West Migration"
  phases:
    1:
      name: "Preparation"
      description: "Add new equipment, establish connectivity"
      duration: "2 weeks"
    2: 
      name: "Parallel Operation"
      description: "Run both DCs in parallel"
      duration: "1 week"
    3:
      name: "Traffic Migration"
      description: "Move traffic to new DC"
      duration: "3 days"
    4:
      name: "Cleanup"
      description: "Decommission old equipment"
      duration: "1 week"
EOF

# Create phase-specific templates
cat > templates/migration/phase1-prep.j2 << 'EOF'
{# Phase 1: Add new DC connectivity #}
hostname {{ node.name }}
!
{# Existing configuration maintained #}
{% include node.vendor + "/" + node.type + ".j2" %}
!
{# Add migration-specific configuration #}
{% if migration.phase == 1 %}
! Migration Phase 1: Preparation
interface {{ migration.new_dc_interface }}
 description Connection to DC-West
 ip address {{ migration.new_dc_ip }} {{ migration.new_dc_netmask }}
 no shutdown
!
router ospf 1
 network {{ migration.new_dc_ip | ipaddr('network') }} {{ migration.new_dc_netmask | netmask_to_wildcard }} area {{ migration.new_dc_area }}
!
{% endif %}
EOF
```

**[8:00-15:00] Migration Execution**

```bash
# Phase 1: Add new connectivity
for device in core-router-01 core-router-02; do
    unet nodes update "$device" --custom-data '{
        "migration": {
            "phase": 1,
            "new_dc_interface": "TenGigabitEthernet0/1",
            "new_dc_ip": "10.200.1.1",
            "new_dc_netmask": "255.255.255.252",
            "new_dc_area": 1
        }
    }'
done

# Generate phase 1 configurations
unet config generate --all --template migration/phase1-prep

# Validate migration configurations
unet policies check --all --policy-set migration-phase1

# Deploy phase 1
unet deployment execute --filter "migration.phase == 1" --max-parallel 1

# Verify connectivity
unet verification run --test-suite migration-connectivity
```

**[15:00-20:00] Traffic Migration and Monitoring**

```bash
# Phase 3: Traffic migration
cat > templates/migration/phase3-traffic.j2 << 'EOF'
{# Phase 3: Redirect traffic #}
{% if migration.phase == 3 %}
! Migration Phase 3: Traffic Migration
! Adjust OSPF costs to prefer new DC
interface {{ migration.old_dc_interface }}
 ip ospf cost 1000
!
interface {{ migration.new_dc_interface }}
 ip ospf cost 10
!
! Update static routes if needed
{% for route in migration.traffic_routes %}
ip route {{ route.network }} {{ route.netmask }} {{ route.new_next_hop }}
{% endfor %}
{% endif %}
EOF

# Execute traffic migration
unet deployment execute --filter "migration.phase == 3" --verify-connectivity

# Monitor traffic flow
unet monitoring dashboard --migration-view
```

**[20:00-22:00] Rollback Procedures**

```bash
# Emergency rollback procedure
cat > scripts/migration-rollback.sh << 'EOF'
#!/bin/bash
echo "EMERGENCY MIGRATION ROLLBACK"
echo "Reverting to pre-migration configurations..."

# Restore from backup
unet deployment rollback --to-backup migration-pre-phase3

# Verify rollback success
unet verification run --test-suite migration-rollback-verify

echo "Rollback completed. Check network status."
EOF

chmod +x scripts/migration-rollback.sh

# Test rollback procedure (dry run)
./scripts/migration-rollback.sh --dry-run
```

---

## Production Guidelines

### Recording Best Practices

**Video Quality:**

- **Resolution:** 1920x1080 minimum, 4K if possible
- **Frame Rate:** 30fps for screen recording
- **Audio Quality:** 48kHz, noise-free environment
- **Screen Layout:** Use consistent terminal/editor themes

**Content Guidelines:**

- **Pace:** Slow enough to follow, fast enough to maintain interest
- **Explanations:** Explain what you're doing before doing it
- **Mistakes:** Show common mistakes and how to fix them
- **Pauses:** Add pauses after complex commands for comprehension

### Equipment Recommendations

**Software:**

- **Screen Recording:** OBS Studio (free), Camtasia (paid)
- **Audio Editing:** Audacity (free), Adobe Audition (paid)
- **Video Editing:** DaVinci Resolve (free), Final Cut Pro (paid)

**Hardware:**

- **Microphone:** Audio-Technica ATR2100x-USB or similar
- **Computer:** Sufficient RAM for smooth recording (16GB+)
- **Storage:** SSD with plenty of space for large video files

### Publishing and Distribution

**Platforms:**

- **YouTube:** Primary platform for public tutorials
- **Vimeo:** Higher quality option for professional content
- **Company LMS:** Internal training systems
- **Documentation Site:** Embedded in μNet documentation

**SEO Optimization:**

- **Titles:** Include "μNet", "Network Configuration", specific topics
- **Descriptions:** Detailed descriptions with timestamps
- **Tags:** Network automation, infrastructure as code, DevOps
- **Thumbnails:** Consistent branding with μNet logo

### Keeping Content Current

**Update Schedule:**

- **Review quarterly** for accuracy and relevance
- **Re-record** when major features change
- **Add supplementary videos** for new features
- **Maintain playlist organization** for logical learning progression

**Version Management:**

- **Tag videos** with μNet version compatibility
- **Create version-specific playlists** when necessary
- **Add update notices** to older videos when content changes

---

This comprehensive guide provides everything needed to create professional video tutorials for μNet. Each script can be adapted based on your specific environment and audience needs.
