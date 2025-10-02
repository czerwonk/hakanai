// SPDX-License-Identifier: Apache-2.0

/*
 * Hakanai JavaScript Client (TypeScript Implementation)
 *
 * This client implements the same cryptographic protocol as the Rust CLI client,
 * allowing you to send and receive encrypted secrets via the Hakanai API.
 */

import { HakanaiError, HakanaiErrorCodes } from "./errors";
import { InputValidation } from "./validation";
import { UrlParser } from "./url-parser";
import { type CompatibilityCheck, BrowserCompatibility } from "./browser-compat";
import { Base64UrlSafe } from "./base64-utils";
import { ContentAnalysis } from "./content-analysis";
import { CryptoContext } from "./crypto-operations";
import { HashUtils } from "./hash-utils";
import { type PayloadData, PayloadDataImpl } from "./payload";
import { SecureMemory } from "./secure-memory";
import { type DataTransferObserver } from "./progress-observer";

interface SecretResponse {
  id: string;
}

interface SecretRestrictions {
  allowed_ips?: string[];
  allowed_countries?: string[];
  allowed_asns?: number[];
  passphrase_hash?: string;
}

interface SecretRequest {
  data: string;
  expires_in: number;
  restrictions?: SecretRestrictions;
}

/**
 * Manages XHR promise settlement with activity-based timeout
 */
class XhrPromiseManager {
  private isCompleted = false;
  private activityTimeout: number | null = null;
  private readonly ACTIVITY_TIMEOUT_MS = 10000;

  constructor(
    private resolvePromise: (response: Response) => void,
    private rejectPromise: (error: HakanaiError) => void,
    private xhr: XMLHttpRequest,
  ) {}

  /**
   * Reset the activity timeout - call this on progress events
   */
  resetActivityTimeout(): void {
    this.clearActivityTimeout();
    if (!this.isCompleted) {
      this.activityTimeout = window.setTimeout(() => {
        if (!this.isCompleted) {
          this.xhr.abort();
        }
      }, this.ACTIVITY_TIMEOUT_MS);
    }
  }

  /**
   * Resolve the promise (only once)
   */
  resolve(response: Response): void {
    if (this.isCompleted) return;
    this.isCompleted = true;
    this.clearActivityTimeout();
    this.resolvePromise(response);
  }

  /**
   * Reject the promise (only once)
   */
  reject(error: HakanaiError): void {
    if (this.isCompleted) return;
    this.isCompleted = true;
    this.clearActivityTimeout();
    this.rejectPromise(error);
  }

  /**
   * Check if request is still active (for progress updates)
   */
  get isActive(): boolean {
    return !this.isCompleted;
  }

  private clearActivityTimeout(): void {
    if (this.activityTimeout !== null) {
      clearTimeout(this.activityTimeout);
      this.activityTimeout = null;
    }
  }
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
      throw new HakanaiError(HakanaiErrorCodes.EXPECTED_STRING, "Base URL must be a non-empty string");
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
    restrictions?: SecretRestrictions,
  ): void {
    if (!payload || typeof payload !== "object") {
      throw new HakanaiError(HakanaiErrorCodes.INVALID_PAYLOAD, "Payload must be an object");
    }

    if (!payload.data || typeof payload.data !== "string" || payload.data.length === 0) {
      throw new HakanaiError(HakanaiErrorCodes.INVALID_PAYLOAD, "Payload data cannot be empty");
    }

    // Use validation namespace for consistent TTL validation
    InputValidation.validateTTL(ttl);

    // Validate auth token format if provided
    if (authToken !== undefined) {
      InputValidation.validateAuthToken(authToken);
    }

    // Validate restrictions if provided
    if (restrictions !== undefined) {
      InputValidation.validateRestrictions(restrictions);
    }
  }

  /**
   * Create appropriate HakanaiError for HTTP response errors
   * @private
   */
  private createSendPayloadError(response: Response): HakanaiError {
    if (response.status === 401) {
      return new HakanaiError(
        HakanaiErrorCodes.AUTHENTICATION_REQUIRED,
        "Authentication required: Please provide a valid authentication token",
        response.status,
      );
    }

    if (response.status === 403) {
      return new HakanaiError(
        HakanaiErrorCodes.INVALID_TOKEN,
        "Invalid authentication token: Please check your token and try again",
        response.status,
      );
    }

    if (response.status === 413) {
      return new HakanaiError(
        HakanaiErrorCodes.PAYLOAD_TOO_LARGE,
        "The payload size exceeds the limit allowed for the user",
        response.status,
      );
    }

    if (response.status === 501) {
      return new HakanaiError(
        HakanaiErrorCodes.NOT_SUPPORTED,
        "This feature or operation is not supported by the server",
        response.status,
      );
    }

    // Generic error for other status codes
    return new HakanaiError(
      HakanaiErrorCodes.SEND_FAILED,
      `Failed to send secret: ${response.status} ${response.statusText}`,
      response.status,
    );
  }

  /**
   * Process response stream without known content length (e.g., compressed responses)
   * @private
   */
  private async processResponseStreamWithoutContentLength(
    response: Response,
    progressObserver?: DataTransferObserver,
  ): Promise<string> {
    const chunks: Uint8Array[] = [];
    let downloadedBytes = 0;

    const reader = response.body!.getReader();
    try {
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        if (value) {
          chunks.push(value);
          downloadedBytes += value.length;

          if (progressObserver) {
            try {
              await progressObserver.onProgress(downloadedBytes);
            } catch (error) {
              console.warn("Progress observer error:", error);
            }
          }
        }
      }
    } finally {
      reader.releaseLock();
    }

    // Combine all chunks
    const result = new Uint8Array(downloadedBytes);
    let offset = 0;
    for (const chunk of chunks) {
      result.set(chunk, offset);
      offset += chunk.length;
    }

    return new TextDecoder().decode(result);
  }

  /**
   * Process response stream with chunking and optional progress tracking
   * @private
   */
  private async processResponseStream(response: Response, progressObserver?: DataTransferObserver): Promise<string> {
    if (!response.body) {
      throw new HakanaiError(HakanaiErrorCodes.INVALID_SERVER_RESPONSE, "Response body is empty");
    }

    const contentLength = response.headers.get("content-length");

    // If no content-length header, use dynamic buffering
    if (!contentLength) {
      return this.processResponseStreamWithoutContentLength(response, progressObserver);
    }

    const totalBytes = parseInt(contentLength, 10);

    // Error if content-length is zero
    if (totalBytes === 0) {
      throw new HakanaiError(HakanaiErrorCodes.INVALID_SERVER_RESPONSE, "Response body is empty");
    }

    // Pre-allocate result array for efficiency, like Rust client
    const result = new Uint8Array(totalBytes);
    let downloadedBytes = 0;

    const reader = response.body.getReader();

    try {
      while (true) {
        const { done, value } = await reader.read();

        if (done) break;

        if (value) {
          // Copy chunk directly into result array at correct offset
          result.set(value, downloadedBytes);
          downloadedBytes += value.length;

          // Call progress observer if provided
          if (progressObserver) {
            try {
              await progressObserver.onProgress(downloadedBytes, totalBytes);
            } catch (error) {
              console.warn("Progress observer error:", error);
            }
          }
        }
      }
    } finally {
      reader.releaseLock();
    }

    return new TextDecoder().decode(result);
  }

  /**
   * Send data using XMLHttpRequest for real upload progress tracking
   * @private
   */
  private async sendWithXHR(
    bodyData: string,
    headers: Record<string, string>,
    progressObserver?: DataTransferObserver,
  ): Promise<Response> {
    return new Promise((resolve, reject) => {
      const xhr = new XMLHttpRequest();
      const promiseManager = new XhrPromiseManager(resolve, reject, xhr);

      // start timeout, if no progfress is received within 10 seconds, abort the request
      promiseManager.resetActivityTimeout();

      xhr.upload.onprogress = (event) => {
        if (event.lengthComputable && promiseManager.isActive) {
          // reset activity timeout on each progress update to prevent timeout
          promiseManager.resetActivityTimeout();

          progressObserver?.onProgress(event.loaded, event.total);
        }
      };

      xhr.onload = () => {
        // Convert XHR response to fetch Response
        const response = new Response(xhr.responseText, {
          status: xhr.status,
          statusText: xhr.statusText,
          headers: new Headers(),
        });

        // Check if the request was successful
        if (xhr.status >= 200 && xhr.status < 300) {
          promiseManager.resolve(response);
        } else {
          promiseManager.reject(this.createSendPayloadError(response));
        }
      };

      xhr.onerror = () => {
        // Network-level error (connection failed, etc.)
        promiseManager.reject(new HakanaiError(HakanaiErrorCodes.SEND_FAILED, "Network error during request"));
      };

      xhr.onabort = () => {
        promiseManager.reject(new HakanaiError(HakanaiErrorCodes.SEND_FAILED, "Request aborted"));
      };

      xhr.ontimeout = () => {
        promiseManager.reject(new HakanaiError(HakanaiErrorCodes.SEND_FAILED, "Request timed out"));
      };

      xhr.open("POST", `${this.baseUrl}/api/v1/secret`);

      // Set headers
      Object.entries(headers).forEach(([key, value]) => {
        xhr.setRequestHeader(key, value);
      });

      xhr.send(bodyData);
    });
  }

  /**
   * Send encrypted data to the server via HTTP with optional progress tracking
   * @private
   */
  private async sendEncryptedData(
    encryptedData: string,
    ttl: number,
    authToken?: string,
    progressObserver?: DataTransferObserver,
    restrictions?: SecretRestrictions,
  ): Promise<string> {
    const requestBody: SecretRequest = {
      data: encryptedData,
      expires_in: ttl,
    };

    if (restrictions) {
      requestBody.restrictions = restrictions;
    }

    const bodyData = JSON.stringify(requestBody);
    const requestId = crypto.randomUUID();

    const headers: Record<string, string> = {
      "Content-Type": "application/json",
      "X-Request-Id": requestId,
    };

    if (authToken?.length) {
      headers["Authorization"] = `Bearer ${authToken}`;
    }

    // Use XMLHttpRequest for real upload progress tracking
    const response = await this.sendWithXHR(bodyData, headers, progressObserver);

    if (!response.ok) {
      throw this.createSendPayloadError(response);
    }

    const responseData: SecretResponse = await response.json();
    if (!responseData.id || typeof responseData.id !== "string") {
      throw new HakanaiError(HakanaiErrorCodes.INVALID_SERVER_RESPONSE, "Invalid response: missing secret ID");
    }

    return responseData.id;
  }

  /**
   * Encrypt and send a payload to the Hakanai server
   * @param payload - Data to encrypt and send (must have non-empty data field)
   * @param ttl - Time-to-live in seconds (default: 3600)
   * @param authToken - Optional authentication token for server access
   * @param progressObserver - Optional progress observer for upload tracking
   * @param restrictions - Optional access restrictions (IP whitelist, etc.)
   * @returns Full URL with secret ID and decryption key in fragment
   * @throws {HakanaiError} With specific error codes:
   *   - AUTHENTICATION_REQUIRED: Server requires auth token
   *   - INVALID_TOKEN: Provided token is invalid
   *   - INVALID_RESTRICTIONS: Provided restrictions are invalid
   *   - SEND_FAILED: General send failure
   */
  async sendPayload(
    payload: PayloadData,
    ttl: number = 3600,
    authToken?: string,
    progressObserver?: DataTransferObserver,
    restrictions?: SecretRestrictions,
  ): Promise<string> {
    this.validateSendPayloadParams(payload, ttl, authToken, restrictions);

    const cryptoContext = await CryptoContext.generate();
    try {
      const secretPayload = {
        data: payload.data,
        filename: payload.filename ?? null,
      };
      const payloadJson = JSON.stringify(secretPayload);
      const payloadBytes = new TextEncoder().encode(payloadJson);

      const encryptedData = await cryptoContext.encrypt(payloadBytes.buffer);
      const hash = await HashUtils.hashContent(payloadBytes.buffer);

      // Clear payload bytes after encryption
      SecureMemory.clearUint8Array(payloadBytes);

      const secretId = await this.sendEncryptedData(encryptedData, ttl, authToken, progressObserver, restrictions);

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
      throw new HakanaiError(HakanaiErrorCodes.BASE64_ERROR, "Invalid decryption key in URL");
    }

    return { secretId, key, hash };
  }

  /**
   * Handle HTTP response errors for receivePayload
   * @private
   */
  private handleReceivePayloadError(response: Response): never {
    if (response.status === 401) {
      throw new HakanaiError(
        HakanaiErrorCodes.PASSPHRASE_REQUIRED,
        "This secret is protected and requires a passphrase",
        401,
      );
    }
    if (response.status === 403) {
      throw new HakanaiError(HakanaiErrorCodes.ACCESS_DENIED, "Client is not authorized to access this secret", 403);
    }
    if (response.status === 404) {
      throw new HakanaiError(HakanaiErrorCodes.SECRET_NOT_FOUND, "Secret not found or has expired", 404);
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

  private async verifyHash(plaintext: ArrayBuffer, expectedHash: string): Promise<void> {
    const actualHash = await HashUtils.hashContent(plaintext);
    if (actualHash !== expectedHash) {
      throw new HakanaiError(HakanaiErrorCodes.HASH_MISMATCH, "Hash verification failed");
    }
  }

  /**
   * Retrieve and decrypt a payload from the server
   * @param url - Full URL with format: https://server/s/{id}#{key}
   * @param progressObserver - Optional progress observer for download tracking
   * @param passphrase - Optional passphrase if the secret is protected
   * @returns Decrypted payload data with optional filename
   * @throws {HakanaiError} With specific error codes:
   *   - MISSING_DECRYPTION_KEY: No key found in URL fragment
   *   - SECRET_NOT_FOUND: Secret expired or doesn't exist
   *   - SECRET_ALREADY_ACCESSED: Secret was already retrieved
   *   - RETRIEVE_FAILED: General retrieval failure
   * @throws {Error} For invalid URL format or decryption failures
   */
  async receivePayload(
    url: string,
    progressObserver?: DataTransferObserver,
    passphrase?: string,
  ): Promise<PayloadData> {
    const { secretId, key, hash } = this.validateAndParseReceiveUrl(url);
    const requestId = crypto.randomUUID();

    const headers: Record<string, string> = {
      "X-Request-Id": requestId,
      "Accept-Encoding": "identity", // ensure no compression
    };

    // Add passphrase header if provided
    if (passphrase) {
      const passphraseHash = await HashUtils.hashPassphrase(passphrase);
      headers["X-Secret-Passphrase"] = passphraseHash;
    }

    const response = await fetch(`${this.baseUrl}/api/v1/secret/${secretId}`, {
      headers,
    });

    if (!response.ok) {
      this.handleReceivePayloadError(response);
    }

    const encryptedData = await this.processResponseStream(response, progressObserver);

    // Validate encrypted data format
    InputValidation.validateEncryptedData(encryptedData);

    const cryptoContext = await CryptoContext.fromKey(key);
    try {
      const decryptedBytes = await cryptoContext.decrypt(encryptedData);

      if (hash) {
        await this.verifyHash(decryptedBytes.buffer as ArrayBuffer, hash);
      }

      const decryptedJson = new TextDecoder().decode(decryptedBytes);

      // Clear decrypted bytes after converting to string
      SecureMemory.clearUint8Array(decryptedBytes);

      let payload: PayloadData;
      try {
        payload = JSON.parse(decryptedJson);
      } catch (error) {
        throw new HakanaiError(HakanaiErrorCodes.INVALID_PAYLOAD, "Failed to parse decrypted payload");
      }

      // Validate payload structure
      if (!payload || typeof payload !== "object" || typeof payload.data !== "string") {
        throw new HakanaiError(HakanaiErrorCodes.INVALID_PAYLOAD, "Invalid payload structure");
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
  type DataTransferObserver,
  SecretRequest,
  SecretResponse,
  SecretRestrictions,
};
