# μNet Example Data Fixtures

This directory contains example data that demonstrates μNet's capabilities and provides a starting point for new users.

## Quick Start

To import the example data into your μNet database:

```bash
# Initialize a new SQLite database
unet init

# Import the example network topology
unet import --from fixtures/examples/
```

## Directory Structure

```
fixtures/
├── README.md           # This file
├── examples/           # Example network topologies
│   ├── small-office/   # Small office network example
│   ├── datacenter/     # Datacenter network example
│   └── campus/         # Campus network example
└── schemas/            # JSON schemas for validation
```

## Data Format

All fixture files use JSON format with the following structure:

### Locations (`locations.json`)
```json
[
  {
    "name": "Main Building",
    "location_type": "building",
    "parent_id": null,
    "custom_data": {
      "address": "123 Network St",
      "contact": "facilities@example.com"
    }
  }
]
```

### Nodes (`nodes.json`)
```json
[
  {
    "name": "core-sw-01",
    "domain": "example.com",
    "vendor": "Cisco",
    "model": "Catalyst 9300",
    "role": "Core",
    "lifecycle": "Production",
    "location_id": "location-uuid-here",
    "management_ip": "192.168.1.10",
    "custom_data": {
      "purchase_date": "2023-01-15",
      "warranty_end": "2026-01-15"
    }
  }
]
```

### Links (`links.json`)
```json
[
  {
    "name": "core-to-access-01",
    "node_a_id": "node-a-uuid",
    "interface_a": "GigabitEthernet1/0/1",
    "node_z_id": "node-z-uuid", 
    "interface_z": "GigabitEthernet1/0/24",
    "bandwidth_bps": 1000000000,
    "custom_data": {
      "cable_type": "fiber",
      "length_meters": 50
    }
  }
]
```

## Example Topologies

### Small Office
- **Description**: 10-50 user office network
- **Devices**: 1 core switch, 2 access switches, 1 firewall
- **Use Case**: Small business, branch office

### Datacenter  
- **Description**: Modern 3-tier datacenter architecture
- **Devices**: Core switches, distribution switches, ToR switches, servers
- **Use Case**: Enterprise datacenter, cloud infrastructure

### Campus
- **Description**: Multi-building campus network
- **Devices**: Core, distribution, access layers across multiple buildings
- **Use Case**: University, corporate campus, hospital

## Custom Data Examples

Each example includes realistic custom_data fields that demonstrate:

- **Asset Management**: Purchase dates, warranty info, serial numbers
- **Physical**: Rack locations, cable types, power requirements
- **Operational**: Maintenance schedules, contact information
- **Business**: Cost centers, project codes, compliance tags

## Import Process

The import process:

1. **Validates** data against schemas
2. **Creates** database tables if needed
3. **Imports** locations first (for FK references)
4. **Imports** nodes second
5. **Imports** links last
6. **Verifies** all references and constraints

## Customization

To create your own fixtures:

1. Copy an existing example directory
2. Modify the JSON files for your topology
3. Update UUIDs to maintain referential integrity
4. Test with `unet import --validate-only`
5. Import with `unet import --from your-directory/`
