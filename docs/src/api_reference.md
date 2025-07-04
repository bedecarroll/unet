<!-- SPDX-License-Identifier: MIT -->

# API Reference Documentation

> **Audience:** Developers integrating with μNet API, system administrators, and API consumers.  
> **Purpose:** Complete reference for all μNet REST API endpoints, request/response formats, and authentication methods.  
> **Base URL:** `https://your-unet-instance.com/api/v1` or `http://localhost:8080/api/v1`

---

## Table of Contents

1. [Authentication](#1-authentication)
2. [Common Response Formats](#2-common-response-formats)
3. [Error Handling](#3-error-handling)
4. [Pagination](#4-pagination)
5. [Node Management](#5-node-management)
6. [Location Management](#6-location-management)
7. [Link Management](#7-link-management)
8. [Policy Management](#8-policy-management)
9. [Template Management](#9-template-management)
10. [Git Integration](#10-git-integration)
11. [Change Management](#11-change-management)
12. [Certificate Management](#12-certificate-management)
13. [Alerting](#13-alerting)
14. [Network Access Control](#14-network-access-control)
15. [Distributed Locking](#15-distributed-locking)
16. [Metrics and Monitoring](#16-metrics-and-monitoring)
17. [Performance Management](#17-performance-management)
18. [Cluster Coordination](#18-cluster-coordination)
19. [Resource Management](#19-resource-management)
20. [Health Checks](#20-health-checks)

---

## 1. Authentication

### 1.1 JWT Authentication

Most API endpoints require JWT authentication. Include the token in the Authorization header:

```
Authorization: Bearer <jwt-token>
```

### 1.2 Authentication Endpoints

#### Login

```http
POST /api/v1/auth/login
Content-Type: application/json

{
  "username": "user@example.com",
  "password": "password123"
}
```

**Response:**

```json
{
  "data": {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "refresh_token": "refresh_token_here",
    "expires_in": 86400,
    "user": {
      "id": "uuid",
      "username": "user@example.com",
      "roles": ["user"]
    }
  },
  "success": true
}
```

#### Get Current User

```http
GET /api/v1/auth/me
Authorization: Bearer <token>
```

#### Get User Permissions

```http
GET /api/v1/auth/permissions
Authorization: Bearer <token>
```

#### Change Password

```http
POST /api/v1/auth/change-password
Authorization: Bearer <token>
Content-Type: application/json

{
  "current_password": "old_password",
  "new_password": "new_password"
}
```

### 1.3 User Management

#### Create User

```http
POST /api/v1/auth/users
Authorization: Bearer <admin-token>
Content-Type: application/json

{
  "username": "newuser@example.com",
  "password": "password123",
  "roles": ["user"],
  "full_name": "New User",
  "email": "newuser@example.com"
}
```

#### List Users

```http
GET /api/v1/auth/users?page=1&per_page=20
Authorization: Bearer <admin-token>
```

#### Get User

```http
GET /api/v1/auth/users/{user_id}
Authorization: Bearer <admin-token>
```

#### Update User

```http
PUT /api/v1/auth/users/{user_id}
Authorization: Bearer <admin-token>
Content-Type: application/json

{
  "full_name": "Updated Name",
  "roles": ["user", "operator"]
}
```

### 1.4 Role Management

#### Create Role

```http
POST /api/v1/auth/roles
Authorization: Bearer <admin-token>
Content-Type: application/json

{
  "name": "custom_role",
  "description": "Custom role description",
  "permissions": [
    "nodes:read",
    "nodes:write",
    "policies:read"
  ]
}
```

#### List Roles

```http
GET /api/v1/auth/roles
Authorization: Bearer <admin-token>
```

### 1.5 API Key Management

#### Create API Key

```http
POST /api/v1/auth/api-keys
Authorization: Bearer <admin-token>
Content-Type: application/json

{
  "name": "integration-key",
  "description": "API key for external integration",
  "permissions": ["nodes:read", "metrics:read"],
  "expires_at": "2024-12-31T23:59:59Z"
}
```

#### List API Keys

```http
GET /api/v1/auth/api-keys
Authorization: Bearer <admin-token>
```

#### Delete API Key

```http
DELETE /api/v1/auth/api-keys/{key_id}
Authorization: Bearer <admin-token>
```

---

## 2. Common Response Formats

### 2.1 Standard API Response

All successful API responses follow this format:

```json
{
  "data": <response_data>,
  "success": true,
  "message": "Optional message"
}
```

### 2.2 Paginated Response

For endpoints that return lists:

```json
{
  "data": {
    "items": [<array_of_items>],
    "pagination": {
      "page": 1,
      "per_page": 20,
      "total_items": 150,
      "total_pages": 8,
      "has_next": true,
      "has_previous": false
    }
  },
  "success": true
}
```

---

## 3. Error Handling

### 3.1 Error Response Format

```json
{
  "error": "Detailed error message",
  "code": "ERROR_CODE",
  "success": false
}
```

### 3.2 Common Error Codes

| HTTP Status | Error Code | Description |
|------------|------------|-------------|
| 400 | `VALIDATION_ERROR` | Request validation failed |
| 401 | `UNAUTHORIZED` | Authentication required |
| 403 | `FORBIDDEN` | Insufficient permissions |
| 404 | `NOT_FOUND` | Resource not found |
| 409 | `CONFLICT` | Resource conflict |
| 429 | `RATE_LIMITED` | Too many requests |
| 500 | `INTERNAL_ERROR` | Internal server error |

---

## 4. Pagination

Most list endpoints support pagination:

**Query Parameters:**

- `page`: Page number (1-based, default: 1)
- `per_page`: Items per page (default: 20, max: 100)

**Example:**

```http
GET /api/v1/nodes?page=2&per_page=50
```

---

## 5. Node Management

### 5.1 List Nodes

```http
GET /api/v1/nodes
Authorization: Bearer <token>
```

**Query Parameters:**

- `page`: Page number
- `per_page`: Items per page
- `lifecycle`: Filter by lifecycle (lab, staging, production)
- `role`: Filter by role (router, switch, firewall)
- `vendor`: Filter by vendor
- `include_status`: Include derived status (true/false)

**Response:**

```json
{
  "data": {
    "items": [
      {
        "id": "uuid",
        "name": "switch-01",
        "address": "192.168.1.1",
        "vendor": "cisco",
        "device_type": "switch",
        "role": "access",
        "lifecycle": "production",
        "location_id": "uuid",
        "snmp_community": "public",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z",
        "custom_data": {},
        "status": {
          "is_reachable": true,
          "last_seen": "2024-01-01T12:00:00Z",
          "uptime": 86400,
          "snmp_status": "ok"
        }
      }
    ],
    "pagination": {
      "page": 1,
      "per_page": 20,
      "total_items": 1,
      "total_pages": 1
    }
  },
  "success": true
}
```

### 5.2 Create Node

```http
POST /api/v1/nodes
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "new-switch",
  "address": "192.168.1.100",
  "vendor": "cisco",
  "device_type": "switch",
  "role": "access",
  "lifecycle": "production",
  "location_id": "uuid",
  "snmp_community": "public",
  "custom_data": {
    "rack_position": "U24",
    "management_vlan": 100
  }
}
```

### 5.3 Get Node

```http
GET /api/v1/nodes/{node_id}
Authorization: Bearer <token>
```

### 5.4 Update Node

```http
PUT /api/v1/nodes/{node_id}
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "updated-switch",
  "role": "distribution",
  "custom_data": {
    "rack_position": "U25"
  }
}
```

### 5.5 Delete Node

```http
DELETE /api/v1/nodes/{node_id}
Authorization: Bearer <token>
```

### 5.6 Get Node Status

```http
GET /api/v1/nodes/{node_id}/status
Authorization: Bearer <token>
```

**Response:**

```json
{
  "data": {
    "is_reachable": true,
    "last_seen": "2024-01-01T12:00:00Z",
    "uptime": 86400,
    "snmp_status": "ok",
    "response_time": 15.5,
    "packet_loss": 0.0
  },
  "success": true
}
```

### 5.7 Get Node Interfaces

```http
GET /api/v1/nodes/{node_id}/interfaces
Authorization: Bearer <token>
```

### 5.8 Get Node Metrics

```http
GET /api/v1/nodes/{node_id}/metrics
Authorization: Bearer <token>
```

---

## 6. Location Management

### 6.1 List Locations

```http
GET /api/v1/locations
Authorization: Bearer <token>
```

### 6.2 Create Location

```http
POST /api/v1/locations
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "Data Center 1",
  "type": "datacenter",
  "address": "123 Main St, City, State",
  "latitude": 40.7128,
  "longitude": -74.0060,
  "custom_data": {
    "facility_code": "DC01",
    "power_capacity": "500kW"
  }
}
```

### 6.3 Get Location

```http
GET /api/v1/locations/{location_id}
Authorization: Bearer <token>
```

### 6.4 Update Location

```http
PUT /api/v1/locations/{location_id}
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "Updated Data Center",
  "custom_data": {
    "power_capacity": "750kW"
  }
}
```

### 6.5 Delete Location

```http
DELETE /api/v1/locations/{location_id}
Authorization: Bearer <token>
```

---

## 7. Link Management

### 7.1 List Links

```http
GET /api/v1/links
Authorization: Bearer <token>
```

### 7.2 Create Link

```http
POST /api/v1/links
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "link-switch1-switch2",
  "source_node_id": "uuid",
  "target_node_id": "uuid",
  "source_interface": "GigabitEthernet1/0/1",
  "target_interface": "GigabitEthernet1/0/24",
  "link_type": "ethernet",
  "bandwidth": 1000000000,
  "custom_data": {
    "cable_type": "fiber",
    "length_meters": 50
  }
}
```

### 7.3 Get Link

```http
GET /api/v1/links/{link_id}
Authorization: Bearer <token>
```

### 7.4 Update Link

```http
PUT /api/v1/links/{link_id}
Authorization: Bearer <token>
Content-Type: application/json

{
  "bandwidth": 10000000000,
  "custom_data": {
    "cable_type": "fiber",
    "length_meters": 75
  }
}
```

### 7.5 Delete Link

```http
DELETE /api/v1/links/{link_id}
Authorization: Bearer <token>
```

---

## 8. Policy Management

### 8.1 Evaluate Policies

```http
POST /api/v1/policies/evaluate
Authorization: Bearer <token>
Content-Type: application/json

{
  "node_id": "uuid",
  "policy_names": ["cisco-ios-config", "security-baseline"],
  "context": {
    "environment": "production"
  }
}
```

**Response:**

```json
{
  "data": {
    "node_id": "uuid",
    "evaluation_results": [
      {
        "policy_name": "cisco-ios-config",
        "status": "pass",
        "actions": [
          {
            "type": "assert",
            "description": "SNMP community is configured",
            "result": "pass"
          }
        ],
        "execution_time_ms": 15
      }
    ],
    "overall_status": "pass",
    "total_execution_time_ms": 25
  },
  "success": true
}
```

### 8.2 Get Policy Results

```http
GET /api/v1/policies/results?node_id={node_id}&policy_name={policy_name}
Authorization: Bearer <token>
```

### 8.3 Validate Policies

```http
POST /api/v1/policies/validate
Authorization: Bearer <token>
Content-Type: application/json

{
  "policy_content": "rule \"example\" { ... }",
  "syntax_only": false
}
```

### 8.4 Get Policy Status

```http
GET /api/v1/policies/status
Authorization: Bearer <token>
```

---

## 9. Template Management

### 9.1 List Templates

```http
GET /api/v1/templates
Authorization: Bearer <token>
```

### 9.2 Create Template

```http
POST /api/v1/templates
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "cisco-ios-base",
  "description": "Base configuration for Cisco IOS devices",
  "content": "! Base configuration\nhostname {{ node.name }}\n...",
  "template_type": "jinja2",
  "variables": {
    "management_vlan": {
      "type": "integer",
      "default": 100,
      "description": "Management VLAN ID"
    }
  }
}
```

### 9.3 Get Template

```http
GET /api/v1/templates/{template_id}
Authorization: Bearer <token>
```

### 9.4 Update Template

```http
PUT /api/v1/templates/{template_id}
Authorization: Bearer <token>
Content-Type: application/json

{
  "description": "Updated description",
  "content": "! Updated configuration\nhostname {{ node.name }}\n..."
}
```

### 9.5 Delete Template

```http
DELETE /api/v1/templates/{template_id}
Authorization: Bearer <token>
```

### 9.6 Render Template

```http
POST /api/v1/templates/render
Authorization: Bearer <token>
Content-Type: application/json

{
  "template_id": "uuid",
  "node_id": "uuid",
  "variables": {
    "management_vlan": 200
  }
}
```

**Response:**

```json
{
  "data": {
    "rendered_content": "! Base configuration\nhostname switch-01\ninterface Vlan200\n...",
    "template_name": "cisco-ios-base",
    "node_name": "switch-01",
    "variables_used": {
      "management_vlan": 200,
      "node.name": "switch-01"
    }
  },
  "success": true
}
```

### 9.7 Validate Template

```http
POST /api/v1/templates/{template_id}/validate
Authorization: Bearer <token>
Content-Type: application/json

{
  "test_variables": {
    "management_vlan": 100
  }
}
```

### 9.8 Get Template Usage

```http
GET /api/v1/templates/{template_id}/usage
Authorization: Bearer <token>
```

### 9.9 Template Assignments

#### Create Template Assignment

```http
POST /api/v1/template-assignments
Authorization: Bearer <token>
Content-Type: application/json

{
  "template_id": "uuid",
  "node_id": "uuid",
  "variables": {
    "management_vlan": 150
  },
  "priority": 10
}
```

#### Update Template Assignment

```http
PUT /api/v1/template-assignments/{assignment_id}
Authorization: Bearer <token>
Content-Type: application/json

{
  "variables": {
    "management_vlan": 200
  },
  "priority": 15
}
```

#### Delete Template Assignment

```http
DELETE /api/v1/template-assignments/{assignment_id}
Authorization: Bearer <token>
```

#### Get Template Assignments for Node

```http
GET /api/v1/nodes/{node_id}/template-assignments
Authorization: Bearer <token>
```

#### Get Template Assignments for Template

```http
GET /api/v1/templates/{template_id}/assignments
Authorization: Bearer <token>
```

---

## 10. Git Integration

### 10.1 Git Sync Status

```http
GET /api/v1/git/sync/status
Authorization: Bearer <token>
```

**Response:**

```json
{
  "data": {
    "last_sync": "2024-01-01T12:00:00Z",
    "status": "success",
    "policies_repo": {
      "url": "https://github.com/org/policies.git",
      "branch": "main",
      "last_commit": "abc123def456",
      "last_sync": "2024-01-01T12:00:00Z"
    },
    "templates_repo": {
      "url": "https://github.com/org/templates.git",
      "branch": "main",
      "last_commit": "def456abc123",
      "last_sync": "2024-01-01T12:00:00Z"
    }
  },
  "success": true
}
```

### 10.2 Trigger Git Sync

```http
POST /api/v1/git/sync
Authorization: Bearer <token>
Content-Type: application/json

{
  "force": false,
  "repositories": ["policies", "templates"]
}
```

### 10.3 Get Change History

```http
GET /api/v1/git/changes?since=2024-01-01T00:00:00Z&limit=50
Authorization: Bearer <token>
```

### 10.4 Get Change Details

```http
GET /api/v1/git/changes/{change_id}
Authorization: Bearer <token>
```

### 10.5 Get Repository Info

```http
GET /api/v1/git/repository?repo=policies
Authorization: Bearer <token>
```

### 10.6 Git Webhooks

#### Handle Git Webhook

```http
POST /api/v1/git/webhooks
Content-Type: application/json
X-GitHub-Event: push

{
  "ref": "refs/heads/main",
  "repository": {
    "clone_url": "https://github.com/org/policies.git"
  }
}
```

#### Get Webhook Config

```http
GET /api/v1/git/webhooks/config
Authorization: Bearer <token>
```

#### Update Webhook Config

```http
PUT /api/v1/git/webhooks/config
Authorization: Bearer <token>
Content-Type: application/json

{
  "enabled": true,
  "secret": "webhook_secret",
  "auto_sync": true
}
```

---

## 11. Change Management

### 11.1 List Changes

```http
GET /api/v1/changes?status=pending&page=1&per_page=20
Authorization: Bearer <token>
```

**Query Parameters:**

- `status`: Filter by status (pending, approved, rejected, applied)
- `entity_type`: Filter by entity type (node, template, policy)
- `created_by`: Filter by creator
- `since`: Filter by creation date

### 11.2 Create Change

```http
POST /api/v1/changes
Authorization: Bearer <token>
Content-Type: application/json

{
  "title": "Update switch configuration",
  "description": "Update VLAN configuration for switch-01",
  "entity_type": "node",
  "entity_id": "uuid",
  "change_type": "update",
  "proposed_changes": {
    "custom_data": {
      "vlans": [100, 200, 300]
    }
  },
  "justification": "Adding VLANs for new department",
  "scheduled_for": "2024-01-01T20:00:00Z"
}
```

### 11.3 Get Change

```http
GET /api/v1/changes/{change_id}
Authorization: Bearer <token>
```

### 11.4 Approve Change

```http
POST /api/v1/changes/{change_id}/approve
Authorization: Bearer <token>
Content-Type: application/json

{
  "comment": "Approved after security review"
}
```

### 11.5 Reject Change

```http
POST /api/v1/changes/{change_id}/reject
Authorization: Bearer <token>
Content-Type: application/json

{
  "comment": "Rejected due to security concerns",
  "reason": "security_violation"
}
```

### 11.6 Apply Change

```http
POST /api/v1/changes/{change_id}/apply
Authorization: Bearer <token>
Content-Type: application/json

{
  "force": false,
  "dry_run": false
}
```

### 11.7 Rollback Change

```http
POST /api/v1/changes/{change_id}/rollback
Authorization: Bearer <token>
Content-Type: application/json

{
  "comment": "Rolling back due to issues"
}
```

### 11.8 Get Change Audit Trail

```http
GET /api/v1/changes/{change_id}/audit
Authorization: Bearer <token>
```

### 11.9 Get Change History

```http
GET /api/v1/changes/history/{entity_type}/{entity_id}
Authorization: Bearer <token>
```

### 11.10 Get Pending Approvals

```http
GET /api/v1/changes/pending?assigned_to=me
Authorization: Bearer <token>
```

### 11.11 Get Change Statistics

```http
GET /api/v1/changes/stats?period=30d
Authorization: Bearer <token>
```

### 11.12 Get Change Management Status

```http
GET /api/v1/changes/status
Authorization: Bearer <token>
```

---

## 12. Certificate Management

### 12.1 Get Certificate Status

```http
GET /api/v1/certificates/status
Authorization: Bearer <token>
```

**Response:**

```json
{
  "data": {
    "certificates": [
      {
        "name": "server.crt",
        "subject": "CN=unet.example.com",
        "issuer": "Let's Encrypt",
        "not_before": "2024-01-01T00:00:00Z",
        "not_after": "2024-04-01T00:00:00Z",
        "days_until_expiry": 45,
        "status": "valid"
      }
    ],
    "overall_status": "healthy"
  },
  "success": true
}
```

### 12.2 Rotate Certificates

```http
POST /api/v1/certificates/rotate
Authorization: Bearer <token>
Content-Type: application/json

{
  "certificate_names": ["server.crt"],
  "force": false,
  "backup": true
}
```

### 12.3 Backup Certificates

```http
POST /api/v1/certificates/backup
Authorization: Bearer <token>
```

### 12.4 Get Certificate Expiration

```http
GET /api/v1/certificates/expiration?days=30
Authorization: Bearer <token>
```

### 12.5 Certificate Health Check

```http
GET /api/v1/certificates/health
Authorization: Bearer <token>
```

---

## 13. Alerting

### 13.1 Get Alerts

```http
GET /api/v1/alerts?status=active&severity=critical&page=1&per_page=20
Authorization: Bearer <token>
```

**Query Parameters:**

- `status`: Filter by status (active, acknowledged, resolved)
- `severity`: Filter by severity (critical, warning, info)
- `since`: Filter by creation date
- `node_id`: Filter by specific node

### 13.2 Get Alert

```http
GET /api/v1/alerts/{alert_id}
Authorization: Bearer <token>
```

### 13.3 Acknowledge Alert

```http
POST /api/v1/alerts/{alert_id}/acknowledge
Authorization: Bearer <token>
Content-Type: application/json

{
  "comment": "Investigating the issue"
}
```

### 13.4 Resolve Alert

```http
POST /api/v1/alerts/{alert_id}/resolve
Authorization: Bearer <token>
Content-Type: application/json

{
  "comment": "Issue has been resolved",
  "resolution": "configuration_updated"
}
```

### 13.5 Manual Escalate Alert

```http
POST /api/v1/alerts/{alert_id}/escalate
Authorization: Bearer <token>
Content-Type: application/json

{
  "escalation_level": 2,
  "comment": "Escalating due to continued issues"
}
```

### 13.6 Alert Rules

#### Get Alert Rules

```http
GET /api/v1/alert-rules
Authorization: Bearer <token>
```

#### Create Alert Rule

```http
POST /api/v1/alert-rules
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "High CPU Usage",
  "description": "Alert when CPU usage exceeds 90%",
  "condition": "cpu_usage > 90",
  "severity": "warning",
  "notification_channels": ["email", "slack"],
  "enabled": true,
  "evaluation_interval": "5m"
}
```

#### Get Alert Rule

```http
GET /api/v1/alert-rules/{rule_id}
Authorization: Bearer <token>
```

#### Update Alert Rule

```http
PUT /api/v1/alert-rules/{rule_id}
Authorization: Bearer <token>
Content-Type: application/json

{
  "condition": "cpu_usage > 85",
  "severity": "critical"
}
```

#### Delete Alert Rule

```http
DELETE /api/v1/alert-rules/{rule_id}
Authorization: Bearer <token>
```

### 13.7 Notification Channels

#### Get Notification Channels

```http
GET /api/v1/notification-channels
Authorization: Bearer <token>
```

#### Create Notification Channel

```http
POST /api/v1/notification-channels
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "ops-team-slack",
  "type": "slack",
  "configuration": {
    "webhook_url": "https://hooks.slack.com/...",
    "channel": "#ops-alerts"
  },
  "enabled": true
}
```

### 13.8 Test Notifications

```http
POST /api/v1/notifications/test
Authorization: Bearer <token>
Content-Type: application/json

{
  "channel_id": "uuid",
  "message": "Test notification from μNet"
}
```

### 13.9 Escalation Policies

#### Get Escalation Policies

```http
GET /api/v1/escalation/policies
Authorization: Bearer <token>
```

#### Create Escalation Policy

```http
POST /api/v1/escalation/policies
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "critical-escalation",
  "description": "Escalation policy for critical alerts",
  "levels": [
    {
      "level": 1,
      "delay_minutes": 0,
      "notification_channels": ["ops-email"]
    },
    {
      "level": 2,
      "delay_minutes": 15,
      "notification_channels": ["manager-email", "ops-slack"]
    }
  ]
}
```

### 13.10 Get Escalation Stats

```http
GET /api/v1/escalation/stats?period=7d
Authorization: Bearer <token>
```

### 13.11 Alerting Configuration

#### Get Alerting Config

```http
GET /api/v1/alerting/config
Authorization: Bearer <token>
```

#### Update Alerting Config

```http
PUT /api/v1/alerting/config
Authorization: Bearer <token>
Content-Type: application/json

{
  "global_enabled": true,
  "default_severity": "warning",
  "auto_resolve_timeout": "24h",
  "max_escalation_level": 3
}
```

### 13.12 Get Alert Statistics

```http
GET /api/v1/alerting/stats?period=30d&group_by=severity
Authorization: Bearer <token>
```

---

## 14. Network Access Control

### 14.1 Get Network Config

```http
GET /api/v1/network-access/config
Authorization: Bearer <token>
```

### 14.2 Update Network Config

```http
PUT /api/v1/network-access/config
Authorization: Bearer <token>
Content-Type: application/json

{
  "enabled": true,
  "allow_list": ["192.168.1.0/24", "10.0.0.0/8"],
  "deny_list": ["192.168.1.100"],
  "blocked_countries": ["CN", "RU"],
  "enable_geolocation": true,
  "rate_limits": {
    "requests_per_minute": 100,
    "burst_size": 20
  }
}
```

### 14.3 Get Network Stats

```http
GET /api/v1/network-access/stats?period=24h
Authorization: Bearer <token>
```

### 14.4 Test Network Access

```http
POST /api/v1/network-access/test
Authorization: Bearer <token>
Content-Type: application/json

{
  "ip_address": "192.168.1.50",
  "user_agent": "test-client/1.0"
}
```

### 14.5 Get Blocked IPs

```http
GET /api/v1/network-access/blocked-ips?page=1&per_page=50
Authorization: Bearer <token>
```

### 14.6 Block IP

```http
POST /api/v1/network-access/block/{ip_address}
Authorization: Bearer <token>
Content-Type: application/json

{
  "reason": "Suspicious activity detected",
  "duration": "24h"
}
```

### 14.7 Unblock IP

```http
DELETE /api/v1/network-access/unblock/{ip_address}
Authorization: Bearer <token>
```

### 14.8 Get Network Access Health

```http
GET /api/v1/network-access/health
Authorization: Bearer <token>
```

---

## 15. Distributed Locking

### 15.1 Get Lock Stats

```http
GET /api/v1/locks/stats
Authorization: Bearer <token>
```

### 15.2 List Locks

```http
GET /api/v1/locks?status=active&page=1&per_page=20
Authorization: Bearer <token>
```

### 15.3 Acquire Lock

```http
POST /api/v1/locks/acquire
Authorization: Bearer <token>
Content-Type: application/json

{
  "key": "config-update-node-123",
  "owner": "user-456",
  "ttl": 300,
  "metadata": {
    "operation": "configuration_update",
    "node_id": "123"
  }
}
```

### 15.4 Release Lock

```http
DELETE /api/v1/locks/{lock_key}
Authorization: Bearer <token>
```

### 15.5 Get Lock Info

```http
GET /api/v1/locks/{lock_key}
Authorization: Bearer <token>
```

### 15.6 Extend Lock

```http
POST /api/v1/locks/{lock_key}/extend
Authorization: Bearer <token>
Content-Type: application/json

{
  "ttl": 600
}
```

### 15.7 Leader Election

#### Create Leader Election

```http
POST /api/v1/locks/leader-election
Authorization: Bearer <token>
Content-Type: application/json

{
  "election_key": "primary-config-manager",
  "candidate_id": "node-123",
  "ttl": 60
}
```

#### Get Leader Election Status

```http
GET /api/v1/locks/leader-election/{election_key}/status
Authorization: Bearer <token>
```

### 15.8 Get Lock Monitor Report

```http
GET /api/v1/locks/monitor
Authorization: Bearer <token>
```

### 15.9 Lock Configuration

#### Get Lock Config

```http
GET /api/v1/locks/config
Authorization: Bearer <token>
```

#### Update Lock Config

```http
PUT /api/v1/locks/config
Authorization: Bearer <token>
Content-Type: application/json

{
  "default_ttl": 300,
  "max_ttl": 3600,
  "cleanup_interval": 60,
  "deadlock_detection_enabled": true
}
```

### 15.10 Get Lock Health

```http
GET /api/v1/locks/health
Authorization: Bearer <token>
```

### 15.11 Test Distributed Locking

```http
POST /api/v1/locks/test
Authorization: Bearer <token>
Content-Type: application/json

{
  "test_type": "basic_lock",
  "parameters": {
    "lock_count": 10,
    "hold_time": 5
  }
}
```

---

## 16. Metrics and Monitoring

### 16.1 Get Prometheus Metrics

```http
GET /metrics
```

**Response:** Prometheus format metrics

### 16.2 Get System Health

```http
GET /api/v1/metrics/health
Authorization: Bearer <token>
```

### 16.3 Get Performance Metrics

```http
GET /api/v1/metrics/performance?period=1h&granularity=5m
Authorization: Bearer <token>
```

### 16.4 Get Business Metrics

```http
GET /api/v1/metrics/business?metrics=nodes_managed,policies_evaluated&period=24h
Authorization: Bearer <token>
```

### 16.5 Get Metrics Config

```http
GET /api/v1/metrics/config
Authorization: Bearer <token>
```

### 16.6 Query Metrics

```http
GET /api/v1/metrics/query?metric=cpu_usage&node_id=uuid&start=2024-01-01T00:00:00Z&end=2024-01-01T23:59:59Z
Authorization: Bearer <token>
```

---

## 17. Performance Management

### 17.1 Get Performance Metrics

```http
GET /api/v1/performance/metrics?operation=all&period=1h
Authorization: Bearer <token>
```

### 17.2 Get Operation Metrics

```http
GET /api/v1/performance/metrics/{operation}?period=24h
Authorization: Bearer <token>
```

### 17.3 Get Performance Report

```http
GET /api/v1/performance/report?format=summary&period=7d
Authorization: Bearer <token>
```

### 17.4 Reset Performance Metrics

```http
POST /api/v1/performance/metrics/reset
Authorization: Bearer <token>
Content-Type: application/json

{
  "operations": ["node_creation", "policy_evaluation"],
  "confirm": true
}
```

### 17.5 Get Cache Stats

```http
GET /api/v1/performance/cache/stats
Authorization: Bearer <token>
```

### 17.6 Clear Cache

```http
POST /api/v1/performance/cache/clear
Authorization: Bearer <token>
Content-Type: application/json

{
  "cache_types": ["policy", "template"],
  "force": false
}
```

### 17.7 Run Benchmark

```http
POST /api/v1/performance/benchmark
Authorization: Bearer <token>
Content-Type: application/json

{
  "benchmark_type": "policy_evaluation",
  "parameters": {
    "node_count": 100,
    "policy_count": 10,
    "iterations": 5
  }
}
```

### 17.8 Get Benchmark Templates

```http
GET /api/v1/performance/benchmark/templates
Authorization: Bearer <token>
```

### 17.9 Get Performance Status

```http
GET /api/v1/performance/status
Authorization: Bearer <token>
```

### 17.10 Get Optimization Recommendations

```http
GET /api/v1/performance/recommendations?category=all
Authorization: Bearer <token>
```

---

## 18. Cluster Coordination

### 18.1 Get Cluster Stats

```http
GET /api/v1/cluster/stats
Authorization: Bearer <token>
```

### 18.2 Get Cluster Health

```http
GET /api/v1/cluster/health
Authorization: Bearer <token>
```

### 18.3 List Cluster Nodes

```http
GET /api/v1/cluster/nodes?status=active
Authorization: Bearer <token>
```

### 18.4 Register Node

```http
POST /api/v1/cluster/nodes
Authorization: Bearer <token>
Content-Type: application/json

{
  "node_id": "node-123",
  "node_type": "worker",
  "address": "192.168.1.10:8080",
  "capabilities": ["policy_evaluation", "template_rendering"],
  "metadata": {
    "zone": "us-west-1a",
    "instance_type": "c5.large"
  }
}
```

### 18.5 Get Node Details

```http
GET /api/v1/cluster/nodes/{node_id}
Authorization: Bearer <token>
```

### 18.6 Remove Node

```http
DELETE /api/v1/cluster/nodes/{node_id}
Authorization: Bearer <token>
```

### 18.7 Update Node Metrics

```http
POST /api/v1/cluster/nodes/metrics
Authorization: Bearer <token>
Content-Type: application/json

{
  "node_id": "node-123",
  "metrics": {
    "cpu_usage": 45.2,
    "memory_usage": 60.8,
    "active_connections": 150,
    "requests_per_second": 25.5
  }
}
```

### 18.8 Cluster Configuration

#### Get Cluster Config

```http
GET /api/v1/cluster/config
Authorization: Bearer <token>
```

#### Update Cluster Config

```http
PUT /api/v1/cluster/config
Authorization: Bearer <token>
Content-Type: application/json

{
  "auto_scaling_enabled": true,
  "min_nodes": 2,
  "max_nodes": 10,
  "health_check_interval": 30,
  "node_timeout": 300
}
```

### 18.9 Scaling

#### Get Scaling Recommendation

```http
POST /api/v1/cluster/scaling/recommendation
Authorization: Bearer <token>
Content-Type: application/json

{
  "metrics": {
    "current_load": 75.5,
    "response_time": 250,
    "queue_depth": 50
  },
  "time_horizon": "1h"
}
```

#### Trigger Scaling Action

```http
POST /api/v1/cluster/scaling/action
Authorization: Bearer <token>
Content-Type: application/json

{
  "action": "scale_up",
  "target_nodes": 5,
  "reason": "High load detected"
}
```

#### Get Scaling History

```http
GET /api/v1/cluster/scaling/history?period=7d
Authorization: Bearer <token>
```

---

## 19. Resource Management

### 19.1 Get Resource Status

```http
GET /api/v1/resource-management/status
Authorization: Bearer <token>
```

### 19.2 Memory Management

#### Get Memory Status

```http
GET /api/v1/resource-management/memory/status
Authorization: Bearer <token>
```

#### Optimize Memory

```http
POST /api/v1/resource-management/memory/optimize
Authorization: Bearer <token>
Content-Type: application/json

{
  "aggressive": false,
  "target_usage": 70.0
}
```

### 19.3 Resource Limits

#### Get Limits Status

```http
GET /api/v1/resource-management/limits/status
Authorization: Bearer <token>
```

#### Update Limits

```http
PUT /api/v1/resource-management/limits
Authorization: Bearer <token>
Content-Type: application/json

{
  "memory_limit": "4GB",
  "cpu_limit": "2.0",
  "max_connections": 500,
  "request_timeout": "30s"
}
```

### 19.4 Graceful Degradation

#### Get Degradation Status

```http
GET /api/v1/resource-management/degradation/status
Authorization: Bearer <token>
```

#### Trigger Emergency Mode

```http
POST /api/v1/resource-management/emergency
Authorization: Bearer <token>
Content-Type: application/json

{
  "level": "high",
  "reason": "Memory usage critical",
  "duration": "1h"
}
```

### 19.5 Resource Monitoring

#### Get Monitoring Metrics

```http
GET /api/v1/resource-management/metrics?period=1h&granularity=5m
Authorization: Bearer <token>
```

#### Get Resource Alerts

```http
GET /api/v1/resource-management/alerts?status=active
Authorization: Bearer <token>
```

#### Acknowledge Alert

```http
POST /api/v1/resource-management/alerts/{alert_id}/acknowledge
Authorization: Bearer <token>
Content-Type: application/json

{
  "comment": "Investigating high memory usage"
}
```

### 19.6 Capacity Planning

#### Get Capacity Recommendations

```http
GET /api/v1/resource-management/capacity/recommendations?forecast_period=30d
Authorization: Bearer <token>
```

### 19.7 Resource Configuration

#### Get Resource Config

```http
GET /api/v1/resource-management/config
Authorization: Bearer <token>
```

#### Update Resource Config

```http
PUT /api/v1/resource-management/config
Authorization: Bearer <token>
Content-Type: application/json

{
  "memory_optimization_enabled": true,
  "auto_gc_enabled": true,
  "resource_limits_enabled": true,
  "degradation_thresholds": {
    "memory_warning": 80.0,
    "memory_critical": 95.0,
    "cpu_warning": 85.0,
    "cpu_critical": 95.0
  }
}
```

### 19.8 Resource Health Check

```http
GET /api/v1/resource-management/health
Authorization: Bearer <token>
```

---

## 20. Health Checks

### 20.1 Basic Health Check

```http
GET /health
```

**Response:**

```json
{
  "status": "healthy",
  "timestamp": "2024-01-01T12:00:00Z"
}
```

### 20.2 Detailed Health Check

```http
GET /health/detailed
```

**Response:**

```json
{
  "status": "healthy",
  "timestamp": "2024-01-01T12:00:00Z",
  "components": {
    "database": {
      "status": "healthy",
      "response_time": 5.2,
      "connections": 10
    },
    "git_repos": {
      "status": "healthy",
      "last_sync": "2024-01-01T11:45:00Z"
    },
    "snmp_polling": {
      "status": "healthy",
      "active_polls": 250,
      "success_rate": 98.5
    }
  }
}
```

### 20.3 Readiness Check

```http
GET /ready
```

**Response:**

```json
{
  "status": "ready",
  "timestamp": "2024-01-01T12:00:00Z"
}
```

### 20.4 Liveness Check

```http
GET /live
```

**Response:**

```json
{
  "status": "live",
  "timestamp": "2024-01-01T12:00:00Z"
}
```

### 20.5 Load Balancer Status

```http
GET /health/lb
```

**Response:**

```json
{
  "status": "healthy",
  "ready_for_traffic": true,
  "timestamp": "2024-01-01T12:00:00Z"
}
```

---

## Rate Limiting

All API endpoints are subject to rate limiting:

- **Default Limit:** 1000 requests per hour per user
- **Burst Limit:** 100 requests per minute
- **Headers:**
  - `X-RateLimit-Limit`: Total limit
  - `X-RateLimit-Remaining`: Remaining requests
  - `X-RateLimit-Reset`: Reset timestamp

---

## WebSocket Connections

For real-time updates, μNet supports WebSocket connections:

```javascript
const ws = new WebSocket('wss://your-unet-instance.com/ws');

// Subscribe to node status updates
ws.send(JSON.stringify({
  type: 'subscribe',
  topics: ['node_status', 'alerts']
}));
```

---

## SDK and Client Libraries

Official SDKs are available for:

- **Python**: `pip install unet-client`
- **JavaScript/TypeScript**: `npm install @unet/client`
- **Go**: `go get github.com/org/unet-go-client`
- **Rust**: `cargo add unet-client`

---

## Changelog and Versioning

API versioning follows semantic versioning:

- Current version: `v1`
- Breaking changes will increment major version
- New features increment minor version
- Bug fixes increment patch version

For the latest API changes, see the [CHANGELOG.md](https://github.com/org/unet/blob/main/CHANGELOG.md).

---

## Support and Resources

- **GitHub Issues**: <https://github.com/org/unet/issues>
- **Documentation**: <https://docs.unet.example.com>
- **API Status**: <https://status.unet.example.com>
- **Community**: <https://community.unet.example.com>

---

**Note**: This API reference is auto-generated from the OpenAPI specification. For the most up-to-date information, refer to the live API documentation at `/api/docs` when running μNet server.
