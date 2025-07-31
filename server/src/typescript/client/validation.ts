import { HakanaiError, HakanaiErrorCodes } from "./errors";

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
   * Validate the format of the hash
   * @param hash - Hash of the secret key to validate
   * @throws {HakanaiError} If key format or length is invalid
   */
  static validateHash(hash: string): void {
    if (typeof hash !== "string") {
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_HASH,
        "Hash must be a string",
      );
    }

    if (!hash.trim()) {
      // empty hash is allowed (no hash provided)
      return;
    }

    // Validate base64url format and length (32 bytes = 43 chars in base64url without padding)
    if (!/^[0-9a-fA-F]{64}$/.test(hash)) {
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_HASH,
        "Hash must be a 64-character hexadecimal string",
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

export { InputValidation };
