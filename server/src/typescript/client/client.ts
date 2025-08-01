/*
 * Hakanai JavaScript Client (TypeScript Implementation)
 *
 * This client implements the same cryptographic protocol as the Rust CLI client,
 * allowing you to send and receive encrypted secrets via the Hakanai API.
 */

import { HakanaiError, HakanaiErrorCodes } from "./errors";
import { InputValidation } from "./validation";
import { UrlParser } from "./url-parser";
import {
  type CompatibilityCheck,
  BrowserCompatibility,
} from "./browser-compat";
import { Base64UrlSafe } from "./base64-utils";
import { ContentAnalysis } from "./content-analysis";
import { CryptoContext } from "./crypto-operations";
import { type PayloadData, PayloadDataImpl } from "./payload";
import { SecureMemory } from "./secure-memory";

interface SecretResponse {
  id: string;
}

interface SecretRequest {
  data: string;
  expires_in: number;
}

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

    if (response.status === 413) {
      throw new HakanaiError(
        HakanaiErrorCodes.PAYLOAD_TOO_LARGE,
        "The payload size exceeds the limit allowed for the user",
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

  private async hashFromBytes(bytes: Uint8Array): Promise<string> {
    const hashBuffer = await crypto.subtle.digest("SHA-256", bytes);
    const hashArray = new Uint8Array(hashBuffer);
    const truncated = hashArray.slice(0, 16);
    return Base64UrlSafe.encode(truncated);
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

    if (authToken?.length) {
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
        filename: payload.filename ?? null,
      };
      const payloadJson = JSON.stringify(secretPayload);
      const encodedBytes = new TextEncoder().encode(payloadJson);
      const payloadBytes =
        encodedBytes instanceof Uint8Array
          ? encodedBytes
          : new Uint8Array(encodedBytes);

      const encryptedData = await cryptoContext.encrypt(payloadBytes);
      const hash = await this.hashFromBytes(payloadBytes);

      // Clear payload bytes after encryption
      SecureMemory.clearUint8Array(payloadBytes);

      const secretId = await this.sendEncryptedData(
        encryptedData,
        ttl,
        authToken,
      );

      return `${this.baseUrl}/s/${secretId}#${cryptoContext.getKeyBase64()}:${hash}`;
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
    hash?: string;
  } {
    // Use URL parser for parsing
    const { secretId, secretKey, hash } = UrlParser.parseSecretUrl(url);

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

    return { secretId, key, hash };
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

  private async verifyHash(
    plaintext: Uint8Array,
    expectedHash: string,
  ): Promise<void> {
    const actualHash = await this.hashFromBytes(plaintext);
    if (actualHash !== expectedHash) {
      throw new HakanaiError(
        HakanaiErrorCodes.HASH_MISMATCH,
        "Hash verification failed",
      );
    }
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
    const { secretId, key, hash } = this.validateAndParseReceiveUrl(url);

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

      if (hash) {
        await this.verifyHash(decryptedBytes, hash);
      }

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

      return new PayloadDataImpl(payload.data, payload.filename ?? undefined);
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

export {
  HakanaiClient,
  HakanaiError,
  HakanaiErrorCodes,
  Base64UrlSafe,
  ContentAnalysis,
  type PayloadData,
  SecretRequest,
  SecretResponse,
};
