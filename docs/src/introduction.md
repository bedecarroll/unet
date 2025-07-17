# Introduction to μNet

> **μNet** (pronounced "micro-net") is a network configuration management system that helps you document, monitor, and automate your network infrastructure.

---

## What Problems Does μNet Solve?

### 📋 **Network Documentation Chaos**

- **Problem**: Network topology scattered across spreadsheets, diagrams, and tribal knowledge
- **Solution**: Centralized device inventory with relationships, locations, and lifecycle tracking

### 🔍 **Configuration Drift**  

- **Problem**: Devices gradually drift from standards without detection
- **Solution**: Declarative policies that continuously check compliance

### 🔧 **Manual Operations**

- **Problem**: Repetitive configuration tasks prone to human error
- **Solution**: Automated actions triggered by policy rules

### 📊 **Lack of Visibility**

- **Problem**: Limited insight into device status and performance
- **Solution**: Real-time SNMP monitoring with historical tracking

---

## Core Concepts

### Devices & Topology

μNet models your network as **Nodes** (devices), **Links** (connections), and **Locations** (physical sites):

```text
Campus
├── Building A
│   ├── Floor 1
│   │   └── Rack 1
│   │       ├── core-01 (Core Router)
│   │       └── dist-01 (Distribution Switch)
│   └── Floor 2
└── Building B
```

### Policy-Driven Compliance

Define network standards as declarative rules:

```rules
# Ensure production devices are monitored
WHEN node.lifecycle == "Production" 
THEN SET custom_data.monitoring_enabled TO true

# Cisco devices should use SNMPv3
WHEN node.vendor == "Cisco"
THEN ASSERT custom_data.snmp_version == "v3"
```

### Real-Time Monitoring

μNet polls your devices via SNMP to collect:

- System information (uptime, description, contact)
- Interface status and statistics  
- Performance metrics (CPU, memory, temperature)
- Custom vendor-specific data

---

## Key Features

### ✅ **What μNet Does Today (v0.1.0)**

- **Complete Device Management**: Add, update, and organize network devices
- **Topology Mapping**: Document connections and physical locations
- **Policy Engine**: Write and evaluate compliance rules
- **SNMP Monitoring**: Real-time device status collection
- **REST API**: Full HTTP API for automation
- **Rich CLI**: Command-line interface with multiple output formats
- **Data Import/Export**: Bulk operations for network data

### 🚧 **Coming Soon**

- **Configuration Templates**: Generate device configs from templates
- **Historical Analytics**: Time-series data and trend analysis  
- **Configuration Push**: Deploy generated configs to devices
- **Advanced Alerting**: Notifications for policy violations

---

## Architecture Overview

μNet consists of three main components:

```text
┌─────────────┐    HTTP/JSON    ┌─────────────┐
│  unet-cli   │ ◄──────────────► │ unet-server │
│             │                 │             │
│ • Commands  │                 │ • REST API  │
│ • Local DB  │                 │ • SNMP Poll │
│ • Policies  │                 │ • Policies  │
└─────────────┘                 └─────────────┘
       │                               │
       ▼                               ▼
┌─────────────────────────────────────────────┐
│              SQLite Database                │
│ • Devices  • Links  • Locations • Status   │
└─────────────────────────────────────────────┘
```

**Design Principles:**

- **Single Binary**: No external dependencies, easy deployment
- **SQLite Backend**: Zero-ops database for up to 10K+ devices
- **API-First**: Everything available via HTTP API
- **Incremental Adoption**: Start small, grow capabilities over time

---

## Use Cases

### Network Documentation

- **Device Inventory**: Comprehensive database of all network equipment
- **Topology Mapping**: Visual representation of network connections
- **Lifecycle Tracking**: Manage devices from planning to decommission
- **Change Auditing**: Track modifications and policy compliance

### Compliance Monitoring

- **Security Standards**: Ensure devices meet security baselines
- **Configuration Validation**: Check for required settings and features
- **Vendor Consistency**: Enforce consistent configurations by vendor
- **Lifecycle Policies**: Different rules for production vs. staging

### Operational Automation  

- **Bulk Updates**: Apply changes across multiple devices
- **Automated Validation**: Continuous compliance checking
- **Integration Workflows**: Connect with existing automation tools
- **Reporting**: Generate compliance and inventory reports

---

## Getting Started

**Ready to try μNet?**

1. **[Quick Start Guide](quick_start.md)** - Get running in 10 minutes
2. **[CLI Reference](cli_reference.md)** - Learn the commands
3. **[Policy Guide](policy_guide.md)** - Write your first compliance rules

**Questions?** Check the [Troubleshooting Guide](troubleshooting.md) or [open an issue](https://github.com/bedecarroll/unet/issues).

---

## Why Choose μNet?

### Built for Network Engineers

- **Domain-specific**: Designed for network operations, not generic IT
- **Practical**: Solves real daily challenges network teams face
- **Flexible**: Adapt to your existing processes and tools

### Modern Technology

- **Rust Performance**: Fast, memory-safe, reliable
- **Simple Deployment**: Single binary, SQLite database
- **API-First**: Easy integration with existing tools

### Open Source

- **MIT/Apache License**: Use freely in commercial environments
- **Active Development**: Regular releases with new features
- **Community Driven**: Contributions and feedback welcome

*Next: [Quick Start Guide](quick_start.md) →*
