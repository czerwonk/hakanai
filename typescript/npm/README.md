# @hakanai/client

Zero-knowledge client library for [Hakanai](https://github.com/czerwonk/hakanai) one-time secret sharing service.

## Features

- 🔐 **Client-side encryption** using AES-256-GCM
- 🚀 **Zero-knowledge** - server never sees unencrypted data
- 📦 **TypeScript** support with full type definitions
- 🌐 **Works in browsers** and Node.js environments
- 🔑 **Secure key generation** and management

## Installation

```bash
npm install @hakanai/client
```

## Usage

### Basic Example

```javascript
import { HakanaiClient } from "@hakanai/client";

// Create a client instance
const client = new HakanaiClient("https://hakanai.link");

// Create and send a text secret
async function shareSecret() {
  const payload = client.createPayload();
  const encoder = new TextEncoder();
  payload.setFromBytes(encoder.encode("My secret message"));

  // Send with 24-hour expiration
  const url = await client.sendPayload(payload, 86400);
  console.log("Secret URL:", url);
}
```

### With Authentication

```javascript
const authToken = "your-auth-token";
const url = await client.sendPayload(payload, 86400, authToken);
```

### Sharing Files

```javascript
// Create payload with filename
const payload = client.createPayload("document.pdf");

// Set file content (as Uint8Array)
payload.setFromBytes(fileBytes);

const url = await client.sendPayload(payload, 86400);
```

### Retrieving Secrets

```javascript
// Parse URL to get ID and key
const secretUrl = "https://hakanai.link/s/ulid#base64key";
const urlParts = new URL(secretUrl);
const id = urlParts.pathname.split("/").pop();
const key = urlParts.hash.substring(1);

// Retrieve and decrypt
const encryptedData = await client.retrieveSecret(id);
const decryptedPayload = await client.decryptPayload(encryptedData, key);

// Get the content
const text = decryptedPayload.decode(); // For text
const bytes = decryptedPayload.decodeBytes(); // For binary
```

## API Reference

### `HakanaiClient`

#### Constructor

```typescript
new HakanaiClient(serverUrl: string)
```

#### Methods

- `createPayload(filename?: string): PayloadData` - Create a new payload
- `sendPayload(payload: PayloadData, ttlSeconds: number, authToken?: string): Promise<string>` - Encrypt and send payload
- `retrieveSecret(id: string): Promise<string>` - Retrieve encrypted secret
- `decryptPayload(encryptedData: string, key: string): Promise<PayloadData>` - Decrypt retrieved data

### `PayloadData`

#### Properties

- `filename?: string` - Optional filename for file uploads
- `data: string` - Base64-encoded content (readonly)

#### Methods

- `setFromBytes(bytes: Uint8Array): void` - Set content from raw bytes
- `decode(): string` - Decode as UTF-8 text
- `decodeBytes(): Uint8Array` - Get raw bytes

## Browser Compatibility

The client uses Web Crypto API and requires:

- Chrome 37+
- Firefox 34+
- Safari 10.1+
- Edge 79+

## Security Notes

- All encryption happens client-side
- Server never receives encryption keys
- Uses AES-256-GCM for authenticated encryption
- Secrets are automatically deleted after first access

## License

Apache-2.0

## Links

- [GitHub Repository](https://github.com/czerwonk/hakanai)
- [Report Issues](https://github.com/czerwonk/hakanai/issues)
