# Hakanai (儚い)

![Banner](banner.svg)

A minimalist one-time secret sharing service built on zero-knowledge principles.

## Philosophy

Hakanai embodies the Japanese concept of transience - secrets that exist only for a moment before vanishing forever. No accounts, no tracking, no persistence. Just ephemeral data transfer with mathematical privacy guarantees.

## Core Principles

- **Zero-Knowledge**: The server never sees your data. All encryption happens client-side.
- **Single View**: Secrets self-destruct after one access. No second chances.
- **No Metadata**: We store only encrypted bytes and an ID. Nothing else.
- **Minimalist**: One function only - share secrets that disappear.

## How It Works

1. Your client (CLI or browser) encrypts the secret locally
2. Sends only the ciphertext to our server
3. You share the link with the decryption key (either embedded in URL or separately)
4. Recipient views once, then it's gone forever

### Enhanced Security Mode

With the `--separate-key` option, Hakanai provides enhanced security by separating the secret URL from the decryption key:

1. **Traditional mode**: One URL contains both secret ID and key (`/s/uuid#key`)
2. **Separate key mode**: Secret URL (`/s/uuid`) and key are provided separately
3. **Defense in depth**: Share URL and key through different communication channels
4. **Reduced attack surface**: No cryptographic material in any single URL

## Security Model

We implement true client-side encryption - your secrets are encrypted before leaving your device and decrypted only after retrieval. The server is just a temporary dead drop that forgets everything.

**Note**: This project focuses on the application-layer encryption. Transport security (HTTPS/TLS) should be handled by a reverse proxy or load balancer in front of the server.

Built for those who believe privacy isn't about having something to hide - it's about having something to protect.

## Installation

### Prerequisites
- Rust 2024 edition or later
- Redis server (for backend storage)
- Standard Rust toolchain (`cargo`, `rustc`)

### From Source

```bash
# Clone the repository
git clone https://github.com/czerwonk/hakanai
cd hakanai

# Build all components
cargo build --release --workspace

# Binaries will be in:
# - ./target/release/hakanai (CLI)
# - ./target/release/hakanai-server (Server)
```

### Using Docker Compose

The easiest way to run Hakanai is with Docker Compose, which includes both the server and a Valkey (Redis-compatible) database:

```bash
# Start the services
docker-compose up -d

# The server will be available at http://localhost:8080

# View logs
docker-compose logs -f

# Stop the services
docker-compose down

# Stop and remove volumes (clears all stored secrets)
docker-compose down -v
```

For production deployment, create your own `docker-compose.override.yml`:

```yaml
services:
  hakanai:
    environment:
      HAKANAI_ALLOW_ANONYMOUS: "true"
      HAKANAI_ANONYMOUS_UPLOAD_SIZE_LIMIT: "64"
```

## Usage

### Server

```bash
# Start with default settings (port 8080, Redis on localhost)
# Admin token will be generated and logged on first startup
hakanai-server

# Or with custom configuration
hakanai-server --port 3000 --listen 0.0.0.0 --redis-dsn redis://redis.example.com:6379/

# Enable anonymous access (allows public secret creation with size limits)
hakanai-server --allow-anonymous

# Configure anonymous size limits (humanized format)
hakanai-server --allow-anonymous --anonymous-size-limit 64k

# Enable admin token system for token management
hakanai-server --enable-admin-token

# Production setup with admin token and monitoring
hakanai-server --enable-admin-token --allow-anonymous --anonymous-size-limit 32k --upload-size-limit 10m
```

**Token System:**
- **Admin token**: Optional admin API access (use `--enable-admin-token`)
- **User tokens**: Auto-created on first startup with 30-day TTL
- **Anonymous access**: Optional public access with size limits
- **Token recovery**: Use `--reset-admin-token` or `--reset-default-token` flags

### CLI

#### Sending a Secret

```bash
# Send from stdin (default: 24 hour expiration)
echo "my secret data" | hakanai send

# Send from a file
hakanai send --file secret.txt
hakanai send -f /path/to/secret.dat

# Send with custom TTL
echo "temporary password" | hakanai send --ttl 30m

# Send to custom server
echo "secret" | hakanai send --server https://hakanai.example.com

# Send with authentication token (required if server has token whitelist)
echo "secret" | hakanai send --token my-auth-token

# Generate separate key for enhanced security (key and URL shared via different channels)
echo "sensitive data" | hakanai send --separate-key
# Output:
# Secret sent successfully!
# 
# Secret link: https://hakanai.example.com/s/uuid
# Key:         base64-encoded-key

# Combine options
hakanai send -f secret.txt -s https://hakanai.example.com --ttl 1h -t my-token
```

#### Retrieving a Secret

```bash
# Get using the full URL returned by send
hakanai get https://hakanai.example.com/secret/550e8400-e29b-41d4-a716-446655440000

# Get using the short link format
hakanai get https://hakanai.example.com/s/550e8400-e29b-41d4-a716-446655440000

# Get using separate key (when --separate-key was used)
hakanai get https://hakanai.example.com/s/550e8400-e29b-41d4-a716-446655440000 --key base64-encoded-key

# Secret is displayed and immediately destroyed on server
```

#### Creating User Tokens (Admin Only)

```bash
# Create a user token with admin privileges
hakanai token --limit 5m --ttl 7d

# Create token with humanized size limits
hakanai token --limit 500k    # 500 KB
hakanai token --limit 1m      # 1 MB
hakanai token --limit 1024    # 1024 bytes

# Create token with custom server and settings
hakanai token --server https://hakanai.example.com --limit 2m --ttl 30d
```

**Size Format Options:**
- Plain numbers: bytes (e.g., `1024`)
- 'k' suffix: kilobytes (e.g., `500k`)
- 'm' suffix: megabytes (e.g., `1m`)
- Decimal values supported (e.g., `1.5m`, `2.5k`)

**Note**: You can also retrieve secrets using a web browser by visiting the server URL and pasting the secret link.

## Web Interface

Hakanai now includes a web interface for users who prefer not to use the CLI:
- Visit the server root (e.g., `https://hakanai.example.com/`) to access the web interface
- Create new secrets at `/create` - supports both text and file uploads
- Paste a hakanai URL to retrieve secrets directly in your browser
- Use `/share` for clipboard-based sharing (perfect for iOS Shortcuts integration)
- The same zero-knowledge encryption is maintained - all encryption/decryption happens in your browser
- **Dark/Light Mode Toggle**: Automatic system preference detection with manual override
- Mobile-friendly responsive design
- Multi-language support (English and German) with automatic browser language detection

### Clipboard-Based Sharing

The `/share` endpoint enables seamless integration with automation tools like iOS Shortcuts and provides Safari web view compatibility:

#### **Method 1: URL Fragment (Safari Web View Compatible)**
```
https://hakanai.example.com/share#data=base64data&filename=test.txt&token=authtoken&ttl=3600
```
- ✅ **Works in Safari web views** (no clipboard permissions needed)
- ✅ **Zero-knowledge** - data stays client-side in URL fragment
- ✅ **Auto-processes** - no user interaction required
- ⚠️ **Size limit:** ~5KB payload (Mobile Safari ~8KB fragment limit)

#### **Method 2: Clipboard JSON (Fallback)**
1. **Copy JSON payload to clipboard**:
   ```json
   {
     "data": "base64-encoded-content",
     "filename": "document.pdf",  // optional
     "token": "auth-token",        // optional  
     "ttl": 86400                 // optional (seconds)
   }
   ```

2. **Visit `/share`** - the page reads and validates clipboard content
3. **Review the preview** - shows file size, filename, and expiration time
4. **Click "Create Secret"** - encrypts client-side and generates the shareable URL
5. **URL is copied to clipboard** automatically for easy sharing

**iOS Shortcuts Integration**: Use fragment URLs for small secrets (< 5KB) or clipboard method for larger content. Both maintain zero-knowledge architecture. For detailed setup instructions and the ready-to-use shortcut file, see [docs/shortcuts-README.md](docs/shortcuts-README.md).

## API Reference

**📚 For complete API documentation, visit `/docs` on your running server.**

The documentation is automatically generated from the OpenAPI specification, ensuring it always reflects the current API state. Both human-readable docs and machine-readable specs are kept in perfect sync.

### POST /api/v1/secret
Create a new secret.

**Headers:**
- `Authorization: Bearer {token}` (required if server has token whitelist)

**Request:**
```json
{
  "data": "base64-encoded-secret",
  "expires_in": 3600  // seconds
}
```

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Error Responses:**
- `401 Unauthorized`: Invalid or missing token when server requires authentication
- `400 Bad Request`: Invalid request body

### GET /api/v1/secret/{id}
Retrieve a secret (one-time access).

**Response:**
- `200 OK`: Plain text secret data
- `404 Not Found`: Secret doesn't exist or has expired
- `410 Gone`: Secret was already accessed by someone else


### POST /api/v1/admin/tokens
Create user tokens (admin authentication required).

**Headers:**
- `Authorization: Bearer {admin-token}` (required)

**Request:**
```json
{
  "upload_size_limit": 5242880,  // optional, in bytes
  "ttl_seconds": 2592000         // optional, default 30 days
}
```

**Response:**
```json
{
  "token": "user-token-string"
}
```

**Error Responses:**
- `401 Unauthorized`: Invalid or missing admin token
- `400 Bad Request`: Invalid request body

### GET /ready
Readiness check endpoint - returns 200 OK when the server is ready to accept requests.

### GET /healthy
Health check endpoint - returns 200 OK when the server and all dependencies (Redis) are healthy.

## Development

### Project Structure

```
hakanai/
├── lib/          # Core library (client, crypto, models)
├── cli/          # Command-line interface
├── server/       # Actix-web server
└── Cargo.toml    # Workspace configuration
```

### Building

```bash
# Build entire workspace (includes automatic TypeScript bundling via Rollup)
cargo build --workspace --verbose

# Build release version
cargo build --release --workspace

# Manual TypeScript bundling (optional - automatically done by cargo build)
npm run build

# Clean TypeScript compiled files
make clean-ts

# TypeScript type checking only (no compilation)
tsc --noEmit
```

**Build Process:**
- `cargo build` automatically handles TypeScript compilation via `build.rs`
- TypeScript files are bundled using Rollup for optimal performance
- Single JavaScript bundle per page reduces HTTP requests
- Tree shaking eliminates unused code for smaller file sizes


### Testing

```bash
# Run all tests
cargo test --verbose

# Run specific test
cargo test test_name --package hakanai-lib

# Run TypeScript tests
npm test --prefix tests

# Run tests with coverage (if cargo-tarpaulin installed)
cargo tarpaulin --verbose
```

The project includes comprehensive test coverage with 200+ tests across all components.

### Code Quality

```bash
# Format code
cargo fmt

# Run linter (warnings as errors)
RUSTFLAGS="-Dwarnings" cargo clippy --workspace

# TypeScript compilation (automatically checks types)
tsc
```

## Architecture

Hakanai implements a zero-knowledge architecture:

1. **Client-side encryption**: All encryption/decryption happens in the client
2. **Server ignorance**: Server only stores encrypted blobs with UUIDs
3. **Automatic destruction**: Secrets self-destruct after first access or TTL
4. **No persistence**: No logs, no backups, no recovery

### Components

- **hakanai-lib**: Core library with encryption, client traits, and shared models
- **hakanai** (CLI): Command-line interface for sending and retrieving secrets
- **hakanai-server**: RESTful API server with Redis backend
- **TypeScript Client**: Browser-based client with the same zero-knowledge architecture

### Security & Deployment Notes

#### Security Architecture
Hakanai follows a **separation of concerns** security model:
- **Application Layer**: Zero-knowledge encryption, secure token handling, input validation
- **Infrastructure Layer**: TLS termination, rate limiting, DDoS protection (handled by reverse proxy)

#### Production Deployment
The server is designed to run behind a reverse proxy (nginx, Caddy, etc.) which handles:
- **TLS termination and HTTPS enforcement**
- **Rate limiting and DDoS protection**
- **Request filtering and header sanitization**
- **Response compression** (gzip, etc.) for improved performance

For production deployments:
1. **Always use authentication tokens** to prevent unauthorized secret creation
2. **Configure reverse proxy** for TLS, rate limiting, and security headers
3. **Monitor server logs** (structured logging with tracing middleware included)
4. **Set appropriate Redis memory limits** and eviction policies
5. **Enable OpenTelemetry** for comprehensive observability

#### Security Audit Results
- ✅ **Comprehensive security audit completed (2025-07-17)**
- ✅ **Overall security rating: A-** (production ready with minor improvements)
- ✅ **No Critical or High priority vulnerabilities** - only 3 low-priority operational improvements remain
- ✅ **Production ready** with proper infrastructure configuration
- ✅ **Zero-knowledge architecture** properly implemented with strong cryptographic foundations
- ✅ **Comprehensive memory safety** with automatic zeroization and secure cleanup



## Configuration

### Server Environment Variables

- `HAKANAI_PORT`: Server port (default: 8080)
- `HAKANAI_LISTEN_ADDRESS`: Bind address (default: 0.0.0.0)
- `HAKANAI_REDIS_DSN`: Redis connection string (default: redis://127.0.0.1:6379/)
- `HAKANAI_UPLOAD_SIZE_LIMIT`: Maximum upload size (default: 10m, supports humanized format like 1m, 500k, 1024)
- `HAKANAI_ALLOW_ANONYMOUS`: Allow anonymous access (default: false)
- `HAKANAI_ANONYMOUS_UPLOAD_SIZE_LIMIT`: Upload size limit for anonymous users (default: 32k, supports humanized format)
- `HAKANAI_ENABLE_ADMIN_TOKEN`: Enable admin token system (default: false)
- `HAKANAI_CORS_ALLOWED_ORIGINS`: Comma-separated allowed CORS origins (default: none)
- `HAKANAI_MAX_TTL`: Maximum allowed TTL in seconds (default: 604800, 7 days)
- `HAKANAI_IMPRESSUM_FILE`: Path to impressum/legal information text file (displays impressum link in footer when provided)
- `HAKANAI_PRIVACY_FILE`: Path to privacy policy/data protection text file (displays privacy policy link in footer when provided)
- `OTEL_EXPORTER_OTLP_ENDPOINT`: OpenTelemetry collector endpoint (optional, enables OTEL when set)

### Server Command-line Options

- `--port`: Override the port number
- `--listen`: Override the listen address
- `--redis-dsn`: Override the Redis connection string
- `--allow-anonymous`: Allow anonymous access without authentication
- `--anonymous-size-limit`: Set upload size limit for anonymous users (supports humanized format like 32k, 1m)
- `--enable-admin-token`: Enable admin token system for token management
- `--reset-admin-token`: Force regenerate admin token (requires --enable-admin-token)
- `--reset-user-tokens`: Clear all user tokens and create new default token
- `--impressum-file`: Path to impressum/legal information text file (displays impressum link in footer when provided)
- `--privacy-file`: Path to privacy policy/data protection text file (displays privacy policy link in footer when provided)

### Security Features

- Zero-knowledge architecture with client-side AES-256-GCM encryption
- One-time access with automatic secret deletion
- Token-based authentication with SHA-256 hashing
- Security headers (X-Frame-Options, X-Content-Type-Options, HSTS)
- Restrictive CORS policy requiring explicit origin allowlist

### Observability

When `OTEL_EXPORTER_OTLP_ENDPOINT` is set, Hakanai exports:
- **Traces**: Distributed tracing for all HTTP requests
- **Metrics**: Application performance and usage metrics
- **Logs**: Structured logs with trace correlation

The server automatically detects service name and version from Cargo metadata and includes resource information about the OS, process, and runtime environment.

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Write tests for new functionality
4. Ensure all tests pass and clippy is happy
5. Submit a pull request

## License

Licensed under [MIT](LICENSE) license.

