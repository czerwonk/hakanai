// SPDX-License-Identifier: MIT

/**
 * Secure memory clearing utilities for sensitive data
 * @class SecureMemory
 */
class SecureMemory {
  /**
   * Securely clear a Uint8Array with multiple overwrite passes
   * @param array - Array to clear
   */
  static clearUint8Array(array: Uint8Array): void {
    if (!(array instanceof Uint8Array)) {
      return;
    }

    // Multiple overwrite passes with random data
    for (let pass = 0; pass < 3; pass++) {
      try {
        crypto.getRandomValues(array);
      } catch (error) {
        // Fallback to manual random fill if crypto not available
        for (let i = 0; i < array.length; i++) {
          array[i] = Math.floor(Math.random() * 256);
        }
      }
    }

    // Final zero fill
    array.fill(0);
  }

  /**
   * Securely clear multiple Uint8Array objects
   * @param arrays - Arrays to clear
   */
  static clearArrays(...arrays: Uint8Array[]): void {
    arrays.forEach((array) => SecureMemory.clearUint8Array(array));
  }
}

export { SecureMemory };
