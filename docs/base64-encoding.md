# Base64 Encoding in Hakanai

This document describes the Base64 encoding schemes used throughout the Hakanai project and the rationale behind each choice.

## Overview

Hakanai uses two different Base64 encoding schemes for different purposes:

1. **Standard Base64** - For encrypted payloads and file content
2. **URL-safe Base64 without padding** - For encryption keys in URLs

## Encoding Schemes

### Standard Base64

**Purpose**: Used for encoding encrypted data and binary file content before transmission.

**Characteristics**:
- Uses the standard Base64 alphabet: `A-Z`, `a-z`, `0-9`, `+`, `/`
- Includes padding with `=` characters to ensure the output length is a multiple of 4
- Not safe for direct inclusion in URLs due to `+`, `/`, and `=` characters

**Usage in Hakanai**:
- Encrypted payload data (nonce + ciphertext)
- Binary file content before encryption
- Text content in the Payload structure

### URL-safe Base64 without Padding

**Purpose**: Used exclusively for encoding AES-256 encryption keys that are included in URL fragments.

**Characteristics**:
- Uses URL-safe alphabet: `A-Z`, `a-z`, `0-9`, `-`, `_`
- Character replacements:
  - `+` → `-`
  - `/` → `_`
- No padding characters (`=`)
- Can be safely included in URLs without requiring percent-encoding

**Usage in Hakanai**:
- 32-byte AES-256 encryption keys in URL fragments (e.g., `#key=...`)

## Implementation Details

### Rust Implementation

Located in `lib/src/crypto.rs`:

```rust
// For encrypted data
use base64::prelude::BASE64_STANDARD;
let encoded = BASE64_STANDARD.encode(&payload);

// For URL keys
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
let key_url = BASE64_URL_SAFE_NO_PAD.encode(key);
```

### JavaScript Implementation

Located in `server/src/includes/hakanai-client.js`:

```javascript
// Standard Base64
const encoded = btoa(data);
const decoded = atob(encoded);

// Convert standard to URL-safe without padding
const urlSafeKey = btoa(String.fromCharCode(...key))
  .replace(/\+/g, "-")
  .replace(/\//g, "_")
  .replace(/=/g, "");

// Convert URL-safe back to standard with padding
const standardKey = keyBase64
  .replace(/-/g, "+")
  .replace(/_/g, "/")
  .padEnd(keyBase64.length + ((4 - (keyBase64.length % 4)) % 4), "=");
```

## Data Flow

1. **Secret Creation**:
   - User provides secret (text or binary file)
   - Binary files are encoded with standard Base64
   - Data is encrypted with AES-256-GCM
   - Encrypted payload (nonce + ciphertext) is encoded with standard Base64
   - Encryption key is encoded with URL-safe Base64 without padding
   - Server receives and stores the Base64-encoded encrypted payload
   - Client receives URL with key in fragment: `https://example.com/s/{id}#key={url-safe-base64-key}`

2. **Secret Retrieval**:
   - Client extracts URL-safe Base64 key from URL fragment
   - Key is decoded from URL-safe Base64
   - Encrypted payload is retrieved from server (standard Base64)
   - Payload is decoded from standard Base64
   - Data is decrypted using the key
   - If file, the decrypted content is decoded from standard Base64

## Why Two Encoding Schemes?

### URL-safe for Keys

The encryption key must be transmitted via URL fragment to maintain zero-knowledge architecture (fragments are not sent to the server). Using URL-safe encoding ensures:

1. **No URL encoding needed**: Keys can be directly included in URLs
2. **Cleaner URLs**: No percent-encoded characters like `%2F` or `%3D`
3. **Better compatibility**: Some systems have issues with certain characters in URLs
4. **No padding**: Reduces URL length (padding is redundant for 32-byte keys)

### Standard Base64 for Data

Encrypted payloads use standard Base64 because:

1. **Universal support**: `btoa()`/`atob()` in browsers, standard libraries everywhere
2. **No URL constraints**: Data is sent in request/response bodies, not URLs
3. **Existing ecosystem**: Most tools and libraries expect standard Base64
4. **Debugging**: Easier to inspect and debug with common tools

## Security Considerations

- The choice of encoding does not affect security - it's purely for transport
- Both encoding schemes are fully reversible and provide no encryption
- The actual security comes from AES-256-GCM encryption, not the encoding
- Keys in URL fragments are never sent to the server (browser behavior)

## Testing

Both encoding schemes are tested in:
- `lib/src/crypto.rs` - Rust unit tests
- `cli/src/main.rs` - CLI integration tests
- Manual testing with various file types and sizes

## Compatibility

The dual encoding approach ensures compatibility:
- Works with all modern browsers
- No issues with URL length limits (keys are always 43 characters when URL-safe encoded)
- Binary files of any type can be shared
- Unicode text is properly handled through the encoding chain