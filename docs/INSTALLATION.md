# Installation Guide

This guide covers all methods for installing and running Hakanai.

## Prerequisites

- **Redis/Valkey**: Required for backend storage
- **For building from source**: Rust 1.89+, Node.js, npm

## Method 1: Docker Compose (Recommended)

The easiest way to run Hakanai is with Docker Compose, which includes both the server and a Valkey (Redis-compatible) database:

```bash
# Start the services
docker compose up -d

# The server will be available at http://localhost:8080

# View logs
docker compose logs -f

# Stop the services
docker compose down

# Stop and remove volumes (clears all stored secrets)
docker compose down -v
```

For production deployment, create your own `docker-compose.override.yml`:

```yaml
services:
  hakanai:
    environment:
      HAKANAI_ALLOW_ANONYMOUS: "true"
      HAKANAI_ANONYMOUS_UPLOAD_SIZE_LIMIT: "64k"
      HAKANAI_UPLOAD_SIZE_LIMIT: "10m"
      # Add other environment variables as needed
```

## Method 2: Kubernetes Deployment (Helm)

The easiest way to deploy Hakanai in Kubernetes is using our Helm chart:

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

### Helm Chart Features

The Helm chart includes:
- Redis with Sentinel for high availability
- Automatic TLS with ingress-nginx
- Rate limiting and security headers
- Network policies and pod security
- Horizontal pod autoscaling support

See the [Helm chart documentation](helm/hakanai/README.md) for detailed configuration options.

## Method 3: Building from Source

### Install Prerequisites

Ensure you have the required tools installed:
- **Rust 1.89+**: Install from [rustup.rs](https://rustup.rs)
- **Node.js and npm**: Install from [nodejs.org](https://nodejs.org) or your package manager
- **Redis**: Install from your package manager or [redis.io](https://redis.io)

Verify installations:
```bash
rustc --version  # Should be 1.89 or later
node --version   # Should be 16+
redis-cli ping   # Should return PONG
```

### Build Hakanai

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

### Run the Server

```bash
# Start with default settings (port 8080, Redis on localhost)
# Admin token will be generated and logged on first startup
./target/release/hakanai-server

# Or with custom configuration
./target/release/hakanai-server --port 3000 --listen 0.0.0.0 --redis-dsn redis://redis.example.com:6379/
```

### Install CLI Globally (Optional)

```bash
# Install the CLI for system-wide use
cargo install --path cli

# Now you can use 'hakanai' from anywhere
hakanai send --help
```

## Method 4: Pre-built Binaries

Check the [releases page](https://github.com/czerwonk/hakanai/releases) for pre-built binaries for your platform.

```bash
# Download and extract (example for Linux x64)
wget https://github.com/czerwonk/hakanai/releases/latest/download/hakanai-linux-x64.tar.gz
tar -xzf hakanai-linux-x64.tar.gz

# Make executable and run
chmod +x hakanai-server hakanai
./hakanai-server
```

## Verification

Once installed, verify your installation:

```bash
# Check server is running
curl http://localhost:8080/ready

# Test CLI
echo "test secret" | hakanai send
```

## Next Steps

- See [CONFIGURATION.md](CONFIGURATION.md) for server configuration options
- See [CLI.md](CLI.md) for complete CLI documentation
- See [DEPLOYMENT.md](DEPLOYMENT.md) for production deployment guidance