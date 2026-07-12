# Small Office Network Topology

## Overview

This example demonstrates a typical small office network serving 35-50 users in a single building. The topology follows enterprise networking best practices scaled down for a small business environment.

## Network Architecture

### Physical Layout

- **Building**: Single-story office building (5,000 sq ft)
- **Server Room**: Dedicated space with environmental controls
- **Office Areas**: Main floor with workstations and conference rooms  
- **Reception**: Public area with guest network requirements

### Network Topology

```
Internet (100Mbps Fiber)
    │
    └── FortiGate 60F (firewall-01)
        │
        └── Cisco Catalyst 2960X-48 (core-sw-01)
            ├── Cisco Catalyst 2960X-24 (access-sw-01) [Main Floor]
            ├── Cisco Catalyst 2960X-24 (access-sw-02) [Reception]
            ├── Aruba 7005 Controller (wifi-controller-01)
            └── Dell PowerEdge R230 (server-01)
```

## Device Details

### Core Infrastructure

- **Firewall**: FortiGate 60F with UTM bundle
  - Handles internet security, VPN, and traffic filtering
  - 3Gbps throughput capacity
  - WAN IP: 203.0.113.10

- **Core Switch**: Cisco Catalyst 2960X-48FPD-L
  - 48x Gigabit PoE+ ports
  - Central aggregation point for all network traffic
  - VLAN routing and management

### Access Layer

- **Main Floor Switch**: 24-port access switch
  - Serves 35 workstations and 3 conference rooms
  - 18 of 24 ports currently in use
  - VLANs 10-50 for segmentation

- **Reception Switch**: 24-port access switch  
  - Serves reception area and guest network
  - 8 of 24 ports currently in use
  - Guest network isolation enabled

### Wireless Infrastructure

- **WiFi Controller**: Aruba 7005 Mobility Controller
  - Manages up to 16 access points (6 currently deployed)
  - 4 SSIDs: Corporate, Guest, IoT, Management
  - Foundation licensing

### Server Infrastructure  

- **Primary Server**: Dell PowerEdge R230
  - Windows Server 2022
  - Active Directory, File/Print services
  - 32GB RAM, 2TB storage
  - Daily backup to cloud

## Network Addressing

### Management Networks

- **Management VLAN**: 192.168.1.0/24
- **Server Network**: 192.168.10.0/24  
- **User Network**: 192.168.20.0/24
- **Guest Network**: 192.168.100.0/24

### IP Allocations

- Gateway/Firewall: 192.168.1.1
- Core Switch: 192.168.1.10
- Access Switches: 192.168.1.11-12
- WiFi Controller: 192.168.1.20
- Primary Server: 192.168.1.50

## Connectivity Details

### Uplinks

- **Internet**: 100Mbps fiber from Regional ISP
- **Core-to-Access**: Gigabit uplinks on ports 47-48
- **Server**: Direct gigabit connection to core
- **WiFi**: Management connection for controller

### Redundancy

- **Power**: UPS backup in server room
- **Internet**: Single circuit (backup planned)
- **Core**: Single switch (acceptable for this size)

## Custom Data Examples

This topology demonstrates various custom_data use cases:

### Asset Management

- Purchase dates and warranty information
- Serial numbers and model details
- Cost center assignments

### Physical Infrastructure  

- Rack positions and power requirements
- Cable types, lengths, and patch panel assignments
- Room assignments and access controls

### Operational Data

- Firmware versions and maintenance contacts
- Port utilization and VLAN assignments
- License information and expiry dates

### Business Context

- Monthly costs and SLA commitments
- Support contacts and escalation procedures
- Compliance and security requirements

## Growth Planning

This network can easily accommodate:

- **50% growth**: Additional ports available on existing switches
- **WiFi expansion**: Controller supports 10 more access points  
- **Server capacity**: Can add second server to existing rack
- **Internet upgrade**: Firewall supports up to 3Gbps

## Import Instructions

```bash
# Import this small office topology
unet import --from fixtures/examples/small-office/

# Verify the import
unet nodes list
unet locations list  
unet links list

# View specific device details
unet nodes show 6ba7b810-9dad-11d1-80b4-00c04fd430c8 --include-status
```

This topology serves as an excellent starting point for small business networks and demonstrates μNet's capabilities for managing enterprise-class infrastructure.
