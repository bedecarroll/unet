# μNet Troubleshooting Guide

> **Quick Reference**: Common issues and step-by-step resolution procedures

## Common Issues Quick Reference

| Issue | Symptoms | First Action | Reference |
|-------|----------|--------------|-----------|
| Server Won't Start | Service fails to start | Check logs: `journalctl -u unet-server` | [Startup Issues](#startup-issues) |
| High Memory Usage | Memory > 80% | Check process: `ps aux \| grep unet` | [Performance Issues](#performance-issues) |
| Database Errors | CRUD operations fail | Check DB: `sqlite3 /path/to/unet.db "PRAGMA integrity_check;"` | [Database Issues](#database-issues) |
| Authentication Failures | Login/API errors | Review logs: `grep auth_failure /var/log/unet/` | [Authentication Issues](#authentication-issues) |
| Git Sync Problems | Policy/template sync fails | Test Git: `curl /api/v1/git/status` | [Git Integration Issues](#git-integration-issues) |
| API Timeouts | Slow/hanging responses | Check metrics: `curl /api/v1/metrics/snapshot` | [Performance Issues](#performance-issues) |

## Startup Issues

### Service Fails to Start

**Symptoms**:

- `systemctl start unet-server` fails
- Service status shows "failed" or "inactive"

**Diagnostic Steps**:

```bash
# 1. Check service status
systemctl status unet-server

# 2. Review startup logs
journalctl -u unet-server --since "5 minutes ago"

# 3. Check configuration validity
/opt/unet/bin/unet-server --config /etc/unet/config.toml --validate

# 4. Verify file permissions
ls -la /etc/unet/config.toml
ls -la /path/to/unet.db

# 5. Check port availability
netstat -tulpn | grep :8080
```

**Common Causes and Solutions**:

1. **Configuration Error**

   ```bash
   # Check configuration syntax
   /opt/unet/bin/unet-server --config /etc/unet/config.toml --validate
   
   # Fix: Review and correct configuration file
   nano /etc/unet/config.toml
   ```

2. **Port Already in Use**

   ```bash
   # Find process using port
   lsof -i :8080
   
   # Fix: Kill conflicting process or change port
   kill $(lsof -t -i :8080)
   # OR update config to use different port
   ```

3. **Database Access Issues**

   ```bash
   # Check database file permissions
   ls -la /path/to/unet.db
   
   # Fix: Set correct permissions
   chown unet:unet /path/to/unet.db
   chmod 640 /path/to/unet.db
   ```

4. **Missing Dependencies**

   ```bash
   # Check binary dependencies
   ldd /opt/unet/bin/unet-server
   
   # Fix: Install missing libraries
   apt update && apt install -y libssl1.1 libsqlite3-0
   ```

### Configuration Validation Issues

**Symptoms**:

- Server starts but behaves unexpectedly
- API endpoints return configuration errors

**Diagnostic Steps**:

```bash
# 1. Validate configuration structure
/opt/unet/bin/unet-server --config /etc/unet/config.toml --validate

# 2. Check configuration syntax
cat /etc/unet/config.toml | grep -n "syntax\|error"

# 3. Verify environment variables
env | grep UNET

# 4. Test specific configuration sections
curl http://localhost:8080/api/v1/config/database
curl http://localhost:8080/api/v1/config/metrics
```

**Common Configuration Fixes**:

```toml
# Fix: Ensure proper TOML syntax
[database]
url = "sqlite:///path/to/unet.db"  # Correct
# url = sqlite:///path/to/unet.db  # Incorrect - missing quotes

# Fix: Proper boolean values
[metrics]
enabled = true  # Correct
# enabled = "true"  # Incorrect - should be boolean

# Fix: Proper array syntax
[server]
cors_origins = ["http://localhost:3000", "https://ui.example.com"]  # Correct
# cors_origins = "http://localhost:3000"  # Incorrect - should be array
```

## Performance Issues

### High CPU Usage

**Symptoms**:

- CPU usage consistently above 80%
- API responses slow or timing out
- System becomes unresponsive

**Diagnostic Steps**:

```bash
# 1. Check current CPU usage
top -p $(pgrep unet-server)

# 2. Analyze process threads
ps -T -p $(pgrep unet-server)

# 3. Check for runaway operations
curl http://localhost:8080/api/v1/metrics/snapshot | jq '.performance'

# 4. Review recent activity
curl http://localhost:8080/api/v1/activities?limit=20

# 5. Check background tasks
curl http://localhost:8080/api/v1/tasks/status
```

**Common Causes and Solutions**:

1. **SNMP Query Overload**

   ```bash
   # Check SNMP query rate
   curl http://localhost:8080/api/v1/metrics/snapshot | jq '.business.snmp_queries_total'
   
   # Fix: Reduce SNMP polling frequency
   curl -X PUT http://localhost:8080/api/v1/config/snmp \
     -H "Content-Type: application/json" \
     -d '{"polling_interval": 300, "max_concurrent_queries": 10}'
   ```

2. **Policy Evaluation Loops**

   ```bash
   # Check policy evaluation rate
   curl http://localhost:8080/api/v1/metrics/snapshot | jq '.business.policy_evaluations_total'
   
   # Fix: Review and optimize policies
   curl http://localhost:8080/api/v1/policies?status=active
   ```

3. **Database Query Performance**

   ```bash
   # Check slow queries
   sqlite3 /path/to/unet.db "PRAGMA compile_options;" | grep ENABLE_STAT4
   
   # Fix: Optimize database
   sqlite3 /path/to/unet.db "ANALYZE; VACUUM;"
   ```

### High Memory Usage

**Symptoms**:

- Memory usage above 80% of available RAM
- Out of memory errors in logs
- System swap usage increasing

**Diagnostic Steps**:

```bash
# 1. Check memory usage
ps aux | grep unet-server | awk '{print $6, $11}'

# 2. Monitor memory over time
watch "ps aux | grep unet-server | awk '{print \$6}'"

# 3. Check for memory leaks
curl http://localhost:8080/api/v1/metrics/snapshot | jq '.system'

# 4. Analyze heap usage (if available)
curl http://localhost:8080/api/v1/debug/memory
```

**Solutions**:

```bash
# 1. Restart service to clear memory
systemctl restart unet-server

# 2. Optimize configuration for memory
cat > /tmp/memory_optimization.toml << 'EOF'
[database]
connection_pool_size = 10  # Reduce from default

[server]
max_concurrent_requests = 50  # Limit concurrent operations

[cache]
max_size = "100MB"  # Limit cache size
EOF

# 3. Monitor after optimization
watch "curl -s http://localhost:8080/api/v1/metrics/snapshot | jq '.system.memory_usage_bytes'"
```

### API Response Timeouts

**Symptoms**:

- HTTP requests timing out
- 504 Gateway Timeout errors
- Client connections dropping

**Diagnostic Steps**:

```bash
# 1. Check response time percentiles
curl http://localhost:8080/api/v1/metrics/snapshot | jq '.performance.http_request_duration_avg'

# 2. Identify slow endpoints
journalctl -u unet-server --since "1 hour ago" | grep "slow_request"

# 3. Check database performance
sqlite3 /path/to/unet.db ".timer on" "SELECT COUNT(*) FROM nodes;"

# 4. Monitor active connections
curl http://localhost:8080/api/v1/metrics/snapshot | jq '.system.active_connections'
```

**Solutions**:

```bash
# 1. Increase timeout values
curl -X PUT http://localhost:8080/api/v1/config/server \
  -H "Content-Type: application/json" \
  -d '{"request_timeout": 60, "read_timeout": 30}'

# 2. Optimize database queries
sqlite3 /path/to/unet.db "CREATE INDEX IF NOT EXISTS idx_nodes_status ON nodes(status);"

# 3. Enable connection pooling
curl -X PUT http://localhost:8080/api/v1/config/database \
  -H "Content-Type: application/json" \
  -d '{"connection_pool_size": 20, "max_idle_connections": 10}'
```

## Database Issues

### Database Corruption

**Symptoms**:

- SQLite errors in logs
- Data inconsistencies
- Database integrity check failures

**Diagnostic Steps**:

```bash
# 1. Check database integrity
sqlite3 /path/to/unet.db "PRAGMA integrity_check;"

# 2. Check for corruption patterns
sqlite3 /path/to/unet.db "PRAGMA quick_check;"

# 3. Analyze database file
file /path/to/unet.db

# 4. Check disk space and permissions
df -h /path/to/
ls -la /path/to/unet.db
```

**Recovery Steps**:

```bash
# 1. Stop μNet server
systemctl stop unet-server

# 2. Backup corrupted database
cp /path/to/unet.db /path/to/unet.db.corrupted.$(date +%Y%m%d_%H%M%S)

# 3. Attempt repair
sqlite3 /path/to/unet.db << 'EOF'
.mode insert
.output /tmp/dump.sql
.dump
.quit
EOF

# 4. Recreate database from dump
mv /path/to/unet.db /path/to/unet.db.old
sqlite3 /path/to/unet.db < /tmp/dump.sql

# 5. Verify repair
sqlite3 /path/to/unet.db "PRAGMA integrity_check;"

# 6. Restart service
systemctl start unet-server
```

### Migration Failures

**Symptoms**:

- Migration errors during startup
- Schema version mismatches
- Database structure inconsistencies

**Diagnostic Steps**:

```bash
# 1. Check current schema version
sqlite3 /path/to/unet.db "SELECT version FROM schema_migrations ORDER BY version DESC LIMIT 1;"

# 2. List available migrations
ls -la /opt/unet/migrations/

# 3. Check migration status
curl http://localhost:8080/api/v1/system/migrations

# 4. Review migration logs
journalctl -u unet-server | grep migration
```

**Recovery Steps**:

```bash
# 1. Stop service
systemctl stop unet-server

# 2. Backup database
cp /path/to/unet.db /path/to/unet.db.pre-migration.$(date +%Y%m%d_%H%M%S)

# 3. Reset to known good migration
sqlite3 /path/to/unet.db "DELETE FROM schema_migrations WHERE version > '20250601000000';"

# 4. Re-run migrations
/opt/unet/bin/unet-server migrate --config /etc/unet/config.toml

# 5. Verify migration success
sqlite3 /path/to/unet.db "SELECT version FROM schema_migrations ORDER BY version DESC LIMIT 5;"

# 6. Restart service
systemctl start unet-server
```

### Connection Pool Exhaustion

**Symptoms**:

- "Database connection pool exhausted" errors
- New requests failing with database errors
- High database connection usage

**Diagnostic Steps**:

```bash
# 1. Check current connections
curl http://localhost:8080/api/v1/metrics/snapshot | jq '.system.database_active_connections'

# 2. Check pool configuration
curl http://localhost:8080/api/v1/config/database | jq '.connection_pool'

# 3. Monitor connection usage over time
watch "curl -s http://localhost:8080/api/v1/metrics/snapshot | jq '.system.database_active_connections'"

# 4. Check for connection leaks
lsof -p $(pgrep unet-server) | grep sqlite
```

**Solutions**:

```bash
# 1. Increase pool size
curl -X PUT http://localhost:8080/api/v1/config/database \
  -H "Content-Type: application/json" \
  -d '{"connection_pool_size": 50, "max_idle_connections": 25}'

# 2. Restart to clear connections
systemctl restart unet-server

# 3. Monitor pool utilization
curl http://localhost:8080/api/v1/metrics/snapshot | jq '.system.database_pool_size'
```

## Authentication Issues

### Login Failures

**Symptoms**:

- Users cannot log in via web interface
- API authentication failing
- JWT token validation errors

**Diagnostic Steps**:

```bash
# 1. Check authentication logs
journalctl -u unet-server | grep auth | tail -20

# 2. Test authentication endpoint
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "password"}'

# 3. Check user database
sqlite3 /path/to/unet.db "SELECT id, username, status FROM users WHERE username = 'admin';"

# 4. Verify JWT configuration
curl http://localhost:8080/api/v1/config/auth | jq '.jwt'
```

**Common Solutions**:

1. **Password Issues**

   ```bash
   # Reset user password
   curl -X PUT http://localhost:8080/api/v1/users/admin/password \
     -H "Content-Type: application/json" \
     -d '{"password": "newpassword"}'
   ```

2. **Account Locked**

   ```bash
   # Unlock user account
   curl -X PUT http://localhost:8080/api/v1/users/admin/status \
     -H "Content-Type: application/json" \
     -d '{"status": "active"}'
   ```

3. **JWT Secret Issues**

   ```bash
   # Check JWT configuration
   grep jwt_secret /etc/unet/config.toml
   
   # Regenerate JWT secret if corrupted
   echo "jwt_secret = \"$(openssl rand -base64 32)\"" >> /etc/unet/config.toml
   systemctl restart unet-server
   ```

### API Key Authentication

**Symptoms**:

- API key requests returning 401/403
- Valid API keys being rejected
- API key validation errors

**Diagnostic Steps**:

```bash
# 1. Test API key
curl -H "Authorization: Bearer YOUR_API_KEY" http://localhost:8080/api/v1/nodes

# 2. Check API key in database
sqlite3 /path/to/unet.db "SELECT id, name, expires_at FROM api_keys WHERE key_hash = 'hash';"

# 3. Review API key logs
journalctl -u unet-server | grep api_key | tail -10

# 4. Verify API key format
echo "YOUR_API_KEY" | base64 -d | hexdump -C
```

**Solutions**:

```bash
# 1. Create new API key
curl -X POST http://localhost:8080/api/v1/auth/api-keys \
  -H "Content-Type: application/json" \
  -d '{"name": "backup-key", "expires_in": "30d"}'

# 2. Revoke compromised keys
curl -X DELETE http://localhost:8080/api/v1/auth/api-keys/key-id

# 3. List active API keys
curl http://localhost:8080/api/v1/auth/api-keys | jq '.[] | {id, name, expires_at}'
```

## Git Integration Issues

### Git Sync Failures

**Symptoms**:

- Policy files not updating
- Template sync errors
- Git repository connection failures

**Diagnostic Steps**:

```bash
# 1. Check Git sync status
curl http://localhost:8080/api/v1/git/sync/status

# 2. Test Git connectivity
curl http://localhost:8080/api/v1/git/repositories

# 3. Check Git credentials
curl http://localhost:8080/api/v1/git/credentials/test

# 4. Review sync logs
journalctl -u unet-server | grep git_sync | tail -20

# 5. Manual Git test
cd /tmp && git clone https://github.com/your/repo.git test-repo
```

**Common Solutions**:

1. **Authentication Issues**

   ```bash
   # Update Git credentials
   curl -X PUT http://localhost:8080/api/v1/git/credentials \
     -H "Content-Type: application/json" \
     -d '{"username": "user", "token": "new_token"}'
   ```

2. **Network Connectivity**

   ```bash
   # Test network connectivity
   nc -zv github.com 443
   
   # Check DNS resolution
   nslookup github.com
   
   # Test HTTPS connectivity
   curl -I https://github.com
   ```

3. **Repository Configuration**

   ```bash
   # Update repository URL
   curl -X PUT http://localhost:8080/api/v1/git/repositories/repo-id \
     -H "Content-Type: application/json" \
     -d '{"url": "https://github.com/new/repo.git", "branch": "main"}'
   ```

### Git Authentication Problems

**Symptoms**:

- 401/403 errors from Git repositories
- SSH key authentication failing
- Token authentication rejected

**Diagnostic Steps**:

```bash
# 1. Test SSH key (if using SSH)
ssh -T git@github.com

# 2. Check SSH key permissions
ls -la ~/.ssh/id_rsa*

# 3. Test HTTPS authentication
git ls-remote https://token@github.com/user/repo.git

# 4. Verify Git configuration
git config --list | grep user
```

**Solutions**:

```bash
# 1. Generate new SSH key
ssh-keygen -t ed25519 -C "unet@example.com"
cat ~/.ssh/id_ed25519.pub  # Add to GitHub

# 2. Update Git credentials
curl -X PUT http://localhost:8080/api/v1/git/credentials \
  -H "Content-Type: application/json" \
  -d '{"type": "token", "token": "ghp_newtoken123"}'

# 3. Test updated credentials
curl -X POST http://localhost:8080/api/v1/git/sync/test
```

## Network Issues

### SNMP Query Failures

**Symptoms**:

- Network device monitoring failing
- SNMP timeout errors
- Device status showing as "unreachable"

**Diagnostic Steps**:

```bash
# 1. Test SNMP connectivity
snmpget -v2c -c public device.example.com 1.3.6.1.2.1.1.1.0

# 2. Check SNMP configuration
curl http://localhost:8080/api/v1/config/snmp

# 3. Review SNMP logs
journalctl -u unet-server | grep snmp | tail -20

# 4. Check network connectivity
ping device.example.com
nc -u device.example.com 161
```

**Solutions**:

```bash
# 1. Update SNMP community strings
curl -X PUT http://localhost:8080/api/v1/config/snmp \
  -H "Content-Type: application/json" \
  -d '{"community": "newcommunity", "version": "2c"}'

# 2. Increase timeout values
curl -X PUT http://localhost:8080/api/v1/config/snmp \
  -H "Content-Type: application/json" \
  -d '{"timeout": 10, "retries": 3}'

# 3. Test specific device
curl -X POST http://localhost:8080/api/v1/devices/test-snmp \
  -H "Content-Type: application/json" \
  -d '{"host": "device.example.com", "community": "public"}'
```

## Emergency Recovery

### Service Recovery Script

```bash
#!/bin/bash
# Emergency μNet recovery script

echo "Starting μNet emergency recovery..."

# 1. Stop service
systemctl stop unet-server

# 2. Create emergency backup
BACKUP_DIR="/tmp/emergency-backup-$(date +%Y%m%d_%H%M%S)"
mkdir -p "$BACKUP_DIR"
cp /path/to/unet.db "$BACKUP_DIR/"
cp /etc/unet/config.toml "$BACKUP_DIR/"

# 3. Check database integrity
if ! sqlite3 /path/to/unet.db "PRAGMA integrity_check;" | grep -q "ok"; then
    echo "Database corruption detected, restoring from backup..."
    LATEST_BACKUP=$(ls -t /backups/unet/*/unet.db | head -1)
    cp "$LATEST_BACKUP" /path/to/unet.db
fi

# 4. Reset to minimal configuration
cat > /etc/unet/config-minimal.toml << 'EOF'
[server]
host = "127.0.0.1"
port = 8080

[database]
url = "sqlite:///path/to/unet.db"

[logging]
level = "info"
EOF

# 5. Start with minimal config
/opt/unet/bin/unet-server --config /etc/unet/config-minimal.toml &
SERVER_PID=$!

# 6. Wait for startup
sleep 10

# 7. Test basic functionality
if curl -f http://localhost:8080/api/v1/health; then
    echo "Emergency recovery successful"
    kill $SERVER_PID
    systemctl start unet-server
else
    echo "Emergency recovery failed, manual intervention required"
    kill $SERVER_PID
fi
```

### Factory Reset Procedure

```bash
#!/bin/bash
# Complete μNet factory reset

echo "WARNING: This will delete all μNet data!"
read -p "Type 'RESET' to continue: " confirm

if [ "$confirm" != "RESET" ]; then
    echo "Factory reset cancelled"
    exit 1
fi

# 1. Stop service
systemctl stop unet-server

# 2. Backup current state
BACKUP_DIR="/backups/factory-reset-$(date +%Y%m%d_%H%M%S)"
mkdir -p "$BACKUP_DIR"
cp -r /etc/unet/ "$BACKUP_DIR/"
cp /path/to/unet.db "$BACKUP_DIR/"

# 3. Remove database
rm -f /path/to/unet.db

# 4. Reset configuration to defaults
cp /opt/unet/config/default.toml /etc/unet/config.toml

# 5. Initialize fresh database
/opt/unet/bin/unet-server migrate --config /etc/unet/config.toml

# 6. Create default admin user
/opt/unet/bin/unet-server create-user --username admin --password changeme

# 7. Start service
systemctl start unet-server

echo "Factory reset complete. Default admin user: admin/changeme"
```

---

**Document Version**: 1.0  
**Last Updated**: 2025-06-29  
**Next Review**: 2025-09-29  
**Owner**: μNet Operations Team
