// SPDX-License-Identifier: Apache-2.0

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
  /** Secret is too large */
  PAYLOAD_TOO_LARGE: "PAYLOAD_TOO_LARGE",
  /** Hash validation of the received secret has failed */
  HASH_MISMATCH: "HASH_MISMATCH",
  /** Client not allowed to access the secret */
  ACCESS_DENIED: "ACCESS_DENIED",

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
  /** Secret restrictions are invalid or malformed */
  INVALID_RESTRICTIONS: "INVALID_RESTRICTIONS",
  /** Server response is invalid or empty */
  INVALID_SERVER_RESPONSE: "INVALID_SERVER_RESPONSE",
  /** CryptoContext has been disposed */
  CRYPTO_CONTEXT_DISPOSED: "CRYPTO_CONTEXT_DISPOSED",
  /** Hash format is invalid */
  INVALID_HASH: "INVALID_HASH",
  /** URL fragment is missing content integrity hash */
  MISSING_HASH: "MISSING_HASH",
} as const;

// Type for error codes
type HakanaiErrorCode =
  (typeof HakanaiErrorCodes)[keyof typeof HakanaiErrorCodes];

/**
 * Custom error class for Hakanai operations with error codes for internationalization
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

export { HakanaiError, HakanaiErrorCodes };
