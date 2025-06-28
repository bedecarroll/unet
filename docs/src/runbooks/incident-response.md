# μNet Incident Response Runbook

> **Critical Reference**: Follow this runbook for all μNet system incidents

## Incident Classification

### Severity Levels

#### P0 - Critical (System Down)

- **Response Time**: 15 minutes
- **Definition**: Complete system outage, data loss, security breach
- **Examples**:
  - μNet server completely unresponsive
  - Database corruption or loss
  - Authentication system compromised
  - Network configuration causing outages

#### P1 - High (Major Impact)  

- **Response Time**: 1 hour
- **Definition**: Significant feature degradation affecting multiple users
- **Examples**:
  - API endpoints returning 5xx errors
  - Policy evaluation failures
  - Git sync completely broken
  - SNMP monitoring down

#### P2 - Medium (Limited Impact)

- **Response Time**: 4 hours  
- **Definition**: Single feature affected or performance degradation
- **Examples**:
  - Specific API endpoints slow
  - Template rendering issues
  - Single node monitoring failure
  - High memory usage

#### P3 - Low (Minor Impact)

- **Response Time**: 24 hours
- **Definition**: Minor issues not affecting core functionality
- **Examples**:
  - Dashboard display issues
  - Non-critical logging errors
  - Documentation inconsistencies

## Alert Response Procedures

### High CPU Usage Alert

**Alert**: `unet_cpu_usage_percent > 80%`

1. **Immediate Actions** (0-5 minutes)

   ```bash
   # Check system process usage
   top -p $(pgrep unet-server)
   
   # Check μNet server logs
   journalctl -u unet-server --since "10 minutes ago"
   
   # Verify alert is not false positive
   curl http://localhost:8080/api/v1/metrics/snapshot
   ```

2. **Investigation** (5-15 minutes)

   ```bash
   # Check for runaway processes
   ps aux | grep unet | head -20
   
   # Review recent configuration changes
   curl http://localhost:8080/api/v1/changes?limit=10
   
   # Check SNMP query load
   curl http://localhost:8080/api/v1/metrics/snapshot | jq '.performance.snmp_query_duration_avg'
   ```

3. **Mitigation Actions**
   - **Temporary**: Restart μNet server if CPU > 95%
   - **Scaling**: Increase resource allocation
   - **Investigation**: Identify resource-intensive operations

### Authentication Failure Spike

**Alert**: `rate(unet_auth_failures_total[5m]) > 10`

1. **Security Assessment** (0-2 minutes)

   ```bash
   # Check recent authentication attempts
   curl http://localhost:8080/api/v1/auth/attempts?limit=50
   
   # Review security audit log
   journalctl -u unet-server | grep "auth_failure" | tail -20
   
   # Check for IP patterns
   curl http://localhost:8080/api/v1/security/events?type=auth_failure
   ```

2. **Immediate Protection** (2-5 minutes)

   ```bash
   # Block suspicious IPs if identified
   curl -X POST http://localhost:8080/api/v1/security/block-ip \
     -H "Content-Type: application/json" \
     -d '{"ip": "1.2.3.4", "reason": "Authentication attack"}'
   
   # Enable additional rate limiting
   curl -X PUT http://localhost:8080/api/v1/config/rate-limits \
     -H "Content-Type: application/json" \
     -d '{"auth_attempts_per_minute": 3}'
   ```

3. **Investigation Actions**
   - Review application logs for attack patterns
   - Check network access controls
   - Verify user account integrity
   - Consider temporary authentication restrictions

### Database Connection Issues

**Alert**: `unet_database_connections_active == 0`

1. **Database Health Check** (0-3 minutes)

   ```bash
   # Test database connectivity
   sqlite3 /path/to/unet.db "SELECT 1;"
   
   # Check database file permissions
   ls -la /path/to/unet.db
   
   # Verify disk space
   df -h /path/to/database/
   ```

2. **Connection Recovery** (3-10 minutes)

   ```bash
   # Restart μNet server to reset connections
   systemctl restart unet-server
   
   # Monitor connection recovery
   watch "curl -s http://localhost:8080/api/v1/health | jq '.database'"
   
   # Check for connection pool exhaustion
   curl http://localhost:8080/api/v1/metrics/snapshot | jq '.system.database_pool_size'
   ```

3. **Prevention Actions**
   - Increase database connection pool size
   - Implement connection health checks
   - Monitor for connection leaks

### Git Sync Failures

**Alert**: Git sync operations failing consistently

1. **Git Repository Verification** (0-5 minutes)

   ```bash
   # Check Git repository status
   curl http://localhost:8080/api/v1/git/status
   
   # Test Git connectivity
   curl http://localhost:8080/api/v1/git/repositories
   
   # Check recent sync attempts
   curl http://localhost:8080/api/v1/git/sync/history?limit=10
   ```

2. **Connectivity Testing** (5-10 minutes)

   ```bash
   # Test network connectivity to Git repositories
   nc -zv git.example.com 443
   
   # Check SSH key validity (if using SSH)
   ssh-keygen -l -f ~/.ssh/id_rsa
   
   # Verify Git credentials
   curl http://localhost:8080/api/v1/git/credentials/test
   ```

3. **Recovery Actions**
   - Update Git credentials if expired
   - Clear Git cache and retry sync
   - Switch to backup Git repository if available
   - Manual Git operations as fallback

## Emergency Procedures

### System Recovery

#### Complete System Restart

```bash
# 1. Stop μNet server gracefully
systemctl stop unet-server

# 2. Verify no processes remain
pgrep unet-server && pkill -9 unet-server

# 3. Check database integrity
sqlite3 /path/to/unet.db "PRAGMA integrity_check;"

# 4. Restart with logging
systemctl start unet-server
journalctl -u unet-server -f
```

#### Database Recovery

```bash
# 1. Stop μNet server
systemctl stop unet-server

# 2. Backup current database
cp /path/to/unet.db /path/to/unet.db.backup.$(date +%Y%m%d_%H%M%S)

# 3. Run database repair
sqlite3 /path/to/unet.db "VACUUM;"
sqlite3 /path/to/unet.db "REINDEX;"

# 4. Verify database integrity
sqlite3 /path/to/unet.db "PRAGMA integrity_check;"

# 5. Restart service
systemctl start unet-server
```

#### Configuration Rollback

```bash
# 1. Identify last known good configuration
curl http://localhost:8080/api/v1/changes?status=successful&limit=1

# 2. Perform rollback
curl -X POST http://localhost:8080/api/v1/changes/{change_id}/rollback

# 3. Verify system stability
curl http://localhost:8080/api/v1/health
```

### Security Incidents

#### Suspected Compromise

1. **Immediate Isolation**

   ```bash
   # Block all external access
   iptables -A INPUT -p tcp --dport 8080 -j DROP
   
   # Stop μNet server
   systemctl stop unet-server
   ```

2. **Evidence Collection**

   ```bash
   # Collect system logs
   journalctl -u unet-server --since "1 hour ago" > /tmp/unet-incident-logs.txt
   
   # Collect security events
   curl http://localhost:8080/api/v1/security/audit > /tmp/security-audit.json
   
   # System state snapshot
   ps aux > /tmp/processes.txt
   netstat -tulpn > /tmp/network-connections.txt
   ```

3. **Recovery Planning**
   - Contact security team
   - Plan clean system rebuild
   - Credential rotation procedures

#### Data Breach Response

1. **Immediate Actions**
   - Stop all system access
   - Preserve evidence
   - Notify stakeholders

2. **Assessment**
   - Determine scope of access
   - Identify compromised data
   - Timeline reconstruction

3. **Recovery**
   - System rebuild from clean images
   - Credential rotation
   - Access control review

## Communication Procedures

### Incident Communication

#### Internal Escalation

- **P0/P1**: Immediate phone call to on-call engineer
- **P2**: Slack/Teams notification within 30 minutes
- **P3**: Email notification within 4 hours

#### Status Updates

- **P0**: Every 15 minutes during active response
- **P1**: Every 30 minutes during active response  
- **P2/P3**: Hourly updates during business hours

#### External Communication

- **Customer Impact**: Notify affected customers within 1 hour
- **Status Page**: Update status page for P0/P1 incidents
- **Regulatory**: Follow data breach notification requirements

### Post-Incident Review

#### Required Documentation

1. Timeline of events
2. Root cause analysis
3. Impact assessment
4. Response effectiveness review
5. Action items for improvement

#### Review Meeting

- Schedule within 48 hours of resolution
- Include all response team members
- Document lessons learned
- Update runbooks based on findings

## Tools and Resources

### Monitoring and Dashboards

- **Grafana**: <http://grafana.example.com/d/unet-system-health>
- **Prometheus**: <http://prometheus.example.com>
- **μNet Health**: <http://localhost:8080/api/v1/health>

### Log Analysis

```bash
# Real-time log monitoring
journalctl -u unet-server -f

# Error log analysis
journalctl -u unet-server --since "1 hour ago" | grep ERROR

# Performance issue investigation
curl http://localhost:8080/api/v1/metrics/snapshot | jq '.performance'
```

### Emergency Contacts

- **Primary On-Call**: [Phone/Slack]
- **Secondary On-Call**: [Phone/Slack]
- **Engineering Manager**: [Phone/Email]
- **Security Team**: [Phone/Email]

### Recovery Resources

- **Backup Location**: `/backups/unet/`
- **Documentation**: `docs/`
- **Configuration Templates**: `config/templates/`
- **Emergency Scripts**: `scripts/emergency/`

## Preventive Measures

### Regular Health Checks

```bash
# Daily health verification
curl http://localhost:8080/api/v1/health | jq '.'

# Weekly performance review
curl http://localhost:8080/api/v1/metrics/snapshot | jq '.performance'

# Monthly security audit
curl http://localhost:8080/api/v1/security/audit/summary
```

### Maintenance Windows

- **Frequency**: Monthly
- **Duration**: 2 hours
- **Activities**: Updates, patches, performance optimization
- **Notification**: 7 days advance notice

### Training and Preparedness

- **Runbook Review**: Quarterly
- **Incident Response Drill**: Bi-annually
- **Security Training**: Annually
- **Tool Familiarization**: Ongoing

---

**Document Version**: 1.0  
**Last Updated**: 2025-06-29  
**Next Review**: 2025-09-29  
**Owner**: μNet Operations Team
