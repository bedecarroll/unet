# μNet Roadmap

> **Current Status:** ~60% complete. See `/STATUS.md` for detailed development status and immediate next steps.

This document outlines the planned features and release timeline for μNet.

## Current Status (v0.1.0)

μNet has CLI, API, policy engine, and SNMP infrastructure implemented. Current development focuses on completing API endpoints and background task integration. See `/STATUS.md` for detailed completion status.

## Planned Features

### **Milestone 4: Template Engine**

- **Configuration Templates**: Jinja2-compatible templating system
- **Partial Configuration Management**: Template-match headers for selective config management
- **Intelligent Diffing**: Hierarchical configuration comparison
- **CLI Commands**:
  - `unet template render --node <id> --template <file>`
  - `unet template diff --node <id> --template <file>`
  - `unet template validate --path <dir>`

### **Milestone 5: Advanced SNMP CLI**

- **SNMP Polling Controls**: CLI commands to manage background polling
- **Device Discovery**: Automatic capability detection
- **Historical Metrics**: Time-series data storage and retrieval
- **CLI Commands** (planned):
  - `unet nodes polling start/stop/status`
  - `unet nodes history --metrics --since 1d`
  - `unet nodes compare <node1> <node2>`

### **Milestone 6: Configuration Push**

- **Safe Configuration Deployment**: Push generated configs to devices
- **Rollback Mechanisms**: Automatic recovery from failed changes
- **Change Management**: Audit trails and approval workflows

### **Future Enhancements**

#### **Enhanced CLI Experience**

- `unet init --with-examples` - Quick setup with sample data
- `unet locations tree` - Hierarchical location display
- Advanced filtering and output formatting
- Interactive mode for guided operations

#### **Web UI**

- Browser-based interface for non-CLI users
- Visual topology maps
- Policy authoring assistance
- Real-time monitoring dashboards

#### **Enterprise Features**

- **Authentication & Authorization**: Token-based access control
- **Multi-tenancy**: Support for multiple organizations
- **Integration APIs**: Webhooks and event streaming
- **High Availability**: Clustering and replication

#### **Advanced Policy Features**

- **Policy Libraries**: Pre-built compliance templates
- **Custom Actions**: Extensible action framework
- **Conditional Logic**: More sophisticated rule evaluation
- **Policy Testing**: Dry-run and simulation capabilities

#### **Network Discovery**

- **Topology Auto-Discovery**: LLDP/CDP-based link detection
- **Device Fingerprinting**: Automatic vendor/model identification
- **Network Mapping**: Visual topology generation

## Release Sequence

- **v0.2.0**: Template Engine + Configuration Generation
- **v0.3.0**: Advanced SNMP Monitoring + CLI
- **v0.4.0**: Configuration Push + Rollback
- **v1.0.0**: Web UI

## Contributing to the Roadmap

We welcome input on prioritization and feature requests:

- **Feature Requests**: [GitHub Issues](https://github.com/bedecarroll/unet/issues) with `enhancement` label
- **Architecture Discussions**: [GitHub Discussions](https://github.com/bedecarroll/unet/discussions)
- **Implementation Help**: Check `good first issue` labels on planned features

## Design Principles for Future Features

All new features will maintain μNet's core principles:

- **Progressive Adoption**: You can adopt new capabilities gradually
- **Safety First**: Always show what changes before applying them
- **Single Binary**: Minimal deployment complexity
- **Multi-vendor**: Work consistently across network vendors
- **Git Integration**: Version control for all configuration artifacts
