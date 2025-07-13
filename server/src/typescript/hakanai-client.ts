/**
 * Hakanai JavaScript Client (TypeScript Implementation)
 *
 * This client implements the same cryptographic protocol as the Rust CLI client,
 * allowing you to send and receive encrypted secrets via the Hakanai API.
 */

const KEY_LENGTH = 32; // 256 bits
const NONCE_LENGTH = 12; // 96 bits for AES-GCM

// Error code constants
const HakanaiErrorCodes = {
  // Authentication errors
  AUTHENTICATION_REQUIRED: "AUTHENTICATION_REQUIRED",
  INVALID_TOKEN: "INVALID_TOKEN",

  // Request/send errors
  SEND_FAILED: "SEND_FAILED",

  // Retrieval errors
  SECRET_NOT_FOUND: "SECRET_NOT_FOUND",
  SECRET_ALREADY_ACCESSED: "SECRET_ALREADY_ACCESSED",
  RETRIEVE_FAILED: "RETRIEVE_FAILED",
  MISSING_DECRYPTION_KEY: "MISSING_DECRYPTION_KEY",
} as const;

// Type for error codes
type HakanaiErrorCode =
  (typeof HakanaiErrorCodes)[keyof typeof HakanaiErrorCodes];

// Type definitions
interface CompatibilityCheck {
  readonly isCompatible: boolean;
  readonly missingFeatures: readonly string[];
}

// Custom error class for Hakanai errors with error codes
class HakanaiError extends Error {
  readonly code: HakanaiErrorCode;
  readonly statusCode?: number;

  constructor(code: HakanaiErrorCode, message: string, statusCode?: number) {
    super(message);
    this.name = "HakanaiError";
    this.code = code;
    this.statusCode = statusCode;
  }
}

interface PayloadData {
  readonly data: string;
  readonly filename?: string;

  /**
   * Set data from raw bytes (for binary files or text converted to bytes)
   */
  setFromBytes?(bytes: Uint8Array): void;

  /**
   * Decode the base64-encoded data field to a readable string
   */
  decode?(): string;

  /**
   * Decode the base64-encoded data field to bytes for binary data
   */
  decodeBytes?(): Uint8Array;
}

interface SecretResponse {
  id: string;
}

interface SecretRequest {
  data: string;
  expires_in: number;
}

interface HakanaiCryptoKey {
  readonly bytes: Uint8Array;
  readonly length: number;
}

/**
 * Type-safe Base64 URL-Safe encoding utility
 * Uses modern browser APIs for better performance and reliability
 */
class Base64UrlSafe {
  /**
   * Encode Uint8Array to URL-safe base64 string
   * Uses chunked processing to handle large arrays safely
   */
  static encode(data: Uint8Array): string {
    if (!(data instanceof Uint8Array)) {
      throw new Error("Input must be a Uint8Array");
    }

    // Handle empty arrays
    if (data.length === 0) {
      return "";
    }

    // Convert Uint8Array to binary string safely, handling large arrays
    const chunkSize = 8192; // Process in chunks to avoid call stack limits
    const chunks: string[] = [];

    for (let i = 0; i < data.length; i += chunkSize) {
      const chunk = data.subarray(i, i + chunkSize);
      chunks.push(String.fromCharCode(...chunk));
    }

    const binaryString = chunks.join("");

    // Encode to standard base64 then convert to URL-safe
    return btoa(binaryString)
      .replace(/\+/g, "-")
      .replace(/\//g, "_")
      .replace(/=/g, "");
  }

  /**
   * Decode URL-safe base64 string to Uint8Array
   * More robust error handling and validation
   */
  static decode(encoded: string): Uint8Array {
    if (typeof encoded !== "string") {
      throw new Error("Input must be a string");
    }

    if (encoded.length === 0) {
      return new Uint8Array(0);
    }

    if (!/^[A-Za-z0-9_-]*$/.test(encoded)) {
      throw new Error("Invalid base64url characters");
    }

    // Add proper padding
    const paddingLength = (4 - (encoded.length % 4)) % 4;
    const padded = encoded + "=".repeat(paddingLength);

    // Convert back to standard base64
    const standard = padded.replace(/-/g, "+").replace(/_/g, "/");

    try {
      const binaryString = atob(standard);
      const bytes = new Uint8Array(binaryString.length);

      // Convert binary string to Uint8Array more efficiently
      for (let i = 0; i < binaryString.length; i++) {
        bytes[i] = binaryString.charCodeAt(i);
      }

      return bytes;
    } catch (error) {
      throw new Error("Failed to decode base64url string: invalid encoding");
    }
  }

  /**
   * Utility method for encoding text strings directly
   * More efficient than manual TextEncoder + encode
   */
  static encodeText(text: string): string {
    if (typeof text !== "string") {
      throw new Error("Input must be a string");
    }

    const encoder = new TextEncoder();
    const bytes = encoder.encode(text);

    // Convert to Uint8Array if needed (Node.js TextEncoder returns different type)
    const uint8Array =
      bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes);
    return Base64UrlSafe.encode(uint8Array);
  }

  /**
   * Utility method for decoding to text strings directly
   * More efficient than decode + manual TextDecoder
   */
  static decodeText(encoded: string): string {
    const data = Base64UrlSafe.decode(encoded);

    const decoder = new TextDecoder();
    return decoder.decode(data);
  }
}

/**
 * Browser compatibility checker
 */
class BrowserCompatibility {
  /**
   * Get detailed compatibility information
   */
  static getCompatibilityInfo(): CompatibilityCheck {
    const missingFeatures: string[] = [];

    // Check for Web Crypto API (browser environment)
    const cryptoInstance = window?.crypto || crypto;
    if (!cryptoInstance || !cryptoInstance.subtle) {
      missingFeatures.push("Web Crypto API (crypto.subtle)");
    }

    // Check for TextEncoder/TextDecoder
    if (typeof TextEncoder === "undefined") {
      missingFeatures.push("TextEncoder");
    }
    if (typeof TextDecoder === "undefined") {
      missingFeatures.push("TextDecoder");
    }

    // Check for crypto.getRandomValues
    if (
      !cryptoInstance ||
      typeof cryptoInstance.getRandomValues !== "function"
    ) {
      missingFeatures.push("crypto.getRandomValues");
    }

    // Check for fetch API
    if (typeof fetch === "undefined") {
      missingFeatures.push("Fetch API");
    }

    // Check for Uint8Array
    if (typeof Uint8Array === "undefined") {
      missingFeatures.push("Uint8Array");
    }

    // Check for atob/btoa (base64 functions)
    if (typeof atob === "undefined" || typeof btoa === "undefined") {
      missingFeatures.push("Base64 functions (atob/btoa)");
    }

    return {
      isCompatible: missingFeatures.length === 0,
      missingFeatures: Object.freeze(missingFeatures),
    };
  }

  /**
   * Check if browser is compatible (simple boolean)
   */
  static isCompatible(): boolean {
    return BrowserCompatibility.getCompatibilityInfo().isCompatible;
  }
}

/**
 * Type-safe crypto operations
 */
class CryptoOperations {
  /**
   * Get crypto instance (browser environment)
   */
  private static getCrypto(): Crypto {
    const cryptoInstance = window?.crypto || crypto;
    if (!cryptoInstance) {
      throw new Error("Crypto API not available");
    }

    return cryptoInstance;
  }

  /**
   * Generate a random 256-bit AES key
   */
  static generateKey(): HakanaiCryptoKey {
    const bytes = new Uint8Array(KEY_LENGTH);
    CryptoOperations.getCrypto().getRandomValues(bytes);
    return Object.freeze({ bytes, length: KEY_LENGTH });
  }

  /**
   * Import a raw key for use with WebCrypto API
   */
  static async importKey(rawKey: Uint8Array): Promise<CryptoKey> {
    if (!(rawKey instanceof Uint8Array)) {
      throw new Error("Key must be a Uint8Array");
    }

    if (rawKey.length !== KEY_LENGTH) {
      throw new Error(`Invalid key length: must be ${KEY_LENGTH} bytes`);
    }

    return CryptoOperations.getCrypto().subtle.importKey(
      "raw",
      rawKey,
      { name: "AES-GCM", length: KEY_LENGTH * 8 },
      false,
      ["encrypt", "decrypt"],
    );
  }

  /**
   * Encrypt a message with AES-GCM
   */
  static async encrypt(
    plaintext: string,
    key: HakanaiCryptoKey,
  ): Promise<string> {
    if (typeof plaintext !== "string") {
      throw new Error("Plaintext must be a string");
    }

    const encoder = new TextEncoder();
    const plaintextBytes = encoder.encode(plaintext);

    // Generate random nonce
    const nonce = new Uint8Array(NONCE_LENGTH);
    CryptoOperations.getCrypto().getRandomValues(nonce);

    const cryptoKey = await CryptoOperations.importKey(key.bytes);

    const ciphertext = await CryptoOperations.getCrypto().subtle.encrypt(
      { name: "AES-GCM", iv: nonce },
      cryptoKey,
      plaintextBytes,
    );

    // Combine nonce and ciphertext
    const combined = new Uint8Array(nonce.length + ciphertext.byteLength);
    combined.set(nonce);
    combined.set(new Uint8Array(ciphertext), nonce.length);

    // Encode to standard base64 using chunked approach
    let binaryString = "";
    const chunkSize = 8192;

    for (let i = 0; i < combined.length; i += chunkSize) {
      const chunk = combined.subarray(i, i + chunkSize);
      binaryString += String.fromCharCode(...chunk);
    }

    return btoa(binaryString);
  }

  /**
   * Decrypt an encrypted message
   */
  static async decrypt(
    encryptedData: string,
    key: Uint8Array,
  ): Promise<string> {
    if (typeof encryptedData !== "string") {
      throw new Error("Encrypted data must be a string");
    }

    if (!(key instanceof Uint8Array) || key.length !== KEY_LENGTH) {
      throw new Error(`Key must be a ${KEY_LENGTH}-byte Uint8Array`);
    }

    // Decode from standard base64 more efficiently
    const binaryString = atob(encryptedData);
    const combined = new Uint8Array(binaryString.length);

    for (let i = 0; i < binaryString.length; i++) {
      combined[i] = binaryString.charCodeAt(i);
    }

    if (combined.length < NONCE_LENGTH + 1) {
      throw new Error("Invalid encrypted data: too short");
    }

    // Extract nonce and ciphertext
    const nonce = combined.slice(0, NONCE_LENGTH);
    const ciphertext = combined.slice(NONCE_LENGTH);

    const cryptoKey = await CryptoOperations.importKey(key);

    try {
      const plaintextBytes = await CryptoOperations.getCrypto().subtle.decrypt(
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
}

/**
 * PayloadData implementation class
 */
class PayloadDataImpl implements PayloadData {
  private _data: string = "";
  private _filename?: string;

  constructor(data: string = "", filename?: string) {
    this._data = data;
    this._filename = filename;
  }

  get data(): string {
    return this._data;
  }

  get filename(): string | undefined {
    return this._filename;
  }

  setFromBytes(bytes: Uint8Array): void {
    if (!(bytes instanceof Uint8Array)) {
      throw new Error("Data must be a Uint8Array");
    }

    // Convert bytes to base64 for storage
    let binaryString = "";
    const chunkSize = 8192;

    for (let i = 0; i < bytes.length; i += chunkSize) {
      const chunk = bytes.subarray(i, i + chunkSize);
      binaryString += String.fromCharCode(...chunk);
    }

    this._data = btoa(binaryString);
  }

  decode(): string {
    const binaryString = atob(this._data);
    return decodeURIComponent(escape(binaryString));
  }

  decodeBytes(): Uint8Array {
    const binaryString = atob(this._data);
    const bytes = new Uint8Array(binaryString.length);
    for (let i = 0; i < binaryString.length; i++) {
      bytes[i] = binaryString.charCodeAt(i);
    }
    return bytes;
  }
}

/**
 * Main Hakanai client class
 */
class HakanaiClient {
  private readonly baseUrl: string;

  constructor(baseUrl: string) {
    if (typeof baseUrl !== "string" || !baseUrl.trim()) {
      throw new Error("Base URL must be a non-empty string");
    }

    this.baseUrl = baseUrl.replace(/\/$/, ""); // Remove trailing slash

    // Check browser compatibility
    const compatibilityInfo = BrowserCompatibility.getCompatibilityInfo();
    if (!compatibilityInfo.isCompatible) {
      throw new Error(
        `Your browser does not support the required security features for this application. ` +
          `Please use a modern browser with Web Crypto API support.`,
      );
    }
  }

  /**
   * Static method to check browser compatibility without creating an instance
   */
  static isCompatible(): boolean {
    return BrowserCompatibility.isCompatible();
  }

  /**
   * Static method to get compatibility information
   */
  static getCompatibilityInfo(): CompatibilityCheck {
    return BrowserCompatibility.getCompatibilityInfo();
  }

  /**
   * Send a payload to the server
   */
  async sendPayload(
    payload: PayloadData,
    ttl: number = 3600,
    authToken?: string,
  ): Promise<string> {
    if (!payload || typeof payload !== "object") {
      throw new Error("Payload must be an object");
    }

    if (
      !payload.data ||
      typeof payload.data !== "string" ||
      payload.data.length === 0
    ) {
      throw new Error("Payload data cannot be empty");
    }

    if (typeof ttl !== "number" || ttl <= 0 || !Number.isInteger(ttl)) {
      throw new Error("TTL must be a positive integer");
    }

    if (authToken !== undefined && typeof authToken !== "string") {
      throw new Error("Auth token must be a string if provided");
    }

    const key = CryptoOperations.generateKey();

    // Convert PayloadData to Rust-compatible Payload format
    // The data field is already base64-encoded when using setFromBytes
    const rustPayload = {
      data: payload.data,
      filename: payload.filename || null,
    };

    const payloadJson = JSON.stringify(rustPayload);
    const encryptedData = await CryptoOperations.encrypt(payloadJson, key);

    const headers: Record<string, string> = {
      "Content-Type": "application/json",
    };

    if (authToken && authToken.length > 0) {
      headers["Authorization"] = `Bearer ${authToken}`;
    }

    const requestBody: SecretRequest = {
      data: encryptedData,
      expires_in: ttl,
    };

    const response = await fetch(`${this.baseUrl}/api/v1/secret`, {
      method: "POST",
      headers: headers,
      body: JSON.stringify(requestBody),
    });

    if (!response.ok) {
      // Handle authentication errors with specific messages
      if (response.status === 401) {
        throw new HakanaiError(
          HakanaiErrorCodes.AUTHENTICATION_REQUIRED,
          "Authentication required: Please provide a valid authentication token",
          response.status,
        );
      }

      if (response.status === 403) {
        throw new HakanaiError(
          HakanaiErrorCodes.INVALID_TOKEN,
          "Invalid authentication token: Please check your token and try again",
          response.status,
        );
      }

      // Generic error for other status codes
      throw new HakanaiError(
        HakanaiErrorCodes.SEND_FAILED,
        `Failed to send secret: ${response.status} ${response.statusText}`,
        response.status,
      );
    }

    const result: SecretResponse = await response.json();
    if (!result.id || typeof result.id !== "string") {
      throw new Error("Invalid response: missing secret ID");
    }

    const secretId = result.id;
    const keyBase64 = Base64UrlSafe.encode(key.bytes);

    return `${this.baseUrl}/s/${secretId}#${keyBase64}`;
  }

  /**
   * Receive a payload from the server
   */
  async receivePayload(url: string): Promise<PayloadData> {
    if (typeof url !== "string" || !url.trim()) {
      throw new Error("URL must be a non-empty string");
    }

    // Parse the URL
    let urlObj: URL;
    try {
      urlObj = new URL(url);
    } catch (error) {
      throw new Error("Invalid URL format");
    }

    // Extract secret ID from path (expects format /s/{id})
    const pathParts = urlObj.pathname.split("/");
    if (pathParts.length < 3 || pathParts[1] !== "s" || !pathParts[2]) {
      throw new Error("No secret ID found in URL");
    }
    const secretId = pathParts[2];

    const keyBase64 = urlObj.hash.slice(1); // Remove the #
    if (!keyBase64) {
      throw new HakanaiError(
        HakanaiErrorCodes.MISSING_DECRYPTION_KEY,
        "No decryption key found in URL",
      );
    }

    let key: Uint8Array;
    try {
      key = Base64UrlSafe.decode(keyBase64);
    } catch (error) {
      throw new Error("Invalid decryption key in URL");
    }

    if (key.length !== KEY_LENGTH) {
      throw new Error("Invalid key length");
    }

    const response = await fetch(`${this.baseUrl}/api/v1/secret/${secretId}`);

    if (!response.ok) {
      if (response.status === 404) {
        throw new HakanaiError(
          HakanaiErrorCodes.SECRET_NOT_FOUND,
          "Secret not found or has expired",
          404,
        );
      }
      if (response.status === 410) {
        throw new HakanaiError(
          HakanaiErrorCodes.SECRET_ALREADY_ACCESSED,
          "Secret has been accessed and is no longer available",
          410,
        );
      }
      throw new HakanaiError(
        HakanaiErrorCodes.RETRIEVE_FAILED,
        `Failed to retrieve secret: ${response.status} ${response.statusText}`,
        response.status,
      );
    }

    const encryptedData = await response.text();
    if (!encryptedData) {
      throw new Error("Empty response from server");
    }

    const decryptedJson = await CryptoOperations.decrypt(encryptedData, key);

    let payload: PayloadData;
    try {
      payload = JSON.parse(decryptedJson);
    } catch (error) {
      throw new Error("Failed to parse decrypted payload");
    }

    // Validate payload structure
    if (
      !payload ||
      typeof payload !== "object" ||
      typeof payload.data !== "string"
    ) {
      throw new Error("Invalid payload structure");
    }

    return new PayloadDataImpl(payload.data, payload.filename || undefined);
  }

  /**
   * Create a new PayloadData object
   */
  createPayload(filename?: string): PayloadData {
    return new PayloadDataImpl("", filename);
  }

  /**
   * Legacy methods for backward compatibility
   */

  /**
   * @deprecated Use CryptoOperations.generateKey() instead
   */
  async generateKey(): Promise<Uint8Array> {
    return CryptoOperations.generateKey().bytes;
  }

  /**
   * @deprecated Use CryptoOperations.importKey() instead
   */
  async importKey(rawKey: Uint8Array): Promise<CryptoKey> {
    return CryptoOperations.importKey(rawKey);
  }

  /**
   * @deprecated Use CryptoOperations.encrypt() instead
   */
  async encrypt(plaintext: string, key: Uint8Array): Promise<string> {
    return CryptoOperations.encrypt(plaintext, {
      bytes: key,
      length: KEY_LENGTH,
    });
  }

  /**
   * @deprecated Use CryptoOperations.decrypt() instead
   */
  async decrypt(encryptedData: string, key: Uint8Array): Promise<string> {
    return CryptoOperations.decrypt(encryptedData, key);
  }
}

// Export for CommonJS/ES modules
declare var module: any;
if (typeof module !== "undefined" && module.exports) {
  module.exports = {
    HakanaiClient,
    HakanaiError,
    HakanaiErrorCodes,
    Base64UrlSafe,
    CryptoOperations,
  };
}

export {
  HakanaiClient,
  HakanaiError,
  HakanaiErrorCodes,
  Base64UrlSafe,
  CryptoOperations,
  type PayloadData,
  type CompatibilityCheck,
};
