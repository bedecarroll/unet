# Quick Start Guide – Getting Started with μNet

> **Goal:** Get μNet running and managing your first network devices in 10 minutes

---

## Installation

### Download μNet

```bash
# Download the latest release (replace with actual URL when available)
curl -L https://github.com/bedecarroll/unet/releases/latest/download/unet-cli -o unet
curl -L https://github.com/bedecarroll/unet/releases/latest/download/unet-server -o unet-server
chmod +x unet unet-server
```

### Or Build from Source

```bash
git clone https://github.com/bedecarroll/unet.git
cd unet
cargo build --release
cp target/release/unet-cli ./unet
cp target/release/unet-server ./unet-server
```

---

## Quick Setup

### 1. Initialize Database

μNet will automatically create a SQLite database on first use:

```bash
./unet nodes list
# Creates unet.db in current directory
```

### 2. Import Sample Data

```bash
# Import locations first (dependencies)
./unet import docs/static/examples/small-office/locations.json

# Import nodes
./unet import docs/static/examples/small-office/nodes.json

# Import links
./unet import docs/static/examples/small-office/links.json

# Verify import
./unet nodes list
```

---

## Basic Operations

### Managing Nodes

```bash
# Add a new device
./unet nodes add \
  --name core-sw-01 \
  --domain corp.example.com \
  --vendor cisco \
  --model catalyst9300 \
  --role access \
  --lifecycle production \
  --management-ip 192.168.1.10

# List all devices
./unet nodes list

# Show device details
./unet nodes show core-sw-01

# Update device
./unet nodes update core-sw-01 --lifecycle production

# Delete device
./unet nodes delete core-sw-01
```

### Managing Locations

```bash
# Add a location
./unet locations add \
  --name "Building A" \
  --location-type building \
  --address "123 Main Street"

# List locations
./unet locations list

# Show location details
./unet locations show "Building A"
```

### Managing Links

```bash
# Create a point-to-point link
./unet links add \
  --node-a core-sw-01 \
  --interface-a GigE0/0/1 \
  --node-z dist-sw-01 \
  --interface-z GigE1/0/24 \
  --bandwidth 1000000000

# Create an internet circuit
./unet links add \
  --node-a edge-rtr-01 \
  --interface-a GigE0/0/0 \
  --circuit-id "ISP-CIRCUIT-12345"

# List links
./unet links list
```

---

## Working with Policies

### 1. Create Your First Policy

Create a file called `network-standards.rules`:

```rules
# Ensure all production devices are monitored
WHEN node.lifecycle == "Production" 
THEN SET custom_data.monitoring_enabled TO true

# Cisco devices should use SNMPv3
WHEN node.vendor == "Cisco"
THEN SET custom_data.snmp_version TO "v3"

# Core devices need redundancy
WHEN node.role == "Core"
THEN ASSERT custom_data.redundancy_configured IS true
```

### 2. Validate and Run Policies

```bash
# Check syntax
./unet policy validate network-standards.rules

# Evaluate against all nodes
./unet policy eval network-standards.rules

# Show only failures
./unet policy eval network-standards.rules --failures-only
```

---

## Using the HTTP Server

### 1. Start the Server

```bash
# Start server on default port (8080)
./unet-server

# Or specify custom settings
./unet-server --host 0.0.0.0 --port 8080 --database-url sqlite://network.db
```

### 2. Use CLI with Remote Server

```bash
# Set server URL
export UNET_SERVER=http://localhost:8080

# Now CLI commands use the server
./unet nodes list
./unet policy eval network-standards.rules
```

### 3. Use the API Directly

```bash
# List nodes via API
curl http://localhost:8080/api/v1/nodes

# Create a node via API
curl -X POST http://localhost:8080/api/v1/nodes \
  -H "Content-Type: application/json" \
  -d '{
    "name": "new-device",
    "vendor": "Cisco",
    "model": "ISR4331",
    "role": "Edge",
    "lifecycle": "Staging"
  }'

# Evaluate policies via API
curl -X POST http://localhost:8080/api/v1/policies/evaluate
```

---

## Output Formats

μNet supports multiple output formats for different use cases:

```bash
# Human-readable table (default for terminal)
./unet nodes list

# JSON for automation
./unet nodes list --output json

# YAML for readability
./unet nodes list --output yaml

# Pipe to other tools
./unet nodes list --output json | jq '.data.data[].name'
```

---

## Common Workflows

### Network Discovery Documentation

```bash
# 1. Document your physical sites
./unet locations add --name "HQ Campus" --location-type campus
./unet locations add --name "Main Building" --location-type building --parent-id <campus-id>

# 2. Add your network devices
./unet nodes add --name core-01 --vendor cisco --model asr9000 --role core
./unet nodes add --name dist-01 --vendor cisco --model catalyst9400 --role distribution

# 3. Document connections
./unet links add --node-a core-01 --interface-a TenGigE0/0/0/1 \
                 --node-z dist-01 --interface-z TenGigE1/0/1

# 4. Export for backup
./unet export --output-dir backup/
```

### Compliance Monitoring

```bash
# 1. Create compliance policies
cat > compliance.rules << 'EOF'
WHEN node.lifecycle == "Production" THEN ASSERT custom_data.backup_configured IS true
WHEN node.role == "Core" THEN ASSERT custom_data.redundancy_level >= 2
WHEN node.vendor == "Cisco" THEN ASSERT custom_data.snmp_version == "v3"
EOF

# 2. Run compliance check
./unet policy eval compliance.rules --failures-only

# 3. Generate compliance report
./unet policy eval compliance.rules --output json > compliance-report.json
```

### Data Migration

```bash
# Export from existing system
./unet export --output-dir migration/

# Edit exported files as needed
# ...

# Import to new environment
./unet import migration/ --dry-run  # Test first
./unet import migration/              # Actually import
```

---

## Configuration

### Environment Variables

```bash
# Database location
export UNET_DATABASE_URL="sqlite:///path/to/unet.db"

# Server URL for remote operations
export UNET_SERVER="http://unet-server:8080"

# Default output format
export UNET_OUTPUT_FORMAT="json"
```

### Configuration File

Create `~/.config/unet/config.toml`:

```toml
[defaults]
database_url = "sqlite:///home/user/network/unet.db"
server_url = "http://unet-server:8080"
output_format = "table"

[logging]
level = "info"
```

---

## Next Steps

Now that you have μNet running:

1. **Read the [CLI Reference](cli_reference.md)** for complete command documentation
2. **Study the [Policy Guide](policy_guide.md)** for advanced policy authoring
3. **Explore the [API Reference](api_reference.md)** for automation integration
4. **Review the [Architecture Overview](architecture.md)** to understand the system design

### Common Next Actions

- **Scale Up**: Import your real network topology
- **Automate**: Integrate with your CI/CD pipeline
- **Monitor**: Set up continuous policy evaluation  
- **Extend**: Use the API to build custom integrations

### Getting Help

- **Troubleshooting**: See [Troubleshooting Guide](troubleshooting.md)
- **Examples**: Check `docs/static/examples/` directory
- **Issues**: Report bugs at <https://github.com/bedecarroll/unet/issues>

Welcome to μNet! 🎉
