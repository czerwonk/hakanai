# Hakanai (儚い)

![Logo](logo.svg)

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
3. You share the link with the decryption key
4. Recipient views once, then it's gone forever

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
      HAKANAI_TOKENS: "your-secret-token-here"
```

## Usage

### Server

```bash
# Start with default settings (port 8080, Redis on localhost)
hakanai-server

# Or with custom configuration
hakanai-server --port 3000 --listen 0.0.0.0 --redis-dsn redis://redis.example.com:6379/

# Start with authentication tokens (recommended for production)
hakanai-server --tokens secret-token-1 --tokens secret-token-2

# Note: If no tokens are provided, anyone can create secrets (not recommended for production)
```

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

# Combine options
hakanai send -f secret.txt -s https://hakanai.example.com --ttl 1h -t my-token
```

#### Retrieving a Secret

```bash
# Get using the full URL returned by send
hakanai get https://hakanai.example.com/secret/550e8400-e29b-41d4-a716-446655440000

# Get using the short link format
hakanai get https://hakanai.example.com/s/550e8400-e29b-41d4-a716-446655440000

# Secret is displayed and immediately destroyed on server
```

**Note**: You can also retrieve secrets using a web browser by visiting the server URL and pasting the secret link.

## Web Interface

Hakanai now includes a web interface for users who prefer not to use the CLI:
- Visit the server root (e.g., `https://hakanai.example.com/`) to access the web interface
- Paste a hakanai URL to retrieve secrets directly in your browser
- The same zero-knowledge encryption is maintained - decryption happens in your browser
- Mobile-friendly responsive design
- Multi-language support (English and German) with automatic browser language detection

## API Reference

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
- `404 Not Found`: Secret doesn't exist or already accessed

### GET /s/{id}
Short link for secret retrieval.

**Response:**
- For CLI clients: Plain text secret data
- For browsers: HTML page for secret retrieval
- `404 Not Found`: Secret doesn't exist or already accessed

### GET /logo.svg
Serves the hakanai logo.

### GET /icon.svg
Serves the hakanai icon.

### GET /scripts/hakanai-client.js
Serves the JavaScript client library for browser-based decryption.

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
# Build entire workspace
cargo build --workspace --verbose

# Build release version
cargo build --release --workspace
```

### Testing

```bash
# Run all tests
cargo test --verbose

# Run specific test
cargo test test_name --package hakanai-lib
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter (warnings as errors)
RUSTFLAGS="-Dwarnings" cargo clippy --workspace
```

## Architecture

Hakanai implements a zero-knowledge architecture:

1. **Client-side encryption**: All encryption/decryption happens in the client
2. **Server ignorance**: Server only stores encrypted blobs with UUIDs
3. **Automatic destruction**: Secrets self-destruct after first access or TTL
4. **No persistence**: No logs, no backups, no recovery

### Components

- **hakanai-lib**: Core functionality including:
  - Generic `Client<T>` trait for flexible client implementations
  - `SecretClient` for type-safe `Payload` handling with automatic serialization
  - `CryptoClient` for AES-256-GCM encryption/decryption
  - `WebClient` for HTTP transport
  - Shared data models (`Payload`, `PostSecretRequest`, `PostSecretResponse`)
- **hakanai** (CLI): User-friendly command-line interface
- **hakanai-server**: RESTful API server with Redis backend

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

For production deployments:
1. **Always use authentication tokens** to prevent unauthorized secret creation
2. **Configure reverse proxy** for TLS, rate limiting, and security headers
3. **Monitor server logs** (structured logging with tracing middleware included)
4. **Set appropriate Redis memory limits** and eviction policies
5. **Enable OpenTelemetry** for comprehensive observability

#### Security Audit Results
- ✅ **Comprehensive security audit completed (2025-07-06)**
- ✅ **Overall security rating: A-** (1 High, 6 Medium, 8 Low findings identified)
- ✅ **No Critical vulnerabilities** - all findings are operational improvements
- ✅ **Production ready** with proper infrastructure configuration
- ✅ **Zero-knowledge architecture** properly implemented with strong cryptographic foundations

### Current Status
- ✅ One-time access enforcement
- ✅ Automatic expiration with configurable TTL
- ✅ No user tracking or accounts
- ✅ Client-side AES-256-GCM encryption
- ✅ Token-based authentication with SHA-256 hashing
- ✅ Redis backend storage with connection pooling
- ✅ Web interface for browser-based retrieval
- ✅ Binary file support with progress tracking
- ✅ Short link format (`/s/{id}`) for easier sharing
- ✅ Internationalization support (English and German)
- ✅ OpenTelemetry integration for comprehensive observability
- ✅ Comprehensive test coverage (74+ tests)
- ✅ Docker deployment with Valkey/Redis included

### Security Implementation
- ✅ **Zero-knowledge architecture**: All encryption/decryption client-side
- ✅ **AES-256-GCM encryption**: Industry-standard authenticated encryption
- ✅ **Secure random generation**: Cryptographically secure nonces with OsRng
- ✅ **Memory safety**: Pure Rust implementation, no unsafe code
- ✅ **Generic client architecture**: Type-safe payload handling with trait abstractions
- ✅ **Layered design**: `SecretClient` → `CryptoClient` → `WebClient`
- ✅ **Input validation**: Comprehensive UUID, TTL, and data validation
- ✅ **Error security**: Generic error messages prevent information disclosure
- ✅ **Web security**: CSP headers, XSS prevention, secure DOM manipulation

### Identified Improvements
Based on recent code review and security audit:

**High Priority:**
- Implement secure memory clearing for secrets (zeroize crate)
- Add token file support to prevent process list exposure

**Medium Priority:**
- Implement atomic file operations to prevent race conditions
- Add structured error responses in API
- Implement browser compatibility checks in JavaScript client
- Document rate limiting requirements for production

**Low Priority:**
- Add cache headers for static assets
- Create integration tests for full secret lifecycle
- Add health check endpoint
- Consider TypeScript for JavaScript client

### Future Enhancements
- Key derivation from URL fragment (never sent to server)
- Optional password protection with Argon2
- Additional language translations
- Performance benchmarks and optimization

## Configuration

### Server Environment Variables

- `PORT`: Server port (default: 8080)
- `LISTEN_ADDRESS`: Bind address (default: 127.0.0.1)
- `REDIS_DSN`: Redis connection string (default: redis://127.0.0.1:6379/)
- `OTEL_EXPORTER_OTLP_ENDPOINT`: OpenTelemetry collector endpoint (optional, enables OTEL when set)

### Server Command-line Options

- `--port`: Override the port number
- `--listen`: Override the listen address
- `--redis-dsn`: Override the Redis connection string
- `--tokens`: Add authentication tokens (can be specified multiple times)

### Security Features

- **Zero-Knowledge Architecture**: All encryption/decryption happens client-side
- **AES-256-GCM Encryption**: Industry-standard authenticated encryption
- **Secure Random Generation**: Cryptographically secure nonce generation with OsRng
- **Authentication Token Whitelist**: When tokens are provided via `--tokens`, only requests with valid Bearer tokens can create secrets
- **SHA-256 Token Hashing**: Authentication tokens are securely hashed before storage
- **Request Logging**: Built-in request logging middleware for monitoring and debugging
- **One-time Access**: Secrets are automatically deleted after first retrieval
- **Input Validation**: Proper UUID validation and TTL enforcement
- **Error Handling**: Secure error messages that don't leak sensitive information
- **CORS Security**: Restrictive CORS policy by default, explicit origin allowlist required
- **Security Headers**: Built-in security headers (X-Frame-Options, X-Content-Type-Options, HSTS)

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

(c) Daniel Brendgen-Czerwonk, 2025. Licensed under [MIT](LICENSE) license.

