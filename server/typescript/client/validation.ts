// SPDX-License-Identifier: Apache-2.0

import { HakanaiError, HakanaiErrorCodes } from "./errors";
import { SecretRestrictions } from "./client";

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

    // Validate base64url format and length (32 bytes = 43 chars in base64url without padding)
    if (!/^[A-Za-z0-9_-]{22}$/.test(hash)) {
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_HASH,
        "Hash must be a 22-character base64url string (truncated SHA-256)",
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

  /**
   * Validate IP address or CIDR notation format
   * @param ip - IP address string to validate
   * @throws {HakanaiError} If IP address format is invalid
   */
  static validateIPAddress(ip: string): void {
    if (typeof ip !== "string" || ip.trim().length === 0) {
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_RESTRICTIONS,
        "IP address must be a non-empty string",
      );
    }

    // Basic CIDR/IP format validation
    const ipPattern =
      /^(?:(?:\d{1,3}\.){3}\d{1,3}(?:\/\d{1,2})?|(?:[0-9a-f:]+(?:\/\d{1,3})?))$/i;
    if (!ipPattern.test(ip.trim())) {
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_RESTRICTIONS,
        `Invalid IP address or CIDR notation: ${ip}`,
      );
    }
  }

  /**
   * Validate country code format (ISO 3166-1 alpha-2)
   * @param country - Country code string to validate
   * @throws {HakanaiError} If country code format is invalid
   */
  static validateCountryCode(country: string): void {
    if (typeof country !== "string" || country.trim().length === 0) {
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_RESTRICTIONS,
        "Country code must be a non-empty string",
      );
    }

    // Validate ISO 3166-1 alpha-2 format (exactly 2 uppercase letters)
    if (!/^[A-Z]{2}$/.test(country.trim())) {
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_RESTRICTIONS,
        `Invalid country code: ${country}. Must be a 2-letter uppercase ISO 3166-1 alpha-2 code (e.g., US, DE, CA)`,
      );
    }
  }

  /**
   * Validate ASN (Autonomous System Number) format
   * @param asn - ASN number to validate
   * @throws {HakanaiError} If ASN format is invalid
   */
  static validateASN(asn: number): void {
    if (typeof asn !== "number") {
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_RESTRICTIONS,
        "ASN must be a number",
      );
    }

    if (!Number.isInteger(asn)) {
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_RESTRICTIONS,
        "ASN must be an integer",
      );
    }

    // Warn about special/reserved ASN ranges
    if (asn === 0) {
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_RESTRICTIONS,
        "ASN 0 is reserved and cannot be used",
      );
    }

    // ASN range: 1 to 4294967295 (2^32 - 1)
    // ASN 0 is already handled above
    if (asn < 1 || asn > 4294967295) {
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_RESTRICTIONS,
        `Invalid ASN: ${asn}. Must be between 1 and 4294967295`,
      );
    }
  }

  /**
   * Validate secret restrictions format
   * @param restrictions - Secret restrictions object to validate
   * @throws {HakanaiError} If restrictions format is invalid
   */
  static validateRestrictions(restrictions: SecretRestrictions): void {
    if (
      restrictions === null ||
      typeof restrictions !== "object" ||
      Array.isArray(restrictions)
    ) {
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_RESTRICTIONS,
        "Restrictions must be an object",
      );
    }

    if (restrictions.allowed_ips !== undefined) {
      if (!Array.isArray(restrictions.allowed_ips)) {
        throw new HakanaiError(
          HakanaiErrorCodes.INVALID_RESTRICTIONS,
          "allowed_ips must be an array",
        );
      }
      for (const ip of restrictions.allowed_ips) {
        this.validateIPAddress(ip);
      }
    }

    if (restrictions.allowed_countries !== undefined) {
      if (!Array.isArray(restrictions.allowed_countries)) {
        throw new HakanaiError(
          HakanaiErrorCodes.INVALID_RESTRICTIONS,
          "allowed_countries must be an array",
        );
      }
      for (const country of restrictions.allowed_countries) {
        this.validateCountryCode(country);
      }
    }

    if (restrictions.allowed_asns !== undefined) {
      if (!Array.isArray(restrictions.allowed_asns)) {
        throw new HakanaiError(
          HakanaiErrorCodes.INVALID_RESTRICTIONS,
          "allowed_asns must be an array",
        );
      }
      for (const asn of restrictions.allowed_asns) {
        this.validateASN(asn);
      }
    }

    if (restrictions.passphrase_hash !== undefined) {
      if (typeof restrictions.passphrase_hash !== "string") {
        throw new HakanaiError(
          HakanaiErrorCodes.INVALID_RESTRICTIONS,
          "passphrase_hash must be a string",
        );
      }

      // Validate SHA-256 hex format (64 hex characters)
      if (!/^[a-fA-F0-9]{64}$/.test(restrictions.passphrase_hash)) {
        throw new HakanaiError(
          HakanaiErrorCodes.INVALID_RESTRICTIONS,
          "passphrase_hash must be a 64-character hexadecimal string (SHA-256 hash)",
        );
      }
    }
  }
}

export { InputValidation };
