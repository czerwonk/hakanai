/**
 * Hakanai JavaScript Client
 *
 * This client implements the same cryptographic protocol as the Rust CLI client,
 * allowing you to send and receive encrypted secrets via the Hakanai API.
 */

class HakanaiClient {
  constructor(baseUrl) {
    this.baseUrl = baseUrl.replace(/\/$/, ""); // Remove trailing slash
    this.authToken = null;
  }

  /**
   * Set the authentication token for creating secrets
   * @param {string} token - Bearer token for authentication
   */
  setAuthToken(token) {
    this.authToken = token;
  }

  /**
   * Generate a cryptographically secure random key
   * @returns {Uint8Array} 32-byte key for AES-256
   */
  async generateKey() {
    return crypto.getRandomValues(new Uint8Array(32));
  }

  /**
   * Generate a cryptographically secure random nonce
   * @returns {Uint8Array} 12-byte nonce for AES-GCM
   */
  generateNonce() {
    return crypto.getRandomValues(new Uint8Array(12));
  }

  /**
   * Import a raw key for use with WebCrypto API
   * @param {Uint8Array} rawKey - The raw key bytes
   * @returns {Promise<CryptoKey>} The imported key
   */
  async importKey(rawKey) {
    return crypto.subtle.importKey(
      "raw",
      rawKey,
      { name: "AES-GCM", length: 256 },
      false,
      ["encrypt", "decrypt"],
    );
  }

  /**
   * Encrypt a plaintext message
   * @param {string} plaintext - The message to encrypt
   * @param {Uint8Array} key - The encryption key
   * @returns {Promise<string>} Base64-encoded encrypted data (nonce + ciphertext)
   */
  async encrypt(plaintext, key) {
    const nonce = this.generateNonce();
    const cryptoKey = await this.importKey(key);

    const encoder = new TextEncoder();
    const plaintextBytes = encoder.encode(plaintext);

    const ciphertext = await crypto.subtle.encrypt(
      { name: "AES-GCM", iv: nonce },
      cryptoKey,
      plaintextBytes,
    );

    // Combine nonce and ciphertext
    const combined = new Uint8Array(nonce.length + ciphertext.byteLength);
    combined.set(nonce, 0);
    combined.set(new Uint8Array(ciphertext), nonce.length);

    // Encode using standard base64
    return btoa(String.fromCharCode(...combined));
  }

  /**
   * Decrypt an encrypted message
   * @param {string} encryptedData - Base64-encoded encrypted data (nonce + ciphertext)
   * @param {Uint8Array} key - The decryption key
   * @returns {Promise<string>} The decrypted plaintext
   */
  async decrypt(encryptedData, key) {
    // Decode from standard base64
    const combined = Uint8Array.from(atob(encryptedData), (c) =>
      c.charCodeAt(0),
    );

    if (combined.length < 12) {
      throw new Error("Invalid encrypted data: too short");
    }

    // Extract nonce and ciphertext
    const nonce = combined.slice(0, 12);
    const ciphertext = combined.slice(12);

    const cryptoKey = await this.importKey(key);

    try {
      const plaintextBytes = await crypto.subtle.decrypt(
        { name: "AES-GCM", iv: nonce },
        cryptoKey,
        ciphertext,
      );

      const decoder = new TextDecoder();
      return decoder.decode(plaintextBytes);
    } catch (error) {
      throw new Error("Decryption failed: invalid key or corrupted data");
    }
  }

  /**
   * Send a secret to the server
   * @param {string} plaintext - The secret to send
   * @param {number} expiresIn - Expiration time in seconds (default: 3600)
   * @returns {Promise<{url: string, key: Uint8Array}>} The secret URL and encryption key
   */
  async sendSecret(plaintext, expiresIn = 3600) {
    // Generate a new key for this secret
    const key = await this.generateKey();

    // Encrypt the plaintext
    const encryptedData = await this.encrypt(plaintext, key);

    // Prepare the request
    const headers = {
      "Content-Type": "application/json",
    };

    if (this.authToken) {
      headers["Authorization"] = `Bearer ${this.authToken}`;
    }

    const response = await fetch(`${this.baseUrl}/api/secret`, {
      method: "POST",
      headers,
      body: JSON.stringify({
        data: encryptedData,
        expires_in: {
          secs: expiresIn,
          nanos: 0,
        },
      }),
    });

    if (!response.ok) {
      throw new Error(
        `Failed to create secret: ${response.status} ${response.statusText}`,
      );
    }

    const result = await response.json();

    // Encode key using URL-safe base64
    const keyBase64 = btoa(String.fromCharCode(...key))
      .replace(/\+/g, "-")
      .replace(/\//g, "_")
      .replace(/=+$/, "");

    const url = `${this.baseUrl}/api/secret/${result.id}#${keyBase64}`;

    return { url, key };
  }

  /**
   * Receive a secret from the server
   * @param {string} url - The full secret URL including the key fragment
   * @returns {Promise<string>} The decrypted secret
   */
  async receiveSecret(url) {
    // Parse the URL
    const urlObj = new URL(url);
    const secretId = urlObj.pathname.split("/").pop();
    const keyBase64 = urlObj.hash.slice(1); // Remove the #

    if (!keyBase64) {
      throw new Error("No decryption key found in URL");
    }

    // Decode key from URL-safe base64
    const keyBase64Standard = keyBase64
      .replace(/-/g, "+")
      .replace(/_/g, "/")
      .padEnd(keyBase64.length + ((4 - (keyBase64.length % 4)) % 4), "=");

    const key = Uint8Array.from(atob(keyBase64Standard), (c) =>
      c.charCodeAt(0),
    );

    if (key.length !== 32) {
      throw new Error("Invalid key length");
    }

    // Fetch the encrypted data
    const response = await fetch(`${this.baseUrl}/api/secret/${secretId}`);

    if (!response.ok) {
      if (response.status === 404) {
        throw new Error("Secret not found or already accessed");
      }
      throw new Error(
        `Failed to retrieve secret: ${response.status} ${response.statusText}`,
      );
    }

    const encryptedData = await response.text();

    // Decrypt and return
    return this.decrypt(encryptedData, key);
  }
}

// Example usage for Node.js environment
if (typeof module !== "undefined" && module.exports) {
  module.exports = HakanaiClient;
}

// Example usage:
/*
// Browser or Node.js with fetch available
const client = new HakanaiClient('https://hakanai.example.com');

// Optional: Set auth token for creating secrets
client.setAuthToken('your-auth-token');

// Send a secret
const secret = 'This is my secret message';
const { url, key } = await client.sendSecret(secret, 3600);
console.log('Secret URL:', url);

// Receive a secret
const retrievedSecret = await client.receiveSecret(url);
console.log('Retrieved secret:', retrievedSecret);
*/

