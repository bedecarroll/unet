# CLI Reference – μNet Command Line Tool

> **Audience:** Users and operators working with μNet daily  
> **Status:** Documents implemented commands only (v0.1.0)

---

## Overview

μNet provides a comprehensive command-line interface for network configuration management. The CLI supports local SQLite and remote (HTTP API) operation modes.

**Binary Name:** `unet`  
**Current Version:** 0.1.0  
**Source:** `crates/unet-cli/`

## Quick Start

```bash
# Local SQLite mode (default)
unet nodes list

# Remote server mode
unet --server http://localhost:8080 nodes list

# Import sample data (create your own JSON files)
unet import your-locations.json
unet import your-nodes.json
unet import your-links.json
```

---

## Global Options

| Option | Environment Variable | Default | Description |
|--------|---------------------|---------|-------------|
| `-c, --config <FILE>` | `UNET_CONFIG` | - | Configuration file path |
| `-d, --database-url <URL>` | `UNET_DATABASE_URL` | `sqlite://unet.db` | Database connection URL |
| `-s, --server <URL>` | `UNET_SERVER` | - | Server URL for remote operations |
| `-t, --token <TOKEN>` | `UNET_TOKEN` | - | Authentication token |
| `-f, --output <FORMAT>` | - | `table` | Output format: table, json, yaml |
| `-v, --verbose` | - | - | Enable verbose logging |

---

## Commands

### Node Management

#### `unet nodes add`

Create a new network device.

```bash
unet nodes add --name router-01 --domain example.com --vendor cisco --model ISR4431 --role router --lifecycle live
```

**Required Options:**

- `--name <NAME>` - Device hostname
- `--domain <DOMAIN>` - DNS domain
- `--vendor <VENDOR>` - Device vendor (cisco, juniper, arista, fortinet, paloalto, checkpoint, other)
- `--model <MODEL>` - Device model
- `--role <ROLE>` - Device role (router, switch, firewall, load-balancer, wireless-controller, other)
- `--lifecycle <STATE>` - Lifecycle state (planned, implementing, live, decommissioned)

**Optional Options:**

- `--management-ip <IP>` - Management IP address
- `--location-id <UUID>` - Location UUID
- `--custom-data <JSON>` - Additional data as JSON string

#### `unet nodes list`

List all nodes with optional filtering.

```bash
unet nodes list
unet nodes list --vendor cisco --role router
unet nodes list --lifecycle live --page 1 --per-page 20
```

**Options:**

- `--vendor <VENDOR>` - Filter by vendor
- `--role <ROLE>` - Filter by role
- `--lifecycle <STATE>` - Filter by lifecycle
- `--page <NUM>` - Page number (default: 1)
- `--per-page <NUM>` - Items per page (default: 50)

#### `unet nodes show`

Display detailed information about a specific node.

```bash
unet nodes show router-01
unet nodes show --include-status --show-interfaces router-01
```

**Arguments:**

- `<NODE_ID>` - Node name or UUID

**Options:**

- `--include-status` - Include node status from SNMP polling
- `--show-interfaces` - Show interface status
- `--show-system-info` - Show system information

#### `unet nodes update`

Update node properties.

```bash
unet nodes update router-01 --model ISR4451 --lifecycle live
unet nodes update router-01 --management-ip 192.168.1.1
```

**Arguments:**

- `<NODE_ID>` - Node name or UUID

**Options:**

- `--name <NAME>` - Update hostname
- `--domain <DOMAIN>` - Update domain
- `--vendor <VENDOR>` - Update vendor
- `--model <MODEL>` - Update model
- `--role <ROLE>` - Update role
- `--lifecycle <STATE>` - Update lifecycle
- `--management-ip <IP>` - Update management IP
- `--location-id <UUID>` - Update location
- `--custom-data <JSON>` - Update custom data

#### `unet nodes delete`

Remove a node from the system.

```bash
unet nodes delete router-01
unet nodes delete router-01 --yes  # Skip confirmation
```

**Arguments:**

- `<NODE_ID>` - Node name or UUID

**Options:**

- `--yes` - Skip confirmation prompt

#### `unet nodes status`

Show current status information for a node.

```bash
unet nodes status router-01
unet nodes status router-01 --show-interfaces --show-system-info
```

**Arguments:**

- `<NODE_ID>` - Node name or UUID

**Options:**

- `--show-interfaces` - Include interface status
- `--show-system-info` - Include system information

#### `unet nodes metrics`

Display current metrics for a node.

```bash
unet nodes metrics router-01
```

**Arguments:**

- `<NODE_ID>` - Node name or UUID

**Note:** Historical metrics are not yet implemented.

---

### Location Management

#### `unet locations add`

Create a new location.

```bash
unet locations add --name "Building A" --location-type building --address "123 Main St"
```

**Required Options:**

- `--name <NAME>` - Location name
- `--location-type <TYPE>` - Type: campus, building, floor, room, rack

**Optional Options:**

- `--parent-id <UUID>` - Parent location UUID
- `--address <ADDRESS>` - Physical address
- `--custom-data <JSON>` - Additional data as JSON

#### `unet locations list`

List all locations.

```bash
unet locations list
unet locations list --location-type building
```

**Options:**

- `--location-type <TYPE>` - Filter by location type
- `--page <NUM>` - Page number
- `--per-page <NUM>` - Items per page

#### `unet locations show`

Display detailed location information.

```bash
unet locations show "Building A"
```

**Arguments:**

- `<LOCATION_ID>` - Location name or UUID

#### `unet locations update`

Update location properties.

```bash
unet locations update "Building A" --address "456 New St"
```

**Arguments:**

- `<LOCATION_ID>` - Location name or UUID

**Options:**

- `--name <NAME>` - Update name
- `--location-type <TYPE>` - Update type
- `--parent-id <UUID>` - Update parent
- `--address <ADDRESS>` - Update address
- `--custom-data <JSON>` - Update custom data

#### `unet locations delete`

Remove a location.

```bash
unet locations delete "Building A"
```

**Arguments:**

- `<LOCATION_ID>` - Location name or UUID

**Options:**

- `--yes` - Skip confirmation prompt

---

### Link Management

#### `unet links add`

Create a connection between nodes.

```bash
# Point-to-point link
unet links add --node-a router-01 --interface-a GigE0/0/0 --node-z switch-01 --interface-z GigE1/0/1

# Internet circuit (no node-z)
unet links add --node-a router-01 --interface-a GigE0/0/1 --circuit-id "ISP-12345"
```

**Required Options:**

- `--node-a <NODE>` - First node name/UUID
- `--interface-a <INTERFACE>` - Interface on first node

**For Point-to-Point Links:**

- `--node-z <NODE>` - Second node name/UUID
- `--interface-z <INTERFACE>` - Interface on second node

**For Internet Circuits:**

- `--circuit-id <ID>` - Circuit identifier

**Optional Options:**

- `--bandwidth <BPS>` - Link bandwidth in bits per second
- `--custom-data <JSON>` - Additional data as JSON

#### `unet links list`

List all links.

```bash
unet links list
unet links list --node-a router-01
```

**Options:**

- `--node-a <NODE>` - Filter by first node
- `--node-z <NODE>` - Filter by second node
- `--page <NUM>` - Page number
- `--per-page <NUM>` - Items per page

#### `unet links show`

Display detailed link information.

```bash
unet links show <LINK_UUID>
```

**Arguments:**

- `<LINK_ID>` - Link UUID

#### `unet links update`

Update link properties.

```bash
unet links update <LINK_UUID> --bandwidth 10000000000
```

**Arguments:**

- `<LINK_ID>` - Link UUID

**Options:**

- `--bandwidth <BPS>` - Update bandwidth
- `--custom-data <JSON>` - Update custom data

#### `unet links delete`

Remove a link.

```bash
unet links delete <LINK_UUID>
```

**Arguments:**

- `<LINK_ID>` - Link UUID

**Options:**

- `--yes` - Skip confirmation prompt

---

### Policy Management

#### `unet policy validate`

Validate policy file syntax.

```bash
unet policy validate policies/compliance.rules
unet policy validate policies/ --verbose
```

**Arguments:**

- `<PATH>` - Policy file or directory path

**Options:**

- `--verbose` - Show detailed rule information

#### `unet policy eval`

Evaluate policies against nodes.

```bash
unet policy eval policies/compliance.rules
unet policy eval policies/ --verbose --failures-only
```

**Arguments:**

- `<PATH>` - Policy file or directory path

**Options:**

- `--verbose` - Show detailed evaluation results
- `--failures-only` - Only show policy failures
- `--node <NODE>` - Evaluate against specific node

#### `unet policy list`

List available policy files.

```bash
unet policy list policies/
```

**Arguments:**

- `<DIRECTORY>` - Policy directory path

#### `unet policy show`

Display policy file contents and parsed rules.

```bash
unet policy show policies/compliance.rules
unet policy show policies/compliance.rules --ast
```

**Arguments:**

- `<FILE>` - Policy file path

**Options:**

- `--ast` - Show parsed abstract syntax tree

---

### Data Import/Export

#### `unet import`

Import data from JSON files.

```bash
unet import your-nodes.json
unet import your-data-directory/ --dry-run
unet import your-data-directory/ --continue-on-error
```

**Arguments:**

- `<PATH>` - JSON file or directory path

**Options:**

- `--dry-run` - Show what would be imported without making changes
- `--continue-on-error` - Continue importing even if some items fail

#### `unet export`

Export data to JSON/YAML files.

```bash
unet export --output-dir exports/
unet export --output-dir exports/ --format yaml --only nodes
```

**Options:**

- `--output-dir <DIR>` - Output directory (required)
- `--format <FORMAT>` - Export format: json, yaml (default: json)
- `--only <TYPE>` - Export only specific type: nodes, links, locations
- `--force` - Overwrite existing files

---

## Output Formats

### Table Format (Default)

Human-readable tabular output with colors when outputting to terminal.

```bash
unet nodes list
```

### JSON Format

Machine-readable JSON output.

```bash
unet nodes list --output json
```

### YAML Format

Human-readable YAML output.

```bash
unet nodes list --output yaml
```

---

## Configuration

### Configuration File

μNet can load settings from a configuration file:

```bash
unet --config /path/to/config.toml nodes list
```

### Environment Variables

Set environment variables to avoid repeating common options:

```bash
export UNET_DATABASE_URL="sqlite:///path/to/unet.db"
export UNET_SERVER="http://localhost:8080"
export UNET_TOKEN="your-auth-token"
```

---

## Error Codes

| Exit Code | Meaning |
|-----------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Configuration error |
| 3 | Network/connection error |
| 4 | Authentication error |
| 5 | Data validation error |

---

## Examples

### Basic Network Setup

```bash
# Create locations
unet locations add --name "HQ Campus" --location-type campus --address "123 Business Park"
unet locations add --name "Main Building" --location-type building --parent-id <campus-uuid>

# Create nodes
unet nodes add --name core-01 --domain corp.example.com --vendor cisco --model ASR9000 \
    --role router --lifecycle live --management-ip 10.1.1.1

unet nodes add --name dist-01 --domain corp.example.com --vendor cisco --model Catalyst9400 \
    --role switch --lifecycle live --management-ip 10.1.1.2

# Create link
unet links add --node-a core-01 --interface-a TenGigE0/0/0/1 \
    --node-z dist-01 --interface-z TenGigE1/0/1 --bandwidth 10000000000

# View the topology
unet nodes list
unet links list
```

### Policy Validation

```bash
# Validate policy syntax
unet policy validate policies/security.rules

# Check compliance
unet policy eval policies/ --failures-only

# Evaluate specific node
unet policy eval policies/compliance.rules --node core-01
```

### Data Management

```bash
# Export current configuration
unet export --output-dir backup/ --format yaml

# Import from backup
unet import backup/nodes.yaml --dry-run
unet import backup/
```

---

## Limitations (Current Version)

- **Template engine**: Not yet implemented (planned for v0.2.0)
- **SNMP polling controls**: Background polling runs automatically, but CLI controls are not implemented
- **Node comparison and history**: Planned for future versions
- **Table output formatting**: Currently defaults to JSON format
- **Advanced filtering**: jq-style filters not yet implemented

For planned features, see the [Roadmap](roadmap.md).
