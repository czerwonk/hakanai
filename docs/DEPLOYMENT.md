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
version: "3.8"

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

### Log Management

Hakanai provides comprehensive logging with OpenTelemetry integration. For detailed logging configuration, structured log format, dual logging mode, and trace correlation, see the [Observability Guide](OBSERVABILITY.md#logging).

Quick examples:

```bash
# View structured logs
journalctl -u hakanai -f -o json | jq .

# Enable debug logging
RUST_LOG=debug hakanai-server

# Enable OTEL dual logging
OTEL_EXPORTER_OTLP_ENDPOINT="http://collector:4318" hakanai-server
```

### Webhook Monitoring

```bash
# Configure webhook for audit trail
hakanai-server \
  --webhook-url "https://audit.company.com/api/events" \
  --webhook-token "$(cat /etc/hakanai/webhook.token)" \
  --webhook-headers "user-agent,x-forwarded-for,x-request-id"
```

### Token Rotation

```bash
# Rotate admin token
hakanai-server --reset-admin-token --enable-admin-token

# Rotate user tokens (bulk operation)
hakanai-server --reset-user-tokens
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
