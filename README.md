# Hakanai (儚い)

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

## Security Opinion

We chose true client-side encryption over convenience. This means client (Browser with JS, CLI) is required to perform the crypto operations, but your secrets remain yours alone. The server is just a temporary dead drop that forgets everything.

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

## Usage

### Server

```bash
# Start with default settings (port 8080, Redis on localhost)
hakanai-server

# Or with custom configuration
hakanai-server --port 3000 --listen 0.0.0.0 --redis-dsn redis://redis.example.com:6379/
```

### CLI

#### Sending a Secret

```bash
# Send from stdin (default: 24 hour expiration)
echo "my secret data" | hakanai send

# Send with custom TTL
echo "temporary password" | hakanai send --ttl 30m

# Send to custom server
echo "secret" | hakanai send --server https://hakanai.example.com

# Send from file
cat secret.txt | hakanai send
```

#### Retrieving a Secret

```bash
# Get using the full URL returned by send
hakanai get https://hakanai.example.com/secret/550e8400-e29b-41d4-a716-446655440000

# Secret is displayed and immediately destroyed on server
```

## API Reference

### POST /secret
Create a new secret.

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

### GET /secret/{id}
Retrieve a secret (one-time access).

**Response:**
- `200 OK`: Plain text secret data
- `404 Not Found`: Secret doesn't exist or already accessed

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

- **hakanai-lib**: Core functionality including crypto operations and HTTP client
- **hakanai** (CLI): User-friendly command-line interface
- **hakanai-server**: RESTful API server with Redis backend

## Security Considerations

> ⚠️ **Important**: Client-side encryption is not yet implemented. The current version stores secrets in plaintext on the server. Do not use for sensitive data until encryption is complete.

### Current Status
- ✅ One-time access enforcement
- ✅ Automatic expiration
- ✅ No user tracking or accounts
- ❌ Client-side encryption (TODO)

### Future Security Enhancements
- Implementation of AES-256-GCM encryption in `hakanai-lib`
- Key derivation from URL fragment (never sent to server)
- Optional password protection with Argon2

## Configuration

### Server Environment Variables

- `PORT`: Server port (default: 8080)
- `LISTEN_ADDRESS`: Bind address (default: 127.0.0.1)
- `REDIS_DSN`: Redis connection string (default: redis://127.0.0.1:6379/)

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Write tests for new functionality
4. Ensure all tests pass and clippy is happy
5. Submit a pull request

## License

(c) Daniel Brendgen-Czerwonk, 2025. Licensed under [MIT](LICENSE) license.

