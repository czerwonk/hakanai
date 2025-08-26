# Hakanai (儚い)

![Banner](banner.svg)

A minimalist one-time secret sharing service built on zero-knowledge principles.

## Philosophy

Hakanai embodies the Japanese concept of transience - secrets that exist only for a moment before vanishing forever. No accounts, no tracking, no permanent storage. Just ephemeral data transfer with mathematical privacy guarantees and automatic expiration.

## Core Principles

- **Zero-Knowledge**: The server never sees your data. All encryption happens client-side.
- **Single View**: Secrets self-destruct after one access. No second chances.
- **No Metadata**: We store only encrypted bytes and an ID. Nothing else.
- **Minimalist**: One function only - share secrets that disappear.
- **Content Integrity**: hash verification ensures secrets haven't been tampered with.

## How It Works

1. Your client (CLI or browser) encrypts the secret locally
2. Sends only the ciphertext to our server
3. You share the link with the decryption key (either embedded in URL or separately)
4. Recipient views once, then it's gone forever

### Enhanced Security Mode

With the `--separate-key` option, Hakanai provides enhanced security by separating the secret URL from the decryption key:

1. **Traditional mode**: One URL contains both secret ID and key (`/s/uuid#key:hash`)
2. **Separate key mode**: Secret URL (`/s/uuid`) and key are provided separately
3. **Defense in depth**: Share URL and key through different communication channels
4. **Reduced attack surface**: No cryptographic material in any single URL

## Security Model

We implement true client-side encryption - your secrets are encrypted before leaving your device and decrypted only after retrieval. The server is just a temporary dead drop that forgets everything.

**Content Integrity**: Hakanai automatically verifies that secrets haven't been tampered with using SHA-256 hashes truncated to 128 bits. This is a deliberate tradeoff between cryptographic security and usability - the 22-character hash keeps URLs manageable and QR codes scannable while still providing strong tamper detection. All URLs must use the format `#key:hash` where the hash validates the decrypted content for security.

**Note**: This project focuses on the application-layer encryption. Transport security (HTTPS/TLS) should be handled by a reverse proxy or load balancer in front of the server.

Built for those who believe privacy isn't about having something to hide - it's about having something to protect.

## Installation

### Kubernetes Deployment (Helm)

The easiest way to deploy Hakanai is using our Helm chart:

```bash
# Add the Hakanai Helm repository
helm repo add hakanai https://czerwonk.github.io/hakanai
helm repo update

# Install with default values (secure by default)
helm install hakanai hakanai/hakanai

# Or with custom configuration
helm install hakanai hakanai/hakanai \
  --set ingress.hosts[0].host=secrets.example.com \
  --set redis.replica.replicaCount=3
```

The Helm chart includes:
- Redis with Sentinel for high availability
- Automatic TLS with ingress-nginx
- Rate limiting and security headers
- Network policies and pod security
- Horizontal pod autoscaling support

See the [Helm chart documentation](helm/hakanai/README.md) for detailed configuration options.

### Manual Installation

#### Prerequisites
- Rust 1.89 or later (stable toolchain)
- Redis server (for backend storage)
- Node.js and npm (for TypeScript bundling)
- Standard Rust toolchain (`cargo`, `rustc`)

### From Source

```bash
# Clone the repository
git clone https://github.com/czerwonk/hakanai
cd hakanai

# Install npm dependencies for TypeScript bundling
npm install

# Build all components (includes automatic TypeScript bundling)
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

# Enable admin token system for token management (requires trusted IP ranges)
hakanai-server --enable-admin-token --trusted-ip-ranges "127.0.0.0/8,10.0.0.0/8"

# Production setup with admin token and monitoring
hakanai-server --enable-admin-token --trusted-ip-ranges "192.168.1.0/24" --allow-anonymous --anonymous-size-limit 32k --upload-size-limit 10m

# With webhook notifications for audit/monitoring (v2.8+)
hakanai-server --webhook-url https://example.com/webhook --webhook-token "token123"
```

**Token System:**
- **Admin token**: Optional admin API access (use `--enable-admin-token`)
- **User tokens**: Auto-created on first startup with 30-day TTL
- **Anonymous access**: Optional public access with size limits
- **Token recovery**: Use `--reset-admin-token` or `--reset-default-token` flags

**Webhook Notifications (v2.8+):**
- **Lifecycle events**: Get notified when secrets are created or retrieved
- **Zero-knowledge preserved**: Only metadata (UUID, headers) sent, never secret content
- **Extensible**: Corporate deployments can implement custom observers for audit trails
- **Fire-and-forget**: Webhooks don't block secret operations

### CLI

#### Sending a Secret

```bash
# Send from stdin (default: 24 hour expiration)
echo "my secret data" | hakanai send

# Send from a file
hakanai send --file secret.txt
hakanai send -f /path/to/secret.dat

# Send multiple files (automatically creates ZIP archive)
hakanai send -f document.pdf -f image.png -f data.csv
hakanai send --file report.pdf --file appendix.docx

# Send with custom TTL
echo "temporary password" | hakanai send --ttl 30m

# Send to custom server
echo "secret" | hakanai send --server https://hakanai.example.com

# Send with authentication token (required if server has token whitelist)
echo "secret" | HAKANAI_TOKEN=my-auth-token hakanai send
# Or using token file
echo "secret" | hakanai send --token-file /path/to/token.txt

# Generate separate key for enhanced security (key and URL shared via different channels)
echo "sensitive data" | hakanai send --separate-key

# Restrict access to specific IP addresses or CIDR ranges
echo "restricted secret" | hakanai send --allow-ip 192.168.1.0/24 --allow-ip 10.0.0.1
echo "office only" | hakanai send --allow-ip 203.0.113.0/24 --allow-ip 2001:db8:85a3::/48

# Restrict access to specific countries (ISO 3166-1 alpha-2 codes)
echo "US/Canada only" | hakanai send --allow-country US --allow-country CA
echo "EU restricted" | hakanai send --allow-country DE --allow-country FR --allow-country NL

# Restrict access to specific ASNs (Autonomous System Numbers)
echo "Cloudflare only" | hakanai send --allow-asn 13335
echo "Multiple ISPs" | hakanai send --allow-asn 13335 --allow-asn 15169 --allow-asn 32934

# Combine IP, country, and ASN restrictions
echo "comprehensive restrictions" | hakanai send --allow-ip 192.168.1.0/24 --allow-country DE --allow-asn 202739
# Output:
# Secret sent successfully!
# 
# Secret link: https://hakanai.example.com/s/uuid
# Key:         base64-encoded-key

# Display URL as QR code for easy mobile sharing
echo "secret" | hakanai send --qr-code

# Combine options
HAKANAI_TOKEN=my-token hakanai send -f secret.txt -s https://hakanai.example.com --ttl 1h
```

#### Retrieving a Secret

```bash
# Get using the full URL returned by send
hakanai get https://hakanai.example.com/secret/550e8400-e29b-41d4-a716-446655440000

# Get using the short link format
hakanai get https://hakanai.example.com/s/550e8400-e29b-41d4-a716-446655440000

# Get using separate key (when --separate-key was used)
hakanai get https://hakanai.example.com/s/550e8400-e29b-41d4-a716-446655440000 --key base64-encoded-key

# Save to a specific file instead of using payload filename
hakanai get https://hakanai.example.com/s/550e8400 --filename custom-name.txt

# Save files to a custom directory instead of current directory
hakanai get https://hakanai.example.com/s/550e8400 --output-dir /path/to/downloads/

# Extract ZIP archives to specified directory
hakanai get https://hakanai.example.com/s/550e8400 --extract --output-dir /path/to/extract/

# Output to stdout (useful for piping to other commands)
hakanai get https://hakanai.example.com/s/550e8400 --to-stdout

# Secret is displayed and immediately destroyed on server
```

#### Creating User Tokens (Admin Only)

```bash
# Create a user token with admin privileges (will prompt for admin token)
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
  "expires_in": 3600,  // seconds
  "restrictions": {    // optional
    "allowed_ips": ["192.168.1.0/24", "10.0.0.1"],
    "allowed_countries": ["US", "DE", "CA"],
    "allowed_asns": [13335, 15169, 202739]
  }
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
Create user tokens (admin authentication and trusted IP required).

**Headers:**
- `Authorization: Bearer {admin-token}` (required)
- Request must originate from trusted IP range configured with `--trusted-ip-ranges`

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
4. **Temporary storage**: Redis-based temporary persistence with automatic expiration and no permanent backups

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

## Configuration

### Size Limits

**Important:** All size limits apply to the **secret data** before any encryption or encoding. The server automatically accounts for encryption and base64 encoding overhead (approximately 50% additional space), so you can configure limits based on your actual data sizes without worrying about the technical overhead.

For example:
- Setting `--upload-size-limit 10m` allows users to upload files up to 10MB in size
- The server internally handles up to ~15MB to account for encryption/encoding overhead
- This prevents legitimate uploads from failing due to encoding overhead

### Server Environment Variables

- `HAKANAI_PORT`: Server port (default: 8080)
- `HAKANAI_LISTEN_ADDRESS`: Bind address (default: 127.0.0.1)
- `HAKANAI_REDIS_DSN`: Redis connection string (default: redis://127.0.0.1:6379/)
- `HAKANAI_UPLOAD_SIZE_LIMIT`: Maximum upload size for secret data before encryption (default: 10m, supports humanized format like 1m, 500k, 1024)
- `HAKANAI_ALLOW_ANONYMOUS`: Allow anonymous access (default: false)
- `HAKANAI_ANONYMOUS_UPLOAD_SIZE_LIMIT`: Upload size limit for anonymous users' secret data before encryption (default: 32k, supports humanized format)
- `HAKANAI_ENABLE_ADMIN_TOKEN`: Enable admin token system (default: false)
- `HAKANAI_CORS_ALLOWED_ORIGINS`: Comma-separated allowed CORS origins (default: none)
- `HAKANAI_MAX_TTL`: Maximum allowed TTL in seconds (default: 604800, 7 days)
- `HAKANAI_IMPRESSUM_FILE`: Path to impressum/legal information text file (displays impressum link in footer when provided)
- `HAKANAI_PRIVACY_FILE`: Path to privacy policy/data protection text file (displays privacy policy link in footer when provided)
- `HAKANAI_WEBHOOK_URL`: Webhook URL for secret lifecycle notifications (optional)
- `HAKANAI_WEBHOOK_TOKEN`: Bearer token for webhook authentication (optional)
- `HAKANAI_WEBHOOK_HEADERS`: Comma-separated list of HTTP headers to include in webhook requests (default: user-agent,x-forwarded-for,x-forwarded-proto,x-real-ip,x-request-id)
- `HAKANAI_SHOW_TOKEN_INPUT`: Show authentication token input field in web interface (default: false)
- `HAKANAI_TRUSTED_IP_RANGES`: Comma-separated IP ranges (CIDR notation) that bypass size limits (optional)
- `HAKANAI_TRUSTED_IP_HEADER`: HTTP header to check for client IP when behind a proxy (default: x-forwarded-for)
- `HAKANAI_COUNTRY_HEADER`: HTTP header to check for client country code (optional, enables geo-restrictions when set)
- `HAKANAI_ASN_HEADER`: HTTP header to check for client ASN (optional, enables ASN-based restrictions when set)
- `OTEL_EXPORTER_OTLP_ENDPOINT`: OpenTelemetry collector endpoint (optional, enables OTEL when set)

### CLI Environment Variables

- `HAKANAI_SERVER`: Default server URL for CLI commands
- `HAKANAI_TOKEN`: Authentication token for CLI operations
- `HAKANAI_TTL`: Default TTL for send command
- `HAKANAI_TOKEN_TTL`: Default TTL for token command
- `HAKANAI_QR_CODE`: Enable QR code output by default
- `HAKANAI_TO_STDOUT`: Output secrets to stdout by default
- `HAKANAI_OUTPUT_DIR`: Default output directory for retrieved secrets and extracted files

### CLI Command-line Options

#### Send Command Options
- `-s, --server`: Hakanai server URL (default: http://localhost:8080)
- `--ttl`: Time after the secret vanishes (default: 24h, supports humanized format like 30m, 1h, 7d)
- `--token-file`: File containing authorization token (environment variable takes precedence)
- `-f, --file`: File to read the secret from (can be specified multiple times for multiple files)
- `-a, --as-file`: Send the secret as a file (auto-detected for binary content)
- `--filename`: Custom filename when sending as a file
- `--separate-key`: Print key separately for enhanced security (share via different channels)
- `--allow-ip`: IP addresses/CIDR ranges allowed to access the secret (can be specified multiple times)
- `--allow-country`: Country codes (ISO 3166-1 alpha-2) allowed to access the secret (can be specified multiple times)
- `--allow-asn`: Autonomous System Numbers (ASNs) allowed to access the secret (can be specified multiple times)
- `-q, --qr-code`: Display URL as QR code for easy mobile sharing

#### Get Command Options  
- `-k, --key`: Base64 encoded secret key (when not included in URL fragment)
- `--to-stdout`: Output secret to stdout (useful for piping to other commands)
- `-f, --filename`: Save to specific file (overrides payload filename)
- `-e, --extract`: Extract ZIP archives to current directory
- `-o, --output-dir`: Save files to this directory instead of current directory

#### Token Command Options (Admin Only)
- `-s, --server`: Hakanai server URL (default: http://localhost:8080) 
- `--ttl`: Token expiration time (default: 30d, supports humanized format)
- `-l, --limit`: Upload size limit for the token (supports humanized format like 1m, 500k)

### Server Command-line Options

- `--port`: Override the port number
- `--listen`: Override the listen address
- `--redis-dsn`: Override the Redis connection string
- `--allow-anonymous`: Allow anonymous access without authentication
- `--anonymous-size-limit`: Set upload size limit for anonymous users (supports humanized format like 32k, 1m)
- `--enable-admin-token`: Enable admin token system for token management (requires --trusted-ip-ranges)
- `--reset-admin-token`: Force regenerate admin token (requires --enable-admin-token) without starting the server
- `--reset-user-tokens`: Clear all user tokens and create new default token without starting the server
- `--impressum-file`: Path to impressum/legal information text file (displays impressum link in footer when provided)
- `--privacy-file`: Path to privacy policy/data protection text file (displays privacy policy link in footer when provided)
- `--webhook-url`: Webhook URL for secret lifecycle notifications
- `--webhook-token`: Bearer token for webhook authentication
- `--show-token-input`: Show authentication token input field in web interface
- `--trusted-ip-ranges`: IP ranges (CIDR notation) that bypass size limits
- `--trusted-ip-header`: HTTP header to check for client IP when behind a proxy (default: x-forwarded-for)
- `--country-header`: HTTP header to check for client country code (enables geo-restrictions)
- `--asn-header`: HTTP header to check for client ASN (enables ASN-based restrictions)

### Access Control & Geo-Restrictions

#### Secret-Level Access Restrictions

Hakanai supports restricting individual secrets to specific IP addresses/CIDR ranges, countries, and/or Autonomous System Numbers (ASNs). This provides additional security layers by ensuring only authorized networks, geographic locations, and network providers can access the secret.

**Geographic Restrictions:**
Country-based restrictions use the `CF-IPCountry` header (commonly provided by Cloudflare and other CDNs) or a configurable country header to determine the client's location. This requires proper reverse proxy configuration.

**Country Header Configuration:**
```bash
# Cloudflare (most common)
hakanai-server --country-header cf-ipcountry

# Custom CDN or proxy setup
hakanai-server --country-header x-country-code

# Environment variable
export HAKANAI_COUNTRY_HEADER="cf-ipcountry"
```

**ASN-based Restrictions:**
ASN restrictions use HTTP headers to determine the client's Autonomous System Number, allowing filtering based on network provider (ISP, cloud provider, CDN, etc.). This is useful for restricting access to specific hosting providers or ISPs.

**Header Configuration:**
```bash
# For ASN detection (requires custom proxy/CDN setup)
hakanai-server --asn-header x-asn-number

# Environment variable
export HAKANAI_ASN_HEADER="x-asn-number"
```

**Note:** Geographic and ASN restrictions require server-side detection via HTTP headers. For deployments behind Cloudflare or similar CDNs that provide geo/ASN information, configure the appropriate headers. Without proper header configuration, these restrictions will return HTTP 501 Not Implemented.

**CLI Usage:**
```bash
# Restrict to specific IPs and networks
hakanai send --allow-ip 192.168.1.0/24 --allow-ip 10.0.0.1 --allow-ip 2001:db8::/32

# Office network and VPN endpoint
echo "sensitive data" | hakanai send --allow-ip 203.0.113.0/24 --allow-ip 198.51.100.1

# Restrict by country (ISO 3166-1 alpha-2 codes)
echo "US only" | hakanai send --allow-country US
echo "EU access" | hakanai send --allow-country DE --allow-country FR --allow-country NL

# Restrict by ASN (network provider)
echo "Cloudflare only" | hakanai send --allow-asn 13335
echo "Major CDNs" | hakanai send --allow-asn 13335 --allow-asn 15169 --allow-asn 16509

# Combine all restriction types
echo "comprehensive restrictions" | hakanai send --allow-ip 10.0.0.0/8 --allow-country DE --allow-asn 202739
```

**Web Interface:**
The web interface includes optional restriction fields:
- "IP Address Restrictions": Enter IP addresses or CIDR ranges (one per line)
- "Country Restrictions": Enter ISO 3166-1 alpha-2 country codes (one per line, e.g., US, DE, CA)
- "ASN Restrictions": Enter Autonomous System Numbers (one per line, e.g., 13335, 15169, 202739)

**API Usage:**
```json
{
  "data": "encrypted_secret_data", 
  "expires_in": 3600,
  "restrictions": {
    "allowed_ips": ["192.168.1.0/24", "10.0.0.1", "2001:db8::/32"],
    "allowed_countries": ["US", "DE", "CA"],
    "allowed_asns": [13335, 15169, 202739]
  }
}
```

**Supported IP Formats:**
- IPv4 addresses: `192.168.1.100`
- IPv4 CIDR: `192.168.1.0/24`
- IPv6 addresses: `2001:db8::1`
- IPv6 CIDR: `2001:db8::/32`

**Supported Country Formats:**
- ISO 3166-1 alpha-2 codes: `US`, `DE`, `CA`, `GB`, `FR`, `JP`, `AU`, etc.
- Must be exactly 2 uppercase letters
- Examples: `US` (United States), `DE` (Germany), `CA` (Canada), `GB` (United Kingdom)

**Supported ASN Formats:**
- 32-bit unsigned integers: `0` to `4294967295`
- Examples: `13335` (Cloudflare), `15169` (Google), `16509` (Amazon), `32934` (Facebook), `202739` (example German ASN)

#### Server-Level IP Whitelisting

Hakanai supports server-level IP whitelisting that serves two purposes:
1. **Bypass upload size limits** for trusted networks (internal services, monitoring systems, backup operations)
2. **Required for admin API access** when `--enable-admin-token` is used

This provides defense-in-depth security by ensuring administrative operations can only be performed from trusted networks.

**Configuration:**
```bash
# Single IPv4 range
--trusted-ip-ranges 10.0.0.0/8

# Multiple IPv4 and IPv6 ranges
--trusted-ip-ranges 10.0.0.0/8,192.168.1.0/24,2001:db8::/32,::1/128

# Environment variable
export HAKANAI_TRUSTED_IP_RANGES="172.16.0.0/12,2001:db8:85a3::/48"

# Custom proxy header (for Cloudflare, nginx, etc.)
--trusted-ip-header cf-connecting-ip
```

**Common IPv6 ranges:**
- `::1/128` - IPv6 localhost
- `2001:db8::/32` - IPv6 documentation prefix
- `2001::/16` - Global unicast prefix
- `fe80::/10` - Link-local addresses
- `::/0` - All IPv6 addresses (use with caution)

**Security considerations:**
- CIDR notation is validated at server startup
- IPs are extracted from configurable proxy headers (default: `x-forwarded-for`)
- Falls back to connection peer address if header is missing
- Invalid CIDR ranges prevent server startup
- Admin API requires both valid admin token AND trusted IP (defense in depth)
- Admin token system cannot be enabled without configuring trusted IP ranges

### Security Features

- Zero-knowledge architecture with client-side AES-256-GCM encryption
- One-time access with automatic secret deletion
- Token-based authentication with SHA-256 hashing
- IP-based whitelisting for trusted networks
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

Licensed under the [Apache License, Version 2.0](LICENSE).

See [NOTICE](NOTICE) for attribution requirements.

