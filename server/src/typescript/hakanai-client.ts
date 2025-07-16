/**
 * Hakanai JavaScript Client (TypeScript Implementation)
 *
 * This client implements the same cryptographic protocol as the Rust CLI client,
 * allowing you to send and receive encrypted secrets via the Hakanai API.
 */

const KEY_LENGTH = 32; // 256 bits
const NONCE_LENGTH = 12; // 96 bits for AES-GCM

// Error code constants

/**
 * Error codes for Hakanai operations
 * @readonly
 * @enum {string}
 */
const HakanaiErrorCodes = {
  /** Server requires authentication token */
  AUTHENTICATION_REQUIRED: "AUTHENTICATION_REQUIRED",
  /** Provided authentication token is invalid */
  INVALID_TOKEN: "INVALID_TOKEN",
  /** Failed to send secret to server */
  SEND_FAILED: "SEND_FAILED",
  /** Secret not found or has expired */
  SECRET_NOT_FOUND: "SECRET_NOT_FOUND",
  /** Secret has already been accessed once */
  SECRET_ALREADY_ACCESSED: "SECRET_ALREADY_ACCESSED",
  /** General failure retrieving secret */
  RETRIEVE_FAILED: "RETRIEVE_FAILED",
  /** URL missing decryption key in fragment */
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

/**
 * Custom error class for Hakanai operations with specific error codes
 * @class HakanaiError
 * @extends {Error}
 */
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
 * @class Base64UrlSafe
 */
class Base64UrlSafe {
  /**
   * Encode Uint8Array to URL-safe base64 string
   * Uses chunked processing to handle large arrays safely
   * @param data - Raw bytes to encode
   * @returns URL-safe base64 string (no padding)
   * @throws {Error} If input is not a Uint8Array
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
   * @param encoded - URL-safe base64 string to decode
   * @returns Decoded bytes as Uint8Array
   * @throws {Error} If input contains invalid characters or encoding
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
 * Content analysis utilities for detecting binary vs text data
 * @class ContentAnalysis
 */
class ContentAnalysis {
  /**
   * Check if a byte array contains null bytes (indicating binary content)
   *
   * This function uses a simple but effective heuristic: the presence of null bytes.
   * Most text encodings (UTF-8, ASCII, etc.) don't contain null bytes, while binary
   * formats (executables, images, compressed files) commonly do.
   *
   * @param bytes - Byte array to analyze
   * @returns true if the content appears to be binary data, false if it appears to be text
   *
   * @example
   * ```typescript
   * const textBytes = new TextEncoder().encode("Hello, world!");
   * const binaryBytes = new Uint8Array([0x00, 0x01, 0x02, 0xFF]);
   *
   * console.log(ContentAnalysis.isBinary(textBytes)); // false
   * console.log(ContentAnalysis.isBinary(binaryBytes)); // true
   * ```
   */
  static isBinary(bytes: Uint8Array): boolean {
    if (!(bytes instanceof Uint8Array)) {
      throw new Error("Input must be a Uint8Array");
    }

    // Check for null bytes, which are common in binary files
    return bytes.includes(0);
  }
}

/**
 * Secure memory clearing utilities for sensitive data
 * @class SecureMemory
 */
class SecureMemory {
  /**
   * Securely clear a Uint8Array with multiple overwrite passes
   * @param array - Array to clear
   */
  static clearUint8Array(array: Uint8Array): void {
    if (!(array instanceof Uint8Array)) {
      return;
    }

    // Multiple overwrite passes with random data
    for (let pass = 0; pass < 3; pass++) {
      try {
        crypto.getRandomValues(array);
      } catch (error) {
        // Fallback to manual random fill if crypto not available
        for (let i = 0; i < array.length; i++) {
          array[i] = Math.floor(Math.random() * 256);
        }
      }
    }

    // Final zero fill
    array.fill(0);
  }

  /**
   * Securely clear multiple Uint8Array objects
   * @param arrays - Arrays to clear
   */
  static clearArrays(...arrays: Uint8Array[]): void {
    arrays.forEach((array) => SecureMemory.clearUint8Array(array));
  }
}

/**
 * Type-safe cryptographic operations using Web Crypto API
 * @class CryptoOperations
 */
class CryptoOperations {
  /**
   * Get crypto instance (browser environment)
   * @private
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
   * @returns Cryptographic key with 32 random bytes
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
   * Encrypt a message with AES-256-GCM
   * @param plaintext - Text to encrypt
   * @param key - 256-bit encryption key
   * @returns Base64-encoded ciphertext with prepended nonce
   * @throws {Error} If encryption fails
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

    let result: string;
    try {
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

      result = btoa(binaryString);

      // Securely clear sensitive data
      SecureMemory.clearUint8Array(combined);
    } finally {
      // Always clear sensitive data even if encryption fails
      SecureMemory.clearArrays(plaintextBytes, nonce);
    }

    return result;
  }

  /**
   * Decrypt an AES-256-GCM encrypted message
   * @param encryptedData - Base64-encoded ciphertext with nonce
   * @param key - 256-bit decryption key
   * @returns Decrypted plaintext
   * @throws {Error} If decryption fails or key is invalid
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

    let result: string;
    try {
      const plaintextBytes = await CryptoOperations.getCrypto().subtle.decrypt(
        { name: "AES-GCM", iv: nonce },
        cryptoKey,
        ciphertext,
      );

      const decoder = new TextDecoder();
      result = decoder.decode(plaintextBytes);

      // Clear sensitive plaintext bytes
      SecureMemory.clearUint8Array(new Uint8Array(plaintextBytes));
    } catch (error) {
      throw new Error("Decryption failed: invalid key or corrupted data");
    } finally {
      // Always clear sensitive data even if decryption fails
      SecureMemory.clearArrays(combined, nonce, ciphertext);
    }

    return result;
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
    const decoder = new TextDecoder();
    return decoder.decode(this.decodeBytes());
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
 * Main Hakanai client for sending and receiving encrypted secrets
 * @class HakanaiClient
 */
class HakanaiClient {
  private readonly baseUrl: string;

  /**
   * Create a new Hakanai client
   * @param baseUrl - Base URL of the Hakanai server (without trailing slash)
   * @throws {Error} If browser lacks required crypto features
   */
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
   * Validate sendPayload parameters
   * @private
   */
  private validateSendPayloadParams(
    payload: PayloadData,
    ttl: number,
    authToken?: string,
  ): void {
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
  }

  /**
   * Handle HTTP response errors for sendPayload
   * @private
   */
  private handleSendPayloadError(response: Response): never {
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

  /**
   * Send encrypted data to the server via HTTP
   * @private
   */
  private async sendEncryptedData(
    encryptedData: string,
    ttl: number,
    authToken?: string,
  ): Promise<string> {
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
      this.handleSendPayloadError(response);
    }

    const responseData: SecretResponse = await response.json();
    if (!responseData.id || typeof responseData.id !== "string") {
      throw new Error("Invalid response: missing secret ID");
    }

    return responseData.id;
  }

  /**
   * Encrypt and send a payload to the Hakanai server
   * @param payload - Data to encrypt and send (must have non-empty data field)
   * @param ttl - Time-to-live in seconds (default: 3600)
   * @param authToken - Optional authentication token for server access
   * @returns Full URL with secret ID and decryption key in fragment
   * @throws {HakanaiError} With specific error codes:
   *   - AUTHENTICATION_REQUIRED: Server requires auth token
   *   - INVALID_TOKEN: Provided token is invalid
   *   - SEND_FAILED: General send failure
   */
  async sendPayload(
    payload: PayloadData,
    ttl: number = 3600,
    authToken?: string,
  ): Promise<string> {
    this.validateSendPayloadParams(payload, ttl, authToken);

    const key = CryptoOperations.generateKey();

    let result: string;
    try {
      const secretPayload = {
        data: payload.data,
        filename: payload.filename || null,
      };
      const payloadJson = JSON.stringify(secretPayload);

      const encryptedData = await CryptoOperations.encrypt(payloadJson, key);

      const secretId = await this.sendEncryptedData(
        encryptedData,
        ttl,
        authToken,
      );

      const keyBase64 = Base64UrlSafe.encode(key.bytes);

      result = `${this.baseUrl}/s/${secretId}#${keyBase64}`;
    } finally {
      SecureMemory.clearUint8Array(key.bytes);
    }

    return result;
  }

  /**
   * Validate and parse receive URL for secret retrieval
   * @private
   */
  private validateAndParseReceiveUrl(url: string): {
    secretId: string;
    key: Uint8Array;
  } {
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

    return { secretId, key };
  }

  /**
   * Handle HTTP response errors for receivePayload
   * @private
   */
  private handleReceivePayloadError(response: Response): never {
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

  /**
   * Retrieve and decrypt a payload from the server
   * @param url - Full URL with format: https://server/s/{id}#{key}
   * @returns Decrypted payload data with optional filename
   * @throws {HakanaiError} With specific error codes:
   *   - MISSING_DECRYPTION_KEY: No key found in URL fragment
   *   - SECRET_NOT_FOUND: Secret expired or doesn't exist
   *   - SECRET_ALREADY_ACCESSED: Secret was already retrieved
   *   - RETRIEVE_FAILED: General retrieval failure
   * @throws {Error} For invalid URL format or decryption failures
   */
  async receivePayload(url: string): Promise<PayloadData> {
    const { secretId, key } = this.validateAndParseReceiveUrl(url);

    const response = await fetch(`${this.baseUrl}/api/v1/secret/${secretId}`);

    if (!response.ok) {
      this.handleReceivePayloadError(response);
    }

    const encryptedData = await response.text();
    if (!encryptedData) {
      throw new Error("Empty response from server");
    }

    let payload: PayloadData;
    try {
      const decryptedJson = await CryptoOperations.decrypt(encryptedData, key);

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
    } finally {
      // Always clear the key after use
      SecureMemory.clearUint8Array(key);
    }

    return new PayloadDataImpl(payload.data, payload.filename || undefined);
  }

  /**
   * Create a new PayloadData object for building payloads
   * @param filename - Optional filename for file payloads
   * @returns Empty PayloadData object ready for data
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
    ContentAnalysis,
  };
}

export {
  HakanaiClient,
  HakanaiError,
  HakanaiErrorCodes,
  Base64UrlSafe,
  CryptoOperations,
  ContentAnalysis,
  type PayloadData,
  type CompatibilityCheck,
};
