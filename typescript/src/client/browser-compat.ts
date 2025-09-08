// SPDX-License-Identifier: Apache-2.0

interface CompatibilityCheck {
  readonly isCompatible: boolean;
  readonly missingFeatures: readonly string[];
}

/**
 * Browser compatibility checker
 */
class BrowserCompatibility {
  /**
   * Get detailed compatibility information
   */
  static getCompatibilityInfo(): CompatibilityCheck {
    const missingFeatures: string[] = [];

    // Check for Web Crypto API (browser environment)
    const cryptoInstance = window?.crypto ?? crypto;
    if (!cryptoInstance || !cryptoInstance.subtle) {
      missingFeatures.push("Web Crypto API (crypto.subtle)");
    }

    // Check for TextEncoder/TextDecoder
    if (typeof TextEncoder === "undefined") {
      missingFeatures.push("TextEncoder");
    }
    if (typeof TextDecoder === "undefined") {
      missingFeatures.push("TextDecoder");
    }

    // Check for crypto.getRandomValues
    if (!cryptoInstance || typeof cryptoInstance.getRandomValues !== "function") {
      missingFeatures.push("crypto.getRandomValues");
    }

    // Check for fetch API
    if (typeof fetch === "undefined") {
      missingFeatures.push("Fetch API");
    }

    // Check for Uint8Array
    if (typeof Uint8Array === "undefined") {
      missingFeatures.push("Uint8Array");
    }

    // Check for atob/btoa (base64 functions)
    if (typeof atob === "undefined" || typeof btoa === "undefined") {
      missingFeatures.push("Base64 functions (atob/btoa)");
    }

    return {
      isCompatible: missingFeatures.length === 0,
      missingFeatures: Object.freeze(missingFeatures),
    };
  }

  /**
   * Check if browser is compatible (simple boolean)
   */
  static isCompatible(): boolean {
    return BrowserCompatibility.getCompatibilityInfo().isCompatible;
  }
}

export { type CompatibilityCheck, BrowserCompatibility };
