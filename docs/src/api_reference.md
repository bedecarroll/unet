# API Reference – μNet HTTP Server

> **Audience:** Developers integrating with μNet via HTTP API  
> **Status:** Documents implemented endpoints only (v0.1.0)

---

## Overview

μNet provides a RESTful HTTP API for all network configuration management
operations. The API is built with Axum (Rust) and supports JSON
request/response formats.

**Base URL:** `http://localhost:8080` (default)  
**API Version:** v1  
**Authentication:** None (planned for future versions)

## Standard Response Format

All API responses follow a consistent format:

### Success Response

```json
{
  "data": "<response_data>",
  "success": true,
  "message": null
}
```

### Error Response

```json
{
  "error": "Human-readable error message",
  "code": "ERROR_CODE",
  "success": false
}
```

### HTTP Status Codes

- **200** - Success
- **400** - Bad Request (validation errors)
- **404** - Resource not found
- **409** - Conflict (constraint violations)
- **500** - Internal server error
- **503** - Service unavailable

---

## Health Check

### `GET /health`

System health and status check.

### Response

```json
{
  "status": "healthy",
  "service": "μNet",
  "version": "0.1.0",
  "timestamp": "2024-01-15T10:30:00Z",
  "components": {
    "datastore": {
      "status": "healthy",
      "type": "sqlite"
    }
  }
}
```

---

## Node Management

### `GET /api/v1/nodes`

List all nodes with filtering and pagination.

### Query Parameters

- `page` (int) - Page number (default: 1)
- `per_page` (int) - Items per page (default: 20)
- `lifecycle` (string) - Filter by lifecycle state
- `role` (string) - Filter by device role
- `vendor` (string) - Filter by vendor
- `include_status` (bool) - Include SNMP-derived status data

### Example Request

```bash
GET /api/v1/nodes?vendor=cisco&role=router&page=1&per_page=10
```

### Response

```json
{
  "data": {
    "data": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "name": "core-01",
        "domain": "example.com",
        "fqdn": "core-01.example.com",
        "vendor": "Cisco",
        "model": "ASR9000",
        "role": "Core",
        "lifecycle": "Production",
        "management_ip": "10.1.1.1",
        "location_id": "550e8400-e29b-41d4-a716-446655440001",
        "platform": null,
        "version": null,
        "serial_number": null,
        "asset_tag": null,
        "purchase_date": null,
        "warranty_expires": null,
        "custom_data": {},
        "status": null
      }
    ],
    "total": 50,
    "page": 1,
    "per_page": 10,
    "total_pages": 5,
    "has_next": true,
    "has_prev": false
  },
  "success": true,
  "message": null
}
```

### `GET /api/v1/nodes/{id}`

Get a specific node by UUID.

### Path Parameters

- `id` (UUID) - Node identifier

### Response

```json
{
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "core-01",
    "domain": "example.com",
    "fqdn": "core-01.example.com",
    "vendor": "Cisco",
    "model": "ASR9000",
    "role": "Core",
    "lifecycle": "Production",
    "management_ip": "10.1.1.1",
    "location_id": "550e8400-e29b-41d4-a716-446655440001",
    "custom_data": {
      "rack_position": "R1-U20"
    }
  },
  "success": true,
  "message": null
}
```

### `POST /api/v1/nodes`

Create a new node.

### Request Body

```json
{
  "name": "new-router-01",
  "domain": "example.com",
  "vendor": "Cisco",
  "model": "ISR4431",
  "role": "Edge",
  "lifecycle": "Staging",
  "management_ip": "10.1.1.50",
  "location_id": "550e8400-e29b-41d4-a716-446655440001",
  "custom_data": {
    "cost_center": "IT-001",
    "rack_position": "R2-U10"
  }
}
```

**Required Fields:** `name`, `vendor`, `model`, `role`, `lifecycle`  
**Optional Fields:** `domain`, `management_ip`, `location_id`, `custom_data`

**Vendor Values:** `Cisco`, `Juniper`, `Arista`, `HPE`, `Dell`, `Ubiquiti`, `MikroTik`, `Fortinet`, `PaloAlto`, `CheckPoint`, `F5`, `A10`, `Riverbed`, `SilverPeak`, `VMware`, `Linux`, `Windows`, `Other`

**Role Values:** `Core`, `Distribution`, `Access`, `Edge`, `Firewall`, `LoadBalancer`, `WirelessController`, `WirelessAP`, `Server`, `Storage`, `Hypervisor`, `Container`, `IoT`, `Camera`, `Phone`, `Printer`, `Other`

**Lifecycle Values:** `Planning`, `Staging`, `Production`, `Maintenance`, `Decommissioned`

### `PUT /api/v1/nodes/{id}`

Update an existing node (partial updates supported).

### Path Parameters

- `id` (UUID) - Node identifier

**Request Body** (all fields optional)

```json
{
  "name": "updated-router-01",
  "lifecycle": "Production",
  "management_ip": "10.1.1.51",
  "custom_data": {
    "maintenance_window": "Sunday 02:00-04:00"
  }
}
```

### `DELETE /api/v1/nodes/{id}`

Delete a node.

### Path Parameters

- `id` (UUID) - Node identifier

### Response

```json
{
  "data": null,
  "success": true,
  "message": null
}
```

---

## Node Derived State (SNMP Data)

### `GET /api/v1/nodes/{id}/status`

Get current node status from SNMP polling.

### Path Parameters

- `id` (UUID) - Node identifier

### Response

```json
{
  "data": {
    "node_id": "550e8400-e29b-41d4-a716-446655440000",
    "last_updated": "2024-01-15T10:30:00Z",
    "reachable": true,
    "system_info": {
      "description": "Cisco IOS Software, Version 15.2(7)E6",
      "object_id": "1.3.6.1.4.1.9.1.516",
      "uptime_ticks": 123456789,
      "contact": "admin@example.com",
      "name": "core-01",
      "location": "Data Center Rack 1",
      "services": 72
    },
    "interfaces": [],
    "performance": null,
    "environmental": null,
    "vendor_metrics": {},
    "raw_snmp_data": {},
    "last_snmp_success": "2024-01-15T10:29:45Z",
    "last_error": null,
    "consecutive_failures": 0
  },
  "success": true,
  "message": null
}
```

### `GET /api/v1/nodes/{id}/interfaces`

Get interface status for a node.

### Path Parameters

- `id` (UUID) - Node identifier

### Response

```json
{
  "data": [
    {
      "index": 1,
      "name": "GigabitEthernet1/0/1",
      "interface_type": 6,
      "mtu": 1500,
      "speed": 1000000000,
      "physical_address": "aa:bb:cc:dd:ee:ff",
      "admin_status": "Up",
      "oper_status": "Up",
      "last_change": "2024-01-15T08:00:00Z",
      "input_stats": {
        "octets": 1234567890,
        "unicast_packets": 9876543,
        "non_unicast_packets": 1000,
        "discards": 0,
        "errors": 0,
        "unknown_protocols": 0
      },
      "output_stats": {
        "octets": 987654321,
        "unicast_packets": 5432109,
        "non_unicast_packets": 500,
        "discards": 0,
        "errors": 0
      }
    }
  ],
  "success": true,
  "message": null
}
```

### `GET /api/v1/nodes/{id}/metrics`

Get performance metrics for a node.

### Path Parameters

- `id` (UUID) - Node identifier

### Response

```json
{
  "data": {
    "cpu_utilization": 15.5,
    "memory_utilization": 45.2,
    "temperature": 42.0,
    "total_memory_kb": 524288,
    "available_memory_kb": 287309,
    "buffer_hit_ratio": 98.5,
    "uptime_seconds": 1234567
  },
  "success": true,
  "message": null
}
```

---

## Policy Management

### `POST /api/v1/policies/evaluate`

Evaluate policies against nodes.

### Request Body

```json
{
  "node_ids": ["550e8400-e29b-41d4-a716-446655440000"],
  "policies": [
    {
      "id": "cisco-compliance",
      "condition": "node.vendor == 'Cisco' && node.lifecycle == 'Production'",
      "action": "assert('Cisco device in production has proper config')"
    }
  ],
  "store_results": true
}
```

**Fields:**

- `node_ids` (array, optional) - Node UUIDs to evaluate (default: all nodes)
- `policies` (array, optional) - Policy rules to evaluate (default: load from configured source)
- `store_results` (bool, optional) - Store results in database (default: true)

### Response

```json
{
  "data": {
    "results": {
      "550e8400-e29b-41d4-a716-446655440000": [
        {
          "policy_id": "cisco-compliance",
          "status": "Satisfied",
          "message": "Cisco device in production has proper config",
          "execution_time_ms": 12,
          "timestamp": "2024-01-15T10:30:00Z"
        }
      ]
    },
    "nodes_evaluated": 1,
    "policies_evaluated": 1,
    "evaluation_time_ms": 150,
    "summary": {
      "total_rules": 1,
      "satisfied_rules": 1,
      "unsatisfied_rules": 0,
      "error_rules": 0,
      "compliance_failures": 0
    }
  },
  "success": true,
  "message": null
}
```

### `GET /api/v1/policies/results`

Get stored policy evaluation results.

### Query Parameters

- `node_id` (UUID, optional) - Filter by node
- `limit` (int, optional) - Maximum results (default: 100)
- `offset` (int, optional) - Results to skip (default: 0)

### Response

```json
{
  "data": {
    "results": [
      {
        "policy_id": "cisco-compliance",
        "status": "Satisfied",
        "message": "Cisco device in production has proper config",
        "execution_time_ms": 12,
        "timestamp": "2024-01-15T10:30:00Z"
      }
    ],
    "total_count": 1,
    "returned_count": 1
  },
  "success": true,
  "message": null
}
```

### `POST /api/v1/policies/validate`

Validate policy rule syntax.

### Request Body

```json
[
  {
    "id": "valid-policy",
    "condition": "node.vendor == 'Cisco'",
    "action": "assert('Valid Cisco device')"
  },
  {
    "id": "invalid-policy",
    "condition": "",
    "action": "invalid_action()"
  }
]
```

### Response

```json
{
  "total_policies": 2,
  "valid_policies": 1,
  "invalid_policies": 1,
  "validation_results": [
    {
      "index": 0,
      "valid": true,
      "message": "Policy rule is valid"
    },
    {
      "index": 1,
      "valid": false,
      "message": "Policy rule has syntax errors"
    }
  ]
}
```

### `GET /api/v1/policies/status`

Get policy engine status.

### Response

```json
{
  "policy_engine_enabled": true,
  "nodes_available": 25,
  "policies_available": 10,
  "last_evaluation": null,
  "evaluation_frequency": "on-demand"
}
```

---

## Error Handling

### Common Error Codes

| Code | Description |
|------|-------------|
| `VALIDATION_ERROR` | Request validation failed |
| `NOT_FOUND` | Resource not found |
| `CONSTRAINT_VIOLATION` | Database constraint violated |
| `DATASTORE_ERROR` | Database operation failed |
| `POLICY_ERROR` | Policy evaluation failed |
| `SNMP_ERROR` | SNMP operation failed |

### Example Error Response

```json
{
  "error": "Node with name 'duplicate-name' already exists",
  "code": "CONSTRAINT_VIOLATION",
  "success": false
}
```

---

## Examples

### Create a Complete Network Setup

```bash
# Create a location first
curl -X POST http://localhost:8080/api/v1/locations \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Data Center 1",
    "location_type": "Building",
    "address": "123 Server St"
  }'

# Create nodes
curl -X POST http://localhost:8080/api/v1/nodes \
  -H "Content-Type: application/json" \
  -d '{
    "name": "core-01",
    "domain": "example.com",
    "vendor": "Cisco",
    "model": "ASR9000",
    "role": "Core",
    "lifecycle": "Production",
    "management_ip": "10.1.1.1",
    "location_id": "550e8400-e29b-41d4-a716-446655440001"
  }'

# List nodes
curl http://localhost:8080/api/v1/nodes?vendor=Cisco

# Get node status
curl http://localhost:8080/api/v1/nodes/550e8400-e29b-41d4-a716-446655440000/status
```

### Policy Evaluation

```bash
# Evaluate compliance policies
curl -X POST http://localhost:8080/api/v1/policies/evaluate \
  -H "Content-Type: application/json" \
  -d '{
    "policies": [
      {
        "id": "production-check",
        "condition": "node.lifecycle == \"Production\"",
        "action": "assert(\"Device is in production\")"
      }
    ]
  }'

# Get evaluation results
curl http://localhost:8080/api/v1/policies/results?limit=50
```

---

## Configuration

### Server Configuration

The API server can be configured via command line, environment variables, or configuration file:

```bash
# Command line
unet-server --host 0.0.0.0 --port 8080 --database-url sqlite://production.db

# Environment variables
export UNET_HOST=0.0.0.0
export UNET_PORT=8080
export UNET_DATABASE_URL=sqlite://production.db
unet-server

# Configuration file
unet-server --config /etc/unet/server.toml
```

### CORS

The server enables permissive CORS for development. In production, configure appropriate CORS policies.

---

## Future Enhancements

- **Authentication & Authorization** - Token-based access control
- **Rate Limiting** - Request throttling and abuse prevention  
- **WebSocket Support** - Real-time updates and notifications
- **GraphQL API** - Alternative query interface
- **OpenAPI Specification** - Auto-generated API documentation

For planned features, see the [Roadmap](roadmap.md).
