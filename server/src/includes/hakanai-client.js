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

// Receive a secret
const retrievedSecret = await client.receiveSecret(url);
console.log('Retrieved secret:', retrievedSecret);
*/
