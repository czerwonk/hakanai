// SPDX-License-Identifier: MIT

/**
 * Tests for Base64UrlSafe encoding/decoding functionality
 */

import { Base64UrlSafe } from "../../server/src/typescript/hakanai-client";

describe("Base64UrlSafe", () => {
  test("encode and decode round trip with text", () => {
    const original = "Hello, World! 🌍";
    const encoder = new TextEncoder();
    const bytes = encoder.encode(original);

    // Convert to Uint8Array if needed (Node.js TextEncoder returns different type)
    const uint8Array =
      bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes);

    const encoded = Base64UrlSafe.encode(uint8Array);
    const decoded = Base64UrlSafe.decode(encoded);

    const decoder = new TextDecoder();
    const result = decoder.decode(decoded);

    expect(result).toBe(original);
  });

  test("encode produces URL-safe characters", () => {
    const testBytes = new Uint8Array([255, 254, 253, 252, 251, 250]);
    const encoded = Base64UrlSafe.encode(testBytes);

    // Should not contain +, /, or = characters
    expect(encoded).not.toMatch(/[+/=]/);
    // Should only contain URL-safe characters
    expect(encoded).toMatch(/^[A-Za-z0-9_-]*$/);
  });

  test("decode handles padding correctly", () => {
    const testBytes = new Uint8Array([1, 2, 3, 4, 5]);
    const encoded = Base64UrlSafe.encode(testBytes);
    const decoded = Base64UrlSafe.decode(encoded);

    expect(Array.from(decoded)).toEqual([1, 2, 3, 4, 5]);
  });

  test("encodeText and decodeText convenience methods", () => {
    const original = "Test string with special chars: åëïöü";
    const encoded = Base64UrlSafe.encodeText(original);
    const decoded = Base64UrlSafe.decodeText(encoded);

    expect(decoded).toBe(original);
  });
});
