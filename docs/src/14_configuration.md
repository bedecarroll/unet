<!-- SPDX-License-Identifier: MIT -->

# 14 Configuration Management – Runtime Options & Deployment

> **Audience:** System administrators, DevOps engineers, and developers deploying μNet  
> **Purpose:** Comprehensive guide to configuring μNet server and CLI for production deployment  
> **Scope:** Runtime configuration options, file formats, environment variables, and deployment patterns

---

## Table of Contents

1. [Configuration Overview](#1-configuration-overview)
2. [Configuration File Format](#2-configuration-file-format)
3. [Core Configuration Sections](#3-core-configuration-sections)
4. [Server Configuration](#4-server-configuration)
5. [CLI Configuration](#5-cli-configuration)
6. [Environment Variables](#6-environment-variables)
7. [Git Repository Integration](#7-git-repository-integration)
8. [Security Considerations](#8-security-considerations)
9. [Deployment Examples](#9-deployment-examples)
10. [Troubleshooting](#10-troubleshooting)

---

## 1. Configuration Overview

μNet uses a hierarchical configuration system supporting multiple sources with precedence ordering:

1. **Command-line flags** (highest precedence)
2. **Environment variables** (`UNET_*` prefix)
3. **Configuration files** (TOML format)
4. **Built-in defaults** (lowest precedence)

### Configuration File Locations

μNet searches for configuration files in the following order:

```bash
# Server configuration search paths
./unet-server.toml          # Current directory
./config/unet-server.toml   # Config subdirectory
~/.config/unet/server.toml  # User config directory
/etc/unet/server.toml       # System-wide configuration

# CLI configuration search paths  
./unet-cli.toml             # Current directory
./config/unet-cli.toml      # Config subdirectory
~/.config/unet/cli.toml     # User config directory
/etc/unet/cli.toml          # System-wide configuration
```

---

## 2. Configuration File Format

μNet uses TOML (Tom's Obvious Minimal Language) for configuration files. Here's the complete structure:

```toml
# /etc/unet/server.toml - Complete μNet Server Configuration

[database]
url = "sqlite:./unet.db?mode=rwc"
max_connections = 10
timeout_secs = 30

[logging]
level = "info"                    # trace, debug, info, warn, error
format = "pretty"                 # pretty, json, compact
file = "/var/log/unet/server.log" # Optional log file
max_size_mb = 100                 # Log rotation size
max_files = 5                     # Number of rotated files

[server]
host = "0.0.0.0"
port = 8080
max_request_size = 1048576        # 1MB in bytes
tls_cert_path = "/etc/unet/tls/cert.pem"     # Optional TLS
tls_key_path = "/etc/unet/tls/key.pem"      # Optional TLS
cors_origins = ["https://dashboard.corp.local"]
worker_threads = 4                # Optional thread pool size

[snmp]
community = "public"
timeout_secs = 5
retries = 3
max_connections = 100
pool_timeout_secs = 30

[git]
policies_repo = "https://github.com/corp/unet-policies.git"
templates_repo = "https://github.com/corp/unet-templates.git"
branch = "main"
sync_interval_secs = 900          # 15 minutes
credentials_path = "/etc/unet/git-credentials"
clone_timeout_secs = 300
shallow_clone = true              # Performance optimization

[domains]
default = "corp.example.com"
search_domains = [
    "corp.example.com",
    "lab.example.com", 
    "mgmt.example.com"
]

[auth]
method = "none"                   # none, token, oidc (future)
token_file = "/etc/unet/api-token"
session_timeout_secs = 3600

[policy]
evaluation_interval_secs = 300    # 5 minutes
max_concurrent_evaluations = 10
enable_dry_run = false
cache_size = 1000
timeout_secs = 30

[templates]
cache_size = 100
render_timeout_secs = 30
allow_unsafe_functions = false
template_dirs = ["/etc/unet/templates"]

[monitoring]
enable_metrics = true
metrics_port = 9090
health_check_path = "/health"
```

---

## 3. Core Configuration Sections

### 3.1 Database Configuration

Controls how μNet connects to and manages the database:

```toml
[database]
# SQLite (default for development)
url = "sqlite:./unet.db?mode=rwc"

# PostgreSQL (recommended for production)
# url = "postgresql://unet:password@localhost:5432/unet"

max_connections = 10      # Connection pool size
timeout_secs = 30        # Query timeout
```

**Environment Variable Override:**

```bash
export UNET_DATABASE__URL="postgresql://unet:secret@db.corp.local:5432/unet"
export UNET_DATABASE__MAX_CONNECTIONS=20
```

### 3.2 Logging Configuration

Controls log output format, level, and destinations:

```toml
[logging]
level = "info"           # trace, debug, info, warn, error
format = "json"          # pretty (development), json (production)
file = "/var/log/unet/server.log"
max_size_mb = 100        # Rotation threshold
max_files = 5            # Keep 5 rotated files
```

**Log Level Guidelines:**

- **trace**: Extremely verbose, debugging only
- **debug**: Function entry/exit, variable values
- **info**: Normal operation events (default)
- **warn**: Unusual but recoverable conditions
- **error**: Error conditions requiring attention

### 3.3 SNMP Configuration

Controls SNMP client behavior for device polling:

```toml
[snmp]
community = "public"              # Default SNMP community
timeout_secs = 5                 # Per-request timeout
retries = 3                      # Retry attempts
max_connections = 100            # Connection pool size
pool_timeout_secs = 30           # Pool checkout timeout
```

**Per-Device Overrides:**
SNMP settings can be overridden per-device using the `custom_data` field:

```json
{
  "snmp": {
    "community": "private",
    "timeout_secs": 10,
    "version": "2c"
  }
}
```

---

## 4. Server Configuration

### 4.1 HTTP Server Settings

```toml
[server]
host = "0.0.0.0"                 # Bind address (0.0.0.0 for all interfaces)
port = 8080                      # HTTP port
max_request_size = 1048576       # 1MB request size limit
worker_threads = 4               # Optional: defaults to CPU count
```

### 4.2 TLS Configuration (Optional)

For HTTPS deployment:

```toml
[server]
tls_cert_path = "/etc/unet/tls/cert.pem"
tls_key_path = "/etc/unet/tls/key.pem"
```

**Certificate Requirements:**

- PEM format certificates
- Include full certificate chain
- Private key must be readable by μNet user
- Consider automated renewal (Let's Encrypt, cert-manager)

### 4.3 CORS Configuration

For web dashboard integration:

```toml
[server]
cors_origins = [
    "https://dashboard.corp.local",
    "https://monitoring.corp.local"
]
```

---

## 5. CLI Configuration

### 5.1 CLI Global Settings

```toml
# ~/.config/unet/cli.toml
[client]
server_url = "https://unet.corp.local:8080"
timeout_secs = 30
output_format = "table"          # table, json, yaml
auth_token_file = "~/.unet/token"

[output]
color = "auto"                   # auto, always, never
verbose = false
max_width = 120                  # Table formatting
```

### 5.2 CLI Command-Line Flags

All configuration options can be overridden via command-line flags:

```bash
# Global flags (available on all commands)
unet --server https://unet.corp.local:8080 \
     --config /custom/config.toml \
     --token-file /secure/token \
     --output json \
     --verbose \
     nodes list

# Server selection shorthand
unet --server prod nodes list    # Uses predefined server alias
unet --server dev nodes list     # Uses development server
```

### 5.3 Server Aliases

Define server shortcuts in CLI configuration:

```toml
[servers]
prod = "https://unet.corp.local:8080"
dev = "https://unet-dev.corp.local:8080"
local = "http://localhost:8080"
```

---

## 6. Environment Variables

All configuration options support environment variable overrides using the `UNET_` prefix:

### 6.1 Variable Naming Convention

```bash
# Section__Key format (double underscore)
UNET_DATABASE__URL="sqlite:./test.db"
UNET_LOGGING__LEVEL="debug"
UNET_SERVER__PORT="9080"
UNET_SNMP__COMMUNITY="private"

# Array values use comma separation
UNET_DOMAINS__SEARCH_DOMAINS="corp.local,lab.local"
UNET_SERVER__CORS_ORIGINS="https://dash1.local,https://dash2.local"
```

### 6.2 Docker Environment File

For containerized deployment:

```bash
# /etc/unet/env
UNET_DATABASE__URL=postgresql://unet:${DB_PASSWORD}@postgres:5432/unet
UNET_LOGGING__FORMAT=json
UNET_LOGGING__LEVEL=info
UNET_SERVER__HOST=0.0.0.0
UNET_SERVER__PORT=8080
UNET_GIT__POLICIES_REPO=https://github.com/corp/policies.git
UNET_GIT__TEMPLATES_REPO=https://github.com/corp/templates.git
UNET_DOMAINS__DEFAULT=corp.local
```

---

## 7. Git Repository Integration

### 7.1 Repository Configuration

```toml
[git]
policies_repo = "https://github.com/corp/unet-policies.git"
templates_repo = "https://github.com/corp/unet-templates.git"
branch = "main"
sync_interval_secs = 900         # 15 minutes
credentials_path = "/etc/unet/git-credentials"
clone_timeout_secs = 300
shallow_clone = true             # Performance: only latest commit
```

### 7.2 Git Credentials

**Option 1: Git Credentials File**

```bash
# /etc/unet/git-credentials
https://username:token@github.com
```

**Option 2: SSH Key Authentication**

```toml
[git]
policies_repo = "git@github.com:corp/unet-policies.git"
ssh_key_path = "/etc/unet/ssh/id_rsa"
```

**Option 3: Environment Variables**

```bash
export GIT_USERNAME="automation"
export GIT_TOKEN="ghp_xxxxxxxxxxxx"
```

### 7.3 Repository Structure

Expected repository layout:

```
unet-policies/
├── global/
│   ├── dns-compliance.rules
│   └── security-baseline.rules
├── environments/
│   ├── production/
│   └── development/
└── vendor-specific/
    ├── juniper/
    └── cisco/

unet-templates/
├── interfaces/
│   ├── ethernet.j2
│   └── loopback.j2
├── protocols/
│   ├── bgp.j2
│   └── ospf.j2
└── vendor/
    ├── juniper/
    └── cisco/
```

---

## 8. Security Considerations

### 8.1 File Permissions

```bash
# Configuration files should be readable only by unet user
chmod 600 /etc/unet/server.toml
chown unet:unet /etc/unet/server.toml

# Credential files should be highly restricted
chmod 400 /etc/unet/git-credentials
chmod 400 /etc/unet/api-token
```

### 8.2 Credential Management

**Best Practices:**

- Use dedicated service accounts for git repositories
- Rotate API tokens regularly
- Store sensitive values in environment variables, not config files
- Use secret management systems (Vault, k8s secrets) in production

**Example with External Secret Management:**

```toml
[database]
url = "postgresql://unet@postgres:5432/unet"  # No password in config

[git]
policies_repo = "https://github.com/corp/policies.git"
# Credentials injected via environment: GIT_TOKEN
```

### 8.3 Network Security

```toml
[server]
host = "127.0.0.1"              # Bind only to localhost for security
tls_cert_path = "/etc/tls/cert.pem"  # Always use TLS in production
cors_origins = ["https://trusted-dashboard.corp.local"]  # Restrict CORS
```

---

## 9. Deployment Examples

### 9.1 Development Environment

**Local Development:**

```toml
# ./unet-dev.toml
[database]
url = "sqlite:./dev.db"

[logging]
level = "debug"
format = "pretty"

[server]
host = "127.0.0.1"
port = 8080

[git]
policies_repo = "file:///path/to/local/policies"
templates_repo = "file:///path/to/local/templates"
sync_interval_secs = 60  # Fast sync for development
```

```bash
# Start development server
cargo run --bin unet-server -- --config ./unet-dev.toml
```

### 9.2 Production Deployment

**Production Configuration:**

```toml
# /etc/unet/server.toml
[database]
url = "postgresql://unet@postgres.corp.local:5432/unet"
max_connections = 20

[logging]
level = "info"
format = "json"
file = "/var/log/unet/server.log"

[server]
host = "0.0.0.0"
port = 8080
tls_cert_path = "/etc/tls/unet.crt"
tls_key_path = "/etc/tls/unet.key"

[git]
policies_repo = "https://github.com/corp/unet-policies.git"
templates_repo = "https://github.com/corp/unet-templates.git"
credentials_path = "/etc/unet/git-credentials"
```

### 9.3 Kubernetes Deployment

**ConfigMap:**

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: unet-config
data:
  server.toml: |
    [database]
    url = "postgresql://unet@postgres:5432/unet"
    
    [logging]
    level = "info"
    format = "json"
    
    [server]
    host = "0.0.0.0"
    port = 8080
```

**Secret:**

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: unet-secrets
data:
  git-token: <base64-encoded-token>
  db-password: <base64-encoded-password>
```

### 9.4 Docker Compose

```yaml
version: '3.8'
services:
  unet-server:
    image: unet/server:latest
    ports:
      - "8080:8080"
    environment:
      - UNET_DATABASE__URL=postgresql://unet:${DB_PASSWORD}@postgres:5432/unet
      - UNET_LOGGING__FORMAT=json
      - UNET_GIT__POLICIES_REPO=https://github.com/corp/policies.git
    env_file:
      - /etc/unet/env
    volumes:
      - /etc/unet:/etc/unet:ro
      - /var/log/unet:/var/log/unet
    depends_on:
      - postgres
```

---

## 10. Troubleshooting

### 10.1 Configuration Validation

**Check configuration loading:**

```bash
# Test configuration parsing
unet-server --config /etc/unet/server.toml --validate-config

# Show effective configuration (with overrides applied)
unet-server --config /etc/unet/server.toml --show-config
```

### 10.2 Common Issues

**Database Connection Issues:**

```bash
# Check database URL format
UNET_LOGGING__LEVEL=debug unet-server

# Test database connectivity
psql "postgresql://unet@postgres:5432/unet" -c "SELECT 1;"
```

**Git Repository Issues:**

```bash
# Test git credentials
git clone https://github.com/corp/policies.git /tmp/test-clone

# Check git configuration
UNET_LOGGING__LEVEL=debug unet-server  # Look for git sync logs
```

**Permission Issues:**

```bash
# Check file permissions
ls -la /etc/unet/
stat /etc/unet/server.toml

# Check user context
id $(whoami)
sudo -u unet cat /etc/unet/server.toml
```

### 10.3 Configuration Precedence Debugging

Enable debug logging to see configuration loading:

```bash
UNET_LOGGING__LEVEL=debug unet-server 2>&1 | grep -i config
```

Expected log output:

```
DEBUG Loading configuration from file: /etc/unet/server.toml
DEBUG Applied environment override: DATABASE__URL
DEBUG Applied command-line override: --port 9080
DEBUG Final configuration loaded successfully
```

---

## Configuration Migration Guide

When upgrading μNet versions, configuration format may change. Use the migration tool:

```bash
# Backup current configuration
cp /etc/unet/server.toml /etc/unet/server.toml.backup

# Migrate to new format
unet-server --migrate-config \
  --from /etc/unet/server.toml.backup \
  --to /etc/unet/server.toml \
  --format v2
```

---

*This configuration guide covers μNet v0.1.0+. For the latest configuration options, see the built-in help: `unet-server --help` and `unet --help`.*
