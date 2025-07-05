/**
 * Hakanai JavaScript Client
 *
 * This client implements the same cryptographic protocol as the Rust CLI client,
 * allowing you to send and receive encrypted secrets via the Hakanai API.
 */

class HakanaiClient {
  constructor(baseUrl) {
    this.baseUrl = baseUrl.replace(/\/$/, ""); // Remove trailing slash
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
   * Generate a random 256-bit AES key
   * @returns {Promise<Uint8Array>} The generated key
   */
  async generateKey() {
    const key = new Uint8Array(32);
    crypto.getRandomValues(key);
    return key;
  }

  /**
   * Encrypt a message with AES-GCM
   * @param {string} plaintext - The message to encrypt
   * @param {Uint8Array} key - The encryption key
   * @returns {Promise<string>} Base64-encoded encrypted data (nonce + ciphertext)
   */
  async encrypt(plaintext, key) {
    const encoder = new TextEncoder();
    const plaintextBytes = encoder.encode(plaintext);

    // Generate random nonce
    const nonce = new Uint8Array(12);
    crypto.getRandomValues(nonce);

    const cryptoKey = await this.importKey(key);

    const ciphertext = await crypto.subtle.encrypt(
      { name: "AES-GCM", iv: nonce },
      cryptoKey,
      plaintextBytes,
    );

    // Combine nonce and ciphertext
    const combined = new Uint8Array(nonce.length + ciphertext.byteLength);
    combined.set(nonce);
    combined.set(new Uint8Array(ciphertext), nonce.length);

    // Encode to standard base64
    return btoa(String.fromCharCode(...combined));
  }

  /**
   * Send a payload to the server
   * @param {Object} payload - The payload object with data and optional filename
   * @param {number} ttl - Time to live in seconds (optional)
   * @param {string} authToken - Authentication token (optional)
   * @returns {Promise<string>} The shareable URL with the key fragment
   */
  async sendPayload(payload, ttl = 3600, authToken = null) {
    if (!payload || !payload.data || payload.data.trim().length === 0) {
      throw new Error("Payload data cannot be empty");
    }

    // Generate encryption key
    const key = await this.generateKey();

    // Serialize payload to JSON and encrypt
    const payloadJson = JSON.stringify(payload);
    const encryptedData = await this.encrypt(payloadJson, key);

    // Prepare headers
    const headers = {
      "Content-Type": "application/json",
    };

    // Add authorization header if token is provided
    if (authToken && authToken.length > 0) {
      headers["Authorization"] = `Bearer ${authToken}`;
    }

    // Send to server
    const response = await fetch(`${this.baseUrl}/api/v1/secret`, {
      method: "POST",
      headers: headers,
      body: JSON.stringify({
        data: encryptedData,
        expires_in: ttl,
      }),
    });

    if (!response.ok) {
      throw new Error(
        `Failed to send secret: ${response.status} ${response.statusText}`,
      );
    }

    const result = await response.json();
    const secretId = result.id;

    // Encode key to URL-safe base64
    const keyBase64 = btoa(String.fromCharCode(...key))
      .replace(/\+/g, "-")
      .replace(/\//g, "_")
      .replace(/=/g, "");

    // Return the shareable URL
    return `${this.baseUrl}/s/${secretId}#${keyBase64}`;
  }

  /**
   * Receive a payload from the server
   * @param {string} url - The full secret URL including the key fragment
   * @returns {Promise<Object>} The decrypted payload object
   */
  async receivePayload(url) {
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
    const response = await fetch(`${this.baseUrl}/api/v1/secret/${secretId}`);

    if (!response.ok) {
      if (response.status === 404) {
        throw new Error("Secret not found or already accessed");
      }
      throw new Error(
        `Failed to retrieve secret: ${response.status} ${response.statusText}`,
      );
    }

    const encryptedData = await response.text();

    // Decrypt and parse JSON
    const payloadJson = await this.decrypt(encryptedData, key);
    return JSON.parse(payloadJson);
  }

  /**
   * Send a text secret to the server
   * @param {string} secret - The secret text to encrypt and send
   * @param {number} ttl - Time to live in seconds (optional)
   * @param {string} authToken - Authentication token (optional)
   * @returns {Promise<string>} The shareable URL with the key fragment
   */
  async sendSecret(secret, ttl = 3600, authToken = null) {
    if (!secret || secret.trim().length === 0) {
      throw new Error("Secret cannot be empty");
    }

    // Base64 encode the text
    const base64Data = btoa(unescape(encodeURIComponent(secret)));
    const payload = { data: base64Data };
    return this.sendPayload(payload, ttl, authToken);
  }

  /**
   * Receive a secret from the server and return just the decoded text
   * @param {string} url - The full secret URL including the key fragment
   * @returns {Promise<string>} The decrypted secret text
   */
  async receiveSecret(url) {
    const payload = await this.receivePayload(url);
    // Base64 decode the data
    return decodeURIComponent(escape(atob(payload.data)));
  }

  /**
   * Send a file to the server
   * @param {string} base64Data - The base64-encoded file data
   * @param {string} filename - The filename
   * @param {number} ttl - Time to live in seconds (optional)
   * @param {string} authToken - Authentication token (optional)
   * @returns {Promise<string>} The shareable URL with the key fragment
   */
  async sendFile(base64Data, filename, ttl = 3600, authToken = null) {
    if (!base64Data || base64Data.trim().length === 0) {
      throw new Error("File data cannot be empty");
    }
    if (!filename || filename.trim().length === 0) {
      throw new Error("Filename cannot be empty");
    }

    const payload = { data: base64Data, filename };
    return this.sendPayload(payload, ttl, authToken);
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

// Send a text secret
const url = await client.sendSecret('My secret message', 3600);
console.log('Secret URL:', url);

// Send a payload directly
const payload = { data: 'My secret message' };
const payloadUrl = await client.sendPayload(payload, 3600);
console.log('Payload URL:', payloadUrl);

// Send a file
const fileUrl = await client.sendFile('SGVsbG8gV29ybGQ=', 'hello.txt', 3600);
console.log('File URL:', fileUrl);

// Receive a secret (returns just the data string)
const retrievedSecret = await client.receiveSecret(url);
console.log('Retrieved secret:', retrievedSecret);

// Receive a payload (returns the full payload object)
const retrievedPayload = await client.receivePayload(payloadUrl);
console.log('Retrieved payload:', retrievedPayload);
// retrievedPayload = { data: 'My secret message' }

// Receive a file payload
const retrievedFile = await client.receivePayload(fileUrl);
console.log('Retrieved file:', retrievedFile);
// retrievedFile = { data: 'SGVsbG8gV29ybGQ=', filename: 'hello.txt' }
*/
