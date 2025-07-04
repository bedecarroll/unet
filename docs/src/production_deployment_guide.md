<!-- SPDX-License-Identifier: MIT -->

# Production Deployment and Operations Guide

> **Audience:** DevOps and Infrastructure teams deploying μNet in production environments.  
> **Purpose:** Comprehensive guide for production deployment, monitoring, scaling, and operational procedures.  
> **Prerequisites:** Read [Architecture Overview](01_architecture.md) and [Deployment](08_deployment.md) before proceeding.

---

## Table of Contents

1. [Production Environment Requirements](#1-production-environment-requirements)
2. [Kubernetes Production Deployment](#2-kubernetes-production-deployment)
3. [High Availability Configuration](#3-high-availability-configuration)
4. [Security Configuration](#4-security-configuration)
5. [Monitoring and Observability](#5-monitoring-and-observability)
6. [Backup and Disaster Recovery](#6-backup-and-disaster-recovery)
7. [Performance Tuning](#7-performance-tuning)
8. [Scaling Operations](#8-scaling-operations)
9. [Operational Procedures](#9-operational-procedures)
10. [Incident Response](#10-incident-response)

---

## 1. Production Environment Requirements

### 1.1 Minimum System Requirements

| Component | Minimum Specification | Recommended for Production |
|-----------|----------------------|----------------------------|
| **CPU** | 4 vCPU | 8 vCPU |
| **Memory** | 8 GB RAM | 16 GB RAM |
| **Storage** | 50 GB SSD | 200 GB SSD (IOPS > 3000) |
| **Network** | 1 Gbps | 10 Gbps |
| **OS** | Ubuntu 22.04 LTS / RHEL 9 | Latest LTS version |

### 1.2 Network Requirements

#### Inbound Ports

- `8080/tcp` - HTTP API (behind load balancer)
- `8443/tcp` - HTTPS API (direct access)
- `9090/tcp` - Metrics endpoint (internal only)

#### Outbound Ports

- `443/tcp` - Git repository access, external APIs
- `161/udp` - SNMP polling to network devices
- `53/tcp,udp` - DNS resolution
- `5432/tcp` - PostgreSQL (if external database)

### 1.3 Database Requirements

#### PostgreSQL (Recommended for Production)

- **Version:** PostgreSQL 14+ with TimescaleDB extension
- **Configuration:**
  - `shared_buffers`: 25% of total RAM
  - `effective_cache_size`: 75% of total RAM
  - `max_connections`: 200
  - `checkpoint_completion_target`: 0.7
  - `wal_buffers`: 16MB
  - `default_statistics_target`: 100

#### Connection Pooling

- **Tool:** PgBouncer or built-in connection pooling
- **Pool Size:** 20-50 connections per μNet instance
- **Transaction Mode:** Recommended for better connection reuse

---

## 2. Kubernetes Production Deployment

### 2.1 Namespace and RBAC Setup

```bash
# Apply namespace and RBAC configurations
kubectl apply -f k8s/base/namespace.yaml
kubectl apply -f k8s/base/rbac.yaml
```

### 2.2 Production Configuration

Deploy using Kustomize overlays:

```bash
# Production deployment
kubectl apply -k k8s/overlays/prod/

# Verify deployment
kubectl get pods -n unet-system
kubectl get services -n unet-system
kubectl get ingress -n unet-system
```

### 2.3 Helm Chart Deployment

For Helm-based deployments:

```bash
# Add custom values for production
helm install unet ./helm/unet \
  --namespace unet-system \
  --create-namespace \
  --values helm/unet/values-production.yaml
```

### 2.4 Production Values Configuration

Create `values-production.yaml`:

```yaml
# Production Helm values
replicaCount: 3

image:
  repository: ghcr.io/your-org/unet-server
  tag: "v1.0.0"
  pullPolicy: IfNotPresent

resources:
  limits:
    cpu: 2000m
    memory: 4Gi
  requests:
    cpu: 1000m
    memory: 2Gi

autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 10
  targetCPUUtilizationPercentage: 70
  targetMemoryUtilizationPercentage: 80

database:
  type: postgresql
  host: postgres.unet-system.svc.cluster.local
  port: 5432
  name: unet_production
  sslmode: require

persistence:
  enabled: true
  storageClass: fast-ssd
  size: 100Gi

ingress:
  enabled: true
  className: nginx
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/rate-limit: "100"
  hosts:
    - host: unet.example.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: unet-tls
      hosts:
        - unet.example.com
```

---

## 3. High Availability Configuration

### 3.1 Multi-Region Deployment

#### Primary Region (Active)

```yaml
# k8s/overlays/prod-primary/kustomization.yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

resources:
  - ../../base

patchesStrategicMerge:
  - deployment-primary.yaml
  - configmap-primary.yaml

replicas:
  - name: unet-server
    count: 5
```

#### Secondary Region (Standby)

```yaml
# k8s/overlays/prod-secondary/kustomization.yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

resources:
  - ../../base

patchesStrategicMerge:
  - deployment-secondary.yaml
  - configmap-secondary.yaml

replicas:
  - name: unet-server
    count: 2
```

### 3.2 Database High Availability

#### PostgreSQL with Streaming Replication

```toml
# Production database configuration
[database]
type = "postgresql"
primary_host = "postgres-primary.unet-system.svc.cluster.local"
replica_hosts = [
    "postgres-replica-1.unet-system.svc.cluster.local",
    "postgres-replica-2.unet-system.svc.cluster.local"
]
port = 5432
database = "unet_production"
username = "unet_user"
password_file = "/etc/secrets/db-password"
sslmode = "require"
max_connections = 50
connection_timeout = "30s"
```

### 3.3 Load Balancer Configuration

#### AWS Application Load Balancer

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: unet-alb
  annotations:
    kubernetes.io/ingress.class: alb
    alb.ingress.kubernetes.io/scheme: internet-facing
    alb.ingress.kubernetes.io/target-type: ip
    alb.ingress.kubernetes.io/healthcheck-path: /health/ready
    alb.ingress.kubernetes.io/healthcheck-interval-seconds: "15"
    alb.ingress.kubernetes.io/healthy-threshold-count: "2"
    alb.ingress.kubernetes.io/unhealthy-threshold-count: "3"
spec:
  rules:
    - host: unet.example.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: unet-server
                port:
                  number: 8080
```

---

## 4. Security Configuration

### 4.1 TLS/SSL Configuration

#### Certificate Management with cert-manager

```yaml
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: admin@example.com
    privateKeySecretRef:
      name: letsencrypt-prod
    solvers:
      - http01:
          ingress:
            class: nginx
```

#### TLS Configuration in μNet

```toml
[server]
bind = "0.0.0.0:8443"
tls_cert_path = "/etc/tls/tls.crt"
tls_key_path = "/etc/tls/tls.key"
tls_min_version = "1.2"
tls_ciphers = [
    "TLS_AES_256_GCM_SHA384",
    "TLS_CHACHA20_POLY1305_SHA256",
    "TLS_AES_128_GCM_SHA256"
]
```

### 4.2 Authentication and Authorization

#### JWT Configuration

```toml
[auth]
jwt_secret_file = "/etc/secrets/jwt-secret"
jwt_expiry = "24h"
jwt_refresh_expiry = "168h"  # 7 days
jwt_issuer = "unet-production"
jwt_audience = "unet-api"

# RBAC configuration
[rbac]
enabled = true
default_role = "viewer"
admin_users = ["admin@example.com"]
```

#### API Key Management

```toml
[api_keys]
enabled = true
key_length = 32
rate_limit_per_key = 1000  # requests per hour
require_https = true
```

### 4.3 Network Security

#### NetworkPolicy for Kubernetes

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: unet-network-policy
  namespace: unet-system
spec:
  podSelector:
    matchLabels:
      app: unet-server
  policyTypes:
    - Ingress
    - Egress
  ingress:
    - from:
        - namespaceSelector:
            matchLabels:
              name: ingress-nginx
      ports:
        - protocol: TCP
          port: 8080
  egress:
    - to:
        - namespaceSelector:
            matchLabels:
              name: kube-system
      ports:
        - protocol: TCP
          port: 53
        - protocol: UDP
          port: 53
    - to: []
      ports:
        - protocol: TCP
          port: 443
        - protocol: TCP
          port: 5432
        - protocol: UDP
          port: 161
```

---

## 5. Monitoring and Observability

### 5.1 Prometheus Configuration

#### ServiceMonitor for Prometheus Operator

```yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: unet-metrics
  namespace: unet-system
spec:
  selector:
    matchLabels:
      app: unet-server
  endpoints:
    - port: metrics
      interval: 30s
      path: /metrics
```

#### Key Metrics to Monitor

- `unet_nodes_total` - Total number of managed nodes
- `unet_snmp_polls_total` - SNMP polling success/failure rates
- `unet_config_changes_total` - Configuration change frequency
- `unet_policy_evaluations_duration_seconds` - Policy evaluation performance
- `unet_database_connections_active` - Database connection pool usage
- `unet_http_requests_duration_seconds` - API response times

### 5.2 Alerting Rules

```yaml
groups:
  - name: unet.rules
    rules:
      - alert: UNetInstanceDown
        expr: up{job="unet-server"} == 0
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "μNet instance is down"
          description: "μNet instance {{ $labels.instance }} has been down for more than 5 minutes."

      - alert: UNetHighErrorRate
        expr: rate(unet_http_requests_total{status=~"5.."}[5m]) > 0.1
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "High error rate in μNet API"
          description: "μNet API error rate is {{ $value }} errors per second."

      - alert: UNetDatabaseConnectionsHigh
        expr: unet_database_connections_active / unet_database_connections_max > 0.8
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Database connection pool nearly exhausted"
          description: "Database connection usage is at {{ $value }}% of maximum."
```

### 5.3 Logging Configuration

#### Structured Logging Setup

```toml
[logging]
level = "info"
format = "json"
output = "stdout"

# Log aggregation
[logging.fields]
service = "unet-server"
environment = "production"
version = "1.0.0"

[logging.targets]
# Send logs to centralized logging system
syslog = { address = "logs.example.com:514", facility = "local0" }
file = { path = "/var/log/unet/server.log", rotation = "daily", keep = 7 }
```

#### Log Forwarding to ELK/EFK Stack

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: fluent-bit-config
data:
  fluent-bit.conf: |
    [INPUT]
        Name tail
        Path /var/log/containers/*unet*.log
        Parser docker
        Tag kube.*
        Refresh_Interval 5
        Mem_Buf_Limit 50MB

    [OUTPUT]
        Name elasticsearch
        Match kube.*
        Host elasticsearch.logging.svc.cluster.local
        Port 9200
        Index unet-logs
        Type _doc
```

---

## 6. Backup and Disaster Recovery

### 6.1 Database Backup Strategy

#### Automated PostgreSQL Backups

```bash
#!/bin/bash
# /opt/scripts/backup-unet-db.sh

# Configuration
BACKUP_DIR="/backups/unet"
DB_HOST="postgres.unet-system.svc.cluster.local"
DB_NAME="unet_production"
DB_USER="backup_user"
RETENTION_DAYS=30

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Perform backup
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="$BACKUP_DIR/unet_backup_$TIMESTAMP.sql.gz"

pg_dump -h "$DB_HOST" -U "$DB_USER" -d "$DB_NAME" \
    --no-password --verbose --clean --create \
    | gzip > "$BACKUP_FILE"

# Verify backup
if [ ${PIPESTATUS[0]} -eq 0 ]; then
    echo "Backup completed successfully: $BACKUP_FILE"
    
    # Upload to S3
    aws s3 cp "$BACKUP_FILE" "s3://unet-backups/database/"
    
    # Clean old local backups
    find "$BACKUP_DIR" -name "unet_backup_*.sql.gz" -mtime +$RETENTION_DAYS -delete
else
    echo "Backup failed!" >&2
    exit 1
fi
```

#### Schedule with CronJob

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: unet-db-backup
  namespace: unet-system
spec:
  schedule: "0 2 * * *"  # Daily at 2 AM
  jobTemplate:
    spec:
      template:
        spec:
          containers:
            - name: postgres-backup
              image: postgres:14
              command: ["/scripts/backup-unet-db.sh"]
              volumeMounts:
                - name: backup-scripts
                  mountPath: /scripts
                - name: backup-storage
                  mountPath: /backups
              env:
                - name: PGPASSWORD
                  valueFrom:
                    secretKeyRef:
                      name: postgres-backup-secret
                      key: password
          restartPolicy: OnFailure
          volumes:
            - name: backup-scripts
              configMap:
                name: backup-scripts
                defaultMode: 0755
            - name: backup-storage
              persistentVolumeClaim:
                claimName: backup-pvc
```

### 6.2 Configuration Backup

#### Git Repository Backup

```bash
#!/bin/bash
# Backup Git repositories containing policies and templates

BACKUP_DIR="/backups/git"
mkdir -p "$BACKUP_DIR"

# Backup policy repository
git clone --mirror https://github.com/org/unet-policies.git \
    "$BACKUP_DIR/unet-policies-$(date +%Y%m%d).git"

# Backup template repository  
git clone --mirror https://github.com/org/unet-templates.git \
    "$BACKUP_DIR/unet-templates-$(date +%Y%m%d).git"

# Upload to S3
aws s3 sync "$BACKUP_DIR" "s3://unet-backups/git/"
```

### 6.3 Disaster Recovery Procedures

#### RTO/RPO Targets

- **Recovery Time Objective (RTO):** 30 minutes
- **Recovery Point Objective (RPO):** 1 hour

#### Recovery Steps

1. **Database Recovery:**

   ```bash
   # Restore from latest backup
   gunzip -c unet_backup_latest.sql.gz | psql -h new-db-host -U postgres
   ```

2. **Application Recovery:**

   ```bash
   # Deploy to new cluster
   kubectl apply -k k8s/overlays/prod/
   
   # Update database connection
   kubectl patch secret unet-config -p '{"data":{"database_host":"<new-host>"}}'
   
   # Rolling restart
   kubectl rollout restart deployment/unet-server -n unet-system
   ```

3. **DNS Failover:**

   ```bash
   # Update DNS to point to backup region
   aws route53 change-resource-record-sets --hosted-zone-id Z123456789 \
       --change-batch file://dns-failover.json
   ```

---

## 7. Performance Tuning

### 7.1 Application Performance

#### Configuration Optimization

```toml
[server]
# Increase worker threads for high concurrency
worker_threads = 16
max_connections = 1000
request_timeout = "30s"
keepalive_timeout = "75s"

# Enable HTTP/2
http2 = true

[database]
# Connection pooling
max_connections = 50
min_connections = 5
connection_timeout = "30s"
idle_timeout = "600s"

[cache]
# Enable caching for better performance
enabled = true
type = "redis"
redis_url = "redis://redis-cluster.unet-system.svc.cluster.local:6379"
ttl = "300s"
max_size = "100MB"

[snmp]
# Optimize SNMP polling
concurrent_polls = 100
timeout = "10s"
retries = 3
```

### 7.2 Database Performance Tuning

#### PostgreSQL Configuration

```sql
-- Performance tuning queries
ALTER SYSTEM SET shared_buffers = '4GB';
ALTER SYSTEM SET effective_cache_size = '12GB';
ALTER SYSTEM SET maintenance_work_mem = '1GB';
ALTER SYSTEM SET checkpoint_completion_target = 0.7;
ALTER SYSTEM SET wal_buffers = '16MB';
ALTER SYSTEM SET default_statistics_target = 100;
ALTER SYSTEM SET random_page_cost = 1.1;

-- Reload configuration
SELECT pg_reload_conf();

-- Create performance indexes
CREATE INDEX CONCURRENTLY idx_nodes_last_seen ON nodes (last_seen);
CREATE INDEX CONCURRENTLY idx_changes_created_at ON configuration_changes (created_at);
CREATE INDEX CONCURRENTLY idx_policy_evaluations_node_id ON policy_evaluations (node_id);
```

### 7.3 Caching Strategy

#### Redis Configuration

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: redis-config
data:
  redis.conf: |
    maxmemory 2gb
    maxmemory-policy allkeys-lru
    save 900 1
    save 300 10
    save 60 10000
    tcp-keepalive 300
    timeout 0
```

---

## 8. Scaling Operations

### 8.1 Horizontal Pod Autoscaling

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: unet-server-hpa
  namespace: unet-system
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: unet-server
  minReplicas: 3
  maxReplicas: 20
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
    - type: Resource
      resource:
        name: memory
        target:
          type: Utilization
          averageUtilization: 80
    - type: Pods
      pods:
        metric:
          name: unet_active_connections
        target:
          type: AverageValue
          averageValue: "100"
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
        - type: Pods
          value: 2
          periodSeconds: 60
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
        - type: Pods
          value: 1
          periodSeconds: 60
```

### 8.2 Cluster Autoscaling

#### AWS EKS Cluster Autoscaler

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cluster-autoscaler
  namespace: kube-system
spec:
  template:
    spec:
      containers:
        - name: cluster-autoscaler
          image: k8s.gcr.io/autoscaling/cluster-autoscaler:v1.21.0
          command:
            - ./cluster-autoscaler
            - --v=4
            - --stderrthreshold=info
            - --cloud-provider=aws
            - --skip-nodes-with-local-storage=false
            - --expander=least-waste
            - --node-group-auto-discovery=asg:tag=k8s.io/cluster-autoscaler/enabled,k8s.io/cluster-autoscaler/unet-cluster
            - --balance-similar-node-groups
            - --scale-down-delay-after-add=10m
            - --scale-down-unneeded-time=10m
```

### 8.3 Database Scaling

#### Read Replicas Configuration

```toml
[database]
# Primary database for writes
primary_host = "postgres-primary.unet-system.svc.cluster.local"

# Read replicas for query distribution
read_replicas = [
    "postgres-replica-1.unet-system.svc.cluster.local",
    "postgres-replica-2.unet-system.svc.cluster.local",
    "postgres-replica-3.unet-system.svc.cluster.local"
]

# Load balancing strategy
read_strategy = "round_robin"  # or "least_connections"
```

---

## 9. Operational Procedures

### 9.1 Deployment Procedures

#### Blue-Green Deployment

```bash
#!/bin/bash
# Blue-Green deployment script

NEW_VERSION="$1"
NAMESPACE="unet-system"

# Validate inputs
if [ -z "$NEW_VERSION" ]; then
    echo "Usage: $0 <version>"
    exit 1
fi

# Deploy green environment
kubectl apply -k k8s/overlays/prod-green/ --namespace="$NAMESPACE"
kubectl set image deployment/unet-server-green unet-server="ghcr.io/org/unet-server:$NEW_VERSION" -n "$NAMESPACE"

# Wait for green to be ready
kubectl rollout status deployment/unet-server-green -n "$NAMESPACE" --timeout=600s

# Health check green environment
GREEN_ENDPOINT=$(kubectl get svc unet-server-green -n "$NAMESPACE" -o jsonpath='{.status.loadBalancer.ingress[0].hostname}')
if curl -f "http://$GREEN_ENDPOINT:8080/health/ready"; then
    echo "Green environment is healthy"
    
    # Switch traffic to green
    kubectl patch service unet-server -n "$NAMESPACE" -p '{"spec":{"selector":{"version":"green"}}}'
    
    # Wait and verify
    sleep 30
    
    # Scale down blue
    kubectl scale deployment unet-server-blue --replicas=0 -n "$NAMESPACE"
    
    echo "Deployment completed successfully"
else
    echo "Green environment failed health check, rolling back"
    kubectl delete -k k8s/overlays/prod-green/ --namespace="$NAMESPACE"
    exit 1
fi
```

#### Rolling Update

```bash
#!/bin/bash
# Rolling update procedure

NEW_VERSION="$1"
NAMESPACE="unet-system"

# Update image
kubectl set image deployment/unet-server unet-server="ghcr.io/org/unet-server:$NEW_VERSION" -n "$NAMESPACE"

# Monitor rollout
kubectl rollout status deployment/unet-server -n "$NAMESPACE" --timeout=600s

# Verify deployment
kubectl get pods -n "$NAMESPACE" -l app=unet-server
```

### 9.2 Configuration Updates

#### Hot Configuration Reload

```bash
#!/bin/bash
# Update configuration without restart

NEW_CONFIG="$1"

# Validate configuration
kubectl create configmap unet-config-new --from-file="$NEW_CONFIG" --dry-run=client -o yaml | kubectl apply -f -

# Apply configuration
kubectl patch configmap unet-config -p "$(kubectl create configmap unet-config-new --from-file="$NEW_CONFIG" --dry-run=client -o yaml | sed 's/unet-config-new/unet-config/')"

# Trigger configuration reload
kubectl exec -n unet-system deployment/unet-server -- kill -USR1 1
```

### 9.3 Maintenance Procedures

#### Scheduled Maintenance Window

```bash
#!/bin/bash
# Maintenance window procedure

echo "Starting maintenance window at $(date)"

# 1. Scale down to maintenance mode
kubectl scale deployment unet-server --replicas=1 -n unet-system

# 2. Enable maintenance mode
kubectl patch configmap unet-config -p '{"data":{"maintenance_mode":"true"}}'

# 3. Reload configuration
kubectl rollout restart deployment/unet-server -n unet-system

# 4. Perform maintenance tasks
echo "Performing database maintenance..."
kubectl exec -it postgres-primary-0 -- psql -U postgres -d unet_production -c "VACUUM ANALYZE;"

echo "Updating Git repositories..."
kubectl exec -it deployment/unet-server -- unet git sync --force

# 5. Exit maintenance mode
kubectl patch configmap unet-config -p '{"data":{"maintenance_mode":"false"}}'

# 6. Scale back to normal operation
kubectl scale deployment unet-server --replicas=3 -n unet-system

echo "Maintenance window completed at $(date)"
```

---

## 10. Incident Response

### 10.1 Incident Classification

| Severity | Description | Response Time | Examples |
|----------|-------------|---------------|----------|
| **P0 - Critical** | Complete service outage | 15 minutes | All instances down, database failure |
| **P1 - High** | Major feature unavailable | 1 hour | API errors, SNMP polling failures |
| **P2 - Medium** | Minor feature impact | 4 hours | Slow responses, partial functionality |
| **P3 - Low** | Cosmetic issues | 24 hours | UI glitches, documentation errors |

### 10.2 Runbook Templates

#### Database Connection Issues

```bash
# Diagnosis
kubectl logs deployment/unet-server -n unet-system | grep -i "database\|connection"
kubectl exec -it postgres-primary-0 -- psql -U postgres -c "SELECT * FROM pg_stat_activity;"

# Resolution
kubectl scale deployment unet-server --replicas=0 -n unet-system
kubectl exec -it postgres-primary-0 -- psql -U postgres -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname='unet_production';"
kubectl scale deployment unet-server --replicas=3 -n unet-system
```

#### High Memory Usage

```bash
# Diagnosis
kubectl top pods -n unet-system
kubectl describe pods -n unet-system -l app=unet-server

# Resolution
kubectl patch deployment unet-server -n unet-system -p '{"spec":{"template":{"spec":{"containers":[{"name":"unet-server","resources":{"limits":{"memory":"8Gi"}}}]}}}}'
kubectl rollout restart deployment/unet-server -n unet-system
```

#### SNMP Polling Failures

```bash
# Diagnosis
kubectl exec -it deployment/unet-server -- unet nodes list --status=unreachable
kubectl logs deployment/unet-server -n unet-system | grep -i snmp

# Resolution
kubectl exec -it deployment/unet-server -- unet snmp test --node=<node-id>
# Update SNMP configuration if needed
kubectl patch configmap unet-config -p '{"data":{"snmp_timeout":"15s"}}'
```

### 10.3 Emergency Procedures

#### Complete System Recovery

```bash
#!/bin/bash
# Emergency recovery procedure

# 1. Deploy minimal instance
kubectl apply -f - <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: unet-emergency
  namespace: unet-system
spec:
  replicas: 1
  selector:
    matchLabels:
      app: unet-emergency
  template:
    metadata:
      labels:
        app: unet-emergency
    spec:
      containers:
        - name: unet-server
          image: ghcr.io/org/unet-server:stable
          env:
            - name: UNET_MODE
              value: "emergency"
          resources:
            requests:
              memory: "512Mi"
              cpu: "250m"
EOF

# 2. Restore database from backup
kubectl exec -it postgres-primary-0 -- psql -U postgres -d template1 -c "DROP DATABASE IF EXISTS unet_production;"
kubectl exec -it postgres-primary-0 -- psql -U postgres -c "CREATE DATABASE unet_production;"
gunzip -c /backups/latest/unet_backup.sql.gz | kubectl exec -i postgres-primary-0 -- psql -U postgres -d unet_production

# 3. Verify emergency instance
kubectl port-forward deployment/unet-emergency 8080:8080 &
curl http://localhost:8080/health/ready

# 4. Scale back to normal operation
kubectl scale deployment unet-server --replicas=3 -n unet-system
kubectl delete deployment unet-emergency -n unet-system
```

---

## Conclusion

This production deployment and operations guide provides comprehensive procedures for deploying, monitoring, and maintaining μNet in production environments. Regular review and updates of these procedures ensure optimal system performance and reliability.

**Key Success Factors:**

- Follow security best practices throughout deployment
- Implement comprehensive monitoring and alerting
- Maintain regular backup and disaster recovery testing
- Document all operational procedures and keep them current
- Conduct regular performance reviews and optimization

For additional support, refer to the [Troubleshooting Guide](troubleshooting_guide.md) and [API Reference](api_reference.md).
