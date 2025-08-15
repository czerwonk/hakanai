// SPDX-License-Identifier: Apache-2.0

import { HakanaiError, HakanaiErrorCodes } from "./errors";

/**
 * Content analysis utilities for detecting binary vs text data
 * @class ContentAnalysis
 */
class ContentAnalysis {
  /**
   * Check if a byte array contains null bytes (indicating binary content)
   *
   * This function uses a simple but effective heuristic: the presence of null bytes.
   * Most text encodings (UTF-8, ASCII, etc.) don't contain null bytes, while binary
   * formats (executables, images, compressed files) commonly do.
   *
   * @param bytes - Byte array to analyze
   * @returns true if the content appears to be binary data, false if it appears to be text
   *
   * @example
   * ```typescript
   * const textBytes = new TextEncoder().encode("Hello, world!");
   * const binaryBytes = new Uint8Array([0x00, 0x01, 0x02, 0xFF]);
   *
   * console.log(ContentAnalysis.isBinary(textBytes)); // false
   * console.log(ContentAnalysis.isBinary(binaryBytes)); // true
   * ```
   */
  static isBinary(bytes: Uint8Array): boolean {
    if (!(bytes instanceof Uint8Array)) {
      throw new HakanaiError(
        HakanaiErrorCodes.EXPECTED_UINT8_ARRAY,
        "Input must be a Uint8Array",
      );
    }

    // Check for null bytes, which are common in binary files
    return bytes.includes(0);
  }
}

export { ContentAnalysis };
