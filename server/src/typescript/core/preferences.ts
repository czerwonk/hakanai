// SPDX-License-Identifier: Apache-2.0

/**
 * Simple preference storage for user settings
 */

const STORAGE_KEYS = {
  LAST_TTL: "hakanai:lastTTL",
  SEPARATE_KEY_MODE: "hakanai:separateKeyMode",
  GENERATE_QR_CODE: "hakanai:generateQrCode",
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
      localStorage.setItem(
        STORAGE_KEYS.SEPARATE_KEY_MODE,
        enabled ? "true" : "false",
      );
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

  /**
   * Save the generate QR code preference
   */
  static saveGenerateQrCode(enabled: boolean): void {
    try {
      localStorage.setItem(
        STORAGE_KEYS.GENERATE_QR_CODE,
        enabled ? "true" : "false",
      );
    } catch (error) {
      console.warn("Failed to save generate QR code preference:", error);
    }
  }

  /**
   * Get the generate QR code preference or undefined if not set
   */
  static getGenerateQrCode(): boolean | undefined {
    try {
      const value = localStorage.getItem(STORAGE_KEYS.GENERATE_QR_CODE);
      if (value === "true") return true;
      if (value === "false") return false;
    } catch (error) {
      console.warn("Failed to get generate QR code preference:", error);
    }

    return undefined;
  }
}

/**
 * Initialize separate key checkbox on any page
 * @param checkboxId - The ID of the checkbox element
 */
export function initSeparateKeyCheckbox(checkbox: HTMLInputElement): void {
  if (!checkbox) {
    return;
  }

  // Restore saved preference
  const savedPreference = PreferenceStorage.getSeparateKeyMode();
  if (savedPreference !== undefined) {
    checkbox.checked = savedPreference;
  }

  // Save preference when checkbox changes
  checkbox.addEventListener("change", () => {
    PreferenceStorage.saveSeparateKeyMode(checkbox.checked);
  });
}

/**
 * Initialize generate QR code checkbox on any page
 * @param checkbox - The checkbox element
 */
export function initGenerateQrCodeCheckbox(checkbox: HTMLInputElement): void {
  if (!checkbox) {
    return;
  }

  // Restore saved preference (default to true if not set)
  const savedPreference = PreferenceStorage.getGenerateQrCode();
  if (savedPreference !== undefined) {
    checkbox.checked = savedPreference;
  }

  // Save preference when checkbox changes
  checkbox.addEventListener("change", () => {
    PreferenceStorage.saveGenerateQrCode(checkbox.checked);
  });
}
