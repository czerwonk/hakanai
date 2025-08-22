# Hakanai Helm Chart

A Helm chart for deploying Hakanai - a minimalist one-time secret sharing service implementing zero-knowledge principles.

## Prerequisites

- Kubernetes 1.19+
- Helm 3.8.0+
- PV provisioner support in the underlying infrastructure (for Redis persistence)
- ingress-nginx controller (or alternative ingress controller)

## Installation

### Add the repository

```bash
helm repo add hakanai https://czerwonk.github.io/hakanai
helm repo update
```

### Basic Installation

```bash
helm install my-hakanai hakanai/hakanai
```

### Installation with custom values

```bash
helm install my-hakanai hakanai/hakanai \
  --set ingress.hosts[0].host=secrets.example.com \
  --set hakanai.server.allowAnonymous=true
```

## Configuration

### Key Configuration Options

| Parameter | Description | Default |
|-----------|-------------|---------|
| `replicaCount` | Number of Hakanai replicas | `2` |
| `image.repository` | Hakanai image repository | `ghcr.io/czerwonk/hakanai` |
| `image.tag` | Hakanai image tag | `v2.9.7` |
| `ingress.enabled` | Enable ingress | `true` |
| `ingress.className` | Ingress class name | `nginx` |
| `ingress.hosts[0].host` | Hostname for ingress | `hakanai.example.com` |
| `hakanai.server.uploadSizeLimit` | Upload size limit | `10m` |
| `hakanai.server.allowAnonymous` | Allow anonymous access | `false` |
| `hakanai.server.anonymousSizeLimit` | Anonymous upload size limit | `32k` |
| `hakanai.server.enableAdminToken` | Enable admin token system | `false` |
| `redis.enabled` | Deploy Redis with Sentinel for HA | `true` |
| `redis.architecture` | Redis architecture (standalone/replication) | `replication` |
| `redis.sentinel.enabled` | Enable Redis Sentinel for HA | `true` |
| `redis.master.persistence.enabled` | Enable Redis master persistence | `true` |
| `redis.replica.replicaCount` | Number of Redis replicas | `2` |

### Security Features

#### Rate Limiting (ingress-nginx)

The chart includes sensible rate limiting defaults:
- 10 requests per second
- 100 requests per minute
- Burst multiplier of 5
- 10 concurrent connections

Adjust these in `values.yaml`:

```yaml
ingress:
  annotations:
    nginx.ingress.kubernetes.io/limit-rps: "20"
    nginx.ingress.kubernetes.io/limit-rpm: "200"
```

#### IP Whitelisting

Allow specific IP ranges to bypass size limits:

```yaml
hakanai:
  server:
    ipWhitelist:
      - "10.0.0.0/8"
      - "192.168.0.0/16"
      - "2001:db8::/32"
    proxyHeader: "x-forwarded-for"
```

#### Network Policies

Network policies are enabled by default to restrict traffic:

```yaml
networkPolicy:
  enabled: true
  policyTypes:
    - Ingress
    - Egress
```

### Deployment Scenarios

#### 1. Development Environment

```yaml
# values-dev.yaml
replicaCount: 1
ingress:
  enabled: false
service:
  type: NodePort
hakanai:
  server:
    allowAnonymous: true
    anonymousSizeLimit: "1m"
valkey:
  persistence:
    enabled: false
resources:
  limits:
    cpu: 200m
    memory: 128Mi
  requests:
    cpu: 50m
    memory: 64Mi
```

```bash
helm install hakanai-dev hakanai/hakanai -f values-dev.yaml
```

#### 2. Production with Authentication

```yaml
# values-prod.yaml
replicaCount: 3
autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 10
ingress:
  hosts:
    - host: secrets.company.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: secrets-tls
      hosts:
        - secrets.company.com
hakanai:
  server:
    allowAnonymous: false
    enableAdminToken: true
    uploadSizeLimit: "100m"
  observability:
    enabled: true
    endpoint: "http://otel-collector:4317"
valkey:
  persistence:
    enabled: true
    size: 10Gi
    storageClass: fast-ssd
  auth:
    enabled: true
resources:
  limits:
    cpu: 1000m
    memory: 512Mi
  requests:
    cpu: 250m
    memory: 256Mi
```

```bash
helm install hakanai-prod hakanai/hakanai -f values-prod.yaml
```

#### 3. High Security Environment

```yaml
# values-secure.yaml
replicaCount: 3
podSecurityContext:
  runAsNonRoot: true
  runAsUser: 65532
  fsGroup: 65532
securityContext:
  allowPrivilegeEscalation: false
  capabilities:
    drop:
    - ALL
  readOnlyRootFilesystem: true
  runAsNonRoot: true
  seccompProfile:
    type: RuntimeDefault
networkPolicy:
  enabled: true
hakanai:
  server:
    allowAnonymous: false
    enableAdminToken: true
    ipWhitelist:
      - "10.0.0.0/8"  # Internal network only
ingress:
  annotations:
    nginx.ingress.kubernetes.io/limit-rps: "5"
    nginx.ingress.kubernetes.io/limit-rpm: "30"
    nginx.ingress.kubernetes.io/ssl-protocols: "TLSv1.3"
    nginx.ingress.kubernetes.io/ssl-ciphers: "TLS_AES_256_GCM_SHA384"
```

```bash
helm install hakanai-secure hakanai/hakanai -f values-secure.yaml
```

#### 4. Using External Redis

```yaml
# values-external-redis.yaml
redis:
  enabled: false
externalRedis:
  host: redis.cache.amazonaws.com
  port: 6379
  password: "" # Or use existingSecret
  existingSecret: redis-auth
  existingSecretPasswordKey: redis-password
```

```bash
helm install hakanai hakanai/hakanai -f values-external-redis.yaml
```

## Token Management

### Generate Admin Token

After deployment with `enableAdminToken: true`:

```bash
# Get the admin token
kubectl get secret my-hakanai -o jsonpath='{.data.admin-token}' | base64 -d

# Create a new user token via CLI
kubectl exec -it deployment/my-hakanai -- hakanai token --limit 5m --ttl 7d
```

### Reset Tokens

```yaml
hakanai:
  tokens:
    resetAdminToken: true    # Reset admin token on next deployment
    resetDefaultToken: true   # Reset default user token
```

## Observability

### OpenTelemetry Integration

```yaml
hakanai:
  observability:
    enabled: true
    endpoint: "http://otel-collector.monitoring:4317"
    serviceName: "hakanai-production"
    env:
      OTEL_TRACES_EXPORTER: "otlp"
      OTEL_METRICS_EXPORTER: "otlp"
      OTEL_LOGS_EXPORTER: "otlp"
```

## Backup and Recovery

### Backup Redis Data

```bash
# Create a backup from master
kubectl exec -it my-hakanai-redis-master-0 -- sh -c 'redis-cli BGSAVE'
kubectl cp my-hakanai-redis-master-0:/data/dump.rdb ./redis-backup-$(date +%Y%m%d).rdb
```

### Restore Redis Data

```bash
# Scale down Hakanai
kubectl scale deployment my-hakanai --replicas=0

# Restore data to master
kubectl cp ./redis-backup.rdb my-hakanai-redis-master-0:/data/dump.rdb

# Redis will replicate to replicas automatically
# Scale up Hakanai
kubectl scale deployment my-hakanai --replicas=2
```

## High Availability Features

The chart includes Redis Sentinel for automatic failover:

- **Master-Replica Replication**: 1 master + 2 replicas by default
- **Automatic Failover**: Sentinel monitors and promotes replicas on master failure
- **Data Persistence**: All Redis nodes persist data to disk
- **Quorum-based Decisions**: 2 out of 3 sentinels must agree for failover

To adjust HA settings:

```yaml
redis:
  replica:
    replicaCount: 3  # Increase replicas
  sentinel:
    quorum: 2  # Adjust quorum
    downAfterMilliseconds: 3000  # Faster failure detection
```

## Upgrading

### From 0.x to 1.0

Breaking changes:
- Redis with Sentinel replaces standalone Valkey
- Uses Bitnami Redis subchart for HA support
- Token configuration moved to `hakanai.tokens.*`
- Network policies are enabled by default

```bash
# Backup your data first!
helm upgrade my-hakanai hakanai/hakanai --version 1.0.0
```

## Monitoring

### Health Checks

The chart configures health probes using Hakanai's built-in endpoints:
- `/healthy` - Liveness probe (checks Redis connectivity)
- `/ready` - Readiness probe (basic service readiness)

### Metrics

When OpenTelemetry is enabled, Hakanai exports:
- Request metrics (latency, status codes)
- Token usage metrics
- Redis connection metrics

## Troubleshooting

### Check Logs

```bash
# Hakanai logs
kubectl logs -l app.kubernetes.io/name=hakanai

# Redis logs
kubectl logs -l app.kubernetes.io/name=redis
```

### Test Connection

```bash
# Port forward to test locally
kubectl port-forward svc/my-hakanai 8080:8080

# Test with curl
curl http://localhost:8080/healthy
```

### Common Issues

1. **Ingress not working**: Ensure ingress-nginx controller is installed
2. **Redis connection errors**: Check network policies and Redis authentication
3. **Size limit errors**: Adjust `uploadSizeLimit` and `anonymousSizeLimit`
4. **Token issues**: Use reset flags to regenerate tokens

## Uninstallation

```bash
helm uninstall my-hakanai

# Clean up PVCs (if persistence was enabled)
kubectl delete pvc -l app.kubernetes.io/instance=my-hakanai
```

## License

This Helm chart is distributed under the MIT License.

## Support

For issues and questions:
- GitHub: https://github.com/czerwonk/hakanai/issues
