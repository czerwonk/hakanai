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

export { UrlParser };

