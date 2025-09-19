// SPDX-License-Identifier: Apache-2.0

import { Base64UrlSafe } from "./base64-utils";

/**
 * Hash utilities for cryptographic operations
 * Provides consistent hashing functions used throughout the application
 */
export class HashUtils {
  /**
   * Hash content using SHA-256 and return truncated base64url (for content integrity)
   * @param bytes - Content data to hash
   * @returns Truncated SHA-256 hash as base64url string (128 bits = 22 chars)
   */
  static async hashContent(bytes: ArrayBuffer): Promise<string> {
    const hashBuffer = await crypto.subtle.digest("SHA-256", bytes);
    const hashArray = new Uint8Array(hashBuffer);
    const truncated = hashArray.slice(0, 16); // 128 bits for URL compactness
    return Base64UrlSafe.encode(truncated);
  }

  /**
   * Hash passphrase using SHA-256 (full hash for authentication)
   * @param passphrase - The passphrase to hash
   * @returns Full SHA-256 hash as hex string (256 bits = 64 hex chars)
   */
  static async hashPassphrase(passphrase: string): Promise<string> {
    const passphraseBytes = new TextEncoder().encode(passphrase);
    const hashBuffer = await crypto.subtle.digest("SHA-256", passphraseBytes.buffer);
    const hashArray = new Uint8Array(hashBuffer);

    // Convert to hex string (like the Rust implementation)
    return Array.from(hashArray)
      .map((b) => b.toString(16).padStart(2, "0"))
      .join("");
  }
}
