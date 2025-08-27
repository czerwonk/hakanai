# CLI Documentation

The Hakanai command-line interface provides full access to secret sharing functionality from the terminal.

## Installation

See [INSTALLATION.md](INSTALLATION.md) for installation instructions.

## Global Options

All commands support these global options:

- `-s, --server`: Hakanai server URL (default: http://localhost:8080)
- `--token-file`: File containing authorization token
- Environment variable `HAKANAI_SERVER`: Sets default server URL
- Environment variable `HAKANAI_TOKEN`: Sets authentication token

## Commands

### `hakanai send` - Send a Secret

Create and share a new secret.

#### Basic Usage

```bash
# Send from stdin (default: 24 hour expiration)
echo "my secret data" | hakanai send

# Send from a file
hakanai send --file secret.txt
hakanai send -f /path/to/secret.dat

# Send multiple files (automatically creates ZIP archive)
hakanai send -f document.pdf -f image.png -f data.csv
hakanai send --file report.pdf --file appendix.docx
```

#### Advanced Options

```bash
# Custom TTL
echo "temporary password" | hakanai send --ttl 30m

# Custom server
echo "secret" | hakanai send --server https://hakanai.example.com

# With authentication token
echo "secret" | HAKANAI_TOKEN=my-auth-token hakanai send
echo "secret" | hakanai send --token-file /path/to/token.txt

# Enhanced security (separate key)
echo "sensitive data" | hakanai send --separate-key

# Display URL as QR code
echo "secret" | hakanai send --qr-code

# Force file upload behavior
echo "data" | hakanai send --as-file --filename "custom.txt"
```

#### Access Restrictions

```bash
# IP/CIDR restrictions
echo "restricted secret" | hakanai send --allow-ip 192.168.1.0/24 --allow-ip 10.0.0.1
echo "office only" | hakanai send --allow-ip 203.0.113.0/24 --allow-ip 2001:db8:85a3::/48

# Country restrictions (ISO 3166-1 alpha-2 codes)
echo "US/Canada only" | hakanai send --allow-country US --allow-country CA
echo "EU restricted" | hakanai send --allow-country DE --allow-country FR --allow-country NL

# ASN restrictions (Autonomous System Numbers)
echo "Cloudflare only" | hakanai send --allow-asn 13335
echo "Multiple ISPs" | hakanai send --allow-asn 13335 --allow-asn 15169 --allow-asn 32934

# Passphrase protection
echo "sensitive document" | hakanai send --require-passphrase mypassword123

# Combine all restriction types
echo "comprehensive restrictions" | hakanai send \
  --allow-ip 192.168.1.0/24 \
  --allow-country DE \
  --allow-asn 202739 \
  --require-passphrase secret123
```

#### Send Command Options

- `-f, --file`: File to read the secret from (can be specified multiple times)
- `--ttl`: Time after the secret vanishes (default: 24h, supports humanized format like 30m, 1h, 7d)
- `-a, --as-file`: Send the secret as a file (auto-detected for binary content)
- `--filename`: Custom filename when sending as a file
- `--separate-key`: Print key separately for enhanced security
- `--allow-ip`: IP addresses/CIDR ranges allowed to access (can be specified multiple times)
- `--allow-country`: Country codes allowed to access (can be specified multiple times)
- `--allow-asn`: ASN numbers allowed to access (can be specified multiple times)
- `--require-passphrase`: Require passphrase for access
- `-q, --qr-code`: Display URL as QR code

### `hakanai get` - Retrieve a Secret

Retrieve and display a secret.

#### Basic Usage

```bash
# Get using full URL
hakanai get https://hakanai.example.com/secret/550e8400-e29b-41d4-a716-446655440000

# Get using short link format  
hakanai get https://hakanai.example.com/s/550e8400-e29b-41d4-a716-446655440000

# Get with separate key (when --separate-key was used)
hakanai get https://hakanai.example.com/s/550e8400 --key base64-encoded-key

# Get passphrase-protected secret
hakanai get https://hakanai.example.com/s/550e8400 --passphrase mypassword123
hakanai get https://hakanai.example.com/s/550e8400 -p secret123
```

#### File Handling

```bash
# Save to specific file (overrides payload filename)
hakanai get https://hakanai.example.com/s/550e8400 --filename custom-name.txt

# Save to custom directory
hakanai get https://hakanai.example.com/s/550e8400 --output-dir /path/to/downloads/

# Extract ZIP archives automatically
hakanai get https://hakanai.example.com/s/550e8400 --extract --output-dir /path/to/extract/

# Output to stdout (useful for piping)
hakanai get https://hakanai.example.com/s/550e8400 --to-stdout
```

#### Get Command Options

- `-k, --key`: Base64 encoded secret key (when not in URL fragment)
- `-p, --passphrase`: Passphrase for protected secrets
- `--to-stdout`: Output secret to stdout
- `-f, --filename`: Save to specific file (overrides payload filename)
- `-e, --extract`: Extract ZIP archives
- `-o, --output-dir`: Save files to this directory

### `hakanai token` - Create User Tokens (Admin Only)

Create user authentication tokens. Requires admin privileges.

#### Usage

```bash
# Create token with admin privileges (prompts for admin token)
hakanai token --limit 5m --ttl 7d

# Create token with humanized size limits
hakanai token --limit 500k    # 500 KB
hakanai token --limit 1m      # 1 MB
hakanai token --limit 1024    # 1024 bytes

# Create token with custom server
hakanai token --server https://hakanai.example.com --limit 2m --ttl 30d
```

#### Token Command Options

- `-l, --limit`: Upload size limit for the token (humanized format supported)
- `--ttl`: Token expiration time (default: 30d, humanized format supported)
- `-s, --server`: Hakanai server URL (default: http://localhost:8080)

## Size Format Options

All commands that accept size values support humanized formats:

- Plain numbers: bytes (e.g., `1024`)
- 'k' suffix: kilobytes (e.g., `500k`)
- 'm' suffix: megabytes (e.g., `1m`)
- Decimal values supported (e.g., `1.5m`, `2.5k`)

## TTL Format Options

Commands that accept TTL values support humanized formats:

- `s`: seconds (e.g., `30s`)
- `m`: minutes (e.g., `30m`)
- `h`: hours (e.g., `1h`, `24h`)
- `d`: days (e.g., `7d`)

## Environment Variables

Configure default behavior with environment variables:

- `HAKANAI_SERVER`: Default server URL for CLI commands
- `HAKANAI_TOKEN`: Authentication token for CLI operations
- `HAKANAI_TTL`: Default TTL for send command
- `HAKANAI_TOKEN_TTL`: Default TTL for token command
- `HAKANAI_QR_CODE`: Enable QR code output by default
- `HAKANAI_TO_STDOUT`: Output secrets to stdout by default
- `HAKANAI_OUTPUT_DIR`: Default output directory for retrieved secrets

## Examples

### Simple Secret Sharing

```bash
# Send a password
echo "mypassword123" | hakanai send

# Send a file
hakanai send -f ~/Documents/contract.pdf

# Quick retrieval to stdout
hakanai get https://hakanai.example.com/s/uuid --to-stdout
```

### Enterprise/Team Usage

```bash
# Send with company restrictions
echo "API key" | hakanai send \
  --allow-ip 10.0.0.0/8 \
  --allow-country US \
  --ttl 1h \
  --require-passphrase company2024

# Send multiple files with restrictions
hakanai send \
  -f report.pdf \
  -f spreadsheet.xlsx \
  -f presentation.pptx \
  --allow-ip 192.168.1.0/24 \
  --ttl 3d
```

### Mobile/QR Code Workflow

```bash
# Generate QR code for mobile sharing
echo "mobile secret" | hakanai send --qr-code --ttl 30m

# Short-lived secret with QR code
echo "meeting password" | hakanai send --qr-code --ttl 5m
```

### Automation/Scripting

```bash
#!/bin/bash
# Backup script with secret sharing
tar -czf backup.tar.gz /important/data
URL=$(hakanai send -f backup.tar.gz --ttl 7d --to-stdout)
echo "Backup available at: $URL"

# API key distribution
echo "$API_KEY" | hakanai send --allow-ip 10.0.0.0/8 --ttl 1d
```

## Error Handling

The CLI returns appropriate exit codes:

- `0`: Success
- `1`: General error (network, parsing, etc.)
- `2`: Authentication error
- `3`: Not found or expired
- `4`: Access denied (restrictions)

## Security Best Practices

1. **Use separate keys** (`--separate-key`) for sensitive data
2. **Apply IP restrictions** for internal secrets
3. **Set short TTLs** for temporary passwords
4. **Use passphrases** for additional security
5. **Store tokens securely** using token files instead of environment variables
6. **Verify URLs** before clicking or retrieving secrets

See [DEVELOPMENT.md](DEVELOPMENT.md) for API integration and development guidance.