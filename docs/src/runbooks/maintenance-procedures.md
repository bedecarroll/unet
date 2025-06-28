# μNet Maintenance Procedures Runbook

> **Essential Guide**: Standard operating procedures for μNet system maintenance

## Scheduled Maintenance

### Monthly Maintenance Window

**Schedule**: First Saturday of each month, 2:00 AM - 4:00 AM UTC  
**Duration**: 2 hours maximum  
**Notification**: 7 days advance notice to stakeholders

#### Pre-Maintenance Checklist (1 week before)

- [ ] Schedule maintenance window announcement
- [ ] Verify backup completion
- [ ] Review planned changes
- [ ] Coordinate with network team
- [ ] Prepare rollback procedures
- [ ] Update maintenance documentation

#### Maintenance Day Checklist

**1. Pre-Maintenance Verification (30 minutes before)**

```bash
# Verify system health
curl http://localhost:8080/api/v1/health | jq '.'

# Check current metrics
curl http://localhost:8080/api/v1/metrics/snapshot | jq '.system'

# Verify backup integrity
ls -la /backups/unet/ | tail -5
sqlite3 /backups/unet/latest/unet.db "PRAGMA integrity_check;"

# Create pre-maintenance snapshot
curl -X POST http://localhost:8080/api/v1/system/snapshot \
  -d '{"name": "pre-maintenance-$(date +%Y%m%d)", "type": "full"}'
```

**2. System Update Procedures**

```bash
# 1. Stop μNet server
systemctl stop unet-server

# 2. Backup current installation
tar -czf /backups/unet/pre-update-$(date +%Y%m%d).tar.gz /opt/unet/

# 3. Update system packages
apt update && apt upgrade -y

# 4. Update μNet binary (if applicable)
cp /path/to/new/unet-server /opt/unet/bin/
chmod +x /opt/unet/bin/unet-server

# 5. Database migration (if needed)
/opt/unet/bin/unet-server migrate --config /etc/unet/config.toml

# 6. Restart service
systemctl start unet-server

# 7. Verify startup
journalctl -u unet-server --since "2 minutes ago"
curl http://localhost:8080/api/v1/health
```

**3. Post-Maintenance Verification**

```bash
# System health check
curl http://localhost:8080/api/v1/health | jq '.'

# Performance verification
curl http://localhost:8080/api/v1/metrics/snapshot | jq '.performance'

# Feature verification
unet nodes list
unet git status
unet policies list

# Database integrity check
sqlite3 /path/to/unet.db "PRAGMA integrity_check;"
```

### Emergency Maintenance

#### Unplanned Security Updates

```bash
# 1. Immediate notification
echo "Emergency security update starting" | \
  curl -X POST https://hooks.slack.com/webhook -d @-

# 2. Create emergency backup
systemctl stop unet-server
cp /path/to/unet.db /backups/emergency-$(date +%Y%m%d_%H%M%S).db

# 3. Apply security patches
apt update && apt install security-updates

# 4. Restart and verify
systemctl start unet-server
curl http://localhost:8080/api/v1/health
```

#### Critical Bug Fixes

```bash
# 1. Assess impact and create rollback plan
systemctl status unet-server
curl http://localhost:8080/api/v1/system/status

# 2. Deploy fix with minimal downtime
systemctl reload unet-server  # If hot-reload supported
# OR
systemctl restart unet-server  # If restart required

# 3. Monitor for regression
watch "curl -s http://localhost:8080/api/v1/health | jq '.status'"
```

## Backup and Recovery

### Automated Daily Backups

**Script Location**: `/opt/unet/scripts/backup.sh`

```bash
#!/bin/bash
# Daily backup script

BACKUP_DIR="/backups/unet/$(date +%Y%m%d)"
mkdir -p "$BACKUP_DIR"

# 1. Database backup
sqlite3 /path/to/unet.db ".backup '$BACKUP_DIR/unet.db'"

# 2. Configuration backup
cp -r /etc/unet/ "$BACKUP_DIR/config/"

# 3. Git repository backup (if local)
if [ -d "/opt/unet/repositories" ]; then
    tar -czf "$BACKUP_DIR/repositories.tar.gz" /opt/unet/repositories/
fi

# 4. Log backup
journalctl -u unet-server --since "24 hours ago" > "$BACKUP_DIR/logs.txt"

# 5. Verify backup integrity
sqlite3 "$BACKUP_DIR/unet.db" "PRAGMA integrity_check;" > "$BACKUP_DIR/integrity.log"

# 6. Clean old backups (keep 30 days)
find /backups/unet/ -type d -mtime +30 -exec rm -rf {} \;

echo "Backup completed: $BACKUP_DIR"
```

### Recovery Procedures

#### Database Recovery from Backup

```bash
# 1. Stop μNet server
systemctl stop unet-server

# 2. Backup current corrupted database
mv /path/to/unet.db /path/to/unet.db.corrupted.$(date +%Y%m%d_%H%M%S)

# 3. Restore from backup
RESTORE_DATE="20250629"  # Specify backup date
cp /backups/unet/$RESTORE_DATE/unet.db /path/to/unet.db

# 4. Verify database integrity
sqlite3 /path/to/unet.db "PRAGMA integrity_check;"

# 5. Set proper permissions
chown unet:unet /path/to/unet.db
chmod 640 /path/to/unet.db

# 6. Restart service
systemctl start unet-server

# 7. Verify recovery
curl http://localhost:8080/api/v1/health
unet nodes list | head -5
```

#### Configuration Recovery

```bash
# 1. Stop service
systemctl stop unet-server

# 2. Backup current config
cp -r /etc/unet/ /etc/unet.backup.$(date +%Y%m%d_%H%M%S)

# 3. Restore configuration
RESTORE_DATE="20250629"
cp -r /backups/unet/$RESTORE_DATE/config/* /etc/unet/

# 4. Verify configuration
/opt/unet/bin/unet-server --config /etc/unet/config.toml --validate

# 5. Restart service
systemctl start unet-server
```

#### Complete System Recovery

```bash
# 1. Stop all μNet services
systemctl stop unet-server

# 2. Restore binary and configuration
RESTORE_DATE="20250629"
tar -xzf /backups/unet/pre-update-$RESTORE_DATE.tar.gz -C /

# 3. Restore database
cp /backups/unet/$RESTORE_DATE/unet.db /path/to/unet.db

# 4. Set permissions
chown -R unet:unet /opt/unet/
chown unet:unet /path/to/unet.db

# 5. Start service
systemctl start unet-server

# 6. Full system verification
curl http://localhost:8080/api/v1/health
unet --version
unet nodes list
```

## Database Maintenance

### Weekly Database Optimization

```bash
# 1. Create backup before optimization
sqlite3 /path/to/unet.db ".backup '/tmp/pre-optimize.db'"

# 2. Analyze database
sqlite3 /path/to/unet.db "ANALYZE;"

# 3. Vacuum database
sqlite3 /path/to/unet.db "VACUUM;"

# 4. Rebuild indexes
sqlite3 /path/to/unet.db "REINDEX;"

# 5. Verify integrity
sqlite3 /path/to/unet.db "PRAGMA integrity_check;"

# 6. Check file size reduction
ls -lh /path/to/unet.db
```

### Database Statistics Collection

```bash
# Monthly database analysis
sqlite3 /path/to/unet.db << 'EOF'
.mode column
.headers on

-- Table sizes
SELECT name, COUNT(*) as row_count 
FROM sqlite_master 
WHERE type='table' 
AND name NOT LIKE 'sqlite_%';

-- Database size analysis
PRAGMA page_count;
PRAGMA page_size;
PRAGMA freelist_count;

-- Index usage statistics
SELECT name, rootpage FROM sqlite_master WHERE type='index';
EOF
```

### Migration Management

```bash
# Check current database version
sqlite3 /path/to/unet.db "SELECT version FROM schema_migrations ORDER BY version DESC LIMIT 1;"

# Run pending migrations
/opt/unet/bin/unet-server migrate --config /etc/unet/config.toml

# Verify migration success
sqlite3 /path/to/unet.db "SELECT * FROM schema_migrations ORDER BY version DESC LIMIT 5;"
```

## Log Management

### Log Rotation Configuration

```bash
# Configure logrotate for μNet logs
cat > /etc/logrotate.d/unet << 'EOF'
/var/log/unet/*.log {
    daily
    missingok
    rotate 30
    compress
    delaycompress
    notifempty
    postrotate
        systemctl reload unet-server
    endscript
}
EOF
```

### Log Analysis Procedures

```bash
# Daily log review
journalctl -u unet-server --since "24 hours ago" | grep ERROR | head -20

# Performance issue analysis
journalctl -u unet-server --since "1 hour ago" | grep "slow_query\|timeout\|performance"

# Security event review
journalctl -u unet-server --since "24 hours ago" | grep "auth_failure\|security\|blocked"

# Error pattern analysis
journalctl -u unet-server --since "7 days ago" | grep ERROR | \
    awk '{print $5}' | sort | uniq -c | sort -nr | head -10
```

### Log Archival

```bash
# Weekly log archival
ARCHIVE_DATE=$(date -d "7 days ago" +%Y%m%d)
journalctl -u unet-server --since "$ARCHIVE_DATE" --until "$(date +%Y%m%d)" \
    > /archives/logs/unet-$ARCHIVE_DATE.log

# Compress archived logs
gzip /archives/logs/unet-$ARCHIVE_DATE.log

# Clean old archives (keep 1 year)
find /archives/logs/ -name "unet-*.log.gz" -mtime +365 -delete
```

## Performance Monitoring

### Daily Performance Checks

```bash
# System resource utilization
curl http://localhost:8080/api/v1/metrics/snapshot | jq '.system'

# Application performance metrics
curl http://localhost:8080/api/v1/metrics/snapshot | jq '.performance'

# Database performance analysis
sqlite3 /path/to/unet.db << 'EOF'
.timer on
SELECT COUNT(*) FROM nodes;
SELECT COUNT(*) FROM users;
SELECT COUNT(*) FROM configuration_changes;
EOF
```

### Performance Optimization

```bash
# Index analysis and optimization
sqlite3 /path/to/unet.db "EXPLAIN QUERY PLAN SELECT * FROM nodes WHERE status = 'active';"

# Connection pool optimization
curl http://localhost:8080/api/v1/config/database | jq '.connection_pool'

# Memory usage optimization
ps aux | grep unet-server | awk '{print $6}' | head -1  # RSS memory
```

### Capacity Planning

```bash
# Monthly capacity analysis
cat > /tmp/capacity_report.sh << 'EOF'
#!/bin/bash
echo "=== μNet Capacity Report $(date) ==="
echo "Database size: $(du -h /path/to/unet.db | cut -f1)"
echo "Total nodes: $(sqlite3 /path/to/unet.db 'SELECT COUNT(*) FROM nodes;')"
echo "Total users: $(sqlite3 /path/to/unet.db 'SELECT COUNT(*) FROM users;')"
echo "Avg response time: $(curl -s http://localhost:8080/api/v1/metrics/snapshot | jq '.performance.http_request_duration_avg')"
echo "Memory usage: $(ps aux | grep unet-server | awk '{print $6}' | head -1) KB"
echo "Disk usage: $(df -h /path/to/ | tail -1 | awk '{print $5}')"
EOF

chmod +x /tmp/capacity_report.sh
/tmp/capacity_report.sh
```

## Security Maintenance

### Certificate Management

```bash
# Check certificate expiration
curl -k https://localhost:8443/api/v1/certificates/status | jq '.expiration'

# Rotate certificates (if needed)
curl -X POST https://localhost:8443/api/v1/certificates/rotate

# Backup certificates
cp /etc/unet/ssl/*.pem /backups/certificates/$(date +%Y%m%d)/
```

### Security Updates

```bash
# Weekly security audit
curl http://localhost:8080/api/v1/security/audit/summary | jq '.'

# Review failed authentication attempts
curl http://localhost:8080/api/v1/security/events?type=auth_failure&limit=50

# Update security policies
curl -X PUT http://localhost:8080/api/v1/security/policies \
    -H "Content-Type: application/json" \
    -d @/etc/unet/security-policies.json
```

### Access Control Review

```bash
# Monthly user access review
curl http://localhost:8080/api/v1/users | jq '.[] | {id, username, last_login, roles}'

# API key audit
curl http://localhost:8080/api/v1/auth/api-keys | jq '.[] | {name, last_used, expires}'

# Remove inactive users (last login > 90 days)
# Manual process with proper approval workflow
```

## Monitoring and Alerting

### Health Check Automation

```bash
# Automated health monitoring script
cat > /opt/unet/scripts/health_check.sh << 'EOF'
#!/bin/bash
HEALTH_ENDPOINT="http://localhost:8080/api/v1/health"
ALERT_WEBHOOK="https://hooks.slack.com/services/YOUR/WEBHOOK/URL"

HEALTH_STATUS=$(curl -s $HEALTH_ENDPOINT | jq -r '.status')

if [ "$HEALTH_STATUS" != "healthy" ]; then
    MESSAGE="μNet Health Check Failed: $HEALTH_STATUS"
    curl -X POST $ALERT_WEBHOOK \
        -H 'Content-type: application/json' \
        --data "{\"text\":\"$MESSAGE\"}"
fi
EOF

# Schedule in crontab (every 5 minutes)
echo "*/5 * * * * /opt/unet/scripts/health_check.sh" | crontab -
```

### Performance Baseline Updates

```bash
# Monthly baseline update
CURRENT_METRICS=$(curl -s http://localhost:8080/api/v1/metrics/snapshot)
echo "$CURRENT_METRICS" > /var/lib/unet/baselines/$(date +%Y%m).json

# Compare with previous month
PREV_MONTH=$(date -d "1 month ago" +%Y%m)
if [ -f "/var/lib/unet/baselines/$PREV_MONTH.json" ]; then
    echo "Performance comparison with $PREV_MONTH:"
    jq -s '.[1].performance.http_request_duration_avg - .[0].performance.http_request_duration_avg' \
        /var/lib/unet/baselines/$PREV_MONTH.json \
        /var/lib/unet/baselines/$(date +%Y%m).json
fi
```

## Documentation Updates

### Procedure Documentation

- **Weekly**: Update troubleshooting guides based on incidents
- **Monthly**: Review and update maintenance procedures
- **Quarterly**: Update capacity planning estimates
- **Annually**: Complete procedure review and optimization

### Change Management

```bash
# Document maintenance changes
cat > /var/log/unet/maintenance-$(date +%Y%m%d).log << EOF
Date: $(date)
Type: [Scheduled/Emergency]
Duration: [Start-End]
Changes: [Description]
Impact: [User Impact]
Rollback: [Rollback procedure]
Results: [Success/Issues]
EOF
```

### Knowledge Base Updates

- Maintain internal wiki with common procedures
- Update external documentation for user-facing changes
- Create video walkthroughs for complex procedures
- Regular knowledge sharing sessions

---

**Document Version**: 1.0  
**Last Updated**: 2025-06-29  
**Next Review**: 2025-12-29  
**Owner**: μNet Operations Team
