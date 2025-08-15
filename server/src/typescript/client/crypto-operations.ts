// SPDX-License-Identifier: Apache-2.0

import { HakanaiError, HakanaiErrorCodes } from "./errors";
import { Base64UrlSafe } from "./base64-utils";
import { SecureMemory } from "./secure-memory";

export const KEY_LENGTH = 32; // 256 bits
export const NONCE_LENGTH = 12; // 96 bits for AES-GCM

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
    const cryptoInstance = window?.crypto ?? crypto;
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

export { CryptoContext };
