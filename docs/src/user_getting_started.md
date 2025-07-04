<!-- SPDX-License-Identifier: MIT -->

# Getting Started with μNet

> **Audience:** Network engineers and administrators who want to start using μNet for configuration management.  
> **Objective:** Get you up and running with μNet in 15 minutes, from installation to managing your first network device.

---

## Table of Contents

1. [What is μNet?](#what-is-μnet)
2. [Installation](#installation)
3. [First Steps](#first-steps)
4. [Managing Your First Device](#managing-your-first-device)
5. [Basic Concepts](#basic-concepts)
6. [Next Steps](#next-steps)

---

## What is μNet?

μNet is a modern network configuration management tool that helps you:

- **Centralize** network device configurations
- **Template** common configuration patterns
- **Validate** configurations against policies
- **Track** configuration changes over time
- **Diff** configurations to see what changed

### Key Benefits

- **Single binary** - No complex dependencies or installation procedures
- **Git integration** - Version control for your network configurations
- **Policy-driven** - Define rules once, apply everywhere
- **Template-based** - Reuse configuration patterns across devices
- **Vendor agnostic** - Works with any text-based configuration format

---

## Installation

### Option 1: Download Pre-built Binary (Recommended)

```bash
# Download the latest release
curl -L https://github.com/your-org/unet/releases/latest/download/unet-linux-x86_64.tar.gz | tar xz

# Move to PATH
sudo mv unet /usr/local/bin/

# Verify installation
unet --version
```

### Option 2: Install via Package Manager

**Ubuntu/Debian:**

```bash
curl -fsSL https://packages.unet.dev/gpg | sudo apt-key add -
echo "deb https://packages.unet.dev/apt stable main" | sudo tee /etc/apt/sources.list.d/unet.list
sudo apt update && sudo apt install unet
```

**CentOS/RHEL:**

```bash
curl -fsSL https://packages.unet.dev/rpm/unet.repo | sudo tee /etc/yum.repos.d/unet.repo
sudo yum install unet
```

**macOS (Homebrew):**

```bash
brew install unet/tap/unet
```

### Option 3: Build from Source

```bash
# Prerequisites: Rust 1.85+
git clone https://github.com/your-org/unet.git
cd unet
cargo build --release
sudo cp target/release/unet /usr/local/bin/
```

---

## First Steps

### 1. Initialize Your Workspace

Create a directory for your network configurations:

```bash
mkdir my-network
cd my-network
unet init
```

This creates:

```
my-network/
├── unet.toml          # Main configuration file
├── nodes/             # Device configurations
├── policies/          # Validation rules
├── templates/         # Configuration templates
└── .git/              # Git repository (auto-initialized)
```

### 2. Configure μNet

Edit `unet.toml` to match your environment:

```toml
[database]
url = "sqlite:///unet.db"

[server]
host = "0.0.0.0"
port = 8080

[git]
auto_commit = true
remote_url = "git@github.com:your-org/network-configs.git"

[authentication]
enabled = false  # Start simple, enable later
```

### 3. Start the Server (Optional)

For team collaboration, start the μNet server:

```bash
unet server start
```

The web interface will be available at `http://localhost:8080`

---

## Managing Your First Device

### 1. Add a Network Device

```bash
# Add a router
unet nodes add \
  --name "border-router-01" \
  --type "router" \
  --vendor "cisco" \
  --management-ip "192.168.1.1" \
  --location "datacenter-east"
```

### 2. Create a Configuration Template

Create `templates/cisco-router-base.j2`:

```jinja2
hostname {{ node.name }}

interface GigabitEthernet0/0
 description Management Interface
 ip address {{ node.management_ip }} 255.255.255.0
 no shutdown

{% if node.location == "datacenter-east" %}
ntp server 10.1.1.100
{% else %}
ntp server 10.2.1.100
{% endif %}

banner motd ^
Managed by μNet - Do not modify manually
Contact: network-team@company.com
^
```

### 3. Generate Configuration

```bash
# Generate config for your device
unet config generate border-router-01 --template cisco-router-base

# Save to file
unet config generate border-router-01 --template cisco-router-base --output nodes/border-router-01.cfg
```

### 4. Create a Validation Policy

Create `policies/cisco-security.rules`:

```
# Ensure management interfaces have descriptions
assert node.type == "router" -> config contains "description Management"

# Require NTP configuration
assert config contains "ntp server"

# Ensure banner is configured
assert config contains "banner motd"

# Security: Ensure console timeout
assert config contains "exec-timeout"
```

### 5. Validate Configuration

```bash
# Check if configuration follows policies
unet config validate border-router-01

# Output:
# ✓ Management interface description check passed
# ✓ NTP server configuration found
# ✓ Banner configuration found
# ✗ Console timeout not configured
```

### 6. Compare Configurations

```bash
# After making changes, see what's different
unet config diff border-router-01

# Compare against another device
unet config diff border-router-01 border-router-02
```

---

## Basic Concepts

### Nodes

**Nodes** represent network devices in your infrastructure. Each node has:

- **Name**: Unique identifier
- **Type**: Device category (router, switch, firewall)
- **Vendor**: Equipment manufacturer
- **Attributes**: IP addresses, location, custom metadata

### Templates

**Templates** are reusable configuration patterns using Jinja2 syntax. They can:

- Reference node attributes: `{{ node.name }}`
- Include conditional logic: `{% if node.type == "router" %}`
- Use loops: `{% for vlan in node.vlans %}`
- Import other templates: `{% include "common/snmp.j2" %}`

### Policies

**Policies** are validation rules written in a simple domain-specific language:

- Check configuration content: `assert config contains "ssh"`
- Validate node attributes: `assert node.management_ip is not null`
- Conditional rules: `assert node.type == "firewall" -> config contains "access-list"`

### Configurations

**Configurations** are the generated text files that get deployed to devices. They're created by applying templates to node data and validated against policies.

---

## Next Steps

Now that you have μNet running, explore these guides:

1. **[Configuration Management Tutorial](user_config_tutorial.md)** - Learn advanced templating techniques
2. **[Policy Creation Guide](user_policy_guide.md)** - Write comprehensive validation rules  
3. **[Template Usage Tutorial](user_template_tutorial.md)** - Master the template system
4. **[Example Configurations](user_examples.md)** - Real-world configuration examples

### Common Workflows

**Daily Operations:**

```bash
# Check what's changed
unet status

# Validate all configurations
unet config validate --all

# Generate configs for deployment
unet config generate --all --output-dir deploy/
```

**Team Collaboration:**

```bash
# Pull latest changes
unet git pull

# Push your changes
unet git push

# Review changes before applying
unet config diff --staged
```

### Getting Help

- **CLI Help**: `unet --help` or `unet <command> --help`
- **Documentation**: All guides are available in this documentation
- **Community**: Join our Discord or GitHub Discussions
- **Issues**: Report bugs on GitHub Issues

---

## Quick Reference

### Essential Commands

| Command | Description | Example |
|---------|-------------|---------|
| `unet init` | Initialize new workspace | `unet init` |
| `unet nodes add` | Add network device | `unet nodes add --name r1 --type router` |
| `unet config generate` | Generate configuration | `unet config generate r1 --template base` |
| `unet config validate` | Validate against policies | `unet config validate r1` |
| `unet config diff` | Compare configurations | `unet config diff r1 r2` |
| `unet server start` | Start web server | `unet server start` |

### File Structure

```
workspace/
├── unet.toml              # Main configuration
├── nodes/                 # Generated configurations
│   ├── router-01.cfg
│   └── switch-01.cfg
├── templates/             # Jinja2 templates
│   ├── cisco/
│   └── juniper/
├── policies/              # Validation rules
│   ├── security.rules
│   └── compliance.rules
└── .git/                  # Version control
```

Welcome to μNet! You're now ready to modernize your network configuration management.
