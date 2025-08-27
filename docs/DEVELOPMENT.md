# Development Guide

This guide covers building, testing, contributing, and developing with Hakanai.

## Development Setup

### Prerequisites

- **Rust 1.89+** with stable toolchain
- **Node.js and npm** for TypeScript bundling  
- **Redis server** for backend storage
- **Git** for version control

### Quick Start

```bash
# Clone repository
git clone https://github.com/czerwonk/hakanai
cd hakanai

# Install and build
npm install
cargo build --workspace

# Run tests
cargo test
npm test --prefix tests

# Start server
cargo run --package hakanai-server
```

## Project Structure

```
hakanai/
├── lib/          # Core library (client, crypto, models)
├── cli/          # Command-line interface
├── server/       # Actix-web server
└── Cargo.toml    # Workspace configuration
```

## Building & Testing

### Build Commands

```bash
# Development build (includes TypeScript bundling)
cargo build --workspace

# Release build
cargo build --release --workspace

# Clean everything
cargo clean && make clean-ts
```

**Note:** TypeScript is automatically compiled via `build.rs` during `cargo build`. Manual TypeScript commands (`npm run build`, `tsc`) are optional for debugging.

### Testing

```bash
# Run all Rust tests
cargo test

# Run specific test
cargo test test_name --package hakanai-lib -- --nocapture

# Run TypeScript tests
npm test --prefix tests

# Watch mode (TypeScript)
npm test --prefix tests -- --watch
```

**Test Coverage:**
- **Rust**: 200+ tests covering crypto, client, server
- **TypeScript**: 177 tests covering UI, crypto, compatibility

## Code Style

### Rust Conventions

```bash
# Format code
cargo fmt

# Lint with all warnings as errors
RUSTFLAGS="-Dwarnings" cargo clippy --workspace

# Check for common issues
cargo audit
```

**Style Guidelines:**
- Follow standard Rust conventions
- Prefer explicit error types with `thiserror`
- Use async/await with Tokio runtime
- Implement proper `Drop` for sensitive data
- Use `Zeroizing<T>` for memory safety

### TypeScript Conventions

```bash
# Type checking
tsc --noEmit

# Linting (if eslint configured)
npm run lint
```

**Style Guidelines:**
- Strict TypeScript mode enabled
- Modern ES2017+ features
- Consistent error handling with custom error classes
- Proper memory management for sensitive data
- DOM utilities for consistent element manipulation

## Development Workflow

### Local Development

1. **Start Redis:**
   ```bash
   redis-server
   ```

2. **Start development server:**
   ```bash
   cargo run --package hakanai-server -- --allow-anonymous
   ```

3. **Test CLI:**
   ```bash
   echo "test secret" | cargo run --package hakanai -- send
   ```

4. **Run tests in watch mode:**
   ```bash
   # Terminal 1: Rust tests
   cargo watch -x test
   
   # Terminal 2: TypeScript tests
   npm test --prefix tests -- --watch
   ```

### Feature Development

1. **Create feature branch:**
   ```bash
   git checkout -b feature/new-feature
   ```

2. **Write tests first (TDD):**
   ```rust
   #[tokio::test]
   async fn test_new_feature() {
       // Arrange
       // Act  
       // Assert
   }
   ```

3. **Implement feature**

4. **Ensure all tests pass:**
   ```bash
   cargo test --workspace
   npm test --prefix tests
   ```

5. **Check code quality:**
   ```bash
   cargo fmt --check
   RUSTFLAGS="-Dwarnings" cargo clippy --workspace
   ```

## Architecture Deep Dive

### Generic Client Architecture

```rust
// Core trait allowing different implementations
pub trait Client<T> {
    async fn send(&self, payload: T) -> Result<String>;
    async fn get(&self, url: &str) -> Result<T>;
}

// Layered implementation
CryptoClient<Payload> -> WebClient<Vec<u8>>
```

### Zero-Knowledge Implementation

```rust
// All encryption happens client-side
pub struct CryptoContext {
    key: Zeroizing<Vec<u8>>,      // Auto-cleared
    nonce: Zeroizing<Vec<u8>>,    // Auto-cleared  
}

impl Drop for CryptoContext {
    fn drop(&mut self) {
        // Sensitive data automatically zeroed
    }
}
```

### Memory Safety

```rust
use zeroize::Zeroizing;

// Sensitive data is automatically cleared
let plaintext = Zeroizing::new(secret_data);
let key = Zeroizing::new(generate_key());

// Memory cleared on drop
```

## API Integration Examples

### Custom Client Implementation

```rust
use hakanai_lib::{Client, Payload, WebClient};

struct CustomClient {
    web_client: WebClient,
    api_key: String,
}

impl Client<Payload> for CustomClient {
    async fn send(&self, payload: Payload) -> Result<String> {
        // Custom logic with authentication
        let mut headers = HashMap::new();
        headers.insert("X-API-Key".to_string(), self.api_key.clone());
        
        self.web_client.send_with_headers(payload, headers).await
    }
}
```

### Integration Testing

```rust
#[tokio::test]
async fn test_full_secret_lifecycle() {
    let client = setup_test_client().await;
    
    // Create secret
    let original = "test secret data";
    let url = client.send(original.into()).await?;
    
    // Retrieve secret  
    let retrieved = client.get(&url).await?;
    
    assert_eq!(original, retrieved.decode()?);
    
    // Verify one-time access
    let result = client.get(&url).await;
    assert!(result.is_err()); // Should fail on second access
}
```

## Contributing

### Code Review Checklist

- [ ] **Tests written** for new functionality
- [ ] **All tests pass** (Rust and TypeScript)
- [ ] **Code formatted** (`cargo fmt`, `tsc`)
- [ ] **No clippy warnings** with `-Dwarnings`
- [ ] **Memory safety** verified for sensitive data
- [ ] **Error handling** is comprehensive
- [ ] **Documentation** updated for public APIs
- [ ] **CHANGELOG** entry added if needed

### Pull Request Process

1. **Fork the repository**
2. **Create feature branch** from `main`
3. **Implement changes** with tests
4. **Update documentation** if needed
5. **Ensure CI passes** (all tests, linting)
6. **Submit pull request** with clear description

### Contribution Guidelines

**Security Contributions:**
- Follow responsible disclosure for vulnerabilities
- Implement defense in depth
- Ensure no information leakage in error messages
- Use constant-time operations for sensitive comparisons

**Performance Contributions:**
- Profile before optimizing
- Maintain zero-knowledge architecture
- Consider memory usage for large secrets
- Benchmark critical paths

## Testing Strategies

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn test_crypto_roundtrip() {
        let ctx = CryptoContext::new();
        let plaintext = b"test data";
        
        let encrypted = ctx.encrypt(plaintext)?;
        let decrypted = ctx.decrypt(&encrypted)?;
        
        assert_eq!(plaintext, &decrypted[..]);
    }
}
```

### Integration Testing

```rust
#[tokio::test]  
async fn test_client_server_integration() {
    let server = setup_test_server().await;
    let client = setup_test_client(&server.url()).await;
    
    // Test full workflow
    let secret = "integration test secret";
    let url = client.send(secret.into()).await?;
    let retrieved = client.get(&url).await?;
    
    assert_eq!(secret, retrieved.decode()?);
}
```

### Property Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn crypto_roundtrip_property(data: Vec<u8>) {
        let ctx = CryptoContext::new();
        let encrypted = ctx.encrypt(&data).unwrap();
        let decrypted = ctx.decrypt(&encrypted).unwrap();
        prop_assert_eq!(&data, &decrypted);
    }
}
```

## Debugging

### Enable Debug Logging

```bash
# Verbose logging
RUST_LOG=debug cargo run --package hakanai-server

# Specific module logging
RUST_LOG=hakanai_lib::crypto=trace cargo run

# JSON structured logging
RUST_LOG=debug cargo run --package hakanai-server 2>&1 | jq
```

### Common Debug Scenarios

**Encryption Issues:**
```bash
# Test crypto operations
RUST_LOG=hakanai_lib::crypto=debug cargo test crypto
```

**Network Issues:**
```bash
# Test HTTP client
RUST_LOG=reqwest=debug cargo test web_client
```

**TypeScript Issues:**
```bash
# Debug TypeScript build
npm run build -- --verbose

# Browser console debugging
console.log('Debug:', error, stackTrace);
```

## Performance Optimization

### Profiling

```bash
# CPU profiling with perf
cargo build --release
perf record --call-graph=dwarf target/release/hakanai-server
perf report

# Memory profiling with valgrind
cargo build
valgrind --tool=massif target/debug/hakanai-server
```

### Benchmarking

For performance testing, consider using the `criterion` crate for Rust benchmarks. See the Criterion documentation for implementation details.

## Security Development

### Threat Modeling

**Assets:**
- Encryption keys (client-side only)
- Secret data (encrypted at rest)
- Authentication tokens

**Threats:**
- Memory disclosure attacks
- Timing attacks on authentication
- Network interception
- Server compromise

**Mitigations:**
- Zero-knowledge architecture
- Memory zeroization
- Constant-time operations
- Comprehensive input validation

### Security Testing

```rust
#[test]
fn test_constant_time_comparison() {
    let token1 = "correct_token";
    let token2 = "wrong_token";
    
    // Use constant-time comparison
    assert!(!constant_time_eq(token1.as_bytes(), token2.as_bytes()));
}
```

## Release Process

### Version Tagging

```bash
# Update version in Cargo.toml files
# Update CHANGELOG.md
git commit -m "chore: bump version to v2.x.x"
git tag -a v2.x.x -m "Release v2.x.x"
git push origin v2.x.x
```

### Release Checklist

- [ ] All tests pass on multiple platforms
- [ ] Security audit completed
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Version numbers updated
- [ ] Tagged release created
- [ ] Binaries built for all platforms
- [ ] Docker images published
- [ ] Helm chart updated


## Docker Development

For Docker development, use the provided `docker-compose.yml`:

```bash
# Start development environment
docker compose up -d

# Watch logs
docker compose logs -f
```

## Troubleshooting

### Common Issues

**Build fails:**
```bash
# Clean everything and rebuild
cargo clean && make clean-ts
npm install && cargo build
```

**Tests fail:**
```bash
# Ensure Redis is running
redis-cli ping
# Run with debug output
RUST_LOG=debug cargo test -- --nocapture
```

**Server won't start:**
```bash
# Check port and Redis
netstat -tlpn | grep :8080
redis-cli ping
RUST_LOG=debug cargo run --package hakanai-server
```

For deployment and production issues, see [DEPLOYMENT.md](DEPLOYMENT.md).
