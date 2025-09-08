// SPDX-License-Identifier: Apache-2.0

/**
 * Simple preference storage for user settings
 */

const STORAGE_KEYS = {
  LAST_TTL: "hakanai:lastTTL",
  SEPARATE_KEY_MODE: "hakanai:separateKeyMode",
} as const;

export class PreferenceStorage {
  /**
   * Save the last used TTL value
   */
  static saveLastTTL(ttl: number): void {
    try {
      localStorage.setItem(STORAGE_KEYS.LAST_TTL, ttl.toString());
    } catch (error) {
      console.warn("Failed to save TTL preference:", error);
    }
  }

  /**
   * Get the last used TTL value or undefined if not set
   */
  static getLastTTL(): number | undefined {
    try {
      const value = localStorage.getItem(STORAGE_KEYS.LAST_TTL);
      if (value) {
        const ttl = parseInt(value, 10);
        return isNaN(ttl) ? undefined : ttl;
      }
    } catch (error) {
      console.warn("Failed to get TTL preference:", error);
    }

    return undefined;
  }

  /**
   * Save the separate key mode preference
   */
  static saveSeparateKeyMode(enabled: boolean): void {
    try {
      localStorage.setItem(STORAGE_KEYS.SEPARATE_KEY_MODE, enabled ? "true" : "false");
    } catch (error) {
      console.warn("Failed to save separate key mode preference:", error);
    }
  }

  /**
   * Get the separate key mode preference or undefined if not set
   */
  static getSeparateKeyMode(): boolean | undefined {
    try {
      const value = localStorage.getItem(STORAGE_KEYS.SEPARATE_KEY_MODE);
      if (value === "true") return true;
      if (value === "false") return false;
    } catch (error) {
      console.warn("Failed to get separate key mode preference:", error);
    }

    return undefined;
  }
}
