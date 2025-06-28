# μNet Grafana Dashboards

This directory contains comprehensive Grafana dashboard templates for monitoring and visualizing μNet network configuration management system metrics.

## Dashboard Overview

### 1. System Overview (`unet-overview.json`)

**Primary Use**: Executive and high-level operational overview

- **Key Metrics**: Total nodes, active connections, CPU/memory usage, HTTP request rates
- **Audience**: Operations teams, executives, stakeholders
- **Refresh**: 30 seconds
- **Features**: Clean overview layout with status indicators and trend graphs

### 2. Business Metrics (`unet-business-metrics.json`)

**Primary Use**: Business KPI monitoring and operational analytics

- **Key Metrics**: User counts, policy evaluations, template renderings, configuration changes
- **Audience**: Product managers, business analysts, operations teams
- **Refresh**: 30 seconds
- **Features**: Business-focused metrics with authentication monitoring

### 3. Performance Monitoring (`unet-performance.json`)

**Primary Use**: Technical performance analysis and optimization

- **Key Metrics**: Request duration percentiles, database performance, system utilization
- **Audience**: DevOps engineers, system administrators, developers
- **Refresh**: 30 seconds
- **Features**: Detailed performance metrics with percentile analysis

### 4. System Health Overview (`unet-system-health.json`)

**Primary Use**: Real-time system health monitoring

- **Key Metrics**: Service uptime, resource utilization, connection health, activity rates
- **Audience**: Operations teams, SRE, on-call engineers
- **Refresh**: 10 seconds (real-time)
- **Features**: Health status indicators with fast refresh for incident response

### 5. Alerting & Monitoring (`unet-alerting.json`)

**Primary Use**: Alert status and threshold monitoring

- **Key Metrics**: Active alerts, critical alerts, authentication failures, response times
- **Audience**: Operations teams, SRE, incident response
- **Refresh**: 10 seconds (real-time)
- **Features**: Alert timeline and threshold visualization for proactive monitoring

### 6. Custom Metrics Visualization (`unet-custom-metrics.json`)

**Primary Use**: Flexible analysis and custom metric exploration

- **Key Metrics**: Configurable metrics with templating variables
- **Audience**: Data analysts, developers, operations teams
- **Refresh**: 30 seconds
- **Features**: Template variables for dynamic metric selection and correlation analysis

## Available Metrics

μNet exposes the following Prometheus metrics:

### Business Metrics

- `unet_nodes_total` - Total number of managed network nodes
- `unet_users_total` - Total number of system users
- `unet_policy_evaluations_total` - Total policy evaluations performed
- `unet_template_renderings_total` - Total template renderings
- `unet_config_changes_total` - Total configuration changes
- `unet_git_sync_operations_total` - Total Git synchronization operations
- `unet_snmp_queries_total` - Total SNMP queries executed

### Authentication Metrics

- `unet_auth_attempts_total` - Total authentication attempts
- `unet_auth_failures_total` - Total authentication failures

### Performance Metrics

- `unet_http_requests_total` - Total HTTP requests
- `unet_http_request_duration_seconds` - HTTP request duration histogram
- `unet_database_query_duration_seconds` - Database query duration histogram

### System Metrics

- `unet_cpu_usage_percent` - CPU usage percentage
- `unet_memory_usage_bytes` - Memory usage in bytes
- `unet_disk_usage_bytes` - Disk usage in bytes
- `unet_active_connections` - Number of active network connections
- `unet_background_tasks_active` - Number of active background tasks

### Database Metrics

- `unet_database_connections_active` - Active database connections
- `unet_database_connections_total` - Total database connection pool size

## Installation and Setup

### Prerequisites

- Grafana 9.0+ installed and running
- Prometheus configured to scrape μNet metrics endpoint
- μNet server running with metrics enabled

### 1. Copy Dashboard Files

```bash
# Copy dashboard files to your Grafana provisioning directory
sudo mkdir -p /etc/grafana/provisioning/dashboards/unet
sudo cp *.json /etc/grafana/provisioning/dashboards/unet/
sudo chown -R grafana:grafana /etc/grafana/provisioning/dashboards/unet
```

### 2. Configure Dashboard Provisioning

```bash
# Copy provisioning configuration
sudo cp provisioning.yml /etc/grafana/provisioning/dashboards/unet.yml
sudo chown grafana:grafana /etc/grafana/provisioning/dashboards/unet.yml
```

### 3. Configure Prometheus Data Source

1. Access Grafana UI (typically <http://localhost:3000>)
2. Go to Configuration → Data Sources
3. Add Prometheus data source
4. Set URL to your Prometheus instance (e.g., <http://localhost:9090>)
5. Test connection and save

### 4. Restart Grafana

```bash
sudo systemctl restart grafana-server
```

### 5. Verify Dashboard Import

1. Navigate to Dashboards → Browse
2. Look for "μNet" folder
3. Verify all 6 dashboards are imported

## Prometheus Configuration

Ensure your Prometheus configuration includes μNet metrics scraping:

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'unet-server'
    static_configs:
      - targets: ['localhost:8080']  # Adjust to your μNet server address
    metrics_path: '/metrics'
    scrape_interval: 15s
```

## μNet Server Configuration

Enable metrics in your μNet server configuration:

```toml
# config.toml
[metrics]
enabled = true
endpoint = "/metrics"
collection_interval = 15
enable_performance_metrics = true
enable_business_metrics = true
enable_system_metrics = true
```

## Customization

### Dashboard Variables

Most dashboards include template variables for customization:

- **DS_PROMETHEUS**: Prometheus data source selector
- **Custom Metric Variables**: Available in custom metrics dashboard for dynamic queries

### Alert Integration

The alerting dashboard displays Prometheus alert states. Configure Prometheus alerting rules to see alert status:

```yaml
# Example alerting rule
groups:
  - name: unet-alerts
    rules:
      - alert: UNetHighCPUUsage
        expr: unet_cpu_usage_percent > 80
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "μNet server high CPU usage"
          description: "CPU usage is {{ $value }}%"
```

### Custom Metrics

Use the custom metrics dashboard to:

- Explore specific metric correlations
- Create ad-hoc visualizations
- Analyze business trends
- Monitor custom business logic

## Troubleshooting

### Common Issues

1. **No data in dashboards**
   - Verify Prometheus is scraping μNet metrics endpoint
   - Check μNet server metrics configuration is enabled
   - Ensure data source is configured correctly

2. **Missing metrics**
   - Check μNet server logs for metric collection errors
   - Verify specific metric categories are enabled in configuration
   - Ensure μNet server version supports all metrics

3. **Performance issues**
   - Adjust dashboard refresh intervals for your environment
   - Consider using recording rules for expensive queries
   - Optimize Prometheus retention settings

### Useful Commands

```bash
# Check μNet metrics endpoint directly
curl http://localhost:8080/metrics

# Verify Prometheus is scraping μNet
curl http://localhost:9090/api/v1/targets

# Check Grafana dashboard provisioning logs
sudo journalctl -u grafana-server -f
```

## Dashboard Maintenance

### Regular Tasks

- Review and update alert thresholds based on system behavior
- Customize time ranges based on operational needs  
- Add new panels for additional metrics as μNet evolves
- Update dashboard variables for new metric types

### Performance Optimization

- Use recording rules for frequently queried complex expressions
- Adjust refresh intervals based on metric update frequency
- Consider using different retention policies for different metric types

## Support and Contributing

For dashboard improvements or custom visualization requests:

1. Check existing GitHub issues for similar requests
2. Create detailed issue describing the visualization need
3. Include sample queries and expected use cases
4. Consider contributing dashboard improvements via pull requests

The dashboards are designed to be community-maintained with regular updates as μNet features evolve.
