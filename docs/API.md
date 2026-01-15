# API Reference

Hakanai provides a RESTful API for programmatic access to secret sharing functionality.

## Interactive Documentation

**📚 For interactive API documentation with examples, visit `/docs` on your running server.**

The interactive docs are automatically generated from the OpenAPI specification and are always up-to-date with the current API implementation.

## Base URL

All API endpoints are relative to your server's base URL:

```
https://your-hakanai-server.com/api/v1
```

## Authentication

Authentication is optional but recommended for production deployments.

### Bearer Token Authentication

```http
Authorization: Bearer {your-token}
```

### Token Sources

- **Admin tokens**: Created with `--enable-admin-token` server flag
- **User tokens**: Created via admin API or auto-generated default token
- **Anonymous access**: Available when `--allow-anonymous` is enabled

## Endpoints

### POST /api/v1/secret - Create Secret

Create a new secret with optional access restrictions.

#### Request

```http
POST /api/v1/secret
Content-Type: application/json
Authorization: Bearer {token}  # Optional if anonymous access enabled

{
  "data": "base64-encoded-secret-data",
  "expires_in": 3600,  // seconds (optional, default: 86400)
  "restrictions": {    // optional
    "allowed_ips": ["192.168.1.0/24", "10.0.0.1", "2001:db8::/32"],
    "allowed_countries": ["US", "DE", "CA"],
    "allowed_asns": [13335, 15169, 202739],
    "passphrase_hash": "5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8"
  }
}
```

#### Fields

- **data** (string, required): Base64-encoded secret data
- **expires_in** (integer, optional): TTL in seconds (default: 86400, max: server configured)
- **restrictions** (object, optional): Access control restrictions
  - **allowed_ips** (array[string], optional): IP addresses and CIDR ranges
  - **allowed_countries** (array[string], optional): ISO 3166-1 alpha-2 country codes
  - **allowed_asns** (array[integer], optional): Autonomous System Numbers
  - **passphrase_hash** (string, optional): SHA-256 hash of required passphrase

#### Response

**Success (201 Created):**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Error Responses:**

- **400 Bad Request**: Invalid request body or malformed data
- **401 Unauthorized**: Invalid or missing token (when authentication required)
- **413 Payload Too Large**: Secret data exceeds size limits
- **422 Unprocessable Entity**: Invalid restrictions format

#### Example Usage

```bash
# Create simple secret
curl -X POST https://hakanai.example.com/api/v1/secret \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-token" \
  -d '{
    "data": "bXkgc2VjcmV0IGRhdGE=",
    "expires_in": 1800
  }'

# Create secret with IP restrictions
curl -X POST https://hakanai.example.com/api/v1/secret \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-token" \
  -d '{
    "data": "cmVzdHJpY3RlZCBzZWNyZXQ=",
    "expires_in": 3600,
    "restrictions": {
      "allowed_ips": ["192.168.1.0/24", "10.0.0.1"],
      "allowed_countries": ["US", "DE"]
    }
  }'

# Create passphrase-protected secret
curl -X POST https://hakanai.example.com/api/v1/secret \
  -H "Content-Type: application/json" \
  -d '{
    "data": "cGFzc3dvcmQgcHJvdGVjdGVk",
    "restrictions": {
      "passphrase_hash": "5994471abb01112afcc18159f6cc74b4f511b99806da59b3caf5a9c173cacfc5"
    }
  }'
```

### GET /api/v1/secret/{id} - Retrieve Secret

Retrieve a secret by its ID. **One-time access only** - the secret is permanently deleted after retrieval.

#### Request

```http
GET /api/v1/secret/550e8400-e29b-41d4-a716-446655440000
X-Secret-Passphrase: sha256-hash-of-passphrase  # Required for passphrase-protected secrets
```

#### Response

**Success (200 OK):**

```
Content-Type: text/plain

decrypted secret data here
```

**Error Responses:**

- **401 Unauthorized**: Missing or incorrect passphrase
- **403 Forbidden**: Access denied due to IP/country/ASN restrictions
- **404 Not Found**: Secret doesn't exist or has expired
- **410 Gone**: Secret was already accessed by someone else
- **501 Not Implemented**: Geo-restrictions used but server not configured

#### Example Usage

```bash
# Simple retrieval
curl https://hakanai.example.com/api/v1/secret/550e8400-e29b-41d4-a716-446655440000

# With passphrase
curl https://hakanai.example.com/api/v1/secret/550e8400-e29b-41d4-a716-446655440000 \
  -H "X-Secret-Passphrase: 5994471abb01112afcc18159f6cc74b4f511b99806da59b3caf5a9c173cacfc5"

# Save to file
curl https://hakanai.example.com/api/v1/secret/550e8400-e29b-41d4-a716-446655440000 \
  -o secret.txt
```

### POST /api/v1/admin/tokens - Create User Token (Admin Only)

Create user authentication tokens. Requires admin authentication and trusted IP access.

#### Request

```http
POST /api/v1/admin/tokens
Content-Type: application/json
Authorization: Bearer {admin-token}

{
  "upload_size_limit": 5242880,  // bytes (optional)
  "ttl_seconds": 2592000         // seconds (optional, default: 30 days)
}
```

#### Response

**Success (201 Created):**

```json
{
  "token": "generated-user-token-string"
}
```

**Error Responses:**

- **401 Unauthorized**: Invalid or missing admin token
- **403 Forbidden**: Request not from trusted IP range
- **400 Bad Request**: Invalid request body

#### Example Usage

```bash
# Create token with custom limits
curl -X POST https://hakanai.example.com/api/v1/admin/tokens \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer admin-token" \
  -d '{
    "upload_size_limit": 1048576,
    "ttl_seconds": 604800
  }'
```

## Health Endpoints

### GET /ready - Readiness Check

Returns 200 OK when the server is ready to accept requests.

```bash
curl https://hakanai.example.com/ready
```

### GET /healthy - Health Check

Returns 200 OK when the server and all dependencies (Redis) are healthy.

```bash
curl https://hakanai.example.com/healthy
```

## Short Link Endpoints

### GET /s/{id} - Short Link Access

Alternative endpoint for secret access with dual behavior:

- **CLI/API clients**: Returns raw secret data
- **Web browsers**: Returns HTML interface for decryption

```bash
# CLI access (returns raw data)
curl https://hakanai.example.com/s/550e8400-e29b-41d4-a716-446655440000

# Browser access (returns HTML interface)
# Visit: https://hakanai.example.com/s/550e8400-e29b-41d4-a716-446655440000
```

## Error Handling

All endpoints return consistent error responses:

```json
{
  "error": "Human readable error message",
  "details": "Additional context when available"
}
```

### HTTP Status Codes

- **200 OK**: Successful secret retrieval
- **201 Created**: Successful secret creation or token creation
- **400 Bad Request**: Invalid request format or parameters
- **401 Unauthorized**: Missing or invalid authentication
- **403 Forbidden**: Access denied (IP/country/ASN restrictions)
- **404 Not Found**: Secret not found or expired
- **410 Gone**: Secret already accessed (one-time use)
- **413 Payload Too Large**: Secret exceeds size limits
- **422 Unprocessable Entity**: Invalid data format
- **501 Not Implemented**: Feature requires server configuration

## Data Encoding

### Secret Data

Secret data is serialized using MessagePack before encryption. The payload structure contains the raw bytes and an optional filename:

```python
import msgpack

# Encode text secret
secret_text = "my secret message"
payload = [secret_text.encode('utf-8'), None]  # [data, filename]
encoded = msgpack.packb(payload)

# Encode binary file with filename
with open('document.pdf', 'rb') as f:
    binary_data = f.read()
    payload = [binary_data, "document.pdf"]  # [data, filename]
    encoded = msgpack.packb(payload)
```

```javascript
// Using msgpack-lite in JavaScript
import msgpack from "msgpack-lite";

// Encode text secret
const secretText = "my secret message";
const payload = [new TextEncoder().encode(secretText), null];
const encoded = msgpack.encode(payload);

// Encode binary file with filename
const payload = [fileBytes, "document.pdf"];
const encoded = msgpack.encode(payload);
```

The MessagePack-encoded payload is then encrypted with AES-256-GCM before being base64-encoded for HTTP transport.

### Passphrase Hashing

Passphrases must be SHA-256 hashed before sending:

```python
import hashlib

passphrase = "my secret passphrase"
hash_object = hashlib.sha256(passphrase.encode('utf-8'))
passphrase_hash = hash_object.hexdigest()
```

```bash
# Using command line
echo -n "my secret passphrase" | sha256sum | cut -d' ' -f1
```

## Access Restrictions

### IP Address Formats

- **IPv4**: `192.168.1.100`
- **IPv4 CIDR**: `192.168.1.0/24`
- **IPv6**: `2001:db8::1`
- **IPv6 CIDR**: `2001:db8::/32`

### Country Codes

Use ISO 3166-1 alpha-2 country codes (uppercase):

- `US` (United States)
- `DE` (Germany)
- `CA` (Canada)
- `GB` (United Kingdom)
- `FR` (France)
- `JP` (Japan)

### ASN Numbers

Autonomous System Numbers as 32-bit unsigned integers:

- `13335` (Cloudflare)
- `15169` (Google)
- `16509` (Amazon)
- `32934` (Facebook)

## Rate Limiting

Rate limiting should be implemented at the reverse proxy level (nginx, Caddy, etc.). The application does not enforce rate limits directly.

## CORS

Cross-Origin Resource Sharing (CORS) is restrictive by default. Configure allowed origins with:

```bash
# Server flag
--cors-allowed-origins "https://trusted-domain.com,https://another-domain.com"

# Environment variable
export HAKANAI_CORS_ALLOWED_ORIGINS="https://trusted-domain.com,https://another-domain.com"
```

## Client Implementation Examples

### Key Concepts for Client Libraries

When implementing a Hakanai client in any language, ensure you:

1. **Base64 encode** all secret data before sending
2. **SHA-256 hash** passphrases before sending (never send plaintext)
3. **Include Bearer token** in Authorization header if authenticated
4. **Handle HTTP status codes** appropriately (401, 403, 404, 410)

### Quick Python Example

```python
import requests
import base64
import hashlib

# Create secret
def create_secret(base_url, data, token=None):
    headers = {'Content-Type': 'application/json'}
    if token:
        headers['Authorization'] = f'Bearer {token}'

    payload = {
        'data': base64.b64encode(data.encode()).decode(),
        'expires_in': 3600
    }

    response = requests.post(f'{base_url}/api/v1/secret',
                            json=payload, headers=headers)
    return response.json()['id']

# Retrieve secret
def get_secret(base_url, secret_id, passphrase=None):
    headers = {}
    if passphrase:
        headers['X-Secret-Passphrase'] = hashlib.sha256(
            passphrase.encode()).hexdigest()

    response = requests.get(f'{base_url}/api/v1/secret/{secret_id}',
                           headers=headers)
    return response.text
```

### Quick JavaScript Example

```javascript
// Create secret
async function createSecret(baseUrl, data, token) {
  const response = await fetch(`${baseUrl}/api/v1/secret`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: token ? `Bearer ${token}` : undefined,
    },
    body: JSON.stringify({
      data: btoa(data),
      expires_in: 3600,
    }),
  });
  return (await response.json()).id;
}

// Retrieve secret
async function getSecret(baseUrl, secretId, passphrase) {
  const headers = {};
  if (passphrase) {
    const encoder = new TextEncoder();
    const data = encoder.encode(passphrase);
    const hashBuffer = await crypto.subtle.digest("SHA-256", data);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    headers["X-Secret-Passphrase"] = hashArray.map((b) => b.toString(16).padStart(2, "0")).join("");
  }

  const response = await fetch(`${baseUrl}/api/v1/secret/${secretId}`, { headers });
  return await response.text();
}
```

## Security Best Practices

1. **Always use HTTPS** in production
2. **Implement proper authentication** with tokens
3. **Apply IP restrictions** for sensitive secrets
4. **Use short TTLs** for temporary data
5. **Hash passphrases** before sending
6. **Validate all inputs** on the client side
7. **Handle errors gracefully** without exposing internal details
8. **Log API access** for audit trails

## Troubleshooting

### Common Issues

**401 Unauthorized:**

- Check token is valid and not expired
- Ensure `Authorization: Bearer {token}` header format
- Verify server has authentication enabled

**403 Forbidden:**

- Check IP address against allowed ranges
- Verify country/ASN restrictions match client location
- Ensure admin API calls come from trusted IP ranges

**413 Payload Too Large:**

- Check secret size against server limits
- Account for base64 encoding overhead (~33% increase)
- Consider splitting large files

**501 Not Implemented:**

- Geo-restrictions require server configuration
- Set `--country-header` or `--asn-header` flags
- Configure reverse proxy to provide location headers

For more troubleshooting, see [DEPLOYMENT.md](DEPLOYMENT.md).
