// SPDX-License-Identifier: Apache-2.0

import { HakanaiError, HakanaiErrorCodes } from "./errors";
import { InputValidation } from "./validation";

/**
 * URL parsing utilities for Hakanai secret URLs
 * @class UrlParser
 */
class UrlParser {
  /**
   * Parse and validate a secret URL, returning its components
   * @param url - Complete secret URL
   * @returns Object with validated secretId, secretKey and hash (optional)
   * @throws {HakanaiError} If URL or its components are invalid
   */
  static parseSecretUrl(url: string): {
    secretId: string;
    secretKey: string;
    hash?: string;
  } {
    // Basic URL validation
    if (typeof url !== "string" || !url.trim()) {
      throw new HakanaiError(HakanaiErrorCodes.INVALID_URL_FORMAT, "URL cannot be empty");
    }

    let urlObj: URL;
    try {
      urlObj = new URL(url);
    } catch {
      throw new HakanaiError(HakanaiErrorCodes.INVALID_URL_FORMAT, "Invalid URL format");
    }

    // Parse and validate URL structure first
    const pathParts = urlObj.pathname.split("/");
    if (pathParts.length !== 3 || pathParts[1] !== "s" || !pathParts[2]) {
      throw new HakanaiError(HakanaiErrorCodes.MISSING_SECRET_ID, "URL must contain secret ID in format /s/{id}");
    }

    // Check if hash fragment is missing from URL
    if (!urlObj.hash) {
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_URL_FORMAT,
        "URL must contain decryption key and hash in fragment",
      );
    }

    const secretId = pathParts[2];
    InputValidation.validateSecretId(secretId);

    const fragmentParts = urlObj.hash.slice(1).split(":");

    const secretKey = fragmentParts[0];
    if (!secretKey) {
      throw new HakanaiError(HakanaiErrorCodes.MISSING_DECRYPTION_KEY, "URL fragment must contain decryption key");
    }
    InputValidation.validateSecretKey(secretKey);

    const hash = fragmentParts[1];
    if (!hash) {
      throw new HakanaiError(
        HakanaiErrorCodes.MISSING_HASH,
        "URL fragment must contain a hash for content integrity verification",
      );
    }
    InputValidation.validateHash(hash);

    return { secretId, secretKey, hash };
  }
}

export { UrlParser };
