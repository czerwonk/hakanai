/*
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

  // Validation error codes - specific for better translations
  /** Input must be a Uint8Array but received different type */
  EXPECTED_UINT8_ARRAY: "EXPECTED_UINT8_ARRAY",
  /** Input must be a string but received different type */
  EXPECTED_STRING: "EXPECTED_STRING",
  /** Input contains invalid characters or format */
  INVALID_INPUT_FORMAT: "INVALID_INPUT_FORMAT",
  /** Cryptographic key is missing */
  MISSING_KEY: "MISSING_KEY",
  /** Cryptographic key has invalid length or format */
  INVALID_KEY: "INVALID_KEY",
  /** Web Crypto API is not available */
  CRYPTO_API_UNAVAILABLE: "CRYPTO_API_UNAVAILABLE",
  /** TTL value is invalid */
  INVALID_TTL: "INVALID_TTL",
  /** Authentication token is missing */
  MISSING_AUTH_TOKEN: "MISSING_AUTH_TOKEN",
  /** Authentication token format is invalid */
  INVALID_AUTH_TOKEN: "INVALID_AUTH_TOKEN",
  /** Base64 encoding/decoding failed */
  BASE64_ERROR: "BASE64_ERROR",
  /** Encrypted data is corrupted or invalid */
  INVALID_ENCRYPTED_DATA: "INVALID_ENCRYPTED_DATA",
  /** Decryption operation failed */
  DECRYPTION_FAILED: "DECRYPTION_FAILED",
  /** URL format is invalid */
  INVALID_URL_FORMAT: "INVALID_URL_FORMAT",
  /** URL is missing secret ID */
  MISSING_SECRET_ID: "MISSING_SECRET_ID",
  /** Secret ID format is invalid */
  INVALID_SECRET_ID: "INVALID_SECRET_ID",
  /** Payload object is invalid or malformed */
  INVALID_PAYLOAD: "INVALID_PAYLOAD",
  /** Server response is invalid or empty */
  INVALID_SERVER_RESPONSE: "INVALID_SERVER_RESPONSE",
  /** CryptoContext has been disposed */
  CRYPTO_CONTEXT_DISPOSED: "CRYPTO_CONTEXT_DISPOSED",
} as const;

// Type for error codes
type HakanaiErrorCode =
  (typeof HakanaiErrorCodes)[keyof typeof HakanaiErrorCodes];

/**
 * Type-safe validation functions for all input data
 * Provides compile-time safety without string copying for better memory security
 * @namespace InputValidation
 */
class InputValidation {
  /**
   * Validate server-generated authentication token format
   * Server tokens are 32 random bytes encoded as base64 (44 characters with padding)
   * @param token - Authentication token string to validate
   * @throws {HakanaiError} If token format doesn't match server-generated format
   */
  static validateAuthToken(token: string): void {
    if (typeof token !== "string") {
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_AUTH_TOKEN,
        "Auth token must be a string",
      );
    }

    // Empty token is valid (represents no authentication)
    if (!token.trim()) {
      return;
    }

    // Server-generated tokens are 32 bytes base64url-encoded = 43 chars without padding
    if (!/^[A-Za-z0-9_-]{43}$/.test(token)) {
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_AUTH_TOKEN,
        "Auth token must be a 43-character base64url string (server-generated format)",
      );
    }
  }

  /**
   * Validate AES-256 secret key format (base64url, 32 bytes)
   * @param key - Base64url encoded secret key to validate
   * @throws {HakanaiError} If key format or length is invalid
   */
  static validateSecretKey(key: string): void {
    if (typeof key !== "string") {
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_KEY,
        "Secret key must be a string",
      );
    }

    if (!key.trim()) {
      throw new HakanaiError(
        HakanaiErrorCodes.MISSING_KEY,
        "Secret key cannot be empty",
      );
    }

    // Validate base64url format and length (32 bytes = 43 chars in base64url without padding)
    if (!/^[A-Za-z0-9_-]{43}$/.test(key)) {
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_KEY,
        "Secret key must be a 43-character base64url string (32 bytes)",
      );
    }
  }

  /**
   * Validate secret ID format (UUID)
   * @param id - Secret ID string to validate
   * @throws {HakanaiError} If ID format is invalid
   */
  static validateSecretId(id: string): void {
    if (typeof id !== "string") {
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_SECRET_ID,
        "Secret ID must be a string",
      );
    }

    if (!id.trim()) {
      throw new HakanaiError(
        HakanaiErrorCodes.MISSING_SECRET_ID,
        "Secret ID cannot be empty",
      );
    }

    // Validate UUID format (flexible - supports v1-v8)
    if (
      !/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(
        id,
      )
    ) {
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_SECRET_ID,
        "Secret ID must be a valid UUID",
      );
    }
  }

  /**
   * Validate encrypted data format (base64)
   * @param data - Base64 encoded encrypted data to validate
   * @throws {HakanaiError} If data format is invalid
   */
  static validateEncryptedData(data: string): void {
    if (typeof data !== "string") {
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_ENCRYPTED_DATA,
        "Encrypted data must be a string",
      );
    }

    if (!data.trim()) {
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_ENCRYPTED_DATA,
        "Encrypted data cannot be empty",
      );
    }

    // Validate base64 format (with optional padding)
    if (!/^[A-Za-z0-9+/]*={0,2}$/.test(data)) {
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_ENCRYPTED_DATA,
        "Encrypted data must be valid base64",
      );
    }

    // Check minimum length (AES-GCM nonce + some encrypted content)
    if (data.length < 16) {
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_ENCRYPTED_DATA,
        "Encrypted data appears too short to be valid",
      );
    }
  }

  /**
   * Validate TTL (Time To Live) value
   * @param ttl - TTL value in seconds
   * @throws {HakanaiError} If TTL is invalid
   */
  static validateTTL(ttl: number): void {
    if (typeof ttl !== "number") {
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_TTL,
        "TTL must be a number",
      );
    }

    if (!Number.isInteger(ttl)) {
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_TTL,
        "TTL must be an integer",
      );
    }

    if (ttl <= 0) {
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_TTL,
        "TTL must be a positive number",
      );
    }
  }
}

/**
 * URL parsing utilities for Hakanai secret URLs
 * @class UrlParser
 */
class UrlParser {
  /**
   * Parse and validate a secret URL, returning its components
   * @param url - Complete secret URL
   * @returns Object with validated secretId and secretKey
   * @throws {HakanaiError} If URL or its components are invalid
   */
  static parseSecretUrl(url: string): {
    secretId: string;
    secretKey: string;
  } {
    // Basic URL validation
    if (typeof url !== "string" || !url.trim()) {
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_URL_FORMAT,
        "URL cannot be empty",
      );
    }

    let urlObj: URL;
    try {
      urlObj = new URL(url);
    } catch {
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_URL_FORMAT,
        "Invalid URL format",
      );
    }

    // Parse and validate URL structure first
    const pathParts = urlObj.pathname.split("/");
    if (pathParts.length !== 3 || pathParts[1] !== "s" || !pathParts[2]) {
      throw new HakanaiError(
        HakanaiErrorCodes.MISSING_SECRET_ID,
        "URL must contain secret ID in format /s/{id}",
      );
    }

    // Check if hash fragment is missing from URL
    if (!urlObj.hash) {
      throw new HakanaiError(
        HakanaiErrorCodes.MISSING_DECRYPTION_KEY,
        "URL must contain decryption key in fragment",
      );
    }

    const secretId = pathParts[2];
    InputValidation.validateSecretId(secretId);

    const secretKey = urlObj.hash.slice(1);
    InputValidation.validateSecretKey(secretKey);

    return { secretId, secretKey };
  }
}

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
  setFromBytes(bytes: Uint8Array): void;

  /**
   * Set data directly from base64-encoded string (optimization for pre-encoded data)
   */
  setFromBase64(base64Data: string): void;

  /**
   * Decode the base64-encoded data field to a readable string
   */
  decode(): string;

  /**
   * Decode the base64-encoded data field to bytes for binary data
   */
  decodeBytes(): Uint8Array;
}

interface SecretResponse {
  id: string;
}

interface SecretRequest {
  data: string;
  expires_in: number;
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
      throw new HakanaiError(
        HakanaiErrorCodes.EXPECTED_UINT8_ARRAY,
        "Input must be a Uint8Array",
      );
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
      throw new HakanaiError(
        HakanaiErrorCodes.EXPECTED_STRING,
        "Input must be a string",
      );
    }

    if (encoded.length === 0) {
      return new Uint8Array(0);
    }

    if (!/^[A-Za-z0-9_-]*$/.test(encoded)) {
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_INPUT_FORMAT,
        "Invalid base64url characters",
      );
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
      throw new HakanaiError(
        HakanaiErrorCodes.BASE64_ERROR,
        "Failed to decode base64url string: invalid encoding",
      );
    }
  }

  /**
   * Utility method for encoding text strings directly
   * More efficient than manual TextEncoder + encode
   */
  static encodeText(text: string): string {
    if (typeof text !== "string") {
      throw new HakanaiError(
        HakanaiErrorCodes.EXPECTED_STRING,
        "Input must be a string",
      );
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
      throw new HakanaiError(
        HakanaiErrorCodes.EXPECTED_UINT8_ARRAY,
        "Input must be a Uint8Array",
      );
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
 * Cryptographic context that encapsulates all crypto operations and automatically cleans up sensitive data
 * Similar to Rust's CryptoContext pattern - all crypto state is contained and automatically disposed
 * @class CryptoContext
 */
class CryptoContext {
  private readonly keyBytes: Uint8Array;
  private readonly cryptoKey: CryptoKey;
  private nonce: Uint8Array;
  private isDisposed = false;
  private isUsed = false;

  private constructor(
    keyBytes: Uint8Array,
    cryptoKey: CryptoKey,
    nonce: Uint8Array,
  ) {
    this.keyBytes = keyBytes;
    this.cryptoKey = cryptoKey;
    this.nonce = nonce;
  }

  /**
   * Get crypto instance (browser environment)
   * @private
   */
  private static getCrypto(): Crypto {
    const cryptoInstance = window?.crypto || crypto;
    if (!cryptoInstance) {
      throw new HakanaiError(
        HakanaiErrorCodes.CRYPTO_API_UNAVAILABLE,
        "Crypto API not available",
      );
    }

    return cryptoInstance;
  }

  /**
   * Create a new CryptoContext with a randomly generated key and nonce
   * @returns Promise resolving to a new CryptoContext
   * @throws {Error} If crypto operations fail
   */
  static async generate(): Promise<CryptoContext> {
    const keyBytes = new Uint8Array(KEY_LENGTH);
    CryptoContext.getCrypto().getRandomValues(keyBytes);

    const nonce = new Uint8Array(NONCE_LENGTH);
    CryptoContext.getCrypto().getRandomValues(nonce);

    const cryptoKey = await CryptoContext.getCrypto().subtle.importKey(
      "raw",
      keyBytes,
      { name: "AES-GCM", length: KEY_LENGTH * 8 },
      false,
      ["encrypt", "decrypt"],
    );

    return new CryptoContext(keyBytes, cryptoKey, nonce);
  }

  /**
   * Create a CryptoContext from an existing key (for decryption)
   * @param keyBytes - 32-byte encryption key
   * @returns Promise resolving to a new CryptoContext
   * @throws {Error} If key is invalid or crypto operations fail
   */
  static async fromKey(keyBytes: Uint8Array): Promise<CryptoContext> {
    if (!(keyBytes instanceof Uint8Array)) {
      throw new HakanaiError(
        HakanaiErrorCodes.EXPECTED_UINT8_ARRAY,
        "Key must be a Uint8Array",
      );
    }

    if (keyBytes.length !== KEY_LENGTH) {
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_KEY,
        `Invalid key length: must be ${KEY_LENGTH} bytes`,
      );
    }

    // Create a copy to avoid external modification
    const keyCopy = new Uint8Array(keyBytes);

    // For decryption, nonce will be extracted from encrypted data
    const nonce = new Uint8Array(NONCE_LENGTH);

    const cryptoKey = await CryptoContext.getCrypto().subtle.importKey(
      "raw",
      keyCopy,
      { name: "AES-GCM", length: KEY_LENGTH * 8 },
      false,
      ["encrypt", "decrypt"],
    );

    return new CryptoContext(keyCopy, cryptoKey, nonce);
  }

  /**
   * Encrypt data with AES-256-GCM using the context's nonce
   * @param plaintextBytes - Raw bytes to encrypt
   * @returns Base64-encoded ciphertext with prepended nonce
   * @throws {Error} If encryption fails, context is disposed, or context already used
   */
  async encrypt(plaintextBytes: Uint8Array): Promise<string> {
    this.checkDisposed();

    if (this.isUsed) {
      throw new HakanaiError(
        HakanaiErrorCodes.CRYPTO_CONTEXT_DISPOSED,
        "CryptoContext has already been used for encryption. Create a new context to prevent nonce reuse.",
      );
    }

    if (!(plaintextBytes instanceof Uint8Array)) {
      throw new HakanaiError(
        HakanaiErrorCodes.EXPECTED_UINT8_ARRAY,
        "Plaintext must be a Uint8Array",
      );
    }

    // Mark context as used to prevent nonce reuse
    this.isUsed = true;

    const ciphertext = await CryptoContext.getCrypto().subtle.encrypt(
      { name: "AES-GCM", iv: this.nonce },
      this.cryptoKey,
      plaintextBytes,
    );

    // Combine nonce and ciphertext
    const combined = new Uint8Array(this.nonce.length + ciphertext.byteLength);
    combined.set(this.nonce);
    combined.set(new Uint8Array(ciphertext), this.nonce.length);

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
   * Decrypt AES-256-GCM encrypted data
   * @param encryptedData - Base64-encoded ciphertext with nonce
   * @returns Decrypted plaintext as bytes
   * @throws {Error} If decryption fails or context is disposed
   */
  async decrypt(encryptedData: string): Promise<Uint8Array> {
    this.checkDisposed();

    if (typeof encryptedData !== "string") {
      throw new HakanaiError(
        HakanaiErrorCodes.EXPECTED_STRING,
        "Encrypted data must be a string",
      );
    }

    // Decode from standard base64
    const binaryString = atob(encryptedData);
    const combined = new Uint8Array(binaryString.length);

    for (let i = 0; i < binaryString.length; i++) {
      combined[i] = binaryString.charCodeAt(i);
    }

    if (combined.length < NONCE_LENGTH + 1) {
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_ENCRYPTED_DATA,
        "Invalid encrypted data: too short",
      );
    }

    // Extract nonce and update context nonce
    this.nonce.set(combined.slice(0, NONCE_LENGTH));
    const ciphertext = combined.slice(NONCE_LENGTH);

    try {
      const plaintextBytes = await CryptoContext.getCrypto().subtle.decrypt(
        { name: "AES-GCM", iv: this.nonce },
        this.cryptoKey,
        ciphertext,
      );

      return new Uint8Array(plaintextBytes);
    } catch (error) {
      throw new HakanaiError(
        HakanaiErrorCodes.DECRYPTION_FAILED,
        "Decryption failed: invalid key or corrupted data",
      );
    }
  }

  /**
   * Get the encryption key as URL-safe base64 string
   * @returns Base64url-encoded key
   * @throws {Error} If context is disposed
   */
  getKeyBase64(): string {
    this.checkDisposed();
    return Base64UrlSafe.encode(this.keyBytes);
  }

  /**
   * Dispose of the crypto context and clear all sensitive data
   * After calling this method, the context cannot be used for crypto operations
   */
  dispose(): void {
    if (!this.isDisposed) {
      SecureMemory.clearArrays(this.keyBytes, this.nonce);
      this.isDisposed = true;
    }
  }

  /**
   * Check if the context has been disposed
   * @private
   */
  private checkDisposed(): void {
    if (this.isDisposed) {
      throw new HakanaiError(
        HakanaiErrorCodes.CRYPTO_CONTEXT_DISPOSED,
        "CryptoContext has been disposed and cannot be used",
      );
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
      throw new HakanaiError(
        HakanaiErrorCodes.EXPECTED_UINT8_ARRAY,
        "Data must be a Uint8Array",
      );
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

  setFromBase64(base64Data: string): void {
    if (typeof base64Data !== "string") {
      throw new HakanaiError(
        HakanaiErrorCodes.EXPECTED_STRING,
        "Base64 data must be a string",
      );
    }

    // Validate base64 format (basic check)
    if (!/^[A-Za-z0-9+/]*={0,2}$/.test(base64Data)) {
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_INPUT_FORMAT,
        "Invalid base64 format",
      );
    }

    this._data = base64Data;
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
      throw new HakanaiError(
        HakanaiErrorCodes.EXPECTED_STRING,
        "Base URL must be a non-empty string",
      );
    }

    this.baseUrl = baseUrl.replace(/\/$/, ""); // Remove trailing slash

    // Check browser compatibility
    const compatibilityInfo = BrowserCompatibility.getCompatibilityInfo();
    if (!compatibilityInfo.isCompatible) {
      throw new HakanaiError(
        HakanaiErrorCodes.CRYPTO_API_UNAVAILABLE,
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
   * Validate sendPayload parameters with comprehensive type checking
   * @private
   */
  private validateSendPayloadParams(
    payload: PayloadData,
    ttl: number,
    authToken?: string,
  ): void {
    if (!payload || typeof payload !== "object") {
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_PAYLOAD,
        "Payload must be an object",
      );
    }

    if (
      !payload.data ||
      typeof payload.data !== "string" ||
      payload.data.length === 0
    ) {
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_PAYLOAD,
        "Payload data cannot be empty",
      );
    }

    // Use validation namespace for consistent TTL validation
    InputValidation.validateTTL(ttl);

    // Validate auth token format if provided
    if (authToken !== undefined) {
      InputValidation.validateAuthToken(authToken);
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
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_SERVER_RESPONSE,
        "Invalid response: missing secret ID",
      );
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

    const cryptoContext = await CryptoContext.generate();
    try {
      const secretPayload = {
        data: payload.data,
        filename: payload.filename || null,
      };
      const payloadJson = JSON.stringify(secretPayload);
      const encodedBytes = new TextEncoder().encode(payloadJson);
      const payloadBytes =
        encodedBytes instanceof Uint8Array
          ? encodedBytes
          : new Uint8Array(encodedBytes);

      const encryptedData = await cryptoContext.encrypt(payloadBytes);

      // Clear payload bytes after encryption
      SecureMemory.clearUint8Array(payloadBytes);

      const secretId = await this.sendEncryptedData(
        encryptedData,
        ttl,
        authToken,
      );

      return `${this.baseUrl}/s/${secretId}#${cryptoContext.getKeyBase64()}`;
    } finally {
      cryptoContext.dispose();
    }
  }

  /**
   * Validate and parse receive URL for secret retrieval using centralized validation
   * @private
   */
  private validateAndParseReceiveUrl(url: string): {
    secretId: string;
    key: Uint8Array;
  } {
    // Use URL parser for parsing
    const { secretId, secretKey } = UrlParser.parseSecretUrl(url);

    // Convert validated key string to bytes
    let key: Uint8Array;
    try {
      key = Base64UrlSafe.decode(secretKey);
    } catch (error) {
      throw new HakanaiError(
        HakanaiErrorCodes.BASE64_ERROR,
        "Invalid decryption key in URL",
      );
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
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_SERVER_RESPONSE,
        "Empty response from server",
      );
    }

    // Validate encrypted data format
    InputValidation.validateEncryptedData(encryptedData);

    const cryptoContext = await CryptoContext.fromKey(key);
    try {
      const decryptedBytes = await cryptoContext.decrypt(encryptedData);
      const decryptedJson = new TextDecoder().decode(decryptedBytes);

      // Clear decrypted bytes after converting to string
      SecureMemory.clearUint8Array(decryptedBytes);

      let payload: PayloadData;
      try {
        payload = JSON.parse(decryptedJson);
      } catch (error) {
        throw new HakanaiError(
          HakanaiErrorCodes.INVALID_PAYLOAD,
          "Failed to parse decrypted payload",
        );
      }

      // Validate payload structure
      if (
        !payload ||
        typeof payload !== "object" ||
        typeof payload.data !== "string"
      ) {
        throw new HakanaiError(
          HakanaiErrorCodes.INVALID_PAYLOAD,
          "Invalid payload structure",
        );
      }

      return new PayloadDataImpl(payload.data, payload.filename || undefined);
    } finally {
      cryptoContext.dispose();
    }
  }

  /**
   * Create a new PayloadData object for building payloads
   * @param filename - Optional filename for file payloads
   * @returns Empty PayloadData object ready for data
   */
  createPayload(filename?: string): PayloadData {
    return new PayloadDataImpl("", filename);
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
    ContentAnalysis,
    InputValidation,
  };
}

export {
  HakanaiClient,
  HakanaiError,
  HakanaiErrorCodes,
  Base64UrlSafe,
  ContentAnalysis,
  type PayloadData,
};
