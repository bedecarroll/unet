# Troubleshooting Guide and FAQ

> **Audience:** System administrators, DevOps engineers, and developers working with μNet.  
> **Purpose:** Comprehensive troubleshooting procedures and frequently asked questions.  
> **Prerequisites:** Basic understanding of μNet architecture and deployment methods.

---

## Table of Contents

1. [Quick Diagnostic Steps](#1-quick-diagnostic-steps)
2. [Common Issues and Solutions](#2-common-issues-and-solutions)
3. [Performance Issues](#3-performance-issues)
4. [Database Problems](#4-database-problems)
5. [Network and SNMP Issues](#5-network-and-snmp-issues)
6. [Authentication and Authorization Issues](#6-authentication-and-authorization-issues)
7. [Git Integration Problems](#7-git-integration-problems)
8. [Kubernetes-Specific Issues](#8-kubernetes-specific-issues)
9. [Frequently Asked Questions (FAQ)](#9-frequently-asked-questions-faq)
10. [Advanced Debugging](#10-advanced-debugging)

---

## 1. Quick Diagnostic Steps

### 1.1 Health Check Commands

```bash
# Basic health check
curl -s http://localhost:8080/health/live | jq
curl -s http://localhost:8080/health/ready | jq

# Check system status
unet status
unet nodes list --status=unhealthy

# Check logs
journalctl -u unet-server -f          # Systemd
kubectl logs deployment/unet-server   # Kubernetes
docker logs unet-server               # Docker
```

### 1.2 Configuration Validation

```bash
# Validate configuration file
unet config validate /etc/unet/config.toml

# Test database connection
unet db test-connection

# Verify Git repository access
unet git test-connection

# Check SNMP connectivity
unet snmp test --node=<node-id>
```

### 1.3 Resource Usage Check

```bash
# Check resource usage
htop
df -h
free -h

# Kubernetes resource check
kubectl top pods -n unet-system
kubectl describe pod <pod-name> -n unet-system
```

---

## 2. Common Issues and Solutions

### 2.1 Service Won't Start

#### Symptoms

- μNet server fails to start
- Exits immediately after starting
- Health checks fail

#### Diagnosis

```bash
# Check logs for startup errors
journalctl -u unet-server --no-pager

# Check configuration syntax
unet config validate

# Verify file permissions
ls -la /etc/unet/
ls -la /var/lib/unet/
```

#### Common Causes and Solutions

**Configuration File Issues:**

```bash
# Problem: Invalid TOML syntax
# Solution: Validate and fix configuration
unet config validate /etc/unet/config.toml

# Problem: Missing required fields
# Solution: Add missing configuration sections
[server]
bind = "0.0.0.0:8080"

[database]
type = "sqlite"
path = "/var/lib/unet/unet.db"
```

**Permission Issues:**

```bash
# Problem: Cannot access database or config files
# Solution: Fix file ownership
sudo chown -R unet:unet /var/lib/unet/
sudo chown root:unet /etc/unet/config.toml
sudo chmod 640 /etc/unet/config.toml
```

**Port Conflicts:**

```bash
# Problem: Port already in use
# Solution: Check and kill conflicting process
sudo netstat -tulpn | grep :8080
sudo kill <pid>

# Or change port in configuration
[server]
bind = "0.0.0.0:8081"
```

### 2.2 High Memory Usage

#### Symptoms

- Out of memory errors
- Slow performance
- Pod restarts in Kubernetes

#### Diagnosis

```bash
# Check memory usage
free -h
ps aux --sort=-%mem | head -20

# Kubernetes memory check
kubectl top pods -n unet-system
kubectl describe pod <pod-name> -n unet-system
```

#### Solutions

```bash
# Increase memory limits (Kubernetes)
kubectl patch deployment unet-server -n unet-system -p '{"spec":{"template":{"spec":{"containers":[{"name":"unet-server","resources":{"limits":{"memory":"4Gi"}}}]}}}}'

# Enable memory optimization in configuration
[performance]
enable_memory_optimization = true
cache_size_limit = "500MB"
gc_frequency = "30s"

# Reduce concurrent operations
[snmp]
concurrent_polls = 50  # Reduce from default
max_retries = 2
```

### 2.3 Slow API Responses

#### Symptoms

- API timeouts
- High response times
- User complaints about slow interface

#### Diagnosis

```bash
# Check API response times
curl -w "@curl-format.txt" -s -o /dev/null http://localhost:8080/api/nodes

# curl-format.txt content:
#     time_namelookup:  %{time_namelookup}\n
#     time_connect:     %{time_connect}\n
#     time_appconnect:  %{time_appconnect}\n
#     time_pretransfer: %{time_pretransfer}\n
#     time_redirect:    %{time_redirect}\n
#     time_starttransfer: %{time_starttransfer}\n
#     time_total:       %{time_total}\n

# Check database performance
unet db performance-report
```

#### Solutions

```bash
# Enable caching
[cache]
enabled = true
ttl = "300s"
type = "memory"  # or "redis"

# Optimize database queries
[database]
max_connections = 20
connection_timeout = "30s"

# Enable connection pooling
[database.pool]
enabled = true
min_connections = 5
max_connections = 25
```

---

## 3. Performance Issues

### 3.1 Database Performance

#### Slow Queries

```sql
-- PostgreSQL: Check slow queries
SELECT query, mean_time, calls, total_time
FROM pg_stat_statements
ORDER BY mean_time DESC
LIMIT 10;

-- Enable query logging temporarily
ALTER SYSTEM SET log_min_duration_statement = 1000;  -- Log queries > 1s
SELECT pg_reload_conf();
```

#### Solutions

```sql
-- Create missing indexes
CREATE INDEX CONCURRENTLY idx_nodes_status ON nodes (status);
CREATE INDEX CONCURRENTLY idx_changes_timestamp ON configuration_changes (created_at);

-- Update table statistics
ANALYZE nodes;
ANALYZE configuration_changes;
ANALYZE policy_evaluations;

-- Vacuum tables
VACUUM ANALYZE;
```

### 3.2 SNMP Polling Performance

#### Symptoms

- Slow SNMP polling
- Timeouts and retries
- High CPU usage during polling

#### Solutions

```toml
# Optimize SNMP configuration
[snmp]
concurrent_polls = 100       # Increase parallelism
timeout = "5s"              # Reduce timeout
retries = 2                 # Reduce retries
batch_size = 50            # Process in batches

# Enable SNMP caching
[snmp.cache]
enabled = true
ttl = "60s"
max_entries = 10000
```

### 3.3 Git Synchronization Performance

#### Symptoms

- Slow Git sync operations
- Large repository sizes
- Frequent sync failures

#### Solutions

```toml
# Optimize Git configuration
[git]
sync_cron = "*/15 * * * *"  # Less frequent syncing
shallow_clone = true        # Reduce repository size
parallel_sync = true        # Enable parallel operations

# Use Git LFS for large files
[git.lfs]
enabled = true
track_patterns = ["*.bin", "*.tar.gz"]
```

---

## 4. Database Problems

### 4.1 Database Connection Errors

#### Symptoms

- "Connection refused" errors
- "Too many connections" errors
- Database timeouts

#### SQLite Issues

```bash
# Check database file permissions
ls -la /var/lib/unet/unet.db

# Fix permissions
sudo chown unet:unet /var/lib/unet/unet.db
sudo chmod 644 /var/lib/unet/unet.db

# Check database integrity
sqlite3 /var/lib/unet/unet.db "PRAGMA integrity_check;"

# Repair database if needed
sqlite3 /var/lib/unet/unet.db "VACUUM;"
```

#### PostgreSQL Issues

```bash
# Check PostgreSQL status
systemctl status postgresql
kubectl get pods -l app=postgres

# Check connection limits
psql -U postgres -c "SELECT * FROM pg_stat_activity;" | wc -l
psql -U postgres -c "SHOW max_connections;"

# Increase connection limits if needed
ALTER SYSTEM SET max_connections = 200;
SELECT pg_reload_conf();
```

### 4.2 Migration Failures

#### Symptoms

- Migration errors during startup
- Version mismatch errors
- Corrupted schema

#### Solutions

```bash
# Check migration status
unet db migrate status

# Force migration reset (caution: data loss)
unet db migrate reset --force

# Apply specific migration
unet db migrate up --target=20240101_001

# Rollback problematic migration
unet db migrate down --steps=1
```

### 4.3 Data Corruption

#### Symptoms

- Inconsistent data
- Foreign key violations
- Unexpected query results

#### Recovery Procedures

```bash
# Backup current database
cp /var/lib/unet/unet.db /var/lib/unet/unet.db.backup

# Check and repair SQLite
sqlite3 /var/lib/unet/unet.db ".backup main backup.db"
mv backup.db /var/lib/unet/unet.db

# PostgreSQL corruption check
psql -U postgres -d unet_production -c "SELECT pg_total_relation_size('nodes');"

# Restore from backup if needed
gunzip -c backup.sql.gz | psql -U postgres -d unet_production
```

---

## 5. Network and SNMP Issues

### 5.1 SNMP Connection Failures

#### Symptoms

- Nodes showing as unreachable
- SNMP timeout errors
- Authentication failures

#### Diagnosis

```bash
# Test SNMP connectivity manually
snmpwalk -v2c -c public 192.168.1.1 1.3.6.1.2.1.1.1.0

# Check μNet SNMP configuration
unet snmp test --node=switch-01 --verbose

# Verify network connectivity
ping 192.168.1.1
telnet 192.168.1.1 161
```

#### Solutions

```toml
# Adjust SNMP timeouts
[snmp]
timeout = "10s"
retries = 3
community = "public"

# Use SNMPv3 for better security
[snmp.v3]
enabled = true
username = "unet_user"
auth_protocol = "SHA"
auth_password_file = "/etc/unet/snmp_auth"
priv_protocol = "AES"
priv_password_file = "/etc/unet/snmp_priv"
```

### 5.2 Firewall Issues

#### Symptoms

- Intermittent connectivity
- Port blocked errors
- Network timeouts

#### Solutions

```bash
# Check firewall rules
sudo iptables -L -n
sudo ufw status

# Allow SNMP traffic
sudo ufw allow out 161/udp
sudo iptables -A OUTPUT -p udp --dport 161 -j ACCEPT

# Check network policies (Kubernetes)
kubectl get networkpolicy -n unet-system
kubectl describe networkpolicy unet-network-policy -n unet-system
```

### 5.3 DNS Resolution Issues

#### Symptoms

- Cannot resolve device hostnames
- DNS lookup timeouts
- Inconsistent name resolution

#### Solutions

```bash
# Test DNS resolution
nslookup switch.example.com
dig switch.example.com

# Configure custom DNS
echo "nameserver 8.8.8.8" >> /etc/resolv.conf

# Use IP addresses instead of hostnames
[nodes.switch-01]
address = "192.168.1.1"  # Instead of "switch.example.com"
```

---

## 6. Authentication and Authorization Issues

### 6.1 JWT Token Problems

#### Symptoms

- "Invalid token" errors
- Token expiration issues
- Authentication failures

#### Solutions

```bash
# Verify JWT configuration
unet auth verify-token <token>

# Generate new tokens
unet auth generate-token --user=admin --role=admin

# Check token expiration
[auth]
jwt_expiry = "24h"
jwt_refresh_expiry = "168h"  # 7 days
```

### 6.2 API Key Issues

#### Symptoms

- API key authentication failures
- Rate limiting errors
- Invalid API key format

#### Solutions

```bash
# List API keys
unet api-keys list

# Generate new API key
unet api-keys create --name="production-app" --role=user

# Revoke compromised key
unet api-keys revoke --key-id=<key-id>

# Update rate limits
[api_keys]
rate_limit_per_key = 1000  # requests per hour
burst_limit = 100          # burst requests
```

### 6.3 RBAC Configuration Issues

#### Symptoms

- Permission denied errors
- Users cannot access resources
- Role assignment problems

#### Solutions

```bash
# Check user roles
unet users list --with-roles

# Assign role to user
unet users assign-role --user=john@example.com --role=operator

# Create custom role
unet roles create --name=readonly --permissions="nodes:read,policies:read"

# Update role permissions
unet roles update --name=operator --add-permission="templates:write"
```

---

## 7. Git Integration Problems

### 7.1 Git Authentication Failures

#### Symptoms

- Cannot clone repositories
- SSH key authentication errors
- HTTPS authentication issues

#### Solutions

```bash
# Test SSH key
ssh -T git@github.com

# Generate new SSH key
ssh-keygen -t ed25519 -C "unet@example.com"

# Configure SSH key in μNet
[git]
ssh_key_path = "/etc/unet/ssh/id_ed25519"
ssh_known_hosts_path = "/etc/unet/ssh/known_hosts"

# Add GitHub to known hosts
ssh-keyscan github.com >> /etc/unet/ssh/known_hosts
```

### 7.2 Repository Sync Issues

#### Symptoms

- Outdated policies/templates
- Sync failures
- Merge conflicts

#### Solutions

```bash
# Force repository refresh
unet git sync --force

# Reset repository to clean state
unet git reset --hard

# Check repository status
unet git status

# Resolve merge conflicts
[git]
conflict_resolution = "ours"  # Always use our version
auto_merge = false            # Manual conflict resolution
```

### 7.3 Large Repository Performance

#### Symptoms

- Slow Git operations
- High disk usage
- Memory issues during sync

#### Solutions

```toml
# Enable shallow cloning
[git]
shallow_clone = true
depth = 1

# Use sparse checkout
[git.sparse_checkout]
enabled = true
patterns = [
    "policies/",
    "templates/production/"
]

# Enable Git LFS
[git.lfs]
enabled = true
```

---

## 8. Kubernetes-Specific Issues

### 8.1 Pod Startup Problems

#### Symptoms

- Pods stuck in Pending state
- CrashLoopBackOff errors
- ImagePullBackOff errors

#### Diagnosis

```bash
# Check pod status
kubectl get pods -n unet-system
kubectl describe pod <pod-name> -n unet-system

# Check events
kubectl get events -n unet-system --sort-by='.lastTimestamp'

# Check logs
kubectl logs <pod-name> -n unet-system --previous
```

#### Solutions

```bash
# Resource constraints
kubectl patch deployment unet-server -n unet-system -p '{"spec":{"template":{"spec":{"containers":[{"name":"unet-server","resources":{"requests":{"memory":"1Gi","cpu":"500m"}}}]}}}}'

# Image pull issues
kubectl create secret docker-registry regcred \
  --docker-server=ghcr.io \
  --docker-username=<username> \
  --docker-password=<token> \
  -n unet-system

# Update deployment to use secret
kubectl patch deployment unet-server -n unet-system -p '{"spec":{"template":{"spec":{"imagePullSecrets":[{"name":"regcred"}]}}}}'
```

### 8.2 Service Discovery Issues

#### Symptoms

- Services unreachable
- DNS resolution failures
- Load balancer problems

#### Solutions

```bash
# Check service endpoints
kubectl get endpoints -n unet-system
kubectl describe service unet-server -n unet-system

# Test service connectivity
kubectl run test-pod --image=busybox -it --rm -- nslookup unet-server.unet-system.svc.cluster.local

# Fix service selector
kubectl patch service unet-server -n unet-system -p '{"spec":{"selector":{"app":"unet-server"}}}'
```

### 8.3 Persistent Volume Issues

#### Symptoms

- Data loss after pod restart
- Volume mount failures
- Storage capacity issues

#### Solutions

```bash
# Check persistent volumes
kubectl get pv,pvc -n unet-system

# Expand volume if needed
kubectl patch pvc unet-data -n unet-system -p '{"spec":{"resources":{"requests":{"storage":"200Gi"}}}}'

# Fix volume permissions
kubectl patch deployment unet-server -n unet-system -p '{"spec":{"template":{"spec":{"securityContext":{"fsGroup":1000}}}}}'
```

---

## 9. Frequently Asked Questions (FAQ)

### 9.1 General Questions

**Q: How do I check the μNet version?**

```bash
unet --version
unet-server --version
```

**Q: Where are the configuration files located?**

- Systemd: `/etc/unet/config.toml`
- Docker: Usually mounted from host
- Kubernetes: ConfigMap in the namespace

**Q: How do I enable debug logging?**

```toml
[logging]
level = "debug"
format = "pretty"  # or "json" for structured logging
```

**Q: Can I run multiple μNet instances?**
Yes, but ensure:

- Different bind ports
- Separate database files (SQLite) or schemas (PostgreSQL)
- Unique Git repository paths

### 9.2 Performance Questions

**Q: How many nodes can μNet handle?**

- SQLite: Up to 10,000 nodes
- PostgreSQL: 100,000+ nodes with proper tuning
- Performance depends on polling frequency and hardware

**Q: How often should I poll SNMP devices?**

- Default: 5-15 minutes for most devices
- Critical devices: 1-5 minutes
- Less critical: 30-60 minutes

**Q: Should I use SQLite or PostgreSQL?**

- SQLite: Development, small deployments (<1,000 nodes)
- PostgreSQL: Production, large deployments, HA requirements

### 9.3 Security Questions

**Q: How do I rotate JWT secrets?**

```bash
# Generate new secret
openssl rand -base64 32 > /etc/unet/jwt-secret-new

# Update configuration
[auth]
jwt_secret_file = "/etc/unet/jwt-secret-new"

# Restart service
sudo systemctl restart unet-server
```

**Q: How do I secure SNMP credentials?**

```toml
# Use encrypted credential storage
[snmp]
community_file = "/etc/unet/snmp/community"  # 600 permissions
enable_encryption = true

# Or use SNMPv3
[snmp.v3]
enabled = true
auth_password_file = "/etc/unet/snmp/auth"
priv_password_file = "/etc/unet/snmp/priv"
```

**Q: How do I enable HTTPS?**

```toml
[server]
bind = "0.0.0.0:8443"
tls_cert_path = "/etc/unet/tls/cert.pem"
tls_key_path = "/etc/unet/tls/key.pem"
```

### 9.4 Operational Questions

**Q: How do I backup μNet data?**

```bash
# SQLite backup
sqlite3 /var/lib/unet/unet.db ".backup /backup/unet-$(date +%Y%m%d).db"

# PostgreSQL backup
pg_dump -h localhost -U unet_user unet_production > backup.sql
```

**Q: How do I restore from backup?**

```bash
# Stop service
sudo systemctl stop unet-server

# Restore SQLite
cp /backup/unet-20240101.db /var/lib/unet/unet.db

# Restore PostgreSQL
psql -h localhost -U unet_user unet_production < backup.sql

# Start service
sudo systemctl start unet-server
```

**Q: How do I migrate from SQLite to PostgreSQL?**

```bash
# Export data from SQLite
unet db export --format=sql --output=export.sql

# Import to PostgreSQL
unet db import --source=export.sql --target=postgresql://user:pass@host:5432/dbname
```

---

## 10. Advanced Debugging

### 10.1 Debug Mode

```bash
# Enable debug mode
export RUST_LOG=debug
export UNET_DEBUG=true

# Run with additional debugging
unet-server --config /etc/unet/config.toml --debug --verbose
```

### 10.2 Profiling

```bash
# CPU profiling
RUST_LOG=info cargo flamegraph --bin unet-server

# Memory profiling
valgrind --tool=massif ./target/release/unet-server

# Performance analysis
perf record -g ./target/release/unet-server
perf report
```

### 10.3 Database Debugging

```sql
-- PostgreSQL: Enable query logging
ALTER SYSTEM SET log_statement = 'all';
ALTER SYSTEM SET log_min_duration_statement = 0;
SELECT pg_reload_conf();

-- Monitor connections
SELECT pid, usename, application_name, client_addr, state 
FROM pg_stat_activity 
WHERE datname = 'unet_production';

-- Check locks
SELECT * FROM pg_locks WHERE NOT granted;
```

### 10.4 Network Debugging

```bash
# Capture SNMP traffic
sudo tcpdump -i any -s 0 -w snmp.pcap port 161

# Analyze with Wireshark
wireshark snmp.pcap

# Monitor network connections
ss -tulpn | grep unet-server
netstat -tulpn | grep 8080
```

---

## Getting Help

If you encounter issues not covered in this guide:

1. **Check logs** for specific error messages
2. **Search** the issue tracker for similar problems
3. **Create** a detailed bug report with:
   - μNet version
   - Operating system and version
   - Configuration file (with secrets redacted)
   - Complete error logs
   - Steps to reproduce

For immediate assistance:

- GitHub Issues: <https://github.com/your-org/unet/issues>
- Documentation: See [Architecture Overview](01_architecture.md)
- Community: Join our Discord/Slack channel

---

**Remember**: When reporting issues, always include relevant logs and configuration details, but be sure to redact any sensitive information like passwords, API keys, or SNMP community strings.
