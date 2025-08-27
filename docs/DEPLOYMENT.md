# Production Deployment Guide

This guide covers production deployment, architecture, security considerations, and operational best practices for Hakanai.

## Architecture Overview

Hakanai implements a **separation of concerns** security model:

- **Application Layer**: Zero-knowledge encryption, secure token handling, input validation
- **Infrastructure Layer**: TLS termination, rate limiting, DDoS protection (handled by reverse proxy)

### Zero-Knowledge Flow

1. **Client encrypts** secret with AES-256-GCM
2. **Server stores** only encrypted blob + UUID
3. **URL contains** decryption key (or shared separately)
4. **Recipient decrypts** client-side
5. **Secret auto-destructs** after first access

## Production Deployment Options

### Option 1: Kubernetes with Helm (Recommended)

```bash
# Add Helm repository
helm repo add hakanai https://czerwonk.github.io/hakanai
helm repo update

# Production deployment
helm install hakanai hakanai/hakanai \
  --set ingress.enabled=true \
  --set ingress.hosts[0].host=secrets.company.com \
  --set ingress.tls[0].secretName=hakanai-tls \
  --set redis.replica.replicaCount=3 \
  --set config.allowAnonymous=false \
  --set config.enableAdminToken=true \
  --set config.trustedIpRanges="10.0.0.0/8,192.168.0.0/16"
```

**Features:**
- High availability with Redis Sentinel
- Automatic TLS with cert-manager
- Horizontal pod autoscaling
- Network policies and pod security standards
- Built-in monitoring and logging

### Option 2: Docker Compose

Use the provided `docker-compose.yml` with a production override file:

```yaml
# docker-compose.override.yml
version: '3.8'

services:
  hakanai:
    environment:
      HAKANAI_ALLOW_ANONYMOUS: "false"
      HAKANAI_ENABLE_ADMIN_TOKEN: "true"
      HAKANAI_TRUSTED_IP_RANGES: "10.0.0.0/8"
      HAKANAI_UPLOAD_SIZE_LIMIT: "50m"
      OTEL_EXPORTER_OTLP_ENDPOINT: "http://otel-collector:4318"
```

```bash
# Deploy with overrides
docker compose up -d

# Scale if needed
docker compose up -d --scale hakanai=3
```

### Option 3: Systemd Service

```bash
# Build and install
cargo build --release
sudo cp target/release/hakanai-server /usr/local/bin/
sudo useradd -r -s /bin/false hakanai
```

```ini
# /etc/systemd/system/hakanai.service
[Unit]
Description=Hakanai Secret Sharing Service
After=network.target redis.service
Requires=redis.service

[Service]
Type=exec
User=hakanai
Group=hakanai
ExecStart=/usr/local/bin/hakanai-server \
  --redis-dsn redis://127.0.0.1:6379/0 \
  --enable-admin-token \
  --trusted-ip-ranges "127.0.0.0/8" \
  --upload-size-limit 10m
Restart=always
RestartSec=5
Environment=HAKANAI_WEBHOOK_TOKEN_FILE=/etc/hakanai/webhook.token
EnvironmentFile=/etc/hakanai/hakanai.env

# Security hardening
NoNewPrivileges=yes
PrivateTmp=yes
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=/var/log/hakanai

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl enable hakanai
sudo systemctl start hakanai
```

## Reverse Proxy Configuration

Hakanai is designed to run behind a reverse proxy for production deployments. The proxy should handle:

### Required Features

1. **TLS Termination**: Handle HTTPS and enforce secure connections
2. **Rate Limiting**: Protect against abuse (e.g., 10 req/s for API, 30 req/s for retrieval)
3. **Security Headers**: HSTS, X-Frame-Options, CSP, etc.
4. **Real IP Detection**: Forward client IP via `X-Forwarded-For` or `X-Real-IP`

### Important Headers to Forward

```
Host: <original-host>
X-Real-IP: <client-ip>
X-Forwarded-For: <client-ip>
X-Forwarded-Proto: https
```

### Optional Geo-Location Headers

If using geo-restrictions, configure your proxy to add:
- `CF-IPCountry` or custom country header
- `X-ASN-Number` or custom ASN header

### Common Reverse Proxies

- **nginx**: High-performance, widely used
- **Caddy**: Automatic HTTPS, simple configuration
- **Traefik**: Kubernetes-native, dynamic configuration
- **HAProxy**: Advanced load balancing
- **Cloudflare**: CDN with built-in geo-headers

Consult your reverse proxy's documentation for specific configuration syntax, as these change frequently with new versions.

## Security Configuration

### Network Security

**Recommended firewall configuration:**

- **Hakanai port (8080)**: Only accessible from reverse proxy (typically localhost/127.0.0.1)
- **Redis port (6379)**: Only accessible from Hakanai server
- **Public ports (80/443)**: Only on reverse proxy

Use your system's firewall (iptables, nftables, ufw, firewalld) to enforce these restrictions.

### Token Security

```bash
# Generate secure admin token
hakanai-server \
  --enable-admin-token \
  --trusted-ip-ranges "10.0.0.0/8" \
  --reset-admin-token

# Create limited user tokens  
hakanai token --limit 5m --ttl 7d
```

### Redis Security

**Key Redis settings for Hakanai:**

- **Bind to localhost only** (not exposed to network)
- **Set memory limit** appropriate for your expected load
- **Use allkeys-lru eviction** policy for automatic cleanup
- **Disable persistence** (secrets are ephemeral by design)
- **Set password** if Redis is network-accessible

Consult Redis documentation for your version's specific configuration syntax.

## High Availability

### Redis Cluster/Sentinel

For high availability, configure Redis Sentinel or Redis Cluster according to your Redis version's documentation. Key considerations:

- Multiple Redis instances for failover
- Sentinel monitors for automatic failover
- Connection string format: `redis-sentinel://sentinel1:26379,sentinel2:26379/mymaster`

### Load Balancing

```bash
# Multiple Hakanai instances
hakanai-server --port 8080 --redis-dsn redis://sentinel:26379/hakanai-master
hakanai-server --port 8081 --redis-dsn redis://sentinel:26379/hakanai-master
```

### Health Checks

```bash
# Configure load balancer health checks
curl -f http://localhost:8080/ready || exit 1
curl -f http://localhost:8080/healthy || exit 1
```

## Monitoring & Observability

### Built-in OpenTelemetry Support

Hakanai has native OpenTelemetry integration that automatically exports traces, metrics, and logs:

```bash
# Enable OTEL by setting the endpoint
export OTEL_EXPORTER_OTLP_ENDPOINT="http://jaeger:14268/api/traces"

# Optional: Configure service information
export OTEL_SERVICE_NAME="hakanai"
export OTEL_SERVICE_VERSION="2.0.0" 
export OTEL_RESOURCE_ATTRIBUTES="deployment.environment=production,team=security"

# Start server - OTEL automatically enabled when endpoint is set
hakanai-server
```

**What's Exported:**
- **Traces**: All HTTP requests with detailed spans for crypto operations, Redis calls, and middleware
- **Metrics**: Request counts, response times, secret creation/retrieval rates, token usage
- **Logs**: Structured logs with trace correlation for debugging and audit

**OTEL Features:**
- **Automatic service detection**: Service name and version from Cargo metadata
- **Resource detection**: OS, process, SDK, environment information
- **Dual logging**: Logs to both stdout and OTEL when enabled
- **Custom metrics**: Token count monitoring, secret lifecycle events

### OTEL with Different Collectors

```bash
# Jaeger (traces + logs)
export OTEL_EXPORTER_OTLP_ENDPOINT="http://jaeger:14268/api/traces"

# OTLP HTTP endpoint (Jaeger, SigNoz, etc.)
export OTEL_EXPORTER_OTLP_ENDPOINT="http://collector:4318"

# OTLP gRPC endpoint
export OTEL_EXPORTER_OTLP_ENDPOINT="http://collector:4317"

# Honeycomb
export OTEL_EXPORTER_OTLP_ENDPOINT="https://api.honeycomb.io"
export OTEL_EXPORTER_OTLP_HEADERS="x-honeycomb-team=your-api-key"
```

### Prometheus Metrics (via OTEL)

When OTEL is enabled, metrics are available for Prometheus scraping:

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'hakanai'
    static_configs:
      - targets: ['hakanai:8080']
    metrics_path: '/metrics'
    scrape_interval: 15s
```

**Available Metrics:**
- `hakanai_secrets_created_total`: Total secrets created
- `hakanai_secrets_retrieved_total`: Total secrets retrieved  
- `hakanai_request_duration_seconds`: HTTP request latency
- `hakanai_tokens_active`: Current active tokens
- `hakanai_redis_operations_total`: Redis operation counts

### Log Management

Hakanai provides structured logging that integrates seamlessly with OTEL:

```bash
# JSON structured logs (always available)
journalctl -u hakanai -f -o json | jq .

# Dual logging mode (when OTEL enabled)
# - Logs to stdout for local debugging
# - Logs to OTEL collector for centralized analysis
OTEL_EXPORTER_OTLP_ENDPOINT="http://collector:4318" hakanai-server

# Debug logging for troubleshooting
RUST_LOG=debug hakanai-server

# Specific module logging
RUST_LOG=hakanai_server::handlers=trace hakanai-server
```

**Log Features:**
- **Structured JSON**: All logs in structured JSON format
- **Trace correlation**: Request traces linked to log entries
- **Security events**: Authentication failures, access denials
- **Performance metrics**: Request timing, Redis latency
- **Error context**: Detailed error information for debugging

### Webhook Monitoring

```bash
# Configure webhook for audit trail
hakanai-server \
  --webhook-url "https://audit.company.com/api/events" \
  --webhook-token "$(cat /etc/hakanai/webhook.token)" \
  --webhook-headers "user-agent,x-forwarded-for,x-request-id"
```

## Backup & Recovery

### Configuration Backup

For disaster recovery, backup your configuration files and deployment settings:

```bash
# Backup configuration files
tar -czf hakanai-config-$(date +%Y%m%d).tar.gz \
  /etc/hakanai/ \
  /etc/systemd/system/hakanai.service
```

**Note**: Redis data backup is not recommended as secrets are ephemeral by design. Loss of Redis data means loss of secrets, which aligns with the zero-knowledge, temporary nature of the service.

## Performance Tuning

### Redis Optimization

Consider tuning Redis performance settings based on your load:
- Memory sampling for eviction
- TCP keepalive for connection health
- Connection timeouts
- TCP backlog for high connection rates

Refer to Redis documentation for optimal values for your version and workload.

### System Limits

For high-traffic deployments, increase file descriptor limits:

```bash
# /etc/security/limits.conf
hakanai soft nofile 65536
hakanai hard nofile 65536

# /etc/systemd/system/hakanai.service
[Service]
LimitNOFILE=65536
```

## Maintenance

### Updates

```bash
# Zero-downtime deployment
# 1. Deploy new version alongside current
docker pull hakanai:latest
docker run -d --name hakanai-new hakanai:latest

# 2. Health check new version
curl -f http://hakanai-new:8080/ready

# 3. Update load balancer config
# 4. Stop old version
docker stop hakanai-old
```

### Token Rotation

```bash
# Rotate admin token
hakanai-server --reset-admin-token --enable-admin-token

# Rotate user tokens (bulk operation)
hakanai-server --reset-user-tokens
```

### Log Rotation

```bash
# /etc/logrotate.d/hakanai
/var/log/hakanai/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    create 644 hakanai hakanai
    postrotate
        systemctl reload hakanai
    endscript
}
```

## Disaster Recovery

### Complete System Recovery

1. **Restore Redis data** from backup
2. **Restore configuration** files
3. **Regenerate tokens** if compromised
4. **Update DNS/load balancer** if needed
5. **Verify health checks** pass

### Data Loss Scenarios

Since Hakanai stores ephemeral secrets:
- **Redis failure**: Secrets are lost (by design)
- **Token compromise**: Rotate all tokens immediately
- **Key compromise**: No server-side keys to compromise

## Security Incident Response

### Immediate Actions

1. **Revoke compromised tokens**
2. **Check access logs** for suspicious activity
3. **Rotate webhook tokens** if needed
4. **Update IP restrictions** if required

### Investigation

```bash
# Check access logs
journalctl -u hakanai --since "1 hour ago" | grep ERROR

# Redis command history
redis-cli monitor

# Network connections
netstat -tulpn | grep :8080
```

## Compliance & Legal

### GDPR Compliance

- **No personal data stored** (zero-knowledge architecture)
- **Automatic deletion** after access/expiration
- **No tracking or analytics** by default
- **Audit trail** via webhooks (metadata only)

### Legal Pages Setup

```bash
# Configure legal information
hakanai-server \
  --impressum-file /etc/hakanai/impressum.txt \
  --privacy-file /etc/hakanai/privacy.txt
```

## Troubleshooting

### Common Production Issues

**High memory usage:**
- Check Redis memory settings
- Monitor secret sizes and TTLs
- Verify cleanup processes

**Connection timeouts:**
- Check reverse proxy timeouts
- Verify Redis connectivity
- Monitor network latency

**Authentication failures:**
- Verify token expiration
- Check IP restrictions
- Validate proxy headers

### Debug Commands

```bash
# Server health
curl -v http://localhost:8080/healthy

# Redis connectivity
redis-cli -u $HAKANAI_REDIS_DSN ping

# Configuration check
hakanai-server --help

# Process monitoring
ps aux | grep hakanai
ss -tlpn | grep :8080
```

For development and testing guidance, see [DEVELOPMENT.md](DEVELOPMENT.md).
