# μNet Operational Runbooks

> **Critical Operations Reference**: Essential runbooks for μNet system administration and incident response

## Runbook Overview

This directory contains comprehensive operational runbooks for managing, maintaining, and troubleshooting the μNet network configuration management system. These runbooks are designed for operations teams, system administrators, and on-call engineers.

### Available Runbooks

| Runbook | Purpose | Audience | Urgency |
|---------|---------|----------|---------|
| [Incident Response](incident-response.md) | Emergency incident handling procedures | On-call engineers, SRE | Critical |
| [Maintenance Procedures](maintenance-procedures.md) | Scheduled maintenance and system care | Operations teams, administrators | Routine |
| [Troubleshooting Guide](troubleshooting-guide.md) | Common issues and solutions | Support teams, administrators | As-needed |

## Quick Access by Scenario

### 🚨 Emergency Situations

- **System Down**: [Incident Response → P0 Critical](incident-response.md#p0---critical-system-down)
- **Security Breach**: [Incident Response → Security Incidents](incident-response.md#security-incidents)
- **Data Corruption**: [Troubleshooting → Database Issues](troubleshooting-guide.md#database-corruption)
- **Complete Recovery**: [Troubleshooting → Emergency Recovery](troubleshooting-guide.md#emergency-recovery)

### ⚡ Performance Issues

- **High CPU Usage**: [Incident Response → High CPU Alert](incident-response.md#high-cpu-usage-alert)
- **Memory Problems**: [Troubleshooting → High Memory Usage](troubleshooting-guide.md#high-memory-usage)
- **API Timeouts**: [Troubleshooting → API Response Timeouts](troubleshooting-guide.md#api-response-timeouts)
- **Database Slow**: [Troubleshooting → Database Issues](troubleshooting-guide.md#database-issues)

### 🔧 Maintenance Tasks

- **Scheduled Updates**: [Maintenance → Monthly Maintenance](maintenance-procedures.md#monthly-maintenance-window)
- **Database Optimization**: [Maintenance → Database Maintenance](maintenance-procedures.md#database-maintenance)
- **Backup Procedures**: [Maintenance → Backup and Recovery](maintenance-procedures.md#backup-and-recovery)
- **Security Updates**: [Maintenance → Security Maintenance](maintenance-procedures.md#security-maintenance)

### 🔍 Troubleshooting

- **Startup Problems**: [Troubleshooting → Startup Issues](troubleshooting-guide.md#startup-issues)
- **Authentication Issues**: [Troubleshooting → Authentication Issues](troubleshooting-guide.md#authentication-issues)
- **Git Sync Problems**: [Troubleshooting → Git Integration Issues](troubleshooting-guide.md#git-integration-issues)
- **Network Issues**: [Troubleshooting → Network Issues](troubleshooting-guide.md#network-issues)

## Runbook Usage Guidelines

### When to Use Runbooks

#### Always Use Runbooks For

- **P0/P1 Incidents**: Follow incident response procedures exactly
- **Scheduled Maintenance**: Use maintenance checklists to prevent errors
- **Recurring Issues**: Reference troubleshooting guides for consistency
- **New Team Members**: Onboarding and training reference

#### Adapt Runbooks For

- **Unique Environments**: Modify commands for your specific setup
- **Custom Configurations**: Adjust procedures for local variations
- **New Tools**: Update procedures when adopting new monitoring/management tools

### Runbook Best Practices

#### During Incidents

1. **Follow the Script**: Stick to documented procedures under pressure
2. **Document Deviations**: Note any steps that don't work as expected
3. **Time Tracking**: Record timing for post-incident analysis
4. **Communication**: Use communication templates provided

#### During Maintenance

1. **Pre-Check Lists**: Complete all pre-maintenance verifications
2. **Change Windows**: Respect scheduled maintenance windows
3. **Rollback Ready**: Have rollback procedures prepared before starting
4. **Validation**: Complete all post-maintenance verification steps

#### During Troubleshooting

1. **Start Simple**: Begin with quick diagnostic steps
2. **Document Findings**: Record what you discover for future reference
3. **Escalate Appropriately**: Know when to involve other teams
4. **Update Runbooks**: Improve procedures based on new issues

## Environment-Specific Adaptations

### Development Environment

```bash
# Example environment variables for dev
export UNET_ENV=development
export UNET_LOG_LEVEL=debug
export UNET_DB_PATH=/dev/data/unet.db
export UNET_CONFIG=/etc/unet/dev-config.toml
```

### Staging Environment

```bash
# Example environment variables for staging
export UNET_ENV=staging
export UNET_LOG_LEVEL=info
export UNET_DB_PATH=/staging/data/unet.db
export UNET_CONFIG=/etc/unet/staging-config.toml
```

### Production Environment

```bash
# Example environment variables for production
export UNET_ENV=production
export UNET_LOG_LEVEL=warn
export UNET_DB_PATH=/prod/data/unet.db
export UNET_CONFIG=/etc/unet/prod-config.toml
```

## Common Command References

### Health Checks

```bash
# System health check
curl http://localhost:8080/api/v1/health | jq '.'

# Metrics snapshot
curl http://localhost:8080/api/v1/metrics/snapshot | jq '.'

# Service status
systemctl status unet-server

# Log monitoring
journalctl -u unet-server -f
```

### Emergency Commands

```bash
# Emergency stop
systemctl stop unet-server
pkill -9 unet-server

# Emergency backup
cp /path/to/unet.db /backups/emergency-$(date +%Y%m%d_%H%M%S).db

# Emergency restart
systemctl restart unet-server

# Database integrity check
sqlite3 /path/to/unet.db "PRAGMA integrity_check;"
```

### Monitoring Commands

```bash
# Resource usage
ps aux | grep unet-server
htop -p $(pgrep unet-server)

# Network connections
netstat -tulpn | grep :8080
lsof -i :8080

# Disk usage
df -h /path/to/database/
du -sh /path/to/unet.db
```

## Integration with Monitoring

### Grafana Dashboards

Link to relevant dashboards for visual confirmation:

- **System Health**: `/d/unet-system-health`
- **Performance**: `/d/unet-performance`
- **Alerting**: `/d/unet-alerting`

### Prometheus Alerts

Reference alert names in runbooks:

- `UNetServerDown`
- `UNetHighCPUUsage`
- `UNetHighMemoryUsage`
- `UNetDatabaseConnectionFailure`
- `UNetAuthenticationFailureSpike`

### Log Aggregation

Standard log queries for common issues:

```bash
# Error analysis
journalctl -u unet-server | grep ERROR | tail -20

# Performance issues
journalctl -u unet-server | grep "slow_query\|timeout" | tail -10

# Security events
journalctl -u unet-server | grep "auth_failure\|security" | tail -15
```

## Escalation Procedures

### Internal Escalation Chain

1. **Level 1**: On-call engineer (immediate response)
2. **Level 2**: Senior operations engineer (15 minutes)
3. **Level 3**: Engineering manager (30 minutes)
4. **Level 4**: Director of engineering (1 hour)

### External Dependencies

- **Database Team**: For persistent storage issues
- **Network Team**: For connectivity problems
- **Security Team**: For security incidents
- **Vendor Support**: For third-party component issues

### Communication Channels

- **Primary**: Slack #unet-ops
- **Secondary**: PagerDuty escalation
- **Emergency**: Phone call tree
- **Status Updates**: Email distribution list

## Runbook Maintenance

### Regular Updates

- **Monthly**: Review and update based on new incidents
- **Quarterly**: Complete runbook review with ops team
- **After Incidents**: Update based on lessons learned
- **Version Releases**: Update for new features/changes

### Change Management

- All runbook changes require peer review
- Test procedure changes in non-production first
- Maintain version control with clear change logs
- Notify team of significant procedure updates

### Training and Certification

- **New Team Members**: Complete runbook walkthrough
- **Quarterly Drills**: Practice incident response procedures
- **Annual Review**: Comprehensive training on all runbooks
- **Certification**: Validate team competency with runbook usage

## Support Resources

### Documentation Links

- [System Architecture](../src/01_architecture.md)
- [API Reference](../src/api-reference.md)
- [Configuration Guide](../src/configuration.md)
- [Security Guide](../src/security.md)

### External Resources

- [SQLite Documentation](https://www.sqlite.org/docs.html)
- [Prometheus Query Guide](https://prometheus.io/docs/prometheus/latest/querying/)
- [systemd Service Management](https://www.freedesktop.org/software/systemd/man/systemctl.html)

### Community Support

- **GitHub Issues**: Bug reports and feature requests
- **Discord Channel**: Real-time community support
- **Documentation Wiki**: Community-maintained guides
- **Stack Overflow**: Tagged questions and answers

---

**Document Version**: 1.0  
**Last Updated**: 2025-06-29  
**Next Review**: 2025-12-29  
**Owner**: μNet Operations Team

*These runbooks are living documents. Please update them based on your operational experience and contribute improvements back to the team.*
