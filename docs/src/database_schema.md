# Database Schema Reference

μNet uses a SQLite database with SeaORM for data persistence. The schema is designed with a clear separation between desired state (user-configured network topology) and derived state (real-time data from SNMP polling).

## Overview

The database consists of two main categories of tables:

- **Desired State Tables**: Store user-defined network topology and configuration
- **Derived State Tables**: Store real-time operational data collected via SNMP

This separation ensures that configuration remains stable while allowing operational data to be updated frequently.

## Core Tables

### Locations

Physical or logical locations for organizing network devices in a hierarchical structure.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | TEXT | PRIMARY KEY, NOT NULL | Unique identifier |
| `name` | TEXT | NOT NULL | Human-readable location name |
| `type` | TEXT | NOT NULL | Type of location (datacenter, building, floor, rack, etc.) |
| `path` | TEXT | NOT NULL, UNIQUE | Hierarchical path for tree navigation |
| `parent_id` | TEXT | FOREIGN KEY | Reference to parent location |
| `description` | TEXT | | Optional description |
| `address` | TEXT | | Physical address or location details |
| `coordinates` | TEXT | | GPS coordinates or position data |
| `custom_data` | TEXT | | JSON string for custom attributes |
| `created_at` | TEXT | NOT NULL, DEFAULT CURRENT_TIMESTAMP | Creation timestamp |
| `updated_at` | TEXT | NOT NULL, DEFAULT CURRENT_TIMESTAMP | Last update timestamp |

**Indexes:**

- `idx_location_path` (unique on `path`)
- `idx_location_parent` (on `parent_id`)

### Nodes

Network devices and their static configuration attributes.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | TEXT | PRIMARY KEY, NOT NULL | Unique identifier |
| `name` | TEXT | NOT NULL | Node name (typically hostname) |
| `fqdn` | TEXT | | Fully qualified domain name |
| `domain` | TEXT | | DNS domain |
| `vendor` | TEXT | NOT NULL | Device vendor (see Vendor enum) |
| `model` | TEXT | NOT NULL | Device model number |
| `role` | TEXT | NOT NULL | Device role (see DeviceRole enum) |
| `lifecycle` | TEXT | NOT NULL | Lifecycle stage (see Lifecycle enum) |
| `serial_number` | TEXT | | Device serial number |
| `asset_tag` | TEXT | | Asset tag for inventory tracking |
| `location_id` | TEXT | FOREIGN KEY | Reference to location |
| `management_ip` | TEXT | | Primary management IP address |
| `description` | TEXT | | Optional description |
| `custom_data` | TEXT | | JSON string for custom attributes |
| `created_at` | TEXT | NOT NULL, DEFAULT CURRENT_TIMESTAMP | Creation timestamp |
| `updated_at` | TEXT | NOT NULL, DEFAULT CURRENT_TIMESTAMP | Last update timestamp |

**Indexes:**

- `idx_node_name` (on `name`)
- `idx_node_fqdn` (on `fqdn`)
- `idx_node_location` (on `location_id`)
- `idx_node_role` (on `role`)
- `idx_node_lifecycle` (on `lifecycle`)

### Links

Network connections between devices, including both internal links and internet circuits.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | TEXT | PRIMARY KEY, NOT NULL | Unique identifier |
| `name` | TEXT | NOT NULL | Human-readable link name |
| `node_a_id` | TEXT | NOT NULL, FOREIGN KEY | Reference to first node (required) |
| `interface_a` | TEXT | NOT NULL | Interface name on first node |
| `node_b_id` | TEXT | FOREIGN KEY | Reference to second node (optional for internet circuits) |
| `interface_b` | TEXT | | Interface name on second node |
| `capacity` | BIGINT | | Link capacity in bits per second |
| `utilization` | REAL | | Current utilization as percentage (0.0-1.0) |
| `is_internet_circuit` | INTEGER | NOT NULL, DEFAULT 0 | Whether this is an internet circuit (1) or internal link (0) |
| `circuit_id` | TEXT | | Provider circuit identifier |
| `provider` | TEXT | | Service provider name |
| `description` | TEXT | | Optional description |
| `custom_data` | TEXT | | JSON string for custom attributes |
| `created_at` | TEXT | NOT NULL, DEFAULT CURRENT_TIMESTAMP | Creation timestamp |
| `updated_at` | TEXT | NOT NULL, DEFAULT CURRENT_TIMESTAMP | Last update timestamp |

**Indexes:**

- `idx_link_name` (on `name`)
- `idx_link_node_a` (on `node_a_id`)
- `idx_link_node_b` (on `node_b_id`)
- `idx_link_circuit_id` (on `circuit_id`)

### Vendors

List of supported network equipment vendors.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `name` | TEXT | PRIMARY KEY, NOT NULL | Vendor name |

Seeded vendors include Cisco, Juniper, Arista, and others. Use the CLI to manage the list:

```bash
# List vendors
unet vendors list
# Add vendor
unet vendors add CustomVendor
# Remove vendor
unet vendors delete CustomVendor
```

## Derived State Tables

### Node Status

Real-time operational status and metrics for network nodes, populated by SNMP polling.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | TEXT | PRIMARY KEY, NOT NULL | Unique identifier |
| `node_id` | TEXT | NOT NULL, FOREIGN KEY | Reference to node table |
| `last_updated` | TEXT | NOT NULL | Timestamp of last status update |
| `reachable` | BOOLEAN | NOT NULL, DEFAULT false | Whether node is reachable via SNMP |
| `system_info` | TEXT | | JSON containing system information (uptime, name, description) |
| `performance` | TEXT | | JSON containing performance metrics (CPU, memory) |
| `environmental` | TEXT | | JSON containing environmental data (temperature, fans) |
| `vendor_metrics` | TEXT | | JSON containing vendor-specific metrics |
| `raw_snmp_data` | TEXT | | JSON containing raw SNMP response data |
| `last_snmp_success` | TEXT | | Timestamp of last successful SNMP poll |
| `last_error` | TEXT | | Last error message encountered during polling |
| `consecutive_failures` | INTEGER | NOT NULL, DEFAULT 0 | Number of consecutive failed polling attempts |

**Indexes:**

- `idx_node_status_node_id` (unique on `node_id`)
- `idx_node_status_last_updated` (on `last_updated`)

### Interface Status

Per-interface operational data and statistics from SNMP interface tables.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | TEXT | PRIMARY KEY, NOT NULL | Unique identifier |
| `node_status_id` | TEXT | NOT NULL, FOREIGN KEY | Reference to node_status table |
| `index` | INTEGER | NOT NULL | Interface index from SNMP ifTable |
| `name` | TEXT | NOT NULL | Interface name or description |
| `type` | INTEGER | NOT NULL | Interface type code from SNMP ifType |
| `mtu` | INTEGER | | Maximum transmission unit in bytes |
| `speed` | BIGINT | | Interface speed in bits per second |
| `physical_address` | TEXT | | Physical MAC address of the interface |
| `admin_status` | TEXT | NOT NULL, DEFAULT 'unknown' | Administrative status (up/down/testing) |
| `oper_status` | TEXT | NOT NULL, DEFAULT 'unknown' | Operational status (up/down/testing/unknown/dormant/notPresent/lowerLayerDown) |
| `last_change` | INTEGER | | Time when interface last changed state |
| `input_stats` | TEXT | NOT NULL | JSON containing input statistics (packets, bytes, errors) |
| `output_stats` | TEXT | NOT NULL | JSON containing output statistics (packets, bytes, errors) |

**Indexes:**

- `idx_interface_status_node_status_id` (on `node_status_id`)
- `idx_interface_status_index` (unique on `node_status_id`, `index`)

### Polling Tasks

Configuration for SNMP polling tasks that collect operational data.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | TEXT | PRIMARY KEY, NOT NULL | Unique identifier |
| `node_id` | TEXT | NOT NULL, FOREIGN KEY | Reference to node table |
| `target` | TEXT | NOT NULL | SNMP target address or hostname |
| `oids` | TEXT | NOT NULL | JSON array of OIDs to poll |
| `interval_seconds` | BIGINT | NOT NULL | Polling interval in seconds |
| `session_config` | TEXT | NOT NULL | JSON containing SNMP session configuration |
| `priority` | SMALLINT | NOT NULL, DEFAULT 128 | Task priority for scheduling |
| `enabled` | BOOLEAN | NOT NULL, DEFAULT true | Whether task is currently enabled |
| `created_at` | TEXT | NOT NULL | Task creation timestamp |
| `last_success` | TEXT | | Timestamp of last successful poll |
| `last_error` | TEXT | | Last error message from polling |
| `consecutive_failures` | INTEGER | NOT NULL, DEFAULT 0 | Number of consecutive failed polls |

**Indexes:**

- `idx_polling_tasks_node_id` (on `node_id`)
- `idx_polling_tasks_enabled` (on `enabled`)

## Enumerations

### Device Role

Defines the network function of a device:

- `router` - Network router
- `switch` - Network switch  
- `firewall` - Firewall device
- `loadbalancer` - Load balancer
- `accesspoint` - Wireless access point
- `securityappliance` - Network security appliance
- `monitor` - Network monitoring device
- `server` - Generic server
- `storage` - Storage device
- `other` - Other/unspecified device type

### Lifecycle

Defines the operational state of a device:

- `planned` - Device is planned but not yet deployed
- `implementing` - Device is currently being implemented/deployed
- `live` - Device is live and operational
- `decommissioned` - Device is being decommissioned or is decommissioned

### Vendor

Supported network equipment vendors:

- `cisco` - Cisco Systems
- `juniper` - Juniper Networks
- `arista` - Arista Networks
- `paloalto` - Palo Alto Networks
- `fortinet` - Fortinet
- `hpe` - HPE/Hewlett Packard Enterprise
- `dell` - Dell Technologies
- `extreme` - Extreme Networks
- `mikrotik` - Mikrotik
- `ubiquiti` - Ubiquiti
- `generic` - Generic/unknown vendor

## Schema Design Patterns

### Primary Keys

All tables use text-based UUIDs as primary keys to ensure global uniqueness and enable distributed scenarios.

### JSON Storage

Complex or flexible data is stored as JSON strings in columns like `custom_data`, `system_info`, and statistics fields. This allows for extensibility without schema changes.

### Timestamps

All timestamps are stored as text in ISO 8601 format for consistency and readability.

### Foreign Key Relationships

The schema maintains referential integrity through foreign key relationships:

- Nodes → Locations (optional)
- Links → Nodes (A-side required, B-side optional)
- Node Status → Nodes (one-to-one)
- Interface Status → Node Status (one-to-many)
- Polling Tasks → Nodes (one-to-many)

### Indexing Strategy

Indexes are created on:

- Foreign key columns for efficient joins
- Frequently queried columns (names, roles, status)
- Unique constraints where needed

## Usage Considerations

### Data Separation

The schema enforces separation between:

- **Configuration data** (locations, nodes, links) - changes infrequently
- **Operational data** (status tables) - updated frequently via SNMP

### Extensibility

Custom fields can be added via:

- `custom_data` JSON columns on core entities
- Additional OIDs in polling tasks
- Vendor-specific metrics in status tables

### Performance

- Derived state tables are optimized for frequent updates
- Indexes support common query patterns
- JSON storage provides flexibility without normalization overhead

## Missing Fields?

If you need additional fields for your network automation use case, consider:

1. **Custom Data Fields**: Use the `custom_data` JSON columns for entity-specific extensions
2. **New Enums**: Extend the vendor, role, or lifecycle enums if needed
3. **Additional Tables**: For complex relationships not covered by existing schema
4. **SNMP Extensions**: Add new OIDs to polling tasks for additional metrics

Open a pull request with your proposed schema changes and use cases to help expand μNet's capabilities.
