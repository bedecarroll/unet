# μNet (Micro Net)

[![Build Status](https://github.com/bedecarroll/unet/workflows/CI/badge.svg)](https://github.com/bedecarroll/unet/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)

**Network inventory and compliance management that doesn't suck.**

Stop tracking network devices in spreadsheets and fighting with brittle
scripts. μNet provides a clean, Git-friendly foundation for managing network
infrastructure data with policy-driven compliance checking.

```bash
# Manage your network inventory with confidence
unet nodes add --name core-sw-01 --vendor cisco --model catalyst-9300 --mgmt-ip 192.168.1.10

# Define compliance policies in plain English
echo 'WHEN vendor == "cisco" AND model CONTAINS "catalyst" 
      THEN ASSERT software_version >= "16.12.04"' > policies/cisco-firmware.rules

# Check compliance across your network  
unet policy eval --path policies/cisco-firmware.rules
```

---

## Table of Contents

- [Why μNet?](#why-μnet)
- [What Works Today](#what-works-today)
- [Quick Start](#quick-start)
- [Roadmap](#roadmap)
- [Documentation](#documentation)
- [Contributing](#contributing)

---

## Why μNet?

Network engineers are drowning in manual processes and fragile tooling:

### The Problems

- **Spreadsheet Hell**: Network inventory scattered across Excel files,
SharePoint, and people's heads
- **Configuration Drift**: No systematic way to check if devices comply with standards
- **Manual Auditing**: Quarterly "compliance projects" that are outdated before
they finish
- **Poor Data Quality**: Missing information, stale data, no single source of truth

### The μNet Approach

- **Structured Data**: Proper database with relationships, not flat files
- **Policy as Code**: Define compliance rules that run automatically
- **Git Integration**: Version control your policies like any other code
- **Progressive Adoption**: Start with inventory, add compliance checking when ready

---

## What Works Today

μNet v0.1 provides a solid foundation for network data management:

### 🗄️ **Network Inventory Management**

Store and manage devices, locations, and links with full CRUD operations.

```bash
# Add network infrastructure
unet locations add --name "Main Building" --address "123 Network St"
unet nodes add --name core-sw-01 --vendor cisco --model catalyst-9300 \
    --mgmt-ip 192.168.1.10 --location "Main Building"
unet links add --name "core-to-access" --node-a core-sw-01 --interface-a "GigE1/0/1" \
    --node-z access-sw-01 --interface-z "GigE0/24"

# Query your infrastructure
unet nodes list --vendor cisco --lifecycle active
unet nodes show <node-id> --include-status
```

### 📋 **Policy-Driven Compliance**

Write rules in a simple DSL to check compliance across your network.

```bash
# Create a policy file
cat > policies/firmware-check.rules << 'EOF'
WHEN vendor == "cisco" AND model CONTAINS "catalyst"
THEN ASSERT software_version >= "16.12.04"

WHEN vendor == "juniper" AND model CONTAINS "ex"
THEN ASSERT software_version >= "18.4R1"
EOF

# Validate and run policies
unet policy validate --path policies/
unet policy eval --path policies/firmware-check.rules
```

### 📊 **Data Import/Export**

Migrate from existing tools and integrate with other systems.

```bash
# Export current state to JSON
unet export --to ./network-backup/ --format json

# Import network topology from structured data
unet import --from ./network-data/
# Expects: locations.json, nodes.json, links.json
```

### 🔧 **REST API**

Full HTTP API for integrations and custom tooling.

```bash
# All CLI operations available via API
curl http://localhost:8080/api/nodes
curl -X POST http://localhost:8080/api/locations -H "Content-Type: application/json" \
  -d '{"name": "DR Site", "location_type": "datacenter"}'
```

---

## Quick Start

### Prerequisites

- Rust 1.85+ ([install rustup](https://rustup.rs/))
- Git

### Installation

```bash
# Clone and build
git clone https://github.com/bedecarroll/unet.git
cd unet
cargo build --release
```

### Try it out

```bash
# 1. Start with example data
cp -r docs/static/examples/small-office/ ./network-data/
./target/release/unet import --from ./network-data/

# 2. Explore what was imported
./target/release/unet locations list
./target/release/unet nodes list --output table

# 3. Create a simple policy
echo 'WHEN lifecycle == "live" THEN ASSERT mgmt_ip IS NOT NULL' > check-mgmt.rules

# 4. Run compliance check
./target/release/unet policy eval --path check-mgmt.rules
```

You'll see a realistic small office network with switches, firewalls, and a server.

---

## Roadmap

μNet is designed for progressive enhancement. Today's solid foundation supports
upcoming features:

### **Coming Soon** (see [full roadmap](docs/src/roadmap.md))

- **Template Engine** (v0.2): Configuration generation with Jinja2-compatible templates
- **SNMP Integration** (v0.3): Automated device discovery and monitoring via
CLI  
- **Configuration Push** (v0.4): Safe deployment of generated configs with rollback

### **Current Limitations**

- **Read-only**: μNet manages data about your network, not device
configurations (yet)
- **No templating**: Configuration generation planned for next major release
- **Basic policies**: Simple compliance checking, more advanced logic coming

---

## Project Status

μNet v0.1 is **production-ready** for network inventory and compliance checking:

- ✅ **Core Data Model**: Stable schema with migrations
- ✅ **Policy Engine**: Full DSL with WHEN/THEN logic
- ✅ **CLI Tools**: Complete CRUD operations for all entities
- ✅ **REST API**: Full HTTP interface with OpenAPI docs
- ✅ **Import/Export**: JSON-based data interchange
- ✅ **SQLite Backend**: ACID transactions, concurrent safe

---

## Documentation

📖 **[Complete Documentation](docs/src/index.md)** - Start here for guides and references

**Quick Links:**

- **[Quick Start Guide](docs/src/quick_start.md)** - Get μNet running in 10 minutes
- **[CLI Reference](docs/src/cli_reference.md)** - Complete command
documentation
- **[Policy Guide](docs/src/policy_guide.md)** - Write compliance rules and automation
- **[API Reference](docs/src/api_reference.md)** - HTTP API for integrations
- **[Troubleshooting](docs/src/troubleshooting.md)** - Common issues and solutions

**Planning & Architecture:**

- **[What is μNet?](docs/src/introduction.md)** - Detailed introduction and use cases
- **[Architecture](docs/src/architecture.md)** - How μNet works under the hood
- **[Roadmap](docs/src/roadmap.md)** - Current features and future plans

---

## Contributing

We welcome contributions! μNet is built by network engineers, for network engineers.

### Getting Started

1. Read the [Architecture Guide](docs/src/architecture.md)
2. Check out [Good First Issues](https://github.com/bedecarroll/unet/labels/good%20first%20issue)
3. Join discussions in [GitHub Discussions](https://github.com/bedecarroll/unet/discussions)

### Development Setup

```bash
git clone https://github.com/bedecarroll/unet.git
cd unet
cargo test --workspace  # Make sure everything works
```

### Areas We Need Help

- **Multi-vendor examples** (Arista, F5, Palo Alto device data)
- **Policy libraries** for common compliance scenarios
- **Import utilities** for existing network documentation
- **Integration examples** (Ansible, Terraform, etc.)

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

---

## Support

- 📖 **Documentation**: [https://bedecarroll.github.io/unet](https://bedecarroll.github.io/unet)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/bedecarroll/unet/discussions)
- 🐛 **Bug Reports**: [GitHub Issues](https://github.com/bedecarroll/unet/issues)

## License

Licensed under the [MIT License](LICENSE).
