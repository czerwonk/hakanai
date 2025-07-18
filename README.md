# Hakanai (儚い)

![Logo](logo.svg)

A minimalist one-time secret sharing service built on zero-knowledge principles.

## Version 2.0 Changes

**⚠️ BREAKING CHANGES:**
- **Redis-based token system**: Token authentication now uses Redis instead of files/environment variables
- **Anonymous access support**: New `--allow-anonymous` option enables public access with size limits
- **Configuration changes**: Size limits now specified in KB instead of MB
- **Automatic token generation**: Admin token automatically created on first startup

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

# Configure anonymous size limits (in KB)
hakanai-server --allow-anonymous --anonymous-size-limit 64

# Production setup with anonymous access and monitoring
hakanai-server --allow-anonymous --anonymous-size-limit 32 --upload-size-limit 10240
```

**v2.0 Token System:**
- **Admin token**: Automatically generated on first startup and logged to console
- **User tokens**: Future admin API will allow generating user tokens
- **Anonymous access**: Optional public access with configurable size limits
- **⚠️ BREAKING**: `HAKANAI_TOKENS` environment variable removed in v2.0

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

**Note**: You can also retrieve secrets using a web browser by visiting the server URL and pasting the secret link.

## Web Interface

Hakanai now includes a web interface for users who prefer not to use the CLI:
- Visit the server root (e.g., `https://hakanai.example.com/`) to access the web interface
- Create new secrets at `/create` - supports both text and file uploads
- Paste a hakanai URL to retrieve secrets directly in your browser
- The same zero-knowledge encryption is maintained - all encryption/decryption happens in your browser
- **Dark/Light Mode Toggle**: Automatic system preference detection with manual override
- Mobile-friendly responsive design
- Multi-language support (English and German) with automatic browser language detection

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

### GET /s/{id}
Short link for secret retrieval.

**Response:**
- For CLI clients: Plain text secret data
- For browsers: HTML page for secret retrieval
- `404 Not Found`: Secret doesn't exist or has expired
- `410 Gone`: Secret was already accessed by someone else

### GET /logo.svg
Serves the hakanai logo.

### GET /icon.svg
Serves the hakanai icon.

### GET /scripts/hakanai-client.js
Serves the JavaScript client library for browser-based encryption/decryption. The client is **implemented in TypeScript** and automatically compiled to JavaScript during the build process for browser compatibility.

**TypeScript Client Architecture:**
- **Source**: TypeScript files in `server/src/typescript/`
- **Compiled Output**: JavaScript files in `server/src/includes/`
- **Build Integration**: Automatic compilation via `build.rs` during cargo build
- **Type Safety**: Comprehensive type definitions with strict type checking
- **Browser Compatibility**: Feature detection with graceful fallback

**TypeScript Client API:**
```javascript
// Create a client instance
const client = new HakanaiClient('https://hakanai.example.com');

// Create payload from text
const encoder = new TextEncoder();
const textBytes = encoder.encode("my secret text");
const payload = client.createPayload(); // optional filename parameter
payload.setFromBytes(textBytes);

// Create payload from file bytes
const fileBytes = new Uint8Array(fileData); // from FileReader
const filePayload = client.createPayload("document.pdf");
filePayload.setFromBytes(fileBytes);

// Send payload
const secretUrl = await client.sendPayload(payload, 3600); // TTL in seconds

// Retrieve payload
const retrievedPayload = await client.receivePayload(secretUrl);
const originalText = retrievedPayload.decode(); // for text data
const originalBytes = retrievedPayload.decodeBytes(); // for binary data
```

### GET /
Web interface for retrieving secrets - shows a form to paste hakanai URLs.

### GET /create
Web interface for creating secrets - supports text input and file uploads.

### GET /docs
Auto-generated API documentation page - comprehensive reference for developers using the REST API.

### GET /openapi.json
OpenAPI 3.0 specification file - the source of truth for API documentation and tooling integration (Postman, code generators, etc.).

### GET /ready
Readiness check endpoint - returns 200 OK when the server is ready to accept requests.

### GET /healthy
Health check endpoint - returns 200 OK when the server and all dependencies (Redis) are healthy.

### GET /s/{id}
Short link format for retrieving secrets. Dual-mode endpoint:
- Returns raw decrypted data for CLI clients
- Returns HTML page for browser clients (based on User-Agent)

### Static Assets
- `/style.css` - CSS stylesheet
- `/i18n.js` - Internationalization support (compiled from TypeScript)
- `/get-secret.js` - JavaScript for secret retrieval page (compiled from TypeScript)
- `/create-secret.js` - JavaScript for secret creation page (compiled from TypeScript)
- `/common-utils.js` - Shared utilities (compiled from TypeScript)
- `/hakanai-client.js` - Main client library (compiled from TypeScript)

**Note:** All JavaScript files are compiled from TypeScript sources during the build process.

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
# Build entire workspace (includes TypeScript compilation and HTML generation)
cargo build --workspace --verbose

# Build release version
cargo build --release --workspace

# Build TypeScript client only (compiles .ts to .js)
make build-ts

# Clean TypeScript compiled files
make clean-ts

# TypeScript type checking only (no compilation)
tsc --noEmit
```

#### Build-Time Template Generation

The server uses build-time template generation for consistent and efficient HTML serving:

**Generated Files (auto-generated, do not edit directly):**
- `server/src/includes/docs_generated.html` - API documentation from OpenAPI spec
- `server/src/includes/create-secret.html` - Create secret page with version substitution
- `server/src/includes/get-secret.html` - Retrieve secret page with version substitution

*Note: These files are excluded from git and regenerated on every build.*

**Source Templates (edit these):**
- `server/src/templates/docs.html` - API documentation template
- `server/src/templates/endpoint.html` - Individual endpoint template
- `server/src/templates/create-secret.html` - Create secret page template
- `server/src/templates/get-secret.html` - Retrieve secret page template

**How it works:**
1. `server/build.rs` runs during compilation
2. Templates in `templates/` directory are processed with tinytemplate
3. Variables like `{version}` are substituted with build-time values
4. Generated HTML files are embedded in the binary with `include_bytes!`
5. No runtime string replacement needed - templates are pre-processed

**Template Variables:**
- `{version}` - Cargo package version (e.g., "1.3.2")
- `{title}`, `{description}` - From OpenAPI spec
- `{endpoints}` - Generated endpoint documentation

This approach ensures version consistency and eliminates runtime overhead.

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

The project includes comprehensive test coverage with:
- **190+ total tests** across all components
- **Factory pattern** for dependency injection in CLI tests
- **Mock observers** to prevent console interference during testing
- **Proper test isolation** with tempfile for all file operations

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

- **hakanai-lib**: Core functionality including:
  - Generic `Client<T>` trait for flexible client implementations
  - `CryptoClient` for AES-256-GCM encryption/decryption with integrated `Payload` serialization
  - `WebClient` for HTTP transport
  - Shared data models (`Payload`, `PostSecretRequest`, `PostSecretResponse`)
  - Comprehensive memory safety with automatic zeroization
- **hakanai** (CLI): User-friendly command-line interface
- **hakanai-server**: RESTful API server with Redis backend
- **TypeScript Client**: Modern browser-based implementation with comprehensive type safety
  - **Full TypeScript Implementation**: Complete rewrite in TypeScript with strict type checking
  - **Automatic Compilation**: TypeScript sources compiled to JavaScript during build process
  - **Type-Safe Architecture**: Comprehensive interfaces and type definitions
  - **Zero-Knowledge Maintained**: Same cryptographic security as Rust client
  - **Browser Compatibility**: Comprehensive feature detection with graceful fallback
  - **Enhanced Error Handling**: Structured error types with detailed messages
  - **Chunked Processing**: Efficient handling of large files with 8KB chunks
  - **Modular Architecture**: Clean separation with dedicated classes for crypto, utils, and i18n
  - **Bytes-based PayloadData Interface**: Unified approach for text and binary data
    - `payload.setFromBytes(bytes)` - Sets data from raw bytes with automatic base64 encoding
    - `payload.decode()` - Decodes to text with proper Unicode handling
    - `payload.decodeBytes()` - Decodes to binary data as Uint8Array
    - `payload.data` - Readonly base64-encoded data field

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
- ✅ Comprehensive test coverage (190+ total tests: 120+ Rust, 70+ TypeScript including comprehensive serialization tests)
- ✅ Docker deployment with Valkey/Redis included
- ✅ **Enhanced TypeScript Client**: Bytes-based PayloadData interface with type safety
- ✅ **Unified Data Handling**: Consistent approach for text and binary data across all clients
- ✅ **Access Tracking**: Returns 410 Gone status if secret was already accessed
- ✅ **Dark/Light Mode**: System preference detection with manual toggle and localStorage persistence
- ✅ **Health Endpoints**: `/healthy` and `/ready` endpoints for monitoring and orchestration

### Security Implementation
- ✅ **Zero-knowledge architecture**: All encryption/decryption client-side
- ✅ **AES-256-GCM encryption**: Industry-standard authenticated encryption
- ✅ **Secure random generation**: Cryptographically secure nonces with OsRng
- ✅ **Memory safety**: Pure Rust implementation, no unsafe code
- ✅ **Generic client architecture**: Type-safe payload handling with trait abstractions
- ✅ **Simplified architecture**: `CryptoClient<Payload>` → `WebClient<Vec<u8>>` with clear security boundaries
- ✅ **Input validation**: Comprehensive UUID, TTL, and data validation
- ✅ **Error security**: Generic error messages prevent information disclosure
- ✅ **Web security**: CSP headers, XSS prevention, secure DOM manipulation

## Configuration

### Server Environment Variables

- `HAKANAI_PORT`: Server port (default: 8080)
- `HAKANAI_LISTEN_ADDRESS`: Bind address (default: 0.0.0.0)
- `HAKANAI_REDIS_DSN`: Redis connection string (default: redis://127.0.0.1:6379/)
- `HAKANAI_UPLOAD_SIZE_LIMIT`: Maximum upload size in KB (default: 10240, 10MB)
- `HAKANAI_ALLOW_ANONYMOUS`: Allow anonymous access (default: false)
- `HAKANAI_ANONYMOUS_UPLOAD_SIZE_LIMIT`: Upload size limit for anonymous users in KB (default: 32KB)
- `HAKANAI_CORS_ALLOWED_ORIGINS`: Comma-separated allowed CORS origins (default: none)
- `HAKANAI_MAX_TTL`: Maximum allowed TTL in seconds (default: 604800, 7 days)
- `OTEL_EXPORTER_OTLP_ENDPOINT`: OpenTelemetry collector endpoint (optional, enables OTEL when set)

### Server Command-line Options

- `--port`: Override the port number
- `--listen`: Override the listen address
- `--redis-dsn`: Override the Redis connection string
- `--allow-anonymous`: Allow anonymous access without authentication
- `--anonymous-size-limit`: Set upload size limit for anonymous users in KB

### Security Features

- **Zero-Knowledge Architecture**: All encryption/decryption happens client-side
- **AES-256-GCM Encryption**: Industry-standard authenticated encryption
- **Secure Random Generation**: Cryptographically secure nonce generation with OsRng
- **Token-based Authentication**: Redis-backed token system with automatic admin token generation
- **SHA-256 Token Hashing**: Authentication tokens are securely hashed before storage
- **Request Logging**: Built-in request logging middleware for monitoring and debugging
- **One-time Access**: Secrets are automatically deleted after first retrieval
- **Input Validation**: Proper UUID validation and TTL enforcement
- **Error Handling**: Secure error messages that don't leak sensitive information
- **CORS Security**: Restrictive CORS policy by default, explicit origin allowlist required
- **Security Headers**: The application sets these security headers:
  - `X-Frame-Options: DENY` - Prevents clickjacking attacks
  - `X-Content-Type-Options: nosniff` - Prevents MIME type sniffing
  - `Strict-Transport-Security: max-age=31536000; includeSubDomains` - Enforces HTTPS
  - Additional headers (CSP, etc.) should be configured at the reverse proxy level

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

