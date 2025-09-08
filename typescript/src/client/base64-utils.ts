// SPDX-License-Identifier: Apache-2.0

import { HakanaiError, HakanaiErrorCodes } from "./errors";

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
      throw new HakanaiError(HakanaiErrorCodes.EXPECTED_UINT8_ARRAY, "Input must be a Uint8Array");
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
    return btoa(binaryString).replace(/\+/g, "-").replace(/\//g, "_").replace(/=/g, "");
  }

  /**
   * Decode URL-safe base64 string to Uint8Array
   * @param encoded - URL-safe base64 string to decode
   * @returns Decoded bytes as Uint8Array
   * @throws {Error} If input contains invalid characters or encoding
   */
  static decode(encoded: string): Uint8Array {
    if (typeof encoded !== "string") {
      throw new HakanaiError(HakanaiErrorCodes.EXPECTED_STRING, "Input must be a string");
    }

    if (encoded.length === 0) {
      return new Uint8Array(0);
    }

    if (!/^[A-Za-z0-9_-]*$/.test(encoded)) {
      throw new HakanaiError(HakanaiErrorCodes.INVALID_INPUT_FORMAT, "Invalid base64url characters");
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
      throw new HakanaiError(HakanaiErrorCodes.BASE64_ERROR, "Failed to decode base64url string: invalid encoding");
    }
  }

  /**
   * Utility method for encoding text strings directly
   * More efficient than manual TextEncoder + encode
   */
  static encodeText(text: string): string {
    if (typeof text !== "string") {
      throw new HakanaiError(HakanaiErrorCodes.EXPECTED_STRING, "Input must be a string");
    }

    const encoder = new TextEncoder();
    const bytes = encoder.encode(text);

    // Convert to Uint8Array if needed (Node.js TextEncoder returns different type)
    const uint8Array = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes);
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

export { Base64UrlSafe };
