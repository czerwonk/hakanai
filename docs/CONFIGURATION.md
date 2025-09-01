# Configuration Guide

This guide covers all configuration options for Hakanai server and CLI.

## Server Configuration

The server can be configured using command-line flags or environment variables. Environment variables take precedence over default values, and command-line flags take precedence over environment variables.

### Basic Server Options

| Flag | Environment Variable | Default | Description |
|------|---------------------|---------|-------------|
| `--port` | `HAKANAI_PORT` | `8080` | Server port |
| `--listen` | `HAKANAI_LISTEN_ADDRESS` | `127.0.0.1` | Bind address |
| `--redis-dsn` | `HAKANAI_REDIS_DSN` | `redis://127.0.0.1:6379/` | Redis connection string |

### Size Limits

All size limits apply to the secret data before encryption. The server automatically accounts for encryption overhead.

| Flag | Environment Variable | Default | Description |
|------|---------------------|---------|-------------|
| `--upload-size-limit` | `HAKANAI_UPLOAD_SIZE_LIMIT` | `10m` | Maximum upload size (humanized format supported) |
| `--anonymous-size-limit` | `HAKANAI_ANONYMOUS_UPLOAD_SIZE_LIMIT` | `32k` | Upload limit for anonymous users |

**Humanized Size Format:**
- Plain numbers: bytes (e.g., `1024`)
- 'k' suffix: kilobytes (e.g., `500k`)
- 'm' suffix: megabytes (e.g., `10m`)
- Decimal values: supported (e.g., `1.5m`, `2.5k`)

### Authentication & Access Control

| Flag | Environment Variable | Default | Description |
|------|---------------------|---------|-------------|
| `--allow-anonymous` | `HAKANAI_ALLOW_ANONYMOUS` | `false` | Allow anonymous secret creation |
| `--enable-admin-token` | `HAKANAI_ENABLE_ADMIN_TOKEN` | `false` | Enable admin token system |
| `--show-token-input` | `HAKANAI_SHOW_TOKEN_INPUT` | `false` | Show token input in web interface |
| `--trusted-ip-ranges` | `HAKANAI_TRUSTED_IP_RANGES` | - | IP ranges that bypass size limits (comma-separated) |
| `--trusted-ip-header` | `HAKANAI_TRUSTED_IP_HEADER` | `x-forwarded-for` | HTTP header for client IP detection |

### Token Management

| Flag | Description |
|------|-------------|
| `--reset-admin-token` | Regenerate admin token (requires `--enable-admin-token`) |
| `--reset-user-tokens` | Clear all user tokens and create new default token |

### Security & CORS

| Flag | Environment Variable | Default | Description |
|------|---------------------|---------|-------------|
| `--cors-allowed-origins` | `HAKANAI_CORS_ALLOWED_ORIGINS` | - | Allowed CORS origins (comma-separated) |
| `--max-ttl` | `HAKANAI_MAX_TTL` | `604800` | Maximum TTL in seconds (7 days) |

### Geo-Restrictions

| Flag | Environment Variable | Description |
|------|---------------------|-------------|
| `--country-header` | `HAKANAI_COUNTRY_HEADER` | HTTP header for country detection |
| `--asn-header` | `HAKANAI_ASN_HEADER` | HTTP header for ASN detection |

### Legal & Compliance

| Flag | Environment Variable | Description |
|------|---------------------|-------------|
| `--impressum-file` | `HAKANAI_IMPRESSUM_FILE` | Path to legal information file |
| `--privacy-file` | `HAKANAI_PRIVACY_FILE` | Path to privacy policy file |

### Webhooks (v2.8+)

| Flag | Environment Variable | Description |
|------|---------------------|-------------|
| `--webhook-url` | `HAKANAI_WEBHOOK_URL` | Webhook URL for lifecycle notifications |
| `--webhook-token` | `HAKANAI_WEBHOOK_TOKEN` | Bearer token for webhook authentication |
| `--webhook-headers` | `HAKANAI_WEBHOOK_HEADERS` | Headers to include in webhook requests |

### Observability

| Environment Variable | Description |
|---------------------|-------------|
| `OTEL_EXPORTER_OTLP_ENDPOINT` | OpenTelemetry collector endpoint |

### Customization & Branding

| Flag | Environment Variable | Description |
|------|---------------------|-------------|
| `--custom-assets-dir` | `HAKANAI_CUSTOM_ASSETS_DIR` | Directory containing custom assets (logo, CSS, icons) |

For detailed customization options, see [CUSTOMIZATION.md](CUSTOMIZATION.md).

## Configuration Examples

### Minimal Development Setup

```bash
hakanai-server
```

Uses all defaults: port 8080, local Redis, 10MB upload limit, authentication required.

### Anonymous Access Server

```bash
hakanai-server --allow-anonymous --anonymous-size-limit 64k
```

Or with environment variables:
```bash
export HAKANAI_ALLOW_ANONYMOUS=true
export HAKANAI_ANONYMOUS_UPLOAD_SIZE_LIMIT=64k
hakanai-server
```

### Production Server with Admin Token

```bash
hakanai-server \
  --enable-admin-token \
  --trusted-ip-ranges "10.0.0.0/8,192.168.1.0/24" \
  --allow-anonymous \
  --anonymous-size-limit 32k \
  --upload-size-limit 50m \
  --cors-allowed-origins "https://app.example.com,https://admin.example.com"
```

### Server with Geo-Restrictions

```bash
# Behind Cloudflare
hakanai-server \
  --country-header cf-ipcountry \
  --trusted-ip-header cf-connecting-ip

# Custom proxy setup  
hakanai-server \
  --country-header x-country-code \
  --asn-header x-asn-number \
  --trusted-ip-header x-real-ip
```

### High-Security Server

```bash
hakanai-server \
  --enable-admin-token \
  --trusted-ip-ranges "10.0.0.0/8" \
  --max-ttl 3600 \
  --upload-size-limit 1m \
  --show-token-input false
```

### Server with Webhooks

```bash
hakanai-server \
  --webhook-url https://audit.example.com/hakanai \
  --webhook-token "secure-webhook-token" \
  --webhook-headers "user-agent,x-forwarded-for,x-request-id"
```

### Server with Legal Pages

```bash
hakanai-server \
  --impressum-file /etc/hakanai/impressum.txt \
  --privacy-file /etc/hakanai/privacy.txt
```

## CLI Configuration

The CLI can be configured using command-line flags or environment variables.

### Global CLI Options

| Flag | Environment Variable | Default | Description |
|------|---------------------|---------|-------------|
| `-s, --server` | `HAKANAI_SERVER` | `http://localhost:8080` | Server URL |
| `--token-file` | `HAKANAI_TOKEN` | - | Authentication token |

### Send Command Defaults

| Environment Variable | Description |
|---------------------|-------------|
| `HAKANAI_TTL` | Default TTL for secrets |
| `HAKANAI_QR_CODE` | Enable QR code output by default |

### Get Command Defaults

| Environment Variable | Description |
|---------------------|-------------|
| `HAKANAI_TO_STDOUT` | Output to stdout by default |
| `HAKANAI_OUTPUT_DIR` | Default output directory |

### Token Command Defaults

| Environment Variable | Description |
|---------------------|-------------|
| `HAKANAI_TOKEN_TTL` | Default TTL for created tokens |

## CLI Configuration Examples

### Environment Configuration

```bash
# ~/.bashrc or ~/.zshrc
export HAKANAI_SERVER="https://secrets.company.com"
export HAKANAI_TOKEN="your-user-token"
export HAKANAI_OUTPUT_DIR="$HOME/secrets"
export HAKANAI_QR_CODE=true
```

### Token File Setup

```bash
# Store token securely
echo "your-secret-token" > ~/.hakanai-token
chmod 600 ~/.hakanai-token

# Use token file
hakanai send --token-file ~/.hakanai-token --file document.pdf
```

### Company-Specific Configuration

```bash
# Company alias with defaults
alias company-send='hakanai send --server https://secrets.company.com --allow-ip 10.0.0.0/8 --allow-country US --ttl 24h'

# Use the alias
echo "internal password" | company-send
```

## Redis Configuration

Hakanai requires Redis for storing encrypted secrets and authentication tokens.

### Redis Connection Strings

```bash
# Local Redis (default)
redis://127.0.0.1:6379/

# Remote Redis with auth
redis://username:password@redis.example.com:6379/0

# Redis with SSL
rediss://username:password@redis.example.com:6380/0

# Redis Sentinel
redis-sentinel://sentinel1:26379,sentinel2:26379/mymaster

# Unix socket
redis+unix:///var/run/redis/redis.sock
```

### Redis Memory Settings

For production, configure Redis with appropriate memory limits and eviction policies:

```bash
# redis.conf
maxmemory 1gb
maxmemory-policy allkeys-lru
save ""  # Disable persistence for ephemeral secrets
```

## Docker Configuration

### Docker Compose Environment

```yaml
# docker-compose.override.yml
services:
  hakanai:
    environment:
      HAKANAI_PORT: "8080"
      HAKANAI_ALLOW_ANONYMOUS: "true"
      HAKANAI_ANONYMOUS_UPLOAD_SIZE_LIMIT: "64k"
      HAKANAI_UPLOAD_SIZE_LIMIT: "50m"
      HAKANAI_CORS_ALLOWED_ORIGINS: "https://app.example.com"
      HAKANAI_TRUSTED_IP_RANGES: "10.0.0.0/8,192.168.0.0/16"
      HAKANAI_WEBHOOK_URL: "https://audit.example.com/webhook"
      HAKANAI_WEBHOOK_TOKEN: "webhook-secret"
      OTEL_EXPORTER_OTLP_ENDPOINT: "http://jaeger:14268/api/traces"
    volumes:
      - ./config/impressum.txt:/app/impressum.txt:ro
      - ./config/privacy.txt:/app/privacy.txt:ro
    command: >
      --impressum-file /app/impressum.txt
      --privacy-file /app/privacy.txt
      --enable-admin-token
      --trusted-ip-ranges "172.16.0.0/12"
```

## Kubernetes Configuration

For Kubernetes deployments, use our Helm chart which handles all configuration:

```bash
# Configure via values file
helm install hakanai hakanai/hakanai --values production.yaml
```

Example `production.yaml`:
```yaml
config:
  allowAnonymous: false
  enableAdminToken: true
  uploadSizeLimit: "10m"
  trustedIpRanges: "10.244.0.0/16"
  
redis:
  replica:
    replicaCount: 3
    
ingress:
  enabled: true
  hosts:
    - host: secrets.example.com
```

See the [Helm chart documentation](https://github.com/czerwonk/hakanai/tree/main/helm/hakanai) for all configuration options.

## Security Configuration

### Network Security

```bash
# Restrict to internal networks only
hakanai-server \
  --listen 10.0.0.100 \
  --trusted-ip-ranges "10.0.0.0/8,192.168.0.0/16" \
  --cors-allowed-origins "https://internal-app.company.local"
```

### Token Security

```bash
# Short-lived tokens only
hakanai-server \
  --enable-admin-token \
  --max-ttl 3600 \
  --trusted-ip-ranges "10.0.0.0/8"

# Create short-lived user tokens
hakanai token --ttl 1h --limit 1m
```

### Content Restrictions

```bash
# Small uploads only
hakanai-server \
  --upload-size-limit 1m \
  --anonymous-size-limit 64k \
  --max-ttl 1800
```

## Monitoring Configuration

### OpenTelemetry

```bash
# Enable full observability
export OTEL_EXPORTER_OTLP_ENDPOINT="http://jaeger:14268/api/traces"
export OTEL_SERVICE_NAME="hakanai"
export OTEL_RESOURCE_ATTRIBUTES="deployment.environment=production,service.version=2.0.0"

hakanai-server
```

### Webhook Monitoring

```bash
# Send lifecycle events to monitoring system
hakanai-server \
  --webhook-url "https://monitoring.example.com/api/events" \
  --webhook-token "monitoring-api-key" \
  --webhook-headers "user-agent,x-forwarded-for,x-request-id,authorization"
```

## Validation & Testing

### Configuration Validation

```bash
# Test configuration
hakanai-server --help

# Validate Redis connection
redis-cli -u redis://localhost:6379 ping

# Test server startup (dry run)
hakanai-server --port 0  # Will fail after validation
```

### Health Checks

```bash
# Basic health check
curl http://localhost:8080/ready

# Full health check (includes Redis)
curl http://localhost:8080/healthy
```

## Troubleshooting

### Common Configuration Issues

**Server won't start:**
- Check Redis connection with `redis-cli`
- Verify port availability with `netstat -tlpn | grep :8080`
- Check file permissions for impressum/privacy files

**Authentication issues:**
- Verify token format and expiration
- Check trusted IP ranges match client IPs
- Ensure admin token is enabled for admin operations

**Size limit exceeded:**
- Account for base64 encoding overhead (~33% increase)
- Check both user and anonymous limits
- Verify trusted IP ranges for bypass

**Geo-restrictions not working:**
- Configure appropriate headers (`--country-header`, `--asn-header`)
- Verify reverse proxy sends geo information
- Check server logs for header values

### Debug Configuration

```bash
# Enable debug logging
RUST_LOG=debug hakanai-server

# Validate environment variables
env | grep HAKANAI

# Test with minimal configuration
hakanai-server --allow-anonymous --upload-size-limit 1m
```

For production deployment guidance, see [DEPLOYMENT.md](DEPLOYMENT.md).