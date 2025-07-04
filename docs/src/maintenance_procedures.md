<!-- SPDX-License-Identifier: MIT -->

# Maintenance and Update Procedures

This document outlines the maintenance and update procedures for μNet installations, covering everything from routine maintenance to major version upgrades.

## 🎯 Quick Reference

### Maintenance Schedule

- **Daily**: Log monitoring, health checks
- **Weekly**: Dependency updates, security patches
- **Monthly**: Database maintenance, performance review
- **Quarterly**: Major updates, architecture review

### Emergency Contacts

- **Critical Issues**: Create GitHub issue with `priority/critical` label
- **Security**: <security@your-org.com>
- **Support**: <support@your-org.com>

## 📋 Routine Maintenance

### Daily Operations

#### 1. Health Monitoring

```bash
# Check μNet server status
systemctl status unet-server

# Verify API responsiveness
curl -f http://localhost:8080/health || echo "Health check failed"

# Check log for errors (last 24 hours)
journalctl -u unet-server --since "24 hours ago" --grep ERROR
```

#### 2. Database Health

```bash
# SQLite health check
unet database health-check

# PostgreSQL health check (if using PostgreSQL)
psql -d unet -c "SELECT version();"
psql -d unet -c "SELECT pg_size_pretty(pg_database_size('unet'));"
```

#### 3. Disk Space Monitoring

```bash
# Check available disk space
df -h /var/lib/unet
df -h /var/log/unet

# Log rotation status
ls -la /var/log/unet/*.gz | wc -l
```

### Weekly Maintenance

#### 1. Security Updates

```bash
# Update μNet (check release notes first)
unet --version
curl -s https://api.github.com/repos/your-org/unet/releases/latest

# System security updates
sudo apt update && sudo apt upgrade -y  # Ubuntu/Debian
sudo yum update -y  # RHEL/CentOS
# or
sudo dnf update -y  # Fedora
```

#### 2. Dependency Audit

```bash
# Check for outdated dependencies
cargo audit  # For source installations

# Container security scan
docker scout cves unet:latest  # If using Docker
```

#### 3. Backup Verification

```bash
# Test backup restoration (on staging environment)
unet database backup --output /tmp/test-backup.sql
unet database restore --input /tmp/test-backup.sql --dry-run
```

### Monthly Maintenance

#### 1. Database Optimization

```bash
# SQLite optimization
unet database optimize

# PostgreSQL maintenance
psql -d unet -c "VACUUM ANALYZE;"
psql -d unet -c "REINDEX DATABASE unet;"
```

#### 2. Performance Review

```bash
# Generate performance report
unet system performance-report --period 30d

# Review slow queries
unet database slow-queries --limit 10

# Check resource usage trends
unet system resource-usage --period 30d
```

#### 3. Log Analysis

```bash
# Analyze error patterns
journalctl -u unet-server --since "30 days ago" --grep ERROR | \
  awk '{print $NF}' | sort | uniq -c | sort -nr

# Check for anomalies
unet logs analyze --period 30d --anomalies
```

## 🔄 Update Procedures

### Version Update Strategy

#### Semantic Versioning

- **Patch (0.1.0 → 0.1.1)**: Bug fixes, safe to auto-update
- **Minor (0.1.0 → 0.2.0)**: New features, backward compatible
- **Major (0.1.0 → 1.0.0)**: Breaking changes, requires planning

#### Update Channels

- **Stable**: Production-ready releases
- **Beta**: Feature previews, testing recommended
- **Nightly**: Development builds, not for production

### Pre-Update Checklist

Before any update:

- [ ] Review release notes and changelog
- [ ] Check breaking changes and migration requirements
- [ ] Create full system backup
- [ ] Verify staging environment compatibility
- [ ] Schedule maintenance window
- [ ] Notify stakeholders
- [ ] Prepare rollback plan

### Patch Updates (0.1.0 → 0.1.1)

**Risk Level**: Low  
**Downtime**: Minimal (< 5 minutes)  
**Rollback**: Simple

```bash
# 1. Create backup
unet database backup --output backup-pre-$(date +%Y%m%d).sql

# 2. Download and verify new version
wget https://releases.your-org.com/unet/v0.1.1/unet-linux-amd64.tar.gz
wget https://releases.your-org.com/unet/v0.1.1/checksums.txt
sha256sum -c checksums.txt

# 3. Stop service
sudo systemctl stop unet-server

# 4. Replace binaries
sudo tar -xzf unet-linux-amd64.tar.gz -C /usr/local/bin/

# 5. Start service
sudo systemctl start unet-server

# 6. Verify health
curl -f http://localhost:8080/health
unet --version
```

### Minor Updates (0.1.0 → 0.2.0)

**Risk Level**: Medium  
**Downtime**: 5-15 minutes  
**Rollback**: Moderate complexity

```bash
# 1. Extended pre-update checks
unet system compatibility-check --target-version 0.2.0
unet database migration-preview --target-version 0.2.0

# 2. Create comprehensive backup
unet system full-backup --output backup-$(date +%Y%m%d)

# 3. Stop all services
sudo systemctl stop unet-server
sudo systemctl stop unet-background-tasks  # if separate service

# 4. Update binaries
sudo tar -xzf unet-v0.2.0-linux-amd64.tar.gz -C /usr/local/bin/

# 5. Run database migrations
unet database migrate --version 0.2.0

# 6. Update configuration if needed
unet config migrate --from 0.1.0 --to 0.2.0

# 7. Start services
sudo systemctl start unet-server
sudo systemctl start unet-background-tasks

# 8. Comprehensive verification
unet system post-update-check --version 0.2.0
```

### Major Updates (0.x.0 → 1.0.0)

**Risk Level**: High  
**Downtime**: 30+ minutes  
**Rollback**: Complex, may require full restore

Major updates require comprehensive planning:

#### 1. Planning Phase (1-2 weeks before)

- [ ] Read complete migration guide
- [ ] Test on staging environment
- [ ] Identify breaking changes
- [ ] Plan configuration updates
- [ ] Schedule extended maintenance window
- [ ] Prepare detailed rollback procedures

#### 2. Pre-Update Phase (Day of update)

```bash
# Complete system snapshot
unet system snapshot create --name pre-major-update-$(date +%Y%m%d)

# Export all data
unet export --format json --output full-export-$(date +%Y%m%d).json

# Document current configuration
cp -r /etc/unet /etc/unet.backup.$(date +%Y%m%d)
```

#### 3. Update Phase

```bash
# Follow detailed migration guide for specific version
# This is version-specific and will be provided in release notes

# Example migration steps:
sudo systemctl stop unet-server
unet database backup --output pre-major-$(date +%Y%m%d).sql
sudo tar -xzf unet-v1.0.0-linux-amd64.tar.gz -C /usr/local/bin/
unet database migrate --major-version 1.0.0
unet config migrate --major-version 1.0.0
sudo systemctl start unet-server
```

#### 4. Post-Update Verification

```bash
# Comprehensive system check
unet system verify --all
unet database integrity-check
unet config validate --all-policies --all-templates

# Functional testing
unet nodes list
unet policies list
unet templates list
```

## 🐳 Container Updates

### Docker Updates

#### Single Container

```bash
# 1. Pull new image
docker pull unet:0.2.0

# 2. Stop current container
docker stop unet-server

# 3. Backup data volume
docker run --rm -v unet_data:/data -v $(pwd):/backup \
  ubuntu tar czf /backup/unet-backup-$(date +%Y%m%d).tar.gz /data

# 4. Start new container
docker run -d --name unet-server-new \
  -v unet_data:/data \
  -p 8080:8080 \
  unet:0.2.0

# 5. Verify and cleanup
curl -f http://localhost:8080/health
docker rm unet-server
docker rename unet-server-new unet-server
```

#### Docker Compose

```bash
# 1. Update docker-compose.yml with new version
sed -i 's/unet:0.1.0/unet:0.2.0/' docker-compose.yml

# 2. Create backup
docker-compose exec db pg_dump -U unet unet > backup-$(date +%Y%m%d).sql

# 3. Update services
docker-compose pull
docker-compose up -d

# 4. Verify health
docker-compose ps
curl -f http://localhost:8080/health
```

### Kubernetes Updates

#### Using kubectl

```bash
# 1. Update deployment manifest
kubectl patch deployment unet-server \
  -p '{"spec":{"template":{"spec":{"containers":[{"name":"unet","image":"unet:0.2.0"}]}}}}'

# 2. Monitor rollout
kubectl rollout status deployment/unet-server

# 3. Verify health
kubectl get pods -l app=unet-server
kubectl logs -l app=unet-server --tail=100
```

#### Using Helm

```bash
# 1. Update values.yaml or set new image
helm upgrade unet ./helm/unet \
  --set image.tag=0.2.0 \
  --wait --timeout=300s

# 2. Verify deployment
helm status unet
kubectl get pods -l app.kubernetes.io/name=unet
```

## 🔧 Configuration Management

### Configuration Validation

```bash
# Validate all configurations
unet config validate --strict

# Check for deprecated options
unet config check-deprecated

# Verify policy syntax
unet policies validate --all

# Test template rendering
unet templates test --all
```

### Configuration Migration

```bash
# Backup current config
cp /etc/unet/config.toml /etc/unet/config.toml.backup

# Migrate configuration format
unet config migrate --from-version 0.1.0 --to-version 0.2.0

# Validate migrated config
unet config validate /etc/unet/config.toml
```

### Environment-Specific Updates

#### Development Environment

```bash
# More aggressive update schedule
unet update --channel beta --auto-confirm

# Enable debug logging
unet config set logging.level debug
```

#### Staging Environment

```bash
# Test production procedures
unet update --dry-run --target-version 0.2.0
unet system compatibility-test --version 0.2.0
```

#### Production Environment

```bash
# Conservative update approach
unet update --stable-only --require-confirmation
unet system pre-update-check --comprehensive
```

## 📊 Monitoring and Alerting

### Health Monitoring

```bash
# Set up monitoring script
cat > /usr/local/bin/unet-health-check << 'EOF'
#!/bin/bash
if ! curl -f http://localhost:8080/health >/dev/null 2>&1; then
    echo "CRITICAL: μNet health check failed"
    # Send alert (email, Slack, PagerDuty, etc.)
    exit 1
fi
echo "OK: μNet is healthy"
EOF

chmod +x /usr/local/bin/unet-health-check
```

### Automated Monitoring Setup

```bash
# Add to crontab
crontab -e
# Add line: */5 * * * * /usr/local/bin/unet-health-check

# Systemd timer (alternative)
sudo systemctl enable unet-health-check.timer
sudo systemctl start unet-health-check.timer
```

### Performance Monitoring

```bash
# Set up performance baselines
unet metrics baseline create --name initial-deployment

# Regular performance checks
unet metrics compare --baseline initial-deployment --current
```

## 🚨 Troubleshooting Updates

### Common Update Issues

#### Service Won't Start After Update

```bash
# Check logs
journalctl -u unet-server --since "10 minutes ago"

# Verify configuration
unet config validate

# Check permissions
ls -la /usr/local/bin/unet*
ls -la /etc/unet/

# Test configuration
unet server start --dry-run
```

#### Database Migration Failures

```bash
# Check migration status
unet database migration status

# Retry failed migration
unet database migrate --retry --verbose

# Rollback if necessary
unet database rollback --to-version 0.1.0
```

#### Configuration Incompatibilities

```bash
# Check for breaking changes
unet config check-compatibility --target-version 0.2.0

# Generate migration script
unet config migration-script --from 0.1.0 --to 0.2.0

# Apply manual fixes
unet config fix-compatibility --interactive
```

### Rollback Procedures

#### Quick Rollback (Patch/Minor)

```bash
# Stop service
sudo systemctl stop unet-server

# Restore previous binaries
sudo tar -xzf unet-v0.1.0-linux-amd64.tar.gz -C /usr/local/bin/

# Rollback database if needed
unet database rollback --to-version 0.1.0

# Start service
sudo systemctl start unet-server
```

#### Full System Restore (Major)

```bash
# Stop all services
sudo systemctl stop unet-server

# Restore database
unet database restore --input backup-pre-major-$(date +%Y%m%d).sql

# Restore configuration
sudo rm -rf /etc/unet
sudo mv /etc/unet.backup.$(date +%Y%m%d) /etc/unet

# Restore binaries
sudo tar -xzf unet-v0.1.0-linux-amd64.tar.gz -C /usr/local/bin/

# Start services
sudo systemctl start unet-server

# Verify restoration
unet system verify --all
```

## 📋 Maintenance Checklists

### Pre-Maintenance Checklist

- [ ] Review planned changes
- [ ] Create system backup
- [ ] Verify staging environment
- [ ] Schedule maintenance window
- [ ] Notify stakeholders
- [ ] Prepare rollback plan
- [ ] Document current system state

### Post-Maintenance Checklist

- [ ] Verify system health
- [ ] Test critical functionality
- [ ] Check performance metrics
- [ ] Review error logs
- [ ] Update documentation
- [ ] Notify stakeholders of completion
- [ ] Schedule follow-up checks

### Emergency Response Checklist

- [ ] Assess impact and severity
- [ ] Implement immediate mitigation
- [ ] Create incident report
- [ ] Communicate with stakeholders
- [ ] Execute fix or rollback
- [ ] Verify resolution
- [ ] Conduct post-incident review

## 📚 Additional Resources

### Documentation

- [Troubleshooting Guide](troubleshooting_guide.md)
- [Production Deployment Guide](production_deployment_guide.md)
- [Security Compliance Guide](security_compliance_guide.md)
- [API Reference](api_reference.md)

### External Tools

- **Monitoring**: Prometheus, Grafana, Nagios
- **Backup**: Restic, Borgbackup, native database tools
- **Automation**: Ansible, Puppet, Chef, SaltStack

### Support Channels

- [GitHub Issues](https://github.com/your-org/unet/issues)
- [Community Discussions](https://github.com/your-org/unet/discussions)
- [Community Support](community_support.md)

---

*This maintenance guide is updated with each release. Always refer to the version-specific documentation for your μNet installation.*
