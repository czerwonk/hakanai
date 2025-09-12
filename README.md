# Hakanai (はかない)

![Banner](banner.svg)

A minimalist one-time secret sharing service built on zero-knowledge principles.

## Philosophy

Hakanai embodies the Japanese concept of transience - secrets that exist only for a moment before vanishing forever. No accounts, no tracking, no permanent storage. Just ephemeral data transfer with mathematical privacy guarantees and automatic expiration.

## Core Principles

- **Zero-Knowledge**: The server never sees your data. All encryption happens client-side.
- **Single View**: Secrets self-destruct after one access. No second chances.
- **No Metadata**: We store only encrypted bytes and an ID. Nothing else.
- **Minimalist**: One function only - share secrets that disappear.
- **Content Integrity**: Hash verification ensures secrets haven't been tampered with.

## How It Works

1. Your client (CLI or browser) encrypts the secret locally
2. Sends only the ciphertext to our server
3. You share the link with the decryption key (either embedded in URL or separately)
4. Recipient views once, then it's gone forever

**Enhanced Security Mode**: With `--separate-key`, the secret URL and decryption key are provided separately, allowing you to share them through different communication channels for defense in depth.

## Quick Start

### Docker Compose (Recommended)

```bash
# Start the services
docker compose up -d

# The server will be available at http://localhost:8080
```

### From Source

```bash
# Prerequisites: Rust 1.89+, Node.js, Redis
git clone https://github.com/czerwonk/hakanai
cd hakanai
npm install
cargo build --release --workspace

# Start server (admin token will be generated and logged)
./target/release/hakanai-server
```

## Usage Examples

### Send a Secret (CLI)

```bash
# Send from stdin
echo "my secret data" | hakanai send

# Send a file
hakanai send --file document.pdf

# Send multiple files (creates ZIP archive)
hakanai send -f file1.txt -f file2.pdf -f image.png

# Enhanced security (separate key)
echo "sensitive data" | hakanai send --separate-key

# With access restrictions
echo "restricted secret" | hakanai send --allow-ip 192.168.1.0/24 --allow-country US
```

### Retrieve a Secret (CLI)

```bash
# Get using the URL returned by send
hakanai get https://hakanai.example.com/s/uuid-here

# Get with separate key (when --separate-key was used)
hakanai get https://hakanai.example.com/s/uuid-here --key base64-key

# Save to custom location
hakanai get https://hakanai.example.com/s/uuid-here --output-dir /downloads/
```

### Web Interface

Visit your server URL (e.g., `http://localhost:8080`) to:

- Create new secrets with text or file uploads
- Apply access restrictions (IP, country, ASN, passphrase)
- Retrieve secrets directly in your browser
- Use clipboard-based sharing for automation

## Security Model

We implement true client-side encryption - your secrets are encrypted before leaving your device and decrypted only after retrieval. The server is just a temporary dead drop that forgets everything.

**Content Integrity**: All secrets include SHA-256 hash validation (truncated to 128 bits for manageable URLs) to detect tampering.

**Note**: This project focuses on application-layer encryption. Transport security (HTTPS/TLS) should be handled by a reverse proxy in production.

## Documentation

📚 **Complete documentation is available in the [docs/](docs/) directory:**

- **[Installation Guide](docs/INSTALLATION.md)** - All installation methods (Helm, Docker, source)
- **[CLI Documentation](docs/CLI.md)** - Complete command-line reference
- **[API Reference](docs/API.md)** - REST API documentation and examples
- **[Configuration](docs/CONFIGURATION.md)** - Server and CLI configuration options
- **[Customization](docs/CUSTOMIZATION.md)** - Asset overrides and white-labeling
- **[Deployment](docs/DEPLOYMENT.md)** - Production deployment and architecture
- **[Development](docs/DEVELOPMENT.md)** - Building, testing, and contributing
- **[Observability](docs/OBSERVABILITY.md)** - OpenTelemetry metrics, traces, and monitoring

**Live API Docs**: Visit `/docs` on your running server for interactive OpenAPI documentation.

## Key Features

- **Zero-knowledge encryption** (AES-256-GCM, client-side)
- **Multiple file support** with automatic ZIP archiving
- **Access restrictions** (IP/CIDR, country, ASN, passphrase)
- **Web interface** with dark/light mode and i18n support
- **Flexible authentication** (tokens, anonymous access)
- **Content integrity verification** with embedded hashes
- **OpenTelemetry observability** for production monitoring
- **Mobile-friendly** with QR code support and iOS Shortcuts integration

## Contributing

Contributions are welcome! Please see [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) for setup instructions and guidelines.

## License

Licensed under the [Apache License, Version 2.0](LICENSE).

See [NOTICE](NOTICE) for attribution requirements.
