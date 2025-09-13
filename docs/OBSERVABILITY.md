# Observability Guide

Hakanai provides comprehensive observability through OpenTelemetry (OTEL) integration, including metrics, traces, and logs.

## Quick Start

Enable OpenTelemetry by setting the OTLP endpoint:

```bash
export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4317"
cargo run --package hakanai-server
```

## Configuration

### Environment Variables

- `OTEL_EXPORTER_OTLP_ENDPOINT`: OTLP endpoint URL (e.g., `http://localhost:4317`)
- `OTEL_SERVICE_NAME`: Override service name (default: `hakanai-server`)
- `OTEL_SERVICE_VERSION`: Override service version (default: from Cargo.toml)

### Docker Compose Example

```yaml
services:
  hakanai:
    image: hakanai:latest
    environment:
      - OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector:4317
    ports:
      - "8080:8080"

  otel-collector:
    image: otel/opentelemetry-collector-contrib:latest
    ports:
      - "4317:4317" # OTLP gRPC
      - "4318:4318" # OTLP HTTP
```

## Available Metrics

### Secret Lifecycle Metrics

| Metric                                    | Type      | Description                              | Labels              |
| ----------------------------------------- | --------- | ---------------------------------------- | ------------------- |
| `hakanai_secrets_created_total`           | Counter   | Total number of secrets created          | `user_type`         |
| `hakanai_secrets_retrieved_total`         | Counter   | Total number of secrets retrieved        | -                   |
| `hakanai_secret_size_bytes`               | Histogram | Distribution of secret sizes in bytes    | `user_type`         |
| `hakanai_secret_ttl_seconds`              | Histogram | Distribution of TTL values in seconds    | `user_type`         |
| `hakanai_secrets_with_restrictions_total` | Counter   | Secrets created with access restrictions | `user_type`, `type` |

### System Metrics

| Metric                          | Type  | Description                                        | Labels |
| ------------------------------- | ----- | -------------------------------------------------- | ------ |
| `hakanai_active_tokens`         | Gauge | Number of active user tokens                       | -      |
| `hakanai_active_secrets`        | Gauge | Number of secrets currently stored (not retrieved) | -      |
| `hakanai_expired_secrets_total` | Gauge | Number of secrets that expired without retrieval   | -      |

### Restriction Type Bitfield

The `type` label in `hakanai_secrets_with_restrictions_total` uses a bitfield encoding:

- **Bit 0 (1)**: IP restrictions
- **Bit 1 (2)**: Country restrictions
- **Bit 2 (4)**: ASN restrictions
- **Bit 3 (8)**: Passphrase protection

Common combinations:

- `0`: No restrictions
- `1`: IP only
- `2`: Country only
- `3`: IP + Country
- `4`: ASN only
- `8`: Passphrase only
- `9`: IP + Passphrase
- `15`: All restrictions

## Prometheus Queries

### Basic Queries

```promql
# Total secrets created in the last hour
increase(hakanai_secrets_created_total[1h])

# Secret creation rate per minute
rate(hakanai_secrets_created_total[5m]) * 60

# Average secret size
hakanai_secret_size_bytes_sum / hakanai_secret_size_bytes_count

# 95th percentile of secret sizes
histogram_quantile(0.95, hakanai_secret_size_bytes_bucket)
```

### Restriction Analysis

```promql
# Percentage of secrets with restrictions
sum(hakanai_secrets_with_restrictions_total) / sum(hakanai_secrets_created_total) * 100

# Most common restriction combinations (top 5)
topk(5, hakanai_secrets_with_restrictions_total)

# Secrets with IP restrictions (bit 0 set: 1, 3, 5, 7, 9, 11, 13, 15)
sum(hakanai_secrets_with_restrictions_total{type=~"1|3|5|7|9|11|13|15"})

# Secrets with passphrase (bit 3 set: 8-15)
sum(hakanai_secrets_with_restrictions_total{type=~"[89]|1[0-5]"})
```

### User Type Analysis

```promql
# Secret creation by user type
sum by (user_type) (rate(hakanai_secrets_created_total[5m]))

# Average secret size by user type
avg by (user_type) (hakanai_secret_size_bytes_sum / hakanai_secret_size_bytes_count)
```

## Grafana Dashboard

A sample Grafana dashboard configuration is available in [`monitoring/grafana/hakanai-dashboard.json`](../monitoring/grafana/hakanai-dashboard.json) (if available).

Key panels to include:

1. Secret creation rate
2. Active secrets gauge
3. Secret size distribution
4. TTL distribution
5. Restriction type breakdown (pie chart)
6. User type comparison

## Tracing

When OTEL is enabled, Hakanai provides distributed tracing for:

- HTTP request handling
- Redis operations
- Secret encryption/decryption
- Token validation

Trace context is propagated through standard HTTP headers, enabling correlation across services.

## Logging

Hakanai provides structured logging with OpenTelemetry integration. When OTEL is enabled, logs are sent to the configured endpoint alongside stdout for dual visibility.

### Configuration

Control log levels via the `RUST_LOG` environment variable:

```bash
# Set global log level
RUST_LOG=info hakanai-server

# Debug logging for troubleshooting
RUST_LOG=debug hakanai-server

# Module-specific logging
RUST_LOG=hakanai_server=debug,hakanai_lib=info hakanai-server

# Trace-level logging for specific handlers
RUST_LOG=hakanai_server::handlers=trace hakanai-server
```

### Dual Logging Mode

When OTEL is enabled, Hakanai provides dual logging:

- **Stdout**: Local debugging and container logs
- **OTEL Collector**: Centralized log aggregation and analysis

```bash
# Enable dual logging
OTEL_EXPORTER_OTLP_ENDPOINT="http://collector:4318" hakanai-server
```

### Log Features

- **Structured JSON Format**: All logs use structured JSON for easy parsing
- **Trace Correlation**: Automatic correlation between logs and distributed traces
- **Security Events**: Authentication failures, access denials, token validation
- **Performance Metrics**: Request timing, Redis latency, encryption operations
- **Error Context**: Detailed error information with stack traces when appropriate

### Viewing Logs

```bash
# View JSON structured logs
journalctl -u hakanai -f -o json | jq .

# Filter by log level
journalctl -u hakanai -f -o json | jq 'select(.level == "ERROR")'

# Search for specific events
journalctl -u hakanai -f -o json | jq 'select(.message | contains("secret_created"))'
```

### Log Correlation

Logs automatically include trace context when OTEL is enabled:

- `trace_id`: Correlate logs across services
- `span_id`: Identify specific operations
- `service.name`: Source service identification

## Monitoring Best Practices

1. **Alert on Error Rates**: Monitor failed secret creations/retrievals
2. **Track Token Usage**: Alert when token count approaches limits
3. **Monitor Secret Sizes**: Detect unusual patterns that might indicate abuse
4. **Analyze Restrictions**: Understand security feature adoption
5. **Set SLOs**: Define service level objectives for latency and availability

## Troubleshooting

### No Metrics Appearing

1. Verify OTEL endpoint is reachable:

   ```bash
   curl -v http://localhost:4317
   ```

2. Check server logs for OTEL initialization:

   ```
   INFO hakanai_server: OpenTelemetry tracing initialized
   ```

3. Ensure OTEL collector is configured to receive OTLP

### Missing Metrics

Some metrics update periodically (default: 30 seconds):

- `hakanai_active_tokens`
- `hakanai_active_secrets`
- `hakanai_expired_secrets_total`

Event-based metrics update in real-time:

- `hakanai_secrets_created_total`
- `hakanai_secrets_retrieved_total`

## Related Documentation

- [OpenTelemetry Documentation](https://opentelemetry.io/docs/)
- [Prometheus Query Language](https://prometheus.io/docs/prometheus/latest/querying/basics/)
- [Grafana Dashboards](https://grafana.com/docs/grafana/latest/dashboards/)
